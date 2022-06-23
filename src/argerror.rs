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
    /// Not a real error, useful when handling `--help`. Returned when the
    /// application has already printed usage info and should exit succesfully.
    /// The helper functions in module [`main_support`][`crate::main_support`]
    /// check for this and turn `Err(ArgError::ExitSuccesfully)` into
    /// `Ok(ExitCode::SUCCESS)`.
    ExitSuccessfully,

    /// An argument could not be decoded as valid Unicode.
    InvalidUnicode(OsString),

    /// Returned, usually through [`Item::unexpected`][`crate::Item::unexpected`]
    /// or [`ItemOs::unexpected`][`crate::ItemOs::unexpected`],
    /// when user code does not recognize a given flag.
    UnexpectedFlag(String),

    /// Returned by [`ArgSplitter::no_more_stashed`]
    /// if a stashed argument was found when no more arguments were expected.
    UnexpectedArgument(OsString),

    /// Returned by [`ArgSplitter::item`] and [`ArgSplitter::item_os`]
    /// if the previous long option had a parameter which has not been
    /// retrieved with [`ArgSplitter::param`], for example `--fruit=banana`.
    UnexpectedParameter(String),

    /// Returned by [`ArgSplitter::param`] and [`ArgSplitter::param_os`]
    /// if no parameter is available.
    ParameterMissing(String),

    /// Returned by [`ArgSplitter::stashed`] and [`ArgSplitter::stashed_os`]
    /// when another argument was requested but none is available.
    ArgumentMissing(String),

    /// For use by user code, usually through [`ArgError::message`].
    ErrorMessage(String),
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
            UnexpectedFlag(flag) => {
                write!(f, "unexpected flag: `{}`", flag)
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
    /// Create an [`ArgError::ErrorMessage`].
    pub fn message(msg: impl fmt::Display) -> Self {
        ArgError::ErrorMessage(msg.to_string())
    }

    /// Create an [`ArgError::UnexpectedFlag`].
    pub fn unknown_flag(flag: &str) -> Self {
        ArgError::UnexpectedFlag(flag.to_owned())
    }

    /// Create an [`ArgError::UnexpectedArgument`].
    pub fn unexpected_argument(arg: impl AsRef<OsStr>) -> Self {
        ArgError::UnexpectedArgument(arg.as_ref().to_owned())
    }

    /// Create an [`ArgError::ExitSuccessfully`].
    pub fn exit_successfully() -> Self {
        ArgError::ExitSuccessfully
    }
}
