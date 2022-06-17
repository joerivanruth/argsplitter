use std::{ffi::OsString, fmt};

use crate::ArgError;

#[cfg(doc)]
use crate::{core::Core, ArgSplitter};

/**
 * Item returned from [`Core::take_item`]
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnedItem {
    /// Long option such as --verbose or --file=data.csv
    Flag(String),
    /// An argument that didn't start with a dash, or the two special cases
    /// `"-"` and `"--"`
    Word(OsString),
}

/**
Item returned from [`ArgSplitter::item_os`]
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemOs<'a> {
    Flag(&'a str),
    Word(OsString),
}

/**
Item returned from [`ArgSplitter::item`]
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item<'a> {
    Flag(&'a str),
    Word(String),
}

impl fmt::Display for Item<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Item::Flag(flag) => flag.fmt(f),
            Item::Word(word) => word.fmt(f),
        }
    }
}

impl fmt::Display for ItemOs<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemOs::Flag(flag) => flag.fmt(f),
            ItemOs::Word(word) => word.to_string_lossy().fmt(f),
        }
    }
}

impl ItemOs<'_> {
    pub fn unexpected(&self) -> Result<(), ArgError> {
        let err = match self {
            ItemOs::Flag(f) => ArgError::unknown_flag(f),
            ItemOs::Word(w) => ArgError::unexpected_argument(w),
        };
        Err(err)
    }
}

impl Item<'_> {
    pub fn unexpected(&self) -> Result<(), ArgError> {
        let err = match self {
            Item::Flag(f) => ArgError::unknown_flag(f),
            Item::Word(w) => ArgError::unexpected_argument(w),
        };
        Err(err)
    }
}
