use argsplitter::{main_support, ArgError, ArgSplitter};
use std::{error::Error, path::PathBuf, process::ExitCode};

const USAGE: &str = r###"
Usage: demo OPTIONS [MESSAGE] OUTFILE
Arguments:
    MESSAGE             Message to write, only if --file not given
    OUTFILE             File to write message to
Options:
    -v --verbose        Be chatty
    -f --file=INFILE	File to read message from, only if MESSAGE not given
    -h --help           Show this help
"###;

fn main() -> ExitCode {
    let ret = main_program();
    main_support::report_errors(USAGE, ret)
}

#[derive(Debug)]
enum Source {
    Str(String),
    File(PathBuf),
}

fn main_program() -> Result<(), Box<dyn Error>> {
    let mut verbose = false;
    let mut source: Option<Source> = None;
    let dest: PathBuf;

    let mut argsplitter = ArgSplitter::new();

    while let Some(f) = argsplitter.flag()? {
        match f {
            "-h" | "help" => {
                // to stdout
                println!("{}", USAGE.trim());
                return Err(ArgError::ExitSuccessfully)?;
            }
            "-v" | "--verbose" => verbose = true,
            "-f" | "--file" => source = Some(Source::File(argsplitter.param_os()?.into())),
            f => return Err(ArgError::unknown_flag(f))?,
        }
    }

    if source.is_none() {
        let msg = argsplitter.stashed("MESSAGE")?;
        source = Some(Source::Str(msg));
    }
    dest = argsplitter.stashed_os("OUTFILE")?.into();
    argsplitter.verify_no_more_stashed()?;

    println!("Hello! verbose={verbose} source={source:?} dest={dest:?}");
    Ok(())
}
