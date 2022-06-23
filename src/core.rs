use std::{ffi::OsString, mem, vec};

use crate::{item::OwnedItem, oschars, ArgError};

type AResult<T> = Result<T, ArgError>;

/// This enum represents the argument currently under consideration.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ArgState {
    /// The current argument is either '-', '--', or it does not start with a dash at all
    Word(OsString),
    /// The current argument is a set of single letter flags that was preceded by a single dash.
    ShortOptionsNew(char, Vec<char>, OsString),
    /// The current argument is the set of single letter flags remaining after at least one has been processed
    ShortOptionsUsed(char, Vec<char>, OsString),
    /// The current argument is a long option (--flag[=value]) with optional value.  Includes the dashes
    LongOption(String, Option<OsString>),
    /// A long option --flag=value has been consumed but its value remains
    RemainingParameter(String, OsString),
    /// There was a bad character directly after the - or --
    CannotDecode(OsString),
    /// No more tokens remain
    End,
}
use ArgState::*;

impl ArgState {
    /// Take a new argument into consideration
    fn from(raw_arg: Option<OsString>) -> Self {
        let s = match raw_arg {
            Some(a) => a,
            None => return End,
        };

        let (head, tail) = oschars::split_valid(s.as_os_str());

        let has_undecodable = !tail.is_empty();
        match (head.as_str(), has_undecodable) {
            // Special case
            ("-", false) => return Word("-".into()),
            // Flags must start with at least one decodable character
            ("-" | "--", true) => return CannotDecode(s),
            _ => {}
        }

        if head.starts_with("--") {
            match head.find('=') {
                None => {
                    if tail.is_empty() {
                        let flag = head.to_string();
                        LongOption(flag, None)
                    } else {
                        // without =, the tail becomes part of the flag but we only allow utf-8 flags
                        CannotDecode(s)
                    }
                }
                Some(idx) => {
                    let flag = head[..idx].to_string();
                    let mut param = OsString::from(&head[idx + 1..]);
                    param.push(tail);
                    LongOption(flag, Some(param))
                }
            }
        } else if let Some(h) = head.strip_prefix('-') {
            let mut chars = h.chars();
            let first = chars.next().unwrap();
            ShortOptionsNew(first, chars.collect(), tail)
        } else {
            Word(s)
        }
    }

    /// Convenience method that replaces `*self` with `Argument::End` and returns the original value.
    fn take(&mut self) -> Self {
        let mut ret = End;
        mem::swap(self, &mut ret);
        ret
    }
}

/// The state machine inside the argument parser.
#[derive(Debug, Clone)]
pub struct Core {
    cur: ArgState,
    rest: vec::IntoIter<OsString>,
}

impl Core {
    /// Create a new state machine from a set of arguments
    pub fn new(items: Vec<OsString>) -> Self {
        let mut rest = items.into_iter();
        let cur = ArgState::from(rest.next());
        Core { cur, rest }
    }

    /// Take the next item out of the arguments.
    pub fn take_item(&mut self) -> AResult<Option<OwnedItem>> {
        let cur = self.cur.take();

        let mut override_next = None;
        let result = match cur {
            End => Ok(None),
            Word(w) => Ok(Some(OwnedItem::Word(w))),
            CannotDecode(s) => Err(ArgError::InvalidUnicode(s)),
            LongOption(flag, param) => {
                if let Some(p) = param {
                    override_next = Some(RemainingParameter(flag.clone(), p));
                }
                Ok(Some(OwnedItem::Flag(flag)))
            }
            RemainingParameter(f, _) => Err(ArgError::UnexpectedParameter(f)),
            ShortOptionsNew(first, mut more, tail) | ShortOptionsUsed(first, mut more, tail) => {
                let flag = format!("-{first}");
                if !more.is_empty() {
                    let c = more.remove(0);
                    override_next = Some(ShortOptionsUsed(c, more, tail));
                } else if !tail.is_empty() {
                    override_next = Some(RemainingParameter(flag.clone(), tail));
                }
                Ok(Some(OwnedItem::Flag(flag)))
            }
        };

        self.cur = match override_next {
            None => ArgState::from(self.rest.next()),
            Some(s) => s,
        };
        result
    }

    /// If the previous call to [`Core::take_item`] returned `ItemOs::Long`,
    /// return the value attached to it, if any.
    /// If it returned `ItemOs::Short`, return the remainder of the original
    /// argument as an OsString
    pub fn param(&mut self) -> Option<OsString> {
        let ret;
        let cur = self.cur.take();
        let next = match cur {
            RemainingParameter(_, p) => {
                ret = Some(p);
                ArgState::from(self.rest.next())
            }
            ShortOptionsUsed(f, more, tail) => {
                let s: String = [f].into_iter().chain(more.into_iter()).collect();
                let mut p: OsString = s.into();
                p.push(tail);
                ret = Some(p);
                ArgState::from(self.rest.next())
            }
            _ => {
                ret = None;
                cur
            }
        };
        self.cur = next;
        ret
    }

    pub fn param_ready(&self) -> bool {
        matches!(
            self.cur,
            RemainingParameter(_, _) | ShortOptionsUsed(_, _, _)
        )
    }

    /// Return true if the next call to [`Core::take_item`] will return
    /// `ItemOs::Work(_)`.
    pub fn at_word(&self) -> bool {
        matches!(self.cur, Word(_))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use crate::oschars::badly_encoded;

    use super::*;

    fn argstate(s: &str) -> ArgState {
        ArgState::from(Some(s.into()))
    }

    fn os(s: &str) -> OsString {
        s.into()
    }

    #[test]
    fn test_argstate() {
        fn badly(prefix: &str) -> OsString {
            let mut ret = OsString::from(prefix);
            ret.push(badly_encoded());
            ret
        }

        fn bad() -> OsString {
            badly("banana")
        }

        assert_eq!(ArgState::from(None), End);

        assert_eq!(argstate(""), Word(os("")));
        assert_eq!(argstate("-"), Word(os("-")));
        assert_eq!(ArgState::from(Some(bad())), Word(bad()));

        assert_eq!(argstate("--foo"), LongOption("--foo".into(), None));
        assert_eq!(argstate("--foo="), LongOption("--foo".into(), Some(os(""))));
        assert_eq!(
            argstate("--foo=bar"),
            LongOption("--foo".into(), Some(os("bar")))
        );
        assert_eq!(
            argstate("--foo=bar=baz"),
            LongOption("--foo".into(), Some(os("bar=baz")))
        );
        assert_eq!(argstate("--"), LongOption("--".into(), None));
        assert_eq!(
            ArgState::from(Some(badly("--foo=X"))),
            LongOption("--foo".into(), Some(badly("X")))
        );
        assert_eq!(ArgState::from(Some(badly("--"))), CannotDecode(badly("--")));

        assert_eq!(argstate("---"), LongOption("---".into(), None));

        assert_eq!(argstate("-x"), ShortOptionsNew('x', vec![], os("")));
        assert_eq!(
            argstate("-xvw"),
            ShortOptionsNew('x', vec!['v', 'w'], os(""))
        );
        assert_eq!(ArgState::from(Some(badly("-"))), CannotDecode(badly("-")));
        assert_eq!(
            ArgState::from(Some(badly("-f"))),
            ShortOptionsNew('f', vec![], badly(""))
        );
        assert_eq!(
            ArgState::from(Some(badly("-fv"))),
            ShortOptionsNew('f', vec!['v'], badly(""))
        );
    }

    #[test]
    fn test_empty() {
        let mut core = Core::new(vec![]);

        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.take_item(), Ok(None));
        assert_eq!(core.take_item(), Ok(None));
        assert_eq!(core.take_item(), Ok(None));
    }

    #[test]
    fn test_vx_ARG() {
        let mut core = Core::new(vec![os("-vx"), os("ARG")]);

        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.take_item(), Ok(Some(OwnedItem::Flag("-v".into()))));
        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), true);
        assert_eq!(core.clone().param(), Some(os("x")));

        assert_eq!(core.take_item(), Ok(Some(OwnedItem::Flag("-x".into()))));
        assert_eq!(core.at_word(), true);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.take_item(), Ok(Some(OwnedItem::Word("ARG".into()))));
        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);
        assert_eq!(core.take_item(), Ok(None));
    }

    #[test]
    fn test_vfFILE_ARG() {
        let mut core = Core::new(vec![os("-vfFILE"), os("ARG")]);

        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.take_item(), Ok(Some(OwnedItem::Flag("-v".into()))));
        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), true);
        assert_eq!(core.clone().param(), Some(os("fFILE")));

        assert_eq!(core.take_item(), Ok(Some(OwnedItem::Flag("-f".into()))));
        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), true);

        assert_eq!(core.param(), Some(os("FILE")));
        assert_eq!(core.at_word(), true);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.take_item(), Ok(Some(OwnedItem::Word("ARG".into()))));
        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);
        assert_eq!(core.take_item(), Ok(None));
    }

    #[test]
    fn test_file_ARG() {
        let mut core = Core::new(vec![os("--file"), os("ARG")]);

        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.take_item(), Ok(Some(OwnedItem::Flag("--file".into()))));
        assert_eq!(core.at_word(), true);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.take_item(), Ok(Some(OwnedItem::Word("ARG".into()))));
        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);
        assert_eq!(core.take_item(), Ok(None));
    }

    #[test]
    fn test_fileFILE_ARG() {
        let mut core = Core::new(vec![os("--file=FILE"), os("ARG")]);

        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.take_item(), Ok(Some(OwnedItem::Flag("--file".into()))));
        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), true);

        assert_eq!(core.param(), Some(os("FILE")));
        assert_eq!(core.at_word(), true);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.take_item(), Ok(Some(OwnedItem::Word("ARG".into()))));
        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);
        assert_eq!(core.take_item(), Ok(None));
    }

    #[test]
    fn test_fileEMPTY_ARG() {
        let mut core = Core::new(vec![os("--file="), os("ARG")]);

        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.take_item(), Ok(Some(OwnedItem::Flag("--file".into()))));
        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), true);

        assert_eq!(core.param(), Some(os("")));
        assert_eq!(core.at_word(), true);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.take_item(), Ok(Some(OwnedItem::Word("ARG".into()))));
        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);

        assert_eq!(core.at_word(), false);
        assert_eq!(core.param_ready(), false);
        assert_eq!(core.clone().param(), None);
        assert_eq!(core.take_item(), Ok(None));
    }

    #[test]
    fn test_dashes() {
        let mut core = Core::new(vec![os("-"), os("--")]);

        assert_eq!(core.take_item(), Ok(Some(OwnedItem::Word("-".into()))));
        assert_eq!(
            core.take_item(),
            Ok(Some(OwnedItem::Flag("--".to_string())))
        );
    }
}
