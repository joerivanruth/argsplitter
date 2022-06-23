use std::{ffi::OsString, fmt};

use crate::ArgError;

#[cfg(doc)]
use crate::{core::Core, ArgSplitter};

/// * Item returned from [`Core::take_item`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnedItem {
    /// An argument that didn't start with a dash, or the special case `"-"`
    Word(OsString),
    /// Long option such as --verbose or --file=data.csv
    Flag(String),
}

/// Item returned from [`ArgSplitter::item_os`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemOs<'a> {
    /// An argument that does not start with a dash
    Word(OsString),
    /// A short flag `-f` or a long flag `--file`. Includes the leading dashes.
    Flag(&'a str),
}

/// Item returned from [`ArgSplitter::item`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item<'a> {
    /// An argument that does not start with a dash
    Word(String),
    /// A short flag `-f` or a long flag `--file`. Includes the leading dashes.
    Flag(&'a str),
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
    /// Return [`ArgError::UnexpectedFlag`] or [`ArgError::UnexpectedArgument`]
    /// depending on the type of item.
    pub fn unexpected(&self) -> ArgError {
        match self {
            ItemOs::Flag(f) => ArgError::unknown_flag(f),
            ItemOs::Word(w) => ArgError::unexpected_argument(w),
        }
    }
}

impl Item<'_> {
    /// Return [`ArgError::UnexpectedFlag`] or [`ArgError::UnexpectedArgument`]
    /// depending on the type of item.
    pub fn unexpected(&self) -> ArgError {
        match self {
            Item::Flag(f) => ArgError::unknown_flag(f),
            Item::Word(w) => ArgError::unexpected_argument(w),
        }
    }
}
