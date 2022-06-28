use argsplitter::{main_support, ArgError, ArgSplitter};
use std::{error::Error, path::PathBuf, process::ExitCode};

const USAGE: &str = r###"
Usage: send_mail [OPTIONS..] RECIPIENT..
Options:
   -v   --verbose       Describe what's going on
   -a   --attach FILE   Attach this file
   -h   --help          Print this help
"###;

fn main() -> ExitCode {
    main_support::report_errors(USAGE, work())
}

fn work() -> Result<(), Box<dyn Error>> {
    let mut attachments: Vec<PathBuf> = vec![];
    let mut verbose = false;

    let mut argsplitter = ArgSplitter::new();

    while let Some(flag) = argsplitter.flag()? {
        match flag {
            "-h" | "--help" => {
                println!("{}", USAGE.trim());
                return Err(ArgError::exit_successfully())?;
            }
            "-v" | "--verbose" => verbose = true,
            "-a" | "--attach" => attachments.push(argsplitter.param_os()?.into()),
            _ => return Err(ArgError::unknown_flag(flag))?,
        }
    }

    // this is how you pick up stashed_args
    let recipients = argsplitter
        .clone()
        .stashed_args(1, "RECIPIENTS")
        .collect::<Result<Vec<_>, _>>()?;

    // this is how you pick up stashed_args_os
    let x = argsplitter
        .stashed_args_os(1, "RECIPIENTS")?
        .collect::<Vec<_>>();
    let _ = x;

    println!("Verbose={verbose} recipients={recipients:?} attachments={attachments:?}");
    Ok(())
}
