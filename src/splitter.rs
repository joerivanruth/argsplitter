use std::{
    env,
    ffi::{OsStr, OsString},
};

use crate::{core::Core, item::OwnedItem, ArgError, ForceUnicode, Item, ItemOs};

type AResult<T> = Result<T, ArgError>;

/// Use type to parse your command line arguments.
#[derive(Debug, Clone)]
pub struct ArgSplitter {
    argv0: Option<OsString>,
    core: Core,
    last_flag: Option<String>,
    stashed_args: Vec<OsString>,
}

impl ArgSplitter {
    /// Create an [`ArgSplitter`] with the arguments from [`std::env::args_os`].
    /// The first argument is assumed to be the program name and will be available
    /// through [`ArgSplitter::argv0`], the rest are arguments and can be accessed
    /// through [`ArgSplitter::item`], [`ArgSplitter::item_os`] and [`ArgSplitter::flag`].
    #[allow(clippy::new_without_default)]
    pub fn from_env() -> Self {
        Self::from(env::args_os())
    }

    /// Create an [`ArgSplitter`] from the given argument list.
    /// The first argument is assumed to be the program name and will be available
    /// through [`ArgSplitter::argv0`], the rest are arguments and can be accessed
    /// through [`ArgSplitter::item`], [`ArgSplitter::item_os`] and [`ArgSplitter::flag`].
    pub fn from<S: AsRef<OsStr>>(argv: impl IntoIterator<Item = S>) -> Self {
        let mut args = argv.into_iter().map(|s| s.as_ref().to_owned());
        let argv0 = args.next();
        let core = Core::new(args.collect());
        ArgSplitter {
            argv0,
            core,
            last_flag: None,
            stashed_args: vec![],
        }
    }

    fn flag_ref(&self) -> &str {
        self.last_flag.as_ref().unwrap().as_str()
    }
}

impl ArgSplitter {
    /// Retrieve the very first item in the argument list, which is generally
    /// the program name. Note that this value is set by the parent process and
    /// can be absent or plain wrong.
    pub fn argv0(&self) -> Option<&OsStr> {
        self.argv0.as_deref()
    }

    /// Retrieve the next item on the command line as an [`ItemOs`]. Bundles of
    /// single-letter arguments such as `-xvf` are split into separate items
    /// `-x`, `-v` and `-f`. This method uses [`OsString`] for word arguments so
    /// every file name can be represented. Use [`ArgSplitter::item`] if you
    /// only care for arguments that are properly encoded.
    pub fn item_os(&mut self) -> AResult<Option<ItemOs>> {
        self.last_flag = None;

        let owned_item = match self.core.take_item()? {
            Some(i) => i,
            None => return Ok(None),
        };

        let itemos = match owned_item {
            OwnedItem::Flag(s) => {
                self.last_flag = Some(s);
                ItemOs::Flag(self.flag_ref())
            }
            OwnedItem::Word(w) => ItemOs::Word(w),
        };

        Ok(Some(itemos))
    }

    /// Retrieve the next item on the command line as an [`Item`].
    /// Bundles of single-letter arguments such as `-xvf` are split into
    /// separate items `-x`, `-v` and `-f`.
    /// This method uses [`String`] for word arguments. Invalidly encoded
    /// arguments will cause an [`ArgError::InvalidUnicode`].
    /// Use [`ArgSplitter::item_os`] if you also want to accept badly encoded
    /// arguments.
    pub fn item(&mut self) -> AResult<Option<Item>> {
        self.item_os().force_unicode()
    }

    /// Return `true` if and only if the parser is currently between arguments,
    /// that is, not in the middle of a bundle (`-xvf`) or between a long
    /// option and its parameter (`--file=data.csv`).
    pub fn at_word(&self) -> bool {
        self.core.at_word()
    }

    /// Return `true` if and only if the item most recently returned by
    /// [`item_os`][`ArgSplitter::item_os`],
    /// [`item`][`ArgSplitter::item`] or
    /// [`flag`][`ArgSplitter::flag`]
    /// was a flag and if a parameter is attached.
    /// In the case of a long flag that means `--file=data.csv` but not
    /// `--file data.csv`. In the case of a short flag it means
    /// `-fdata.csv` but not `-f data.csv`.
    pub fn has_param_attached(&self) -> bool {
        self.core.param_ready()
    }

    /// If the item most recently returned by
    /// [`item_os`][`ArgSplitter::item_os`],
    /// [`item`][`ArgSplitter::item`] or
    /// [`flag`][`ArgSplitter::flag`]
    /// was a flag, return its parameter as an [`OsString`].
    /// If the flag had a parameter attached (see [`has_param_attached`][`ArgSplitter::has_param_attached`]),
    /// return that parameter. Otherwise, if the flag is followed by a word, return
    /// that word. If no more arguments follow or if the next argument is another
    /// flag, return [`ArgError::ParameterMissing`].
    pub fn param_os(&mut self) -> AResult<OsString> {
        assert!(
            self.last_flag.is_some(),
            "only call .parm_os() after .take_item() returned a flag"
        );

        if self.core.param_ready() {
            Ok(self.core.param().unwrap())
        } else if self.core.at_word() {
            let it = self.core.take_item().unwrap().unwrap();
            if let OwnedItem::Word(w) = it {
                Ok(w)
            } else {
                panic!("at_word() inconsistent with take_item()");
            }
        } else {
            Err(ArgError::ParameterMissing(self.flag_ref().to_owned()))
        }
    }

    /// If the item most recently returned by
    /// [`item_os`][`ArgSplitter::item_os`],
    /// [`item`][`ArgSplitter::item`] or
    /// [`flag`][`ArgSplitter::flag`]
    /// was a flag, return its parameter as a [`String`].
    /// If the flag had a parameter attached (see [`has_param_attached`][`ArgSplitter::has_param_attached`]),
    /// return that parameter. Otherwise, if the flag is followed by a word, return
    /// that word. If no more arguments follow or if the next argument is another
    /// flag, return [`ArgError::ParameterMissing`].
    pub fn param(&mut self) -> AResult<String> {
        self.param_os().force_unicode()
    }
}

impl ArgSplitter {
    /// Similar to [`ArgSplitter::item_os`] but only returns flags.
    /// Returns them as an `Option<&str>` rather than [`Item`] or [`ItemOs`]
    /// for better match ergomics.
    /// All non-flag arguments are stashed in a buffer from which they can
    /// be retrieved using 
    /// [`stashed`][`ArgSplitter::stashed`],
    /// [`stashed_os`][`ArgSplitter::stashed_os`],
    /// [`stashed_args`][`ArgSplitter::stashed_args`] or
    /// [`stashed_args_os`][`ArgSplitter::stashed_args_os`].
    pub fn flag(&mut self) -> AResult<Option<&str>> {
        loop {
            let w = match self.item_os()? {
                None => return Ok(None),
                Some(ItemOs::Flag(_)) => break,
                Some(ItemOs::Word(w)) => w,
            };
            self.stashed_args.push(w);
        }
        Ok(Some(self.flag_ref()))
    }

    fn take_stashed(&mut self) -> Option<OsString> {
        if self.stashed_args.is_empty() {
            None
        } else {
            Some(self.stashed_args.remove(0))
        }
    }

    /// Return an argument set aside by [`ArgSplitter::flag`], as an
    /// [`OsString`]. Yields an [`OsString`] or an error if no argument is
    /// present. For optional arguments, see the iterator returned by
    /// [`ArgSplitter::stashed_args_os`].
    pub fn stashed_os(&mut self, desc: &str) -> AResult<OsString> {
        match self.take_stashed() {
            Some(v) => Ok(v),
            None => Err(ArgError::ArgumentMissing(desc.to_owned())),
        }
    }

    /// Return an argument set aside by [`ArgSplitter::flag`], as  [`String`].
    /// Yields a [`String`] or an error if no argument is present. For optional
    /// arguments, see the iterator returned by [`ArgSplitter::stashed_args`].
    pub fn stashed(&mut self, desc: &str) -> AResult<String> {
        self.stashed_os(desc).force_unicode()
    }

    /// Iterate over the arguments set aside by [`ArgSplitter::flag`], as
    /// [`OsString`]. Return an error if no sufficient number of stashed
    /// arguments is available. Use `desc` as a description in the error
    /// message.
    ///
    /// A note about the return type. This function returns a Result of Iterator
    /// over OsString. The Result is outside the Iterator because the only thing
    /// that can go wrong is that there are too few arguments. This way we don't
    /// have to deal with errors when processing individual items from the
    /// iterator. This makes it different from [`ArgSplitter::stashed_args`].
    ///
    /// # Example
    /// ```
    /// # fn main() -> Result<(),Box<dyn std::error::Error>> {
    /// # use std::path::PathBuf;
    /// # use argsplitter::ArgSplitter;
    /// # let mut argsplitter = ArgSplitter::from(["test", "a", "-f"]);
    /// # argsplitter.flag();
    /// let filenames: Vec<PathBuf> = argsplitter
    ///         .stashed_args_os(1, "FILE")?
    ///         .map(PathBuf::from)
    ///         .collect();
    /// # let _ = filenames; Ok(())
    /// # }
    /// ```
    /// Note how the question mark operator comes directly after the call to
    /// `stashed_args_os()`.
    pub fn stashed_args_os(&mut self, expect_at_least: usize, desc: &str) -> AResult<StashedOs> {
        if self.stashed_args.len() >= expect_at_least {
            Ok(StashedOs(self))
        } else {
            Err(ArgError::ArgumentMissing(desc.to_owned()))
        }
    }

    /// Iterate over the arguments set aside by [`ArgSplitter::flag`], as
    /// [`String`]. Return errors if the encoding is wrong or if no sufficient
    /// number of stashed arguments is available. Use `desc` as a description in
    /// the error message.
    ///
    /// A note about the return type. This function returns an Iterator over
    /// Results of String. The Result is inside the Iterator because the errors
    /// can either be about the number of available stashed arguments or about
    /// the encoding of an individual argument. This makes it different from
    /// [`ArgSplitter::stashed_args_os`].
    ///
    /// # Example
    /// ```
    /// # fn main() -> Result<(),Box<dyn std::error::Error>> {
    /// # use argsplitter::ArgSplitter;
    /// # let mut argsplitter = ArgSplitter::from(["test", "a", "-f"]);
    /// # argsplitter.flag();
    /// let recipients: Vec<String> = argsplitter
    ///         .stashed_args(1, "RECIPIENT")
    ///         .collect::<Result<_,_>>()?;
    /// # let _ = recipients; Ok(())
    /// # }
    /// ```
    /// Note how the question mark operator only comes after the `collect` has
    /// moved the Result from inside the iterator to outside.
    pub fn stashed_args(&mut self, expect_at_least: usize, desc: &str) -> Stashed {
        let err = if self.stashed_args.len() >= expect_at_least {
            None
        } else {
            Some(ArgError::ArgumentMissing(desc.to_owned()))
        };
        Stashed {
            splitter: self,
            err,
        }
    }

    /// Return `Ok(())` if all stashed arguments have been consumed,
    /// `Err(ArgError::UnexpectedArgument)` otherwise.
    pub fn no_more_stashed(&self) -> AResult<()> {
        if self.stashed_args.is_empty() {
            Ok(())
        } else {
            Err(ArgError::UnexpectedArgument(self.stashed_args[0].clone()))
        }
    }
}

/// Iterator returned by [`ArgSplitter::stashed_args_os()`].
pub struct StashedOs<'a>(&'a mut ArgSplitter);

impl Iterator for StashedOs<'_> {
    type Item = OsString;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.take_stashed()
    }
}

/// Iterator returned by [`ArgSplitter::stashed_args()`].
pub struct Stashed<'a> {
    splitter: &'a mut ArgSplitter,
    err: Option<ArgError>,
}

impl Iterator for Stashed<'_> {
    type Item = AResult<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref err) = self.err {
            Some(Err(err.clone()))
        } else {
            self.splitter
                .take_stashed()
                .map(ForceUnicode::force_unicode)
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn test_completely_empty() {
        let empty: Vec<OsString> = vec![];
        let mut sp = ArgSplitter::from(empty);

        assert_eq!(sp.argv0(), None);
        assert_eq!(sp.has_param_attached(), false);
        assert_eq!(sp.item_os(), Ok(None));
        assert_eq!(sp.has_param_attached(), false);
    }

    #[test]
    fn test_no_args() {
        let empty: Vec<OsString> = vec!["test".into()];
        let mut sp = ArgSplitter::from(empty);

        assert_eq!(sp.argv0(), Some(OsStr::new("test")));
        assert_eq!(sp.has_param_attached(), false);
        assert_eq!(sp.item_os(), Ok(None));
        assert_eq!(sp.has_param_attached(), false);
    }

    #[test]
    fn test_split_short() {
        let mut sp = ArgSplitter::from(["test", "-vx", "-n", "ARGS"]);

        assert_eq!(sp.has_param_attached(), false);

        assert_eq!(sp.item(), Ok(Some(Item::Flag("-v"))));
        assert_eq!(sp.has_param_attached(), true);
        assert_eq!(sp.clone().param(), Ok("x".to_owned()));

        assert_eq!(sp.item(), Ok(Some(Item::Flag("-x"))));
        assert_eq!(sp.has_param_attached(), false);
        assert_eq!(
            sp.clone().param(),
            Err(ArgError::ParameterMissing("-x".into()))
        );

        assert_eq!(sp.item(), Ok(Some(Item::Flag("-n"))));
        assert_eq!(sp.has_param_attached(), false);
        assert_eq!(sp.clone().param(), Ok("ARGS".into()));

        assert_eq!(sp.item(), Ok(Some(Item::Word("ARGS".into()))));
        assert_eq!(sp.has_param_attached(), false);
        // must not call .parm after getting a Word.
    }

    #[test]
    fn test_split_long() {
        let mut sp = ArgSplitter::from(["test", "--foo", "--bar=BAR", "--baz", "ARGS"]);

        assert_eq!(sp.has_param_attached(), false);

        assert_eq!(sp.item(), Ok(Some(Item::Flag("--foo"))));
        assert_eq!(sp.has_param_attached(), false);
        assert_eq!(
            sp.clone().param(),
            Err(ArgError::ParameterMissing("--foo".into()))
        );

        assert_eq!(sp.item(), Ok(Some(Item::Flag("--bar"))));
        assert_eq!(sp.has_param_attached(), true);
        assert_eq!(sp.param(), Ok("BAR".into()));
        assert_eq!(sp.has_param_attached(), false);
        assert_eq!(
            sp.clone().param(),
            Err(ArgError::ParameterMissing("--bar".into()))
        );

        assert_eq!(sp.item(), Ok(Some(Item::Flag("--baz"))));
        assert_eq!(sp.has_param_attached(), false);
        assert_eq!(sp.clone().param(), Ok("ARGS".into()));

        assert_eq!(sp.item(), Ok(Some(Item::Word("ARGS".into()))));
        assert_eq!(sp.has_param_attached(), false);
        // must not call .parm after getting a Word.
    }
}
