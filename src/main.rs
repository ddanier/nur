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
            r#"echo "Hello World from this eval""#,
            PipelineData::empty(),
        )
        .unwrap();
}
