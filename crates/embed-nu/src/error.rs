use miette::Diagnostic;
use nu_protocol::{ParseError, ShellError};
use thiserror::Error;

pub type CrateResult<T> = std::result::Result<T, CrateError>;

#[derive(Clone, Debug, Error, Diagnostic)]
pub enum CrateError {
    #[error("Shell Error {0}")]
    #[diagnostic()]
    NuShellError(#[from] ShellError),

    #[error("Parse Error {0:?}")]
    #[diagnostic()]
    NuParseErrors(#[related] Vec<ParseError>),

    #[error("Could not find the function {0}")]
    #[diagnostic()]
    FunctionNotFound(String),
}
