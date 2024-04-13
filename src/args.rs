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
use std::env::Args;

pub(crate) fn gather_commandline_args(args: &mut Args) -> (Vec<String>, String, Vec<String>) {
    let mut args_to_nur = Vec::from([NUR_NAME.into()]);
    let mut task_name = String::new();

    args.next(); // Ignore own name
    #[allow(clippy::while_let_on_iterator)]
    while let Some(arg) = args.next() {
        if !arg.starts_with('-') {
            task_name = arg;
            break;
        }

        // let flag_value = match arg.as_ref() {
        //     // "--some-file" => args.next().map(|a| escape_quote_string(&a)),
        //     _ => None,
        // };

        args_to_nur.push(arg);

        // if let Some(flag_value) = flag_value {
        //     args_to_nur.push(flag_value);
        // }
    }

    let args_to_task = if !task_name.is_empty() {
        args.map(|arg| escape_for_script_arg(&arg)).collect()
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
