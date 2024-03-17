mod args;
mod commands;
mod context;
mod engine;
mod errors;
mod names;
mod nu_version;
mod path;

use crate::args::{gather_commandline_args, parse_commandline_args};
use crate::commands::Nur;
use crate::context::Context;
use crate::engine::init_engine_state;
use crate::errors::NurError;
use crate::names::{
    NUR_CONFIG_CONFIG_FILENAME, NUR_CONFIG_ENV_FILENAME, NUR_CONFIG_LIB_PATH, NUR_CONFIG_PATH,
    NUR_FILE, NUR_LOCAL_FILE, NUR_NAME, NUR_VAR_CONFIG_DIR, NUR_VAR_PROJECT_PATH, NUR_VAR_RUN_PATH,
    NUR_VAR_TASK_NAME,
};
#[cfg(feature = "plugin")]
use crate::names::{NUR_CONFIG_PLUGIN_FILENAME, NUR_CONFIG_PLUGIN_PATH};
use crate::path::find_project_path;
use miette::Result;
use nu_ansi_term::Color;
use nu_cli::gather_parent_env_vars;
use nu_cmd_base::util::get_init_cwd;
use nu_protocol::engine::StateWorkingSet;
use nu_protocol::{
    eval_const::create_nu_constant, BufferedReader, PipelineData, RawStream, Record, Span, Type,
    Value, NU_VARIABLE_ID,
};
use nu_std::load_standard_library;
use nu_utils::{get_default_config, get_default_env};
use std::env;
use std::io::BufReader;
use std::process::ExitCode;

fn main() -> Result<ExitCode, miette::ErrReport> {
    // Get initial directory details
    let init_cwd = get_init_cwd();
    let project_path = find_project_path(&init_cwd)?;

    // Initialize nu engine state
    let mut engine_state = init_engine_state(project_path)?;
    let use_color = engine_state.get_config().use_ansi_coloring;

    // Parse args
    let (args_to_nur, task_name, args_to_task) = gather_commandline_args();
    let parsed_nur_args = parse_commandline_args(&args_to_nur.join(" "), &mut engine_state)
        .unwrap_or_else(|_| std::process::exit(1));

    #[cfg(feature = "debug")]
    if parsed_nur_args.debug_output {
        eprintln!("nur args: {:?}", parsed_nur_args);
        eprintln!("task name: {:?}", task_name);
        eprintln!("task args: {:?}", args_to_task);
        eprintln!("init_cwd: {:?}", init_cwd);
        eprintln!("project_path: {:?}", project_path);
    }

    // Base path for nur config/env
    let nur_config_dir = project_path.join(NUR_CONFIG_PATH);
    #[cfg(feature = "debug")]
    if parsed_nur_args.debug_output {
        eprintln!("nur config path: {:?}", nur_config_dir);
    }

    // Set default scripts path
    let mut nur_lib_dir_path = nur_config_dir.clone();
    nur_lib_dir_path.push(NUR_CONFIG_LIB_PATH);
    engine_state.add_env_var(
        "NU_LIB_DIRS".to_string(),
        Value::test_string(nur_lib_dir_path.to_string_lossy()),
    );
    #[cfg(feature = "debug")]
    if parsed_nur_args.debug_output {
        eprintln!("nur scripts path: {:?}", nur_lib_dir_path);
    }

    #[cfg(feature = "plugin")]
    {
        // Set default plugin path
        let mut nur_plugin_dirs_path = nur_config_dir.clone();
        nur_plugin_dirs_path.push(NUR_CONFIG_PLUGIN_PATH);
        engine_state.add_env_var(
            "NU_PLUGIN_DIRS".to_string(),
            Value::test_string(nur_plugin_dirs_path.to_string_lossy()),
        );
        #[cfg(feature = "debug")]
        if parsed_nur_args.debug_output {
            eprintln!("nur plugins path: {:?}", nur_plugin_dirs_path);
        }
    }

    // Set config and env paths to .nur versions
    let mut nur_env_path = nur_config_dir.clone();
    nur_env_path.push(NUR_CONFIG_ENV_FILENAME);
    engine_state.set_config_path("env-path", nur_env_path.clone());
    let mut nur_config_path = nur_config_dir.clone();
    nur_config_path.push(NUR_CONFIG_CONFIG_FILENAME);
    engine_state.set_config_path("config-path", nur_config_path.clone());
    #[cfg(feature = "plugin")]
    let nur_plugin_path = {
        let mut nur_plugin_path = nur_config_dir.clone();
        nur_plugin_path.push(NUR_CONFIG_PLUGIN_FILENAME);
        engine_state.set_config_path("plugin-path", nur_plugin_path.clone());
        nur_plugin_path
    };

    // Set up the $nu constant before evaluating any files (need to have $nu available in them)
    let nu_const = create_nu_constant(
        &engine_state,
        PipelineData::empty().span().unwrap_or_else(Span::unknown),
    )?;
    engine_state.set_variable_const_val(NU_VARIABLE_ID, nu_const);

    // Add $nur constant record (like $nu)
    let mut nur_record = Record::new();
    nur_record.push(
        NUR_VAR_RUN_PATH,
        Value::string(String::from(init_cwd.to_str().unwrap()), Span::unknown()),
    );
    nur_record.push(
        NUR_VAR_PROJECT_PATH,
        Value::string(
            String::from(project_path.to_str().unwrap()),
            Span::unknown(),
        ),
    );
    nur_record.push(
        NUR_VAR_TASK_NAME,
        Value::string(&task_name, Span::unknown()),
    );
    nur_record.push(
        NUR_VAR_CONFIG_DIR,
        Value::string(
            String::from(nur_config_dir.to_str().unwrap()),
            Span::unknown(),
        ),
    );
    let mut working_set = StateWorkingSet::new(&engine_state);
    let nur_var_id = working_set.add_variable(
        NUR_NAME.as_bytes().into(),
        Span::unknown(),
        Type::Record(vec![]),
        false,
    );
    engine_state.merge_delta(working_set.render())?;
    engine_state.set_variable_const_val(nur_var_id, Value::record(nur_record, Span::unknown()));

    // Further engine setup
    gather_parent_env_vars(&mut engine_state, project_path);
    load_standard_library(&mut engine_state)?;

    // Switch to using context
    let mut context = Context::from(engine_state);

    // Load end and context
    #[cfg(feature = "plugin")]
    context.read_plugin_file(&nur_plugin_path);
    if nur_env_path.exists() {
        context.source_and_merge_env(&nur_env_path, PipelineData::empty())?;
    } else {
        context.eval_and_merge_env(get_default_env(), PipelineData::empty())?;
    }
    if nur_config_path.exists() {
        context.source_and_merge_env(&nur_config_path, PipelineData::empty())?;
    } else {
        context.eval_and_merge_env(get_default_config(), PipelineData::empty())?;
    }

    // Load task files
    let nurfile_path = project_path.join(NUR_FILE);
    let local_nurfile_path = project_path.join(NUR_LOCAL_FILE);
    #[cfg(feature = "debug")]
    if parsed_nur_args.debug_output {
        eprintln!("nurfile path: {:?}", nurfile_path);
        eprintln!("nurfile local path: {:?}", local_nurfile_path);
    }
    if nurfile_path.exists() {
        context.source(nurfile_path, PipelineData::empty())?;
    }
    if local_nurfile_path.exists() {
        context.source(local_nurfile_path, PipelineData::empty())?;
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
    let task_def_name = format!("{} {}", NUR_NAME, task_name);
    #[cfg(feature = "debug")]
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
            return Err(miette::ErrReport::from(NurError::TaskNotFound(task_name)));
        }

        std::process::exit(0);
    }

    // Check if requested task exists
    if !context.has_def(&task_def_name) {
        return Err(miette::ErrReport::from(NurError::TaskNotFound(task_name)));
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
    let exit_code: i64;
    let full_task_call = format!("{} {}", task_def_name, args_to_task.join(" "));
    #[cfg(feature = "debug")]
    if parsed_nur_args.debug_output {
        eprintln!("full task call: {}", full_task_call);
    }
    if parsed_nur_args.quiet_execution {
        exit_code = context.eval(full_task_call, input)?;

        #[cfg(feature = "debug")]
        if parsed_nur_args.debug_output {
            println!("Exit code {:?}", exit_code);
        }
    } else {
        println!("nur version {}", env!("CARGO_PKG_VERSION"));
        println!("Project path {:?}", project_path);
        println!("Executing task {}", task_name);
        println!();
        exit_code = context.eval_and_print(full_task_call, input)?;
        #[cfg(feature = "debug")]
        if parsed_nur_args.debug_output {
            println!("Exit code {:?}", exit_code);
        }
        if exit_code == 0 {
            println!(
                "{}Task execution successful{}",
                if use_color {
                    Color::Green.prefix().to_string()
                } else {
                    String::from("")
                },
                if use_color {
                    Color::Green.suffix().to_string()
                } else {
                    String::from("")
                },
            );
        } else {
            println!(
                "{}Task execution failed{}",
                if use_color {
                    Color::Red.prefix().to_string()
                } else {
                    String::from("")
                },
                if use_color {
                    Color::Red.suffix().to_string()
                } else {
                    String::from("")
                },
            );
        }
    }

    Ok(ExitCode::from(exit_code as u8))
}
