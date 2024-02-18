use nu_engine::get_full_help;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{Category, Example, IntoPipelineData, PipelineData, ShellError, Signature, SyntaxShape, Value};
use nu_protocol::ast::Call;

#[derive(Clone)]
pub struct Nur;

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
            .switch(
                "stdin",
                "Attach stdin to nu function call",
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
            .category(Category::Default);

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
