use nu_engine::get_full_help;
use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, IntoPipelineData, PipelineData, ShellError, Signature, SyntaxShape, Value,
};

#[derive(Clone)]
pub(crate) struct Nur;

impl Command for Nur {
    fn name(&self) -> &str {
        "nur"
    }

    fn signature(&self) -> Signature {
        let mut signature = Signature::build("nur");

        signature = signature
            .usage("nur - a taskrunner based on nu shell.")
            .switch("version", "Output version number and exit", Some('v'))
            .switch("list", "List available tasks and then just exit", Some('l'))
            .switch(
                "quiet",
                "Do not output anything but what the task produces",
                None,
            )
            .switch("stdin", "Attach stdin to called nur task", None)
            .named(
                "commands",
                SyntaxShape::String,
                "run the given commands and then exit",
                Some('c'),
            )
            .switch(
                "enter-shell",
                "enter nu shell with nur being setup (use this for debugging)",
                Some('e'),
            )
            .optional(
                "task name",
                SyntaxShape::String,
                "Name of the task to run (you may use sub tasks)",
            )
            .rest(
                "task args",
                SyntaxShape::String,
                "Parameters for the executed task",
            )
            .category(Category::Default);

        #[cfg(feature = "debug")]
        {
            signature = signature.switch("debug", "Show debug details", Some('d'));
        }

        signature
    }

    fn usage(&self) -> &str {
        "nur - a taskrunner based on nu shell."
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
