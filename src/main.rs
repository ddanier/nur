mod engine;
mod args;
mod execute;
mod errors;
mod path;

use nu_cli::gather_parent_env_vars;
use nu_cmd_base::util::get_init_cwd;
use engine::get_engine_state;
use args::{gather_commandline_args, parse_commandline_args};
use nu_protocol::{Value, Span, NU_VARIABLE_ID, eval_const::create_nu_constant, PipelineData, Spanned};
use nu_std::load_standard_library;
use miette::Result;
use execute::eval;
use crate::args::show_nur_help;
use crate::errors::NurError;
use crate::execute::{has_def, source};
use crate::path::find_project_path;

fn main() -> Result<(), miette::ErrReport> {
    // Get initial current working directory.
    let init_cwd = get_init_cwd();
    let project_path = find_project_path(&init_cwd)?;
    let mut engine_state = get_engine_state();

    // Parse args
    let (args_to_nur, task_name, args_to_task) = gather_commandline_args();
    let parsed_nur_args = parse_commandline_args(&args_to_nur.join(" "), &mut engine_state)
        .unwrap_or_else(|_| std::process::exit(1));

    println!("nur args: {:?}", parsed_nur_args);
    println!("task name: {:?}", task_name);
    println!("task args: {:?}", args_to_task);

    // Set some engine flags
    engine_state.is_interactive = false;
    engine_state.is_login = false;
    engine_state.history_enabled = false;

    // Init config
    // TODO
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

    // Add include path in project
    // TODO
    // if let Some(include_path) = &parsed_nu_cli_args.include_path {
    //     let span = include_path.span;
    //     let vals: Vec<_> = include_path
    //         .item
    //         .split('\x1e') // \x1e is the record separator character (a character that is unlikely to appear in a path)
    //         .map(|x| Value::string(x.trim().to_string(), span))
    //         .collect();
    //
    //     engine_state.add_env_var("NU_LIB_DIRS".into(), Value::list(vals, span));
    // }

    // First, set up env vars as strings only
    gather_parent_env_vars(&mut engine_state, &init_cwd);
    engine_state.add_env_var(
        "NU_VERSION".to_string(),
        Value::string(env!("CARGO_PKG_VERSION"), Span::unknown()),
    );

    // Load std library
    load_standard_library(&mut engine_state)?;

    // Initialize input
    let input = PipelineData::empty();

    // Set up the $nu constant before evaluating config files (need to have $nu available in them)
    let nu_const = create_nu_constant(&engine_state, input.span().unwrap_or_else(Span::unknown))?;
    engine_state.set_variable_const_val(NU_VARIABLE_ID, nu_const);

    let mut stack = nu_protocol::engine::Stack::new();

    // Load task files
    let nurfile_path = project_path.join("nurfile");
    let local_nurfile_path = project_path.join("nurfile.local");
    if nurfile_path.exists() {
        source(
            &mut engine_state,
            &mut stack,
            nurfile_path,
            PipelineData::empty(),
        )?;
    }
    if local_nurfile_path.exists() {
        source(
            &mut engine_state,
            &mut stack,
            local_nurfile_path,
            PipelineData::empty(),
        )?;
    }

    // Handle help
    if parsed_nur_args.show_help {
        if task_name.len() == 0 {
            show_nur_help(&mut engine_state);
        } else {
            eval(
                &mut engine_state,
                &mut stack,
                format!("help nur {}", task_name),
                PipelineData::empty(),
            )?.print(
                &engine_state,
                &mut stack,
                false,
                false,
            )?;
        }

        std::process::exit(0);
    }

    // Handle list tasks
    if parsed_nur_args.list_tasks {
        eval(
            &mut engine_state,
            &mut stack,
            r#"scope commands | where name starts-with "nur " and category == "default" | get name | each { |it| $it | str substring 4.. } | sort"#,
            PipelineData::empty(),
        )?.print(
            &engine_state,
            &mut stack,
            false,
            false,
        )?;

        std::process::exit(0);
    }

    // Execute the task
    let task_def_name = format!("nur {}", task_name);
    if !has_def(&engine_state, &task_def_name) {
        return Err(miette::ErrReport::from(
            NurError::NurTaskNotFound(String::from(task_name))
        ));
    }
    println!("Running task {}", task_name);
    eval(
        &mut engine_state,
        &mut stack,
        &task_def_name,
        PipelineData::empty(),
    )?;

    // TEST: Execute 'ls'
    eval(
        &mut engine_state,
        &mut stack,
        "ls",
        PipelineData::empty(),
    )?.print(
        &engine_state,
        &mut stack,
        false,
        false,
    )?;

    // TEST: Show $nu
    eval(
        &mut engine_state,
        &mut stack,
        "$nu",
        PipelineData::empty(),
    )?.print(
        &engine_state,
        &mut stack,
        false,
        false,
    )?;

    Ok(())
}

