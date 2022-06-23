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

    let mut recipients = vec![argsplitter.stashed("RECIPIENT")?];
    // .collect is inconvenient because we need to get rid of the Result<_,ArgError>.
    for a in argsplitter.stashed_iter() {
        recipients.push(a?);
    }

    println!("Verbose={verbose} recipients={recipients:?} attachments={attachments:?}");
    Ok(())
}
