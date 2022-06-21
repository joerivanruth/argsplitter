//! This module provides helper functions for reporting `ArgError`'s
//! to stderr including Usage information if applicable, and for deciding
//! the processes `ExitCode`.
//!
//! [`ArgError`] and other errors in main.

use std::{error::Error, process::ExitCode};

use crate::ArgError;

/// Determine if an error is or is caused by an [`ArgError`].
pub fn find_argerror<'a>(mut err: &'a (dyn Error + 'static)) -> Option<&'a ArgError> {
    loop {
        let x = err.downcast_ref::<ArgError>();
        match x {
            Some(a) => return Some(a),
            None => match err.source() {
                None => return None,
                Some(e) => err = e,
            },
        }
    }
}

/// Decide the `ExitCode` for an [`ArgError`] and write it to `stderr`.
/// Also write the usage information if that makes
/// sense for this ArgError variant. For example,
/// [`ArgError::ExitSuccessfully`] and [`ArgError::InvalidUnicode`]
/// do not need the usage information.
pub fn report_argerror(usage: &str, argerr: &ArgError) -> ExitCode {
    match argerr {
        ArgError::ExitSuccessfully => ExitCode::SUCCESS,
        ArgError::InvalidUnicode(_) => {
            // To stderr, no Usage info
            eprintln!("Error: {}", argerr);
            ExitCode::FAILURE
        }
        _ => {
            // To stderr, with Usage info
            eprintln!("Error: {}", argerr);
            eprintln!("{}", usage.trim());
            ExitCode::FAILURE
        }
    }
}

///
/// For any `Err(_)`  caused by an ArgError, call [`report_argerror`]. For other
/// errors, print the error and its cause chain.
///
/// Return `ExitCode::SUCCESS` for `Ok(_) | Err(ArgError::ExitSuccessfully)`
/// and `ExitCode::FAILURE` for everything else.
///
/// Maybe we should support stacktraces somehow but that's not implemented yet.
///
#[allow(unused)]
pub fn report_errors<E>(usage: &str, result: Result<(), E>) -> ExitCode
where
    E: AsRef<dyn Error + 'static>,
{
    let error = match result {
        Ok(t) => return ExitCode::SUCCESS,
        Err(e) => e,
    };

    let e = error.as_ref();
    if let Some(ae) = find_argerror(e) {
        return report_argerror(usage, ae);
    }

    // Print the whole source-chain
    let mut cur: &dyn Error = e;
    eprintln!("Error: {}", cur);
    while let Some(e) = cur.source() {
        eprintln!("caused by:");
        eprintln!("    {}", e);
        cur = e;
    }

    ExitCode::FAILURE
}
