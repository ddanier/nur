use embed_nu::IntoValue;
use rusty_value::*;
use std::{mem, path::PathBuf};

#[derive(RustyValue, Debug, Clone)]
pub struct TestStruct {
    foo: String,
    bar: Vec<String>,
    baz: TestEnum,
    path: PathBuf,
}

#[derive(RustyValue, Debug, Clone)]
pub enum TestEnum {
    Empty,
    Unnamed(String),
    Named { foo: usize, bar: Box<TestStruct> },
}

impl TestStruct {
    pub fn new_test() -> Self {
        Self {
            foo: "Hello".to_string(),
            bar: vec!["One".to_string(), "Two".to_string()],
            baz: TestEnum::Named {
                foo: 12,
                bar: Box::new(TestStruct {
                    foo: "World".to_string(),
                    bar: vec![],
                    baz: TestEnum::Empty,
                    path: PathBuf::from("/tmp"),
                }),
            },
            path: PathBuf::from("/"),
        }
    }
}

#[test]
fn it_creates_values_from_structs() {
    let test_val = TestStruct::new_test();
    dbg!(mem::size_of::<TestStruct>());
    dbg!(&test_val);
    let rusty_val = test_val.clone().into_rusty_value();
    dbg!(mem::size_of::<rusty_value::Value>());
    dbg!(&rusty_val);
    let val = test_val.into_value();
    dbg!(mem::size_of::<Value>());
    dbg!(&val);

    assert!(val.as_record().is_ok())
}
