use embed_nu::{rusty_value::*, IntoValue, NewEmpty};
use embed_nu::{CommandGroupConfig, Context, PipelineData};
use nu_protocol::engine::Command;
use nu_protocol::{Config, Signature, Span, SyntaxShape};

#[test]
fn it_evals_strings() {
    let mut ctx = get_context();
    let pipeline = ctx
        .eval_raw(
            r#"echo "Hello World from this eval""#,
            PipelineData::empty(),
        )
        .unwrap();
    ctx.print_pipeline(pipeline).unwrap()
}

#[test]
fn it_reports_parse_errors() {
    let mut ctx = get_context();
    let eval_result = ctx.eval_raw(r#"let a = 1 | 2 | 3"#, PipelineData::empty());
    assert!(eval_result.is_err());
}

#[test]
fn it_returns_variables() {
    let mut ctx = get_context();
    ctx.eval_raw(r#"let hello = 'world'"#, PipelineData::empty())
        .unwrap();
    let val = ctx.get_var("hello").expect("No variable returned");
    assert_eq!(val.as_string().unwrap(), String::from("world"))
}

#[test]
fn it_accepts_variables() {
    let mut ctx = get_context();
    ctx.add_var("hello", "world").unwrap();

    let val = ctx.get_var("hello").expect("No variable returned");
    assert_eq!(val.as_string().unwrap(), String::from("world"));

    let val = ctx
        .eval_raw(r#"$hello"#, PipelineData::empty())
        .unwrap()
        .into_value(Span::empty());

    assert_eq!(val.as_string().unwrap(), String::from("world"))
}

#[derive(RustyValue)]
struct TestArg {
    foo: String,
    bar: usize,
}

#[test]
fn it_executes_functions() {
    let mut ctx = get_context();
    ctx.eval_raw(
        r#"
    
        def hello [] {
            echo "Hello World from this script";
            echo # dummy echo so I don't have to print the output
        }        
        
    "#,
        PipelineData::empty(),
    )
    .unwrap();
    ctx.call_fn("hello", [] as [String; 0]).unwrap();
    assert!(ctx.has_fn("world") == false);

    let test_arg = TestArg {
        foo: String::from("Hello World"),
        bar: 12,
    };
    let pipeline = ctx.call_fn("echo", [test_arg]).unwrap();
    ctx.print_pipeline(pipeline).unwrap();
}

#[test]
fn it_executes_custom_commands() {
    let mut ctx = get_context();
    let pipeline = ctx
        .eval_raw(r#"custom_upper "Hello world""#, PipelineData::empty())
        .unwrap();
    let string_output = pipeline.collect_string(" ", &Config::default()).unwrap();
    assert_eq!(string_output, String::from("HELLO WORLD"))
}

fn get_context() -> Context {
    Context::builder()
        .with_command_groups(CommandGroupConfig::default().all_groups(true))
        .unwrap()
        .add_command(CustomCommand)
        .unwrap()
        .add_parent_env_vars()
        .build()
        .unwrap()
}

#[derive(Clone)]
struct CustomCommand;

impl Command for CustomCommand {
    fn name(&self) -> &str {
        "custom_upper"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::new("custom_upper")
            .required(
                "text",
                SyntaxShape::String,
                "Text to print in full uppercase.",
            )
            .category(nu_protocol::Category::Experimental)
    }

    fn usage(&self) -> &str {
        "custom_upper <text>"
    }

    fn run(
        &self,
        _engine_state: &nu_protocol::engine::EngineState,
        _stack: &mut nu_protocol::engine::Stack,
        call: &nu_protocol::ast::Call,
        _input: PipelineData,
    ) -> Result<PipelineData, nu_protocol::ShellError> {
        let string_input = call.positional_nth(0).unwrap();
        let string_input = string_input.as_string().unwrap();
        let upper = string_input.to_uppercase();
        println!("{upper}");

        Ok(PipelineData::Value(upper.into_value(), None))
    }
}
