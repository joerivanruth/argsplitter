/*!

Helper crate for parsing command line arguments

Crates such as [clap] allow you to create a command-line parser with many bells
and whistles such as automatically parsing the configuration into a struct,
generating help text or even starting with the help text and  deriving the
command-line parser from that.
The downside is that to cover all possibilities the API has to be quite large
and elaborate and you have to do quite some learning, documentation reading and
tinkering to get it to do what you want. Moreover, you forget and then have to
relearn all that whenever you want to change an old project or start a new one.

[clap]: https://docs.rs/clap/

The main aim of the `argsplitter` crate is that it should take only a few minutes
to be productive again when coming back after not using it for a while. It tries
to make it easy to process command-line flags using Rust's `match` statement and
it tries to help correctly deal with arguments that have an invalid Unicode
encoding.

**A note about encodings:** Rust strings are defined to be encoded as UTF-8 but both
Unix and Windows allow file names and command-line arguments that are not Unicode.
For these, Rust provides [`OsString`] which is less convenient to work with than
[`String`] but can represent everything.
In the `argsplitter` API, methods suffixed with `_os` have return
types based on [`OsString`] and the others are based on [`String`].
You can switch back and forth between these variants as required.

# Overview

We distinguish short options, long options and words. Short options start with
a single dash and can be bundled, so `-xvf` is equivalent to `-x -v -f`.
Long options such as `--file` start with two dashes and always contain a single
multi-letter flag. Words are arguments that do not start with a dash. Sometimes
they are standalone arguments and sometimes they are a parameter to a preceding
flag. Long options can also have a parameter attached,
for example `--file=data.csv`.

Parsers made with `argsplitter` crate are not declarative but purely procedural.
First you construct an [`ArgSplitter`] and then you repeatedly call the methods
[`item()`][`ArgSplitter::item`] or [`item_os()`][`ArgSplitter::item_os`],
[`param()`][`ArgSplitter::param`] or [`param_os()`][`ArgSplitter::param_os`],
and [`flag()`][`ArgSplitter::flag`]
to consume options and words from the commandline.

For example,

```
# use std::path::PathBuf;
# use argsplitter::{ArgSplitter, Item};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
// Command line to parse
let mut argsplitter = ArgSplitter::from(["test", "-fdata.csv", "hello"]);
// Where to put the parts
let mut message: Option<String> = None;
let mut file: Option<PathBuf> = None;
// Iterate over the arguments
while let Some(item) = argsplitter.item()? {
    match item {
        // Encoding has already been checked, this is a String:
        Item::Word(w) => message = Some(w),
        // Use param_os() to pick up file names:
        Item::Flag("-f" | "--file") => file = Some(argsplitter.param_os()?.into()),
        // Any other flag is an error
        other => other.unexpected()?,
    }
}
assert_eq!(message, Some("hello".to_string()));
assert_eq!(file, Some(PathBuf::from("data.csv")));
#    Ok(())
# }
```

In this example we repeatedly call [`item()`][`ArgSplitter::item`].
The idea is to immediately propagate the errors with `?`, use `while let Some`
to deal with iteration and the Options, and use pattern matching on the items.
The method returns:

* `Ok(Some(Item::Flag(&str)))` if it finds a flag such as `-f`. Returning the
  flag as a `&str` makes it easier to use the `match` statement on it.

* `Ok(Some(Item::Word(String)))` if it found a word. This is an owned value
  because it will probably be stored somewhere. [`ArgSplitter::item_os()`]
  returns `Ok(Some(ItemOs::Word(OsString)))`.

* `Ok(None)` if the command line has been exhausted.

* `Err(ArgError)` if an error occurred, for example a Unicode problem or
  a flag with an unexpected parameter (`--verbose=data.csv`).

If we encounter a flag which has a parameter, we call either
[`param_os()`][`ArgSplitter::param_os`] or
[`param()`][`ArgSplitter::param`] to pick it up.
If these methods cannot find a parameter attached to the flag they also
look for a word following the flag:

| command line                | result of `.param_os()`                  |
| ---                         | ---                                      |
| `-fdata.csv`                | `data.csv`                               |
| `-f` &nbsp; `data.csv`      | `data.csv`                               |
| `--file=data.csv`           | `data.csv`                               |
| `--file` &nbsp; `data.csv`  | `data.csv`                               |
| `-f` &nbsp; `-x`            | `ArgError::ParameterMissing("-f")`       |
| `--file` &nbsp; `-x`        | `ArgError::ParameterMissing("--file")`   |
| `-f`                        | `ArgError::ParameterMissing("-f")`       |
| `--file`                    | `ArgError::ParameterMissing("--file")`   |

If more control is needed, for example to allow `--file=data.csv` but
not `--file` &nbsp;`data.csv`, the methods [`ArgSplitter::has_param_attached`] and
[`ArgSplitter::at_word`] can be used to check what's behind the flag.

If we wouldn't have called
[`param_os()`][`ArgSplitter::param_os`] or
[`param()`][`ArgSplitter::param`]
after the `-f`, subsequent calls would simply have returned `-d`, `-a`, etc.
If we wouldn't have called it after `--file`, the next call would return an
[`ArgError::UnexpectedParameter`].

# Processing the flags first

 */
use std::ffi::{OsStr, OsString};

pub mod main_support;

mod argerror;
mod core;
mod item;
mod oschars;
mod splitter;

pub use argerror::ArgError;
pub use item::{Item, ItemOs};
pub use splitter::ArgSplitter;

trait ForceUnicode {
    type Forced;

    fn force_unicode(self) -> Result<Self::Forced, ArgError>;
}

impl ForceUnicode for OsString {
    type Forced = String;

    fn force_unicode(self) -> Result<String, ArgError> {
        match self.to_str() {
            Some(s) => Ok(s.to_owned()),
            None => Err(ArgError::InvalidUnicode(self)),
        }
    }
}

impl<'a> ForceUnicode for &'a OsStr {
    type Forced = &'a str;

    fn force_unicode(self) -> Result<&'a str, ArgError> {
        match self.to_str() {
            Some(s) => Ok(s),
            None => Err(ArgError::InvalidUnicode(self.to_owned())),
        }
    }
}

impl<'a> ForceUnicode for ItemOs<'a> {
    type Forced = Item<'a>;

    fn force_unicode(self) -> Result<Self::Forced, ArgError> {
        match self {
            ItemOs::Flag(f) => Ok(Item::Flag(f)),
            ItemOs::Word(w) => Ok(Item::Word(w.force_unicode()?)),
        }
    }
}

impl<T: ForceUnicode> ForceUnicode for Option<T> {
    type Forced = Option<<T as ForceUnicode>::Forced>;

    fn force_unicode(self) -> Result<Self::Forced, ArgError> {
        match self {
            None => Ok(None),
            Some(v) => Ok(Some(v.force_unicode()?)),
        }
    }
}

impl<T: ForceUnicode> ForceUnicode for Result<T, ArgError> {
    type Forced = <T as ForceUnicode>::Forced;

    fn force_unicode(self) -> Result<Self::Forced, ArgError> {
        self.and_then(|v| v.force_unicode())
    }
}
