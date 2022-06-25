use std::{
    env,
    ffi::{OsStr, OsString},
};

use crate::{core::Core, item::OwnedItem, ArgError, ForceUnicode, Item, ItemOs};

type AResult<T> = Result<T, ArgError>;

#[derive(Debug, Clone)]
pub struct ArgSplitter {
    argv0: Option<OsString>,
    core: Core,
    last_flag: Option<String>,
    stashed_args: Vec<OsString>,
}

impl ArgSplitter {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::from(env::args_os())
    }

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
    pub fn argv0(&self) -> Option<&OsStr> {
        self.argv0.as_deref()
    }

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

    pub fn item(&mut self) -> AResult<Option<Item>> {
        self.item_os().force_unicode()
    }

    pub fn at_word(&self) -> bool {
        self.core.at_word()
    }

    pub fn has_param_attached(&self) -> bool {
        self.core.param_ready()
    }

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

    pub fn param(&mut self) -> AResult<String> {
        self.param_os().force_unicode()
    }
}

impl ArgSplitter {
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

    pub fn stashed_iter_os(&mut self) -> StashedOs {
        StashedOs(self)
    }

    pub fn stashed_iter(&mut self) -> Stashed {
        Stashed(self.stashed_iter_os())
    }

    pub fn stashed_os(&mut self, desc: &str) -> AResult<OsString> {
        match self.take_stashed() {
            Some(v) => Ok(v),
            None => Err(ArgError::ArgumentMissing(desc.to_owned())),
        }
    }

    pub fn stashed(&mut self, desc: &str) -> AResult<String> {
        self.stashed_os(desc).force_unicode()
    }

    pub fn no_more_stashed(&self) -> AResult<()> {
        if self.stashed_args.is_empty() {
            Ok(())
        } else {
            Err(ArgError::UnexpectedArgument(self.stashed_args[0].clone()))
        }
    }
}

pub struct StashedOs<'a>(&'a mut ArgSplitter);

impl Iterator for StashedOs<'_> {
    type Item = OsString;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.take_stashed()
    }
}

pub struct Stashed<'a>(StashedOs<'a>);

impl Iterator for Stashed<'_> {
    type Item = AResult<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(ForceUnicode::force_unicode)
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
