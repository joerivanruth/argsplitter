use std::{error, ffi::OsString};
use std::{ffi::OsStr, fmt};

#[cfg(doc)]
use super::ArgSplitter;
/**
Error type for [`ArgSplitter`].
*/
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArgError {
    /// Argument could not be decoded as valid Unicode.
    InvalidUnicode(OsString),
    /// Returned by [`ArgSplitter::item`] and [`ArgSplitter::item_os`]
    /// if the previous long option has a parameter which has not been
    /// retrieved with [`ArgSplitter::param`], for example `--fruit=banana`.
    UnexpectedParameter(String),
    /// Argument was
    UnexpectedArgument(OsString),
    /// Returned by [`ArgSplitter::param`] and [`ArgSplitter::param_os`]
    /// if no parameter is available, for example on `-f` in  `-f -v`.
    ParameterMissing(String),
    /// if a required argument is missing.
    ArgumentMissing(String),
    /// For use by user code
    ErrorMessage(String),
    /// Not a real error, application should print usage info and exit
    /// succesfully
    ExitSuccessfully,
}

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ArgError::*;
        match self {
            InvalidUnicode(a) => {
                write!(f, "invalid unicode in argument `{}`", a.to_string_lossy())
            }
            UnexpectedParameter(flag) => {
                write!(f, "unexpected parameter for flag `{}`", flag)
            }
            UnexpectedArgument(arg) => {
                write!(f, "unexpected argument: `{}`", arg.to_string_lossy())
            }
            ParameterMissing(flag) => write!(f, "parameter missing for flag `{}`", flag),
            ArgumentMissing(desc) => write!(f, "missing argument: {desc}"),
            ErrorMessage(msg) => write!(f, "{}", msg),
            ExitSuccessfully => {
                write!(f, "no error")
            }
        }
    }
}

impl error::Error for ArgError {}

impl ArgError {
    pub fn message(msg: impl fmt::Display) -> Self {
        ArgError::ErrorMessage(msg.to_string())
    }

    pub fn unknown_flag(flag: &str) -> Self {
        ArgError::ErrorMessage(format!("Unexpected flag: {flag}"))
    }

    pub fn unexpected_argument(arg: impl AsRef<OsStr>) -> Self {
        ArgError::UnexpectedArgument(arg.as_ref().to_owned())
    }

    pub fn exit_successfully() -> Self {
        ArgError::ExitSuccessfully
    }
}
