use embed_nu::{rusty_value::*, CommandGroupConfig, Context, NewEmpty, PipelineData};

fn main() {
    let mut ctx = Context::builder()
        .with_command_groups(CommandGroupConfig::default().all_groups(true))
        .unwrap()
        .add_parent_env_vars()
        .build()
        .unwrap();

    // eval a nu expression
    let pipeline = ctx
        .eval_raw(
            "date now",
            PipelineData::empty(),
        )
        .unwrap();
    ctx.print_pipeline(pipeline).unwrap();

    // this eval put's the function definition of hello into scope
    let pipeline = ctx.eval_raw(
        r#"
            def "hello world" [] {
                print "Hello World as separet def";
            }

            def hello [] {
                print "Hello World from this script";
            }
        "#,
        PipelineData::empty(),
    )
        .unwrap();
    ctx.print_pipeline(pipeline).unwrap();

    // hello can now be called as a function
    let pipeline = ctx.call_fn("hello world", [] as [String; 0]).unwrap();
    ctx.print_pipeline(pipeline).unwrap();
}
