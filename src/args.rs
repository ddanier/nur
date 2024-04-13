use crate::commands::Nur;
use crate::names::NUR_NAME;
use nu_engine::{get_full_help, CallExt};
use nu_parser::escape_for_script_arg;
use nu_parser::parse;
use nu_protocol::report_error;
use nu_protocol::{
    ast::Expr,
    engine::{Command, EngineState, Stack, StateWorkingSet},
    ShellError,
};
use nu_utils::stdout_write_all_and_flush;

pub(crate) fn gather_commandline_args(args: Vec<String>) -> (Vec<String>, String, Vec<String>) {
    let mut args_to_nur = Vec::from([String::from(NUR_NAME)]);
    let mut task_name = String::new();
    let mut args_iter = args.iter();

    args_iter.next(); // Ignore own name
    #[allow(clippy::while_let_on_iterator)]
    while let Some(arg) = args_iter.next() {
        if !arg.starts_with('-') {
            task_name = arg.clone();
            break;
        }

        // let flag_value = match arg.as_ref() {
        //     // "--some-file" => args.next().map(|a| escape_quote_string(&a)),
        //     _ => None,
        // };

        args_to_nur.push(arg.clone());

        // if let Some(flag_value) = flag_value {
        //     args_to_nur.push(flag_value);
        // }
    }

    let args_to_task = if !task_name.is_empty() {
        args_iter.map(|arg| escape_for_script_arg(arg)).collect()
    } else {
        Vec::default()
    };
    (args_to_nur, task_name, args_to_task)
}

pub(crate) fn parse_commandline_args(
    commandline_args: &str,
    engine_state: &mut EngineState,
) -> Result<NurArgs, ShellError> {
    let (block, delta) = {
        let mut working_set = StateWorkingSet::new(engine_state);

        let output = parse(&mut working_set, None, commandline_args.as_bytes(), false);
        if let Some(err) = working_set.parse_errors.first() {
            report_error(&working_set, err);

            std::process::exit(1);
        }

        (output, working_set.render())
    };

    engine_state.merge_delta(delta)?;

    let mut stack = Stack::new();

    // We should have a successful parse now
    if let Some(pipeline) = block.pipelines.first() {
        if let Some(Expr::Call(call)) = pipeline.elements.first().map(|e| &e.expr.expr) {
            // let config_file = call.get_flag_expr("config");
            let list_tasks = call.has_flag(engine_state, &mut stack, "list")?;
            let quiet_execution = call.has_flag(engine_state, &mut stack, "quiet")?;
            let attach_stdin = call.has_flag(engine_state, &mut stack, "stdin")?;
            let show_help = call.has_flag(engine_state, &mut stack, "help")?;
            #[cfg(feature = "debug")]
            let debug_output = call.has_flag(engine_state, &mut stack, "debug")?;

            if call.has_flag(engine_state, &mut stack, "version")? {
                let version = env!("CARGO_PKG_VERSION").to_string();
                let _ = std::panic::catch_unwind(move || {
                    stdout_write_all_and_flush(format!("{version}\n"))
                });

                std::process::exit(0);
            }

            return Ok(NurArgs {
                list_tasks,
                quiet_execution,
                attach_stdin,
                show_help,
                #[cfg(feature = "debug")]
                debug_output,
            });
        }
    }

    // Just give the help and exit if the above fails
    let full_help = get_full_help(
        &Nur.signature(),
        &Nur.examples(),
        engine_state,
        &mut stack,
        true,
    );
    print!("{full_help}");
    std::process::exit(1);
}

#[derive(Debug, Clone)]
pub(crate) struct NurArgs {
    pub(crate) list_tasks: bool,
    pub(crate) quiet_execution: bool,
    pub(crate) attach_stdin: bool,
    pub(crate) show_help: bool,
    #[cfg(feature = "debug")]
    pub(crate) debug_output: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::create_nur_context;
    use crate::engine::init_engine_state;
    use nu_cmd_base::util::get_init_cwd;

    #[test]
    fn test_gather_commandline_args_splits_on_task_name() {
        let args = vec![
            String::from("nur"),
            String::from("--quiet"),
            String::from("some_task_name"),
            String::from("--task-option"),
            String::from("task-value"),
        ];
        let (nur_args, task_name, task_args) = gather_commandline_args(args);
        assert_eq!(nur_args, vec![String::from("nur"), String::from("--quiet")]);
        assert_eq!(task_name, "some_task_name");
        assert_eq!(
            task_args,
            vec![String::from("--task-option"), String::from("task-value")]
        );
    }

    #[test]
    fn test_gather_commandline_args_handles_missing_nur_args() {
        let args = vec![
            String::from("nur"),
            String::from("some_task_name"),
            String::from("--task-option"),
            String::from("task-value"),
        ];
        let (nur_args, task_name, task_args) = gather_commandline_args(args);
        assert_eq!(nur_args, vec![String::from("nur")]);
        assert_eq!(task_name, "some_task_name");
        assert_eq!(
            task_args,
            vec![String::from("--task-option"), String::from("task-value")]
        );
    }

    #[test]
    fn test_gather_commandline_args_handles_missing_task_name() {
        let args = vec![String::from("nur"), String::from("--help")];
        let (nur_args, task_name, task_args) = gather_commandline_args(args);
        assert_eq!(nur_args, vec![String::from("nur"), String::from("--help")]);
        assert_eq!(task_name, "");
        assert_eq!(task_args, vec![] as Vec<String>);
    }

    #[test]
    fn test_gather_commandline_args_handles_missing_task_args() {
        let args = vec![
            String::from("nur"),
            String::from("--quiet"),
            String::from("some_task_name"),
        ];
        let (nur_args, task_name, task_args) = gather_commandline_args(args);
        assert_eq!(nur_args, vec![String::from("nur"), String::from("--quiet")]);
        assert_eq!(task_name, "some_task_name");
        assert_eq!(task_args, vec![] as Vec<String>);
    }

    #[test]
    fn test_gather_commandline_args_handles_no_args_at_all() {
        let args = vec![String::from("nur")];
        let (nur_args, task_name, task_args) = gather_commandline_args(args);
        assert_eq!(nur_args, vec![String::from("nur")]);
        assert_eq!(task_name, "");
        assert_eq!(task_args, vec![] as Vec<String>);
    }

    fn _create_minimal_engine_for_erg_parsing() -> EngineState {
        let init_cwd = get_init_cwd();
        let engine_state = init_engine_state(&init_cwd).unwrap();
        let engine_state = create_nur_context(engine_state);

        engine_state
    }

    #[test]
    fn test_parse_commandline_args_without_args() {
        let mut engine_state = _create_minimal_engine_for_erg_parsing();

        let nur_args = parse_commandline_args("nur", &mut engine_state).unwrap();
        assert_eq!(nur_args.list_tasks, false);
        assert_eq!(nur_args.quiet_execution, false);
        assert_eq!(nur_args.attach_stdin, false);
        assert_eq!(nur_args.show_help, false);
    }

    #[test]
    fn test_parse_commandline_args_list() {
        let mut engine_state = _create_minimal_engine_for_erg_parsing();

        let nur_args = parse_commandline_args("nur --list", &mut engine_state).unwrap();
        assert_eq!(nur_args.list_tasks, true);
        assert_eq!(nur_args.quiet_execution, false);
        assert_eq!(nur_args.attach_stdin, false);
        assert_eq!(nur_args.show_help, false);
    }

    #[test]
    fn test_parse_commandline_args_quiet() {
        let mut engine_state = _create_minimal_engine_for_erg_parsing();

        let nur_args = parse_commandline_args("nur --quiet", &mut engine_state).unwrap();
        assert_eq!(nur_args.list_tasks, false);
        assert_eq!(nur_args.quiet_execution, true);
        assert_eq!(nur_args.attach_stdin, false);
        assert_eq!(nur_args.show_help, false);
    }

    #[test]
    fn test_parse_commandline_args_stdin() {
        let mut engine_state = _create_minimal_engine_for_erg_parsing();

        let nur_args = parse_commandline_args("nur --stdin", &mut engine_state).unwrap();
        assert_eq!(nur_args.list_tasks, false);
        assert_eq!(nur_args.quiet_execution, false);
        assert_eq!(nur_args.attach_stdin, true);
        assert_eq!(nur_args.show_help, false);
    }

    #[test]
    fn test_parse_commandline_args_help() {
        let mut engine_state = _create_minimal_engine_for_erg_parsing();

        let nur_args = parse_commandline_args("nur --help", &mut engine_state).unwrap();
        assert_eq!(nur_args.list_tasks, false);
        assert_eq!(nur_args.quiet_execution, false);
        assert_eq!(nur_args.attach_stdin, false);
        assert_eq!(nur_args.show_help, true);
    }
}
