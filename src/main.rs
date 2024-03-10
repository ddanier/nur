mod engine;
mod args;
mod context;
mod errors;
mod path;
mod nu_version;
mod commands;

use std::env;
use std::io::BufReader;
use nu_cmd_base::util::get_init_cwd;
use crate::engine::init_engine_state;
use crate::args::{gather_commandline_args, parse_commandline_args};
use nu_protocol::{Span, NU_VARIABLE_ID, eval_const::create_nu_constant, PipelineData, RawStream, BufferedReader, Value, Record, Type};
use miette::Result;
use nu_protocol::engine::StateWorkingSet;
use crate::commands::Nur;
use crate::context::Context;
use crate::errors::NurError;
use crate::path::find_project_path;

fn main() -> Result<(), miette::ErrReport> {
    // Get initial directory details
    let init_cwd = get_init_cwd();
    let project_path = find_project_path(&init_cwd)?;

    // Initialize nu engine state
    let mut engine_state = init_engine_state(project_path)?;

    // Parse args
    let (args_to_nur, task_name, args_to_task) = gather_commandline_args();
    let parsed_nur_args = parse_commandline_args(&args_to_nur.join(" "), &mut engine_state)
        .unwrap_or_else(|_| std::process::exit(1));

    if parsed_nur_args.debug_output {
        eprintln!("nur args: {:?}", parsed_nur_args);
        eprintln!("task name: {:?}", task_name);
        eprintln!("task args: {:?}", args_to_task);
        eprintln!("init_cwd: {:?}", init_cwd);
        eprintln!("project_path: {:?}", project_path);
    }

    // Init config
    // TODO: Setup config/env nu file?
    // engine_state.set_config_path("nur-config", path);
    // set_config_path(
    //     &mut engine_state,
    //     &init_cwd,
    //     "config.nu",
    //     "config-path",
    //     parsed_nu_cli_args.config_file.as_ref(),
    // );
    // set_config_path(
    //     &mut engine_state,
    //     &init_cwd,
    //     "env.nu",
    //     "env-path",
    //     parsed_nu_cli_args.env_file.as_ref(),
    // );

    // Add library path in project
    let nurscripts_path = project_path.join(".nurscripts");
    if parsed_nur_args.debug_output {
        eprintln!("nurscripts path: {:?}", nurscripts_path);
    }
    if nurscripts_path.exists() && nurscripts_path.is_dir() {
        engine_state.add_env_var("NU_LIB_DIRS".into(), Value::list(
            vec![Value::string(String::from(nurscripts_path.to_str().unwrap()), Span::unknown())],
            Span::unknown(),
        ));
    } else {
        // Ensure we do not load user libraries
        engine_state.add_env_var("NU_LIB_DIRS".into(), Value::list(
            vec![],
            Span::unknown(),
        ));
    }

    if parsed_nur_args.config_file.is_some() {
        eprintln!("WARNING: Config files are not supported yet.")
    }

    // Set up the $nu constant before evaluating any files (need to have $nu available in them)
    let nu_const = create_nu_constant(&engine_state, PipelineData::empty().span().unwrap_or_else(Span::unknown))?;
    engine_state.set_variable_const_val(NU_VARIABLE_ID, nu_const);

    // Add $nur constant record (like $nu)
    let mut nur_record = Record::new();
    nur_record.push("run-path", Value::string(
        String::from(init_cwd.to_str().unwrap()),
        Span::unknown(),
    ));
    nur_record.push("project-path", Value::string(
        String::from(project_path.to_str().unwrap()),
        Span::unknown(),
    ));
    nur_record.push("task-name", Value::string(
        &task_name,
        Span::unknown(),
    ));
    let mut working_set = StateWorkingSet::new(&engine_state);
    let nur_var_id = working_set.add_variable(
        "nur".as_bytes().into(),
        Span::unknown(),
        Type::Record(vec![]),
        false,
    );
    engine_state.merge_delta(working_set.render())?;
    engine_state.set_variable_const_val(nur_var_id, Value::record(nur_record, Span::unknown()));

    // Switch to using context
    let mut context = Context::from(engine_state);

    // Load task files
    let nurfile_path = project_path.join("nurfile");
    let local_nurfile_path = project_path.join("nurfile.local");
    if parsed_nur_args.debug_output {
        eprintln!("nurfile path: {:?}", nurfile_path);
        eprintln!("nurfile local path: {:?}", local_nurfile_path);
    }
    if nurfile_path.exists() {
        context.source(
            nurfile_path,
            PipelineData::empty(),
        )?;
    }
    if local_nurfile_path.exists() {
        context.source(
            local_nurfile_path,
            PipelineData::empty(),
        )?;
    }

    // Handle list tasks
    if parsed_nur_args.list_tasks {
        // TODO: Parse and handle commands without eval
        context.eval_and_print(
            r#"scope commands | where name starts-with "nur " and category == "default" | get name | each { |it| $it | str substring 4.. } | sort"#,
            PipelineData::empty(),
        )?;

        std::process::exit(0);
    }

    // Initialize internal data
    let task_def_name = format!("nur {}", task_name);
    if parsed_nur_args.debug_output {
        eprintln!("task def name: {}", task_def_name);
    }

    // Handle help
    if parsed_nur_args.show_help || task_name.is_empty() {
        if task_name.is_empty() {
            context.print_help(&Nur);
        } else if let Some(command) = context.get_def(task_def_name) {
            context.clone().print_help(command);
        } else {
            return Err(miette::ErrReport::from(
                NurError::TaskNotFound(task_name)
            ));
        }

        std::process::exit(0);
    }

    // Check if requested task exists
    if !context.has_def(&task_def_name) {
        return Err(miette::ErrReport::from(
            NurError::TaskNotFound(task_name)
        ));
    }

    // Prepare input data - if requested
    let input = if parsed_nur_args.attach_stdin {
        let stdin = std::io::stdin();
        let buf_reader = BufReader::new(stdin);

        PipelineData::ExternalStream {
            stdout: Some(RawStream::new(
                Box::new(BufferedReader::new(buf_reader)),
                None,
                Span::unknown(),
                None,
            )),
            stderr: None,
            exit_code: None,
            span: Span::unknown(),
            metadata: None,
            trim_end_newline: false,
        }
    } else {
        PipelineData::empty()
    };

    // Execute the task
    let full_task_call = format!("{} {}", task_def_name, args_to_task.join(" "));
    if parsed_nur_args.debug_output {
        eprintln!("full task call: {}", full_task_call);
    }
    if parsed_nur_args.quiet_execution {
        context.eval(
            full_task_call,
            input,
        )?;
    } else {
        println!("nur version {}", env!("CARGO_PKG_VERSION"));
        println!("Project path {:?}", project_path);
        println!("Executing task {}", task_name);
        context.eval_and_print(
            full_task_call,
            input,
        )?;
        println!("Task exited ok");
    }

    Ok(())
}

