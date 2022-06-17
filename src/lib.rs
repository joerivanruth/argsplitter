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
