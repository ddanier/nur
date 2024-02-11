# embed-nu

[![](https://img.shields.io/crates/v/embed-nu?style=for-the-badge)](https://crates.io/crates/embed-nu)
[![](https://img.shields.io/docsrs/embed-nu/latest?style=for-the-badge)](https://docs.rs/embed-nu/)

*embed-nu* can be used to call [nu](https://github.com/nushell/nushell) scripts and expressions
from within your rust application. This crate provides a wrapper around the nu engine to easily build
the nu execution context, parse scripts and call functions. As this crate includes nu as a dependency
calls to nu don't have the overhead of calling an external application. 

## Example Usage

```rust
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

  // print the pipeline of this expression. In this case
  // this pipeline contains the text of the echo expression
  // as it's the last expressin 
  ctx.print_pipeline(pipeline).unwrap();

  // this eval put's the function definition of hello into scope 
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

  // hello can now be called as a function
  ctx.call_fn("hello", [] as [String; 0]).unwrap();
}
```


## Converting data into nu values

This crate uses [rusty-value](https://github.com/Trivernis/rusty-value) to convert any rust
data type into nu values.

```rust
use embed_nu::{rusty_value::*, IntoValue};


// derive rusty value
#[derive(RustyValue)]
struct MyStruct {
    foo: String,
    bar: usize,
}

fn main() {
  let instance = MyStruct {
    foo: String::from("foo"),
    bar: 12
  };
  // convert this struct into a nu value
  // this is also done implicitly when passing the value to the nu context
  // as function arguments or variables
  let value = instance.into_value();
  dbg!(value);
}
```