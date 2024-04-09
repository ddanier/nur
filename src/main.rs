mod args;
mod commands;
mod compat;
mod defaults;
mod engine;
mod errors;
mod names;
mod nu_version;
mod path;
mod state;

use crate::args::{gather_commandline_args, parse_commandline_args};
use crate::commands::Nur;
use crate::compat::show_nurscripts_hint;
use crate::defaults::{get_default_nur_config, get_default_nur_env};
use crate::engine::init_engine_state;
use crate::errors::NurError;
use crate::names::{
    NUR_ENV_NU_LIB_DIRS, NUR_NAME, NUR_VAR_CONFIG_DIR, NUR_VAR_DEFAULT_LIB_DIR,
    NUR_VAR_PROJECT_PATH, NUR_VAR_RUN_PATH, NUR_VAR_TASK_NAME,
};
use crate::state::NurState;
use engine::NurEngine;
use miette::Result;
use nu_ansi_term::Color;
use nu_cmd_base::util::get_init_cwd;
use nu_protocol::engine::StateWorkingSet;
use nu_protocol::{
    eval_const::create_nu_constant, BufferedReader, PipelineData, RawStream, Record, Span, Type,
    Value, NU_VARIABLE_ID,
};
use std::env;
use std::io::BufReader;
use std::process::ExitCode;

fn main() -> Result<ExitCode, miette::ErrReport> {
    // Initialise nur state
    let run_path = get_init_cwd();
    let nur_state = NurState::new(run_path);

    // Setup nur engine
    let engine_state = init_engine_state(&nur_state.project_path)?;
    let mut nur_engine = NurEngine::from(engine_state);

    // Parse args
    let (args_to_nur, task_name, args_to_task) = gather_commandline_args();
    let parsed_nur_args =
        parse_commandline_args(&args_to_nur.join(" "), &mut nur_engine.engine_state)
            .unwrap_or_else(|_| std::process::exit(1));

    #[cfg(feature = "debug")]
    if parsed_nur_args.debug_output {
        eprintln!("nur args: {:?}", parsed_nur_args);
        eprintln!("task name: {:?}", task_name);
        eprintln!("task args: {:?}", args_to_task);
        eprintln!("run path: {:?}", run_path);
        eprintln!("project path: {:?}", project_path);
    }

    // Show hints for compatibility issues
    if nur_state.has_project_path {
        show_nurscripts_hint(&nur_state.project_path, nur_engine.use_color);
    }

    // Handle execution without project path, only allow to show help, abort otherwise
    if !nur_state.has_project_path {
        if parsed_nur_args.show_help {
            nur_engine.print_help(&Nur);

            std::process::exit(0);
        } else {
            return Err(miette::ErrReport::from(NurError::NurfileNotFound()));
        }
    }

    // Base path for nur config/env
    #[cfg(feature = "debug")]
    if parsed_nur_args.debug_output {
        eprintln!("nur config dir: {:?}", nur_state.config_dir);
    }

    // Set default scripts path
    nur_engine.engine_state.add_env_var(
        NUR_ENV_NU_LIB_DIRS.to_string(),
        Value::test_string(nur_state.lib_dir_path.to_string_lossy()),
    );
    #[cfg(feature = "debug")]
    if parsed_nur_args.debug_output {
        eprintln!("nur lib path (scripts): {:?}", nur_state.lib_dir_path);
    }

    // Set config and env paths to .nur versions
    nur_engine
        .engine_state
        .set_config_path("env-path", nur_state.env_path.clone());
    nur_engine
        .engine_state
        .set_config_path("config-path", nur_state.config_path.clone());

    // Set up the $nu constant before evaluating any files (need to have $nu available in them)
    let nu_const = create_nu_constant(
        &nur_engine.engine_state,
        PipelineData::empty().span().unwrap_or_else(Span::unknown),
    )?;
    nur_engine
        .engine_state
        .set_variable_const_val(NU_VARIABLE_ID, nu_const);

    // Set up the $nur constant record (like $nu)
    let mut nur_record = Record::new();
    nur_record.push(
        NUR_VAR_RUN_PATH,
        Value::string(
            String::from(nur_state.run_path.to_str().unwrap()),
            Span::unknown(),
        ),
    );
    nur_record.push(
        NUR_VAR_PROJECT_PATH,
        Value::string(
            String::from(nur_state.project_path.to_str().unwrap()),
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
            String::from(nur_state.config_dir.to_str().unwrap()),
            Span::unknown(),
        ),
    );
    nur_record.push(
        NUR_VAR_DEFAULT_LIB_DIR,
        Value::string(
            String::from(nur_state.lib_dir_path.to_str().unwrap()),
            Span::unknown(),
        ),
    );
    let mut working_set = StateWorkingSet::new(&nur_engine.engine_state);
    let nur_var_id = working_set.add_variable(
        NUR_NAME.as_bytes().into(),
        Span::unknown(),
        Type::Any,
        false,
    );
    nur_engine
        .stack
        .add_var(nur_var_id, Value::record(nur_record, Span::unknown()));
    nur_engine.engine_state.merge_delta(working_set.render())?;

    // Load env and config
    if nur_state.env_path.exists() {
        nur_engine.source_and_merge_env(&nur_state.env_path, PipelineData::empty())?;
    } else {
        nur_engine.eval_and_merge_env(get_default_nur_env(), PipelineData::empty())?;
    }
    if nur_state.config_path.exists() {
        nur_engine.source_and_merge_env(&nur_state.config_path, PipelineData::empty())?;
    } else {
        nur_engine.eval_and_merge_env(get_default_nur_config(), PipelineData::empty())?;
    }

    // Load task files
    #[cfg(feature = "debug")]
    if parsed_nur_args.debug_output {
        eprintln!("nurfile path: {:?}", nur_state.nurfile_path);
        eprintln!("nurfile local path: {:?}", nur_state.local_nurfile_path);
    }
    if nur_state.nurfile_path.exists() {
        nur_engine.source(nur_state.nurfile_path, PipelineData::empty())?;
    }
    if nur_state.local_nurfile_path.exists() {
        nur_engine.source(nur_state.local_nurfile_path, PipelineData::empty())?;
    }

    // Handle list tasks
    if parsed_nur_args.list_tasks {
        // TODO: Parse and handle commands without eval
        nur_engine.eval_and_print(
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
            nur_engine.print_help(&Nur);
        } else if let Some(command) = nur_engine.get_def(task_def_name) {
            nur_engine.clone().print_help(command);
        } else {
            return Err(miette::ErrReport::from(NurError::TaskNotFound(task_name)));
        }

        std::process::exit(0);
    }

    // Check if requested task exists
    if !nur_engine.has_def(&task_def_name) {
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
        exit_code = nur_engine.eval_and_print(full_task_call, input)?;

        #[cfg(feature = "debug")]
        if parsed_nur_args.debug_output {
            println!("Exit code {:?}", exit_code);
        }
    } else {
        println!("nur version {}", env!("CARGO_PKG_VERSION"));
        println!("Project path {:?}", nur_state.project_path);
        println!("Executing task {}", task_name);
        println!();
        exit_code = nur_engine.eval_and_print(full_task_call, input)?;
        #[cfg(feature = "debug")]
        if parsed_nur_args.debug_output {
            println!("Exit code {:?}", exit_code);
        }
        if exit_code == 0 {
            println!(
                "{}Task execution successful{}",
                if nur_engine.use_color {
                    Color::Green.prefix().to_string()
                } else {
                    String::from("")
                },
                if nur_engine.use_color {
                    Color::Green.suffix().to_string()
                } else {
                    String::from("")
                },
            );
        } else {
            println!(
                "{}Task execution failed{}",
                if nur_engine.use_color {
                    Color::Red.prefix().to_string()
                } else {
                    String::from("")
                },
                if nur_engine.use_color {
                    Color::Red.suffix().to_string()
                } else {
                    String::from("")
                },
            );
        }
    }

    Ok(ExitCode::from(exit_code as u8))
}
