mod args;
mod commands;
mod compat;
mod engine;
mod errors;
mod names;
mod nu_version;
mod path;
mod scripts;
mod state;

use crate::commands::Nur;
use crate::compat::show_nurscripts_hint;
use crate::engine::init_engine_state;
use crate::errors::NurError;
use crate::names::NUR_NAME;
use crate::scripts::{get_default_nur_config, get_default_nur_env};
use crate::state::NurState;
use engine::NurEngine;
use miette::Result;
use nu_ansi_term::Color;
use nu_cmd_base::util::get_init_cwd;
use nu_protocol::{BufferedReader, PipelineData, RawStream, Span};
use std::env;
use std::io::BufReader;
use std::process::ExitCode;

fn main() -> Result<ExitCode, miette::ErrReport> {
    // Initialise nur state
    let run_path = get_init_cwd();
    let nur_state = NurState::new(run_path);

    // Create raw nu engine state
    let engine_state = init_engine_state(&nur_state.project_path)?;

    // Setup nur engine from engine state
    let mut nur_engine = NurEngine::new(engine_state, nur_state)?;
    let use_color = nur_engine.engine_state.get_config().use_ansi_coloring;

    // Parse args
    let parsed_nur_args = nur_engine.parse_args();

    #[cfg(feature = "debug")]
    if parsed_nur_args.debug_output {
        eprintln!("nur args: {:?}", parsed_nur_args);
        eprintln!("task name: {:?}", nur_engine.state.task_name);
        eprintln!("task args: {:?}", nur_engine.state.args_to_task);
        eprintln!("run path: {:?}", nur_engine.state.run_path);
        eprintln!("project path: {:?}", nur_engine.state.project_path);
    }

    // Show hints for compatibility issues
    if nur_engine.state.has_project_path {
        show_nurscripts_hint(nur_engine.state.project_path.clone(), use_color);
    }

    // Handle execution without project path, only allow to show help, abort otherwise
    if !nur_engine.state.has_project_path {
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
        eprintln!("nur lib path (scripts): {:?}", nur_state.lib_dir_path);
    }

    // Load env and config
    if nur_engine.state.env_path.exists() {
        nur_engine
            .source_and_merge_env(&nur_engine.state.env_path.clone(), PipelineData::empty())?;
    } else {
        nur_engine.eval_and_merge_env(get_default_nur_env(), PipelineData::empty())?;
    }
    if nur_engine.state.config_path.exists() {
        nur_engine
            .source_and_merge_env(&nur_engine.state.config_path.clone(), PipelineData::empty())?;
    } else {
        nur_engine.eval_and_merge_env(get_default_nur_config(), PipelineData::empty())?;
    }

    // Load task files
    #[cfg(feature = "debug")]
    if parsed_nur_args.debug_output {
        eprintln!("nurfile path: {:?}", nur_state.nurfile_path);
        eprintln!("nurfile local path: {:?}", nur_state.local_nurfile_path);
    }
    if nur_engine.state.nurfile_path.exists() {
        nur_engine.source(nur_engine.state.nurfile_path.clone(), PipelineData::empty())?;
    }
    if nur_engine.state.local_nurfile_path.exists() {
        nur_engine.source(
            nur_engine.state.local_nurfile_path.clone(),
            PipelineData::empty(),
        )?;
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
    let task_def_name = format!("{} {}", NUR_NAME, nur_engine.state.task_name);
    #[cfg(feature = "debug")]
    if parsed_nur_args.debug_output {
        eprintln!("task def name: {}", task_def_name);
    }

    // Handle help
    if parsed_nur_args.show_help || nur_engine.state.task_name.is_empty() {
        if nur_engine.state.task_name.is_empty() {
            nur_engine.print_help(&Nur);
        } else if let Some(command) = nur_engine.get_def(task_def_name) {
            nur_engine.clone().print_help(command);
        } else {
            return Err(miette::ErrReport::from(NurError::TaskNotFound(
                nur_engine.state.task_name,
            )));
        }

        std::process::exit(0);
    }

    // Check if requested task exists
    if !nur_engine.has_def(&task_def_name) {
        return Err(miette::ErrReport::from(NurError::TaskNotFound(
            nur_engine.state.task_name,
        )));
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
    let full_task_call = format!(
        "{} {}",
        task_def_name,
        nur_engine.state.args_to_task.join(" ")
    );
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
        println!("Project path {:?}", nur_engine.state.project_path);
        println!("Executing task {}", nur_engine.state.task_name);
        println!();
        exit_code = nur_engine.eval_and_print(full_task_call, input)?;
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
