use miette::Diagnostic;
use nu_protocol::{ParseError, ShellError};
use thiserror::Error;

pub type NurResult<T> = Result<T, NurError>;

#[derive(Clone, Debug, Error, Diagnostic)]
pub enum NurError {
    #[error("Init nu error {0}")]
    #[diagnostic()]
    InitError(String),

    #[error("IO Error {0}")]
    #[diagnostic()]
    IoError(String),

    #[error("Shell Error {0}")]
    #[diagnostic()]
    ShellError(#[from] ShellError),

    #[error("Parse Error {0:?}")]
    #[diagnostic()]
    ParseErrors(#[related] Vec<ParseError>),

    #[error("Could not find the task {0}")]
    #[diagnostic()]
    TaskNotFound(String),

    #[error("Could not find nurfile in path and parents")]
    #[diagnostic()]
    NurfileNotFound(),
}

impl From<std::io::Error> for NurError {
    fn from(_value: std::io::Error) -> NurError {
        NurError::IoError(String::from("Could not read file"))
    }
}
