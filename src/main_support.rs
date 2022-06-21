use std::{borrow::Borrow, error::Error, process::ExitCode};

use crate::ArgError;

pub fn find_argerror(mut err: &(dyn Error + 'static)) -> Option<ArgError> {
    loop {
        let x = err.downcast_ref::<ArgError>();
        match x {
            Some(a) => return Some(a.clone()),
            None => match err.source() {
                None => return None,
                Some(e) => err = e,
            },
        }
    }
}

pub fn handle_argerror<E>(usage: &str, result: Result<(), E>) -> Result<ExitCode, E>
where
    E: Borrow<dyn Error> + 'static,
{
    let err = match result {
        Ok(_) => return Ok(ExitCode::SUCCESS),
        Err(e) => e,
    };
    let borrowed = err.borrow();
    match find_argerror(borrowed) {
        Some(ArgError::ExitSuccessfully) => Ok(ExitCode::SUCCESS),
        Some(argerr @ ArgError::InvalidUnicode(_)) => {
            // To stderr, no Usage info
            eprintln!("Error: {}", argerr);
            Ok(ExitCode::FAILURE)
        }
        Some(argerr) => {
            // To stderr, with Usage info
            eprintln!("Error: {}", argerr);
            eprintln!("{}", usage.trim());
            Ok(ExitCode::FAILURE)
        }
        None => Err(err),
    }
}

pub fn print_any_errors<E>(usage: &str, result: Result<(), E>) -> ExitCode
where
    E: Borrow<dyn Error> + 'static,
{
    match handle_argerror(usage, result) {
        Ok(code) => code,
        Err(err) => {
            // Print the whole source-chain
            let mut cur: &dyn Error = err.borrow();
            eprintln!("Error: {}", cur);
            while let Some(e) = cur.source() {
                eprintln!("caused by:");
                eprintln!("    {}", e);
                cur = e;
            }
            ExitCode::FAILURE
        }
    }
}
