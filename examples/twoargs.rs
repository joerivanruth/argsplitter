use argsplitter::{ArgError, Splitter};
use std::{error::Error, path::PathBuf, process::ExitCode};

#[derive(Debug, PartialEq, Eq)]
enum Source {
    Text(String),
    File(PathBuf),
}

fn main() -> ExitCode {
    match work() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn work() -> Result<(), Box<dyn Error>> {
    let mut verbose = false;
    let mut src: Option<Source> = None;
    let dest: Option<PathBuf>;

    let mut args = Splitter::new();

    // args.flag stashes any non-flag arguments in a buffer
    while let Some(flag) = args.flag()? {
        match flag {
            "-v" | "--verbose" => verbose = true,
            "-s" | "--source" => src = Some(Source::Text(args.param()?)),
            a => return Err(ArgError::unexpected_argument(a))?,
        }
    }

    // args.stashed_os() returns a stashed argument as an Ok(OsString)
    if src.is_none() {
        let arg = args.stashed_os("source or infile")?;
        src = Some(Source::File(arg.into()));
    }
    dest = Some(args.stashed_os("outfile")?.into());
    args.verify_no_more_stashed()?;

    println!("Verbose={verbose} source={src:?} dest={dest:?}");
    Ok(())
}
