ArgSplitter - helper crate for parsing command line arguments
===============================================================

Helper crate for parsing command line arguments

Crates such as [clap] allow you to create a command-line parser with many bells
and whistles. For example,

* automatically parsing the configuration into a struct,
* generating help text,
* or even starting with the help text and deriving the command-line
  parser from that.

The downside is that to cover all possibilities the API has to be quite large
and elaborate and you have to do quite some learning, documentation reading and
tinkering to get it to do what you want. Moreover, you forget and then have to
relearn all that whenever you want to change an old project or start a new one.

[clap]: https://docs.rs/clap/

The main aim of the `argsplitter` crate is that it should take only a few minutes
to be productive again when coming back after not using it for a while. It tries
to make it easy to process command-line flags using Rust's `match` statement and
it tries to help correctly deal with arguments that have an invalid Unicode
encoding. As such, it only provides the following services:

1) Splitting combined single-dash flags such as `-xvf` into separate flags `-x`,
   `-v` and `-f`.

2) Dealing with flags arguments such as `-fbanana` or `--fruit=banana`.
   The latter may or may not be equivalent with `--fruit banana`.

3) Correctly dealing with non-unicode arguments such as filenames, while
   still working with regular strings wherever possible.
   This is important because both Unix and Windows allow file names which
   cannot be represented as UTF-8 encoded strings.

**A note about encodings:** Item 3) is important because Rust strings are
defined to be encoded as UTF-8 but both Unix and Windows allow file names and
command-line arguments that are not Unicode. For these, Rust provides
[`OsString`] which is less convenient to work with than [`String`] but can
represent everything. In the `argsplitter` API, methods suffixed with `_os` have
return types based on [`OsString`] and the others are based on [`String`]. You
can switch back and forth between these variants as required.


# Overview

We distinguish short options, long options and words. Short options start with
a single dash and can be bundled, so `-xvf` is equivalent to `-x -v -f`.
Long options such as `--file` start with two dashes and always contain a single
multi-letter flag. Words are arguments that do not start with a dash. Sometimes
they are standalone arguments and sometimes they are a parameter to a preceding
flag. Long options can also have a parameter attached,
for example `--file=data.csv`.

Parsers made with `argsplitter` crate are not declarative but purely procedural.
First you construct an `ArgSplitter` and then you repeatedly call the methods
`item()` or `item_os()`, `param()` or `param_os()`, and `flag()` to consume
options and words from the commandline.

# Example

```rust
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
```

# Example output

With -h and --help, the Usage goes to stdout:
```
» send_mail -h
-- stdout --
Usage: send_mail [OPTIONS..] RECIPIENT..
Options:
   -v   --verbose          Describe what's going on
   -a   --attach FILE      Attach this file
   -s   --subject TEXT     Subject: line
   -h   --help             Print this help
```

Without any arguments it complains to stderr:
```
» send_mail
-- stderr --
Error: missing argument: RECIPIENTS
Usage: send_mail [OPTIONS..] RECIPIENT..
Options:
   -v   --verbose          Describe what's going on
   -a   --attach FILE      Attach this file
   -s   --subject TEXT     Subject: line
   -h   --help             Print this help
-- exit status 1
```

Non-flag arguments are recipients, there must be at least 1:
```
» send_mail alice bob
-- stdout --
verbose=false
subject=None
recipients=["alice", "bob"]
attachments=[]
```

It notices "verbose":
```
» send_mail alice -v bob
-- stdout --
verbose=true
subject=None
recipients=["alice", "bob"]
attachments=[]
```

It works correctly if we bundle multiple flags:
```
» send_mail alice -vshello bob
-- stdout --
verbose=true
subject=Some("Hello")
recipients=["alice", "bob"]
attachments=[]
```
