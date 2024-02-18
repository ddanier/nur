use miette::Diagnostic;
use nu_protocol::{ParseError, ShellError};
use thiserror::Error;

pub type NurResult<T> = Result<T, NurError>;

#[derive(Clone, Debug, Error, Diagnostic)]
pub enum NurError {
    #[error("IO Error {0}")]
    #[diagnostic()]
    NurIoError(String),

    #[error("Shell Error {0}")]
    #[diagnostic()]
    NurShellError(#[from] ShellError),

    #[error("Parse Error {0:?}")]
    #[diagnostic()]
    NurParseErrors(#[related] Vec<ParseError>),

    #[error("Could not find the task {0}")]
    #[diagnostic()]
    NurTaskNotFound(String),

    #[error("Could not find nurfile in path and parents")]
    #[diagnostic()]
    NurTaskfileNotFound(),
}

impl From<std::io::Error> for NurError {
    fn from(_value: std::io::Error) -> NurError {
        NurError::NurIoError(String::from("Could not read file"))
    }
}
