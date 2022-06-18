ArgSplitter - helper library for parsing command line arguments
=============================================================

Helper library for command line argument parsing.

Allows you to conveniently iterate over flags and other options. Does not
provide higher level features such as parsing into structs or automatically
generating help text. Instead, it only provides the following services:

1) Splitting combined single-dash flags such as `-xvf` into separate flags `-x`,
   `-v` and `-f`.

2) Dealing with flags with arguments such as `-fbanana` or `--fruit=banana`.
   The latter may or may not be equivalent with `--fruit banana`.

3) Correctly dealing with non-unicode arguments such as filenames, while
   still working with regular strings wherever possible.
   This is important because both Unix and Windows allow file names which
   cannot be represented as UTF-8 encoded strings.

# Example

The program below either wants to get two arguments `infile` and `outfile`,
or only `outfile` plus a flag `--source <some text>`. It also supports
`--verbose`, and abbreviations `-s` and `-v`.

```rust
use argsplitter::{main_support::handle_argerror, ArgError, ArgSplitter};
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
    handle_argerror(USAGE, ret).unwrap_or(ExitCode::FAILURE)
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
```

# Example output

```
» cargo run -q --example=demo -- -h
-- stdout --
Usage: demo OPTIONS [MESSAGE] OUTFILE
Arguments:
    MESSAGE             Message to write, only if --file not given
    OUTFILE             File to write message to
Options:
    -v --verbose        Be chatty
    -f --file=INFILE	File to read message from, only if MESSAGE not given
    -h --help           Show this help
```

Without any arguments it complains:
```
» cargo run -q --example=twoargs
-- stderr --
Error: missing argument: source or infile
-- exit status 1
```


When given two arguments it is happy:
```
» cargo run -q --example=twoargs left right
-- stdout --
Verbose=false source=Some(File("left")) dest=Some("right")
```

It notices "verbose":
```
» cargo run -q --example=twoargs left -v right
-- stdout --
Verbose=true source=Some(File("left")) dest=Some("right")
```

Instead of `infile` you can pass `-s SOMETHING` or `--source SOMETHING`:
```
» cargo run -q --example=twoargs  right -s some-text
-- stdout --
Verbose=false source=Some(Text("some-text")) dest=Some("right")
```

Superfluous arguments are detected by the call to `args.verify_no_more_stashed()`:
```
» cargo run -q --example=twoargs  right -s some-text extra
-- stderr --
Error: unexpected argument: `extra`
-- exit status 1
```
