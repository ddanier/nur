use nu_engine::{get_full_help, CallExt};
use nu_parser::parse;
use nu_parser::{escape_for_script_arg, escape_quote_string};
use nu_protocol::report_error;
use nu_protocol::{
    ast::{Call, Expr, Expression, PipelineElement},
    engine::{Command, EngineState, Stack, StateWorkingSet},
    Category, Example, IntoPipelineData, PipelineData, ShellError, Signature, Spanned, SyntaxShape,
    Value,
};
use nu_utils::stdout_write_all_and_flush;

pub(crate) fn gather_commandline_args() -> (Vec<String>, String, Vec<String>) {
    // Would be nice if we had a way to parse this. The first flags we see will be going to nushell
    // then it'll be the script name
    // then the args to the script
    let mut args_to_nur = Vec::from(["nur".into()]);
    let mut task_name = String::new();
    let mut args = std::env::args();

    args.next();  // Ignore own name
    while let Some(arg) = args.next() {
        if !arg.starts_with('-') {
            task_name = arg;
            break;
        }

        let flag_value = match arg.as_ref() {
            "--config" => args.next().map(|a| escape_quote_string(&a)),
            "--list" | "--quiet" => None,
            _ => None,
        };

        args_to_nur.push(arg);

        if let Some(flag_value) = flag_value {
            args_to_nur.push(flag_value);
        }
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
) -> Result<NurCliArgs, ShellError> {
    let (block, delta) = {
        let mut working_set = StateWorkingSet::new(engine_state);
        working_set.add_decl(Box::new(Nur));

        let output = parse(&mut working_set, None, commandline_args.as_bytes(), false);
        if let Some(err) = working_set.parse_errors.first() {
            report_error(&working_set, err);

            std::process::exit(1);
        }

        working_set.hide_decl(b"nur");
        (output, working_set.render())
    };

    engine_state.merge_delta(delta)?;

    let mut stack = Stack::new();

    // We should have a successful parse now
    if let Some(pipeline) = block.pipelines.first() {
        if let Some(PipelineElement::Expression(
                        _,
                        Expression {
                            expr: Expr::Call(call),
                            ..
                        },
                    )) = pipeline.elements.first()
        {
            let config_file = call.get_flag_expr("config");
            let list_tasks = call.has_flag(engine_state, &mut stack, "list")?;
            let quiet_execution = call.has_flag(engine_state, &mut stack, "quiet")?;
            let show_help = call.has_flag(engine_state, &mut stack, "help")?;

            fn extract_contents(
                expression: Option<&Expression>,
            ) -> Result<Option<Spanned<String>>, ShellError> {
                if let Some(expr) = expression {
                    let str = expr.as_string();
                    if let Some(str) = str {
                        Ok(Some(Spanned {
                            item: str,
                            span: expr.span,
                        }))
                    } else {
                        Err(ShellError::TypeMismatch {
                            err_message: "string".into(),
                            span: expr.span,
                        })
                    }
                } else {
                    Ok(None)
                }
            }

            let config_file = extract_contents(config_file)?;

            if call.has_flag(engine_state, &mut stack, "version")? {
                let version = env!("CARGO_PKG_VERSION").to_string();
                let _ = std::panic::catch_unwind(move || {
                    stdout_write_all_and_flush(format!("{version}\n"))
                });

                std::process::exit(0);
            }

            return Ok(NurCliArgs {
                config_file,
                list_tasks,
                quiet_execution,
                show_help,
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

pub(crate) fn show_nur_help(
    engine_state: &mut EngineState,
) {
    let mut stack = Stack::new();

    let full_help = get_full_help(
        &Nur.signature(),
        &Nur.examples(),
        &engine_state,
        &mut stack,
        true,
    );

    let _ = std::panic::catch_unwind(move || stdout_write_all_and_flush(full_help));
}

#[derive(Debug)]
pub(crate) struct NurCliArgs {
    pub(crate) config_file: Option<Spanned<String>>,
    pub(crate) list_tasks: bool,
    pub(crate) quiet_execution: bool,
    pub(crate) show_help: bool,
}

#[derive(Clone)]
struct Nur;

impl Command for Nur {
    fn name(&self) -> &str {
        "nur"
    }

    fn signature(&self) -> Signature {
        let signature = Signature::build("nur")
            .usage("nu run - simple task runner.")
            .named(
                "config",
                SyntaxShape::String,
                "path to config",
                None,
            )
            .switch(
                "version",
                "output version number and exit",
                None,
            )
            .switch(
                "list",
                "list available tasks and then just exit",
                None,
            )
            .switch(
                "quiet",
                "Do not output anything but what tasks produce",
                None,
            )
            .optional(
                "task name",
                SyntaxShape::Filepath,
                "name of the task to run",
            )
            .rest(
                "task args",
                SyntaxShape::String,
                "parameters to the executed task",
            )
            .category(Category::System);

        signature
    }

    fn usage(&self) -> &str {
        "nu run - simple task runner."
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        Ok(Value::string(
            get_full_help(&Nur.signature(), &Nur.examples(), engine_state, stack, true),
            call.head,
        )
            .into_pipeline_data())
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Execute a task",
                example: "nur task-name",
                result: None,
            },
            Example {
                description: "List available tasks",
                example: "nur --list",
                result: None,
            },
        ]
    }
}

