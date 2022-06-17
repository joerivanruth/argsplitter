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
```

Without any arguments it complains:
```
» cargo run --example=twoargs
Error: missing argument: source or infile
```

When given two arguments it is happy:
```
»  cargo run --example=twoargs left right
Verbose=false source=Some(File("left")) dest=Some("right")
```

It notices "verbose":
```
» cargo run --example=twoargs left -v right
Verbose=true source=Some(File("left")) dest=Some("right")
```

Instead of `infile` you can pass `-s SOMETHING` or `--source SOMETHING`:
```
» cargo run --example=twoargs  right -s some-text
Verbose=false source=Some(Text("some-text")) dest=Some("right")
```

Superfluous arguments are detected by the call to `args.verify_no_more_stashed()`:
```
» cargo run --example=twoargs  right -s some-text extra
Error: unexpected argument: `extra`
»  cargo run --example=twoargs  right -s some-text -x
Error: unexpected argument: `-x`
»  cargo run --example=twoargs  right -s some-text --verbose=please
Error: unexpected parameter for flag `--verbose`
```
