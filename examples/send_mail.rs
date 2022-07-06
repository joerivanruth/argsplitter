use argsplitter::{main_support, ArgError, ArgSplitter};
use std::{error::Error, path::PathBuf, process::ExitCode};

const USAGE: &str = r###"
Usage: send_mail [OPTIONS..] RECIPIENT..
Options:
   -v   --verbose          Describe what's going on
   -a   --attach FILE      Attach this file
   -s   --subject TEXT     Subject: line
   -h   --help             Print this help
"###;

fn main() -> ExitCode {
    main_support::report_errors(USAGE, work())
}

fn work() -> Result<(), Box<dyn Error>> {
    // To be configured using arguments
    let mut verbose = false;
    let mut subject: Option<String> = None;
    let mut attachments: Vec<PathBuf> = vec![];

    let mut argsplitter = ArgSplitter::from_env();

    // .flag() skips non-flag arguments and stashes them for later use.
    while let Some(flag) = argsplitter.flag()? {
        match flag {
            "-h" | "--help" => {
                println!("{}", USAGE.trim());
                return Err(ArgError::exit_successfully())?;
            }

            "-v" | "--verbose" => verbose = true,

            // subject is a String so we use .param()
            "-s" | "--subject" => subject = Some(argsplitter.param()?),

            // attachment is a file name  so we use .param_os()
            "-a" | "--attach" => attachments.push(argsplitter.param_os()?.into()),

            flag => return Err(ArgError::unknown_flag(flag))?,
        }
    }

    // Pick up the recipients stashed by .flag().
    // The first argument states the minimum number that must be present.
    // The second argument is used in the error messages.
    let recipients: Result<Vec<_>, _> = argsplitter.stashed_args(1, "RECIPIENTS").collect();
    // Handle ArgError::ArgumentMissing and ArgError::InvalidUnicode
    let recipients = recipients?;

    println!("verbose={verbose}");
    println!("subject={subject:?}");
    println!("recipients={recipients:?}");
    println!("attachments={attachments:?}");
    Ok(())
}
