use std::{
    ffi::{OsStr, OsString},
    fmt,
};

#[cfg(not(windows))]
fn osstr_to_wide(_s: &OsStr) -> Vec<u16> {
    unimplemented!("osstr_to_wide() is only available on Windows")
}

#[cfg(not(windows))]
fn osstring_from_wide(_b: &[u16]) -> OsString {
    unimplemented!("osstring_from_wide is only available on Windows")
}

#[cfg(windows)]
fn osstr_to_wide(s: &OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    OsStr::encode_wide(s).collect()
}

#[cfg(windows)]
fn osstring_from_wide(b: &[u16]) -> OsString {
    use std::os::windows::ffi::OsStringExt;
    OsString::osstring_from_wide(b)
}

/// Return an example of a badly encoded OsString
pub fn badly_encoded() -> OsString {
    osstring_from_wide(&[0xD800, 0xD840, 0x42, 0x41, 0x44])
}

/// Split the OsString into the prefix that is UTF-16 valid, and the tail that isn't.
pub fn split_valid(os: &OsStr) -> (String, OsString) {
    let wide = osstr_to_wide(os);
    let idx = find_invalid(&wide).unwrap_or(wide.len());
    let head = String::from_utf16(&wide[..idx]).unwrap();
    let tail = osstring_from_wide(&wide[idx..]);
    (head, tail)
}

fn find_invalid(wide: &[u16]) -> Option<usize> {
    let mut units = wide.iter().copied().enumerate().peekable();

    while let Some((i, u)) = units.next() {
        let kind = Kind::of(u);
        let next_kind = units.peek().map(|&(_, x)| Kind::of(x));
        match (kind, next_kind) {
            (Kind::Normal, _) => {}
            (Kind::HighSurrogate, Some(Kind::LowSurrogate)) => {
                // skip the low surrogate
                units.next();
            }
            _ => return Some(i),
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Kind {
    Normal,
    LowSurrogate,
    HighSurrogate,
}

impl Kind {
    fn of(unit: u16) -> Self {
        use Kind::*;
        match unit {
            0x0000..=0xD7FF => Normal,
            0xD800..=0xDBFF => HighSurrogate,
            0xDC00..=0xDFFF => LowSurrogate,
            0xE000..=0xFFFF => Normal,
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Kind::Normal => "normal",
            Kind::LowSurrogate => "low",
            Kind::HighSurrogate => "high",
        };
        f.write_str(s)
    }
}

#[test]
fn test_find_invalid() {
    // example code units
    let ok = &[0x0000, 0x0040, 0xD7FF, 0xE000, 0xFFFF];
    let hi = &[0xD800, 0xD840, 0xDBFF];
    let lo = &[0xDC00, 0xDC40, 0xDFFF];

    fn generate_variants(pattern: &[&[u16]], buffer: &mut Vec<u16>, output: &mut Vec<Vec<u16>>) {
        if pattern.is_empty() {
            output.push(buffer.clone());
            return;
        }
        let first = pattern[0];
        let rest = &pattern[1..];
        for &u in first {
            buffer.push(u);
            generate_variants(rest, buffer, output);
            buffer.pop().unwrap();
        }
    }

    fn verify(pattern: &[&[u16]], expected: Option<usize>) {
        let mut buffer: Vec<u16> = vec![];
        let mut testcases: Vec<Vec<u16>> = vec![];
        generate_variants(pattern, &mut buffer, &mut testcases);
        for testcase in testcases {
            verify_one(testcase, expected);
        }
    }

    fn verify_one(testcase: Vec<u16>, expected: Option<usize>) {
        let mut text = String::new();
        for u in &testcase {
            use std::fmt::Write;
            write!(text, "{:04x}({}) ", u, Kind::of(*u)).unwrap();
        }
        let text = text.trim_end();
        let solution = find_invalid(&testcase);
        assert_eq!(
            solution, expected,
            "find_invalid does not give expected result for input {:?}",
            text
        );
        let actually_valid = String::from_utf16(&testcase).is_ok();
        assert_eq!(solution.is_none(), actually_valid);
    }

    verify(&[], None);

    verify(&[ok], None);
    verify(&[hi], Some(0));
    verify(&[lo], Some(0));

    // all pairs
    verify(&[ok, ok], None);
    verify(&[ok, hi], Some(1));
    verify(&[ok, lo], Some(1));
    verify(&[hi, ok], Some(0));
    verify(&[hi, hi], Some(0));
    verify(&[hi, lo], None);
    verify(&[lo, ok], Some(0));
    verify(&[lo, hi], Some(0));
    verify(&[lo, lo], Some(0));

    // all pairs, with something valid after
    verify(&[ok, ok, ok], None);
    verify(&[ok, hi, ok], Some(1));
    verify(&[ok, lo, ok], Some(1));
    verify(&[hi, ok, ok], Some(0));
    verify(&[hi, hi, ok], Some(0));
    verify(&[hi, lo, ok], None);
    verify(&[lo, ok, ok], Some(0));
    verify(&[lo, hi, ok], Some(0));
    verify(&[lo, lo, ok], Some(0));
    //
    verify(&[ok, ok, hi, lo], None);
    verify(&[ok, hi, hi, lo], Some(1));
    verify(&[ok, lo, hi, lo], Some(1));
    verify(&[hi, ok, hi, lo], Some(0));
    verify(&[hi, hi, hi, lo], Some(0));
    verify(&[hi, lo, hi, lo], None);
    verify(&[lo, ok, hi, lo], Some(0));
    verify(&[lo, hi, hi, lo], Some(0));
    verify(&[lo, lo, hi, lo], Some(0));

    // all pairs, with something valid before
    verify(&[ok, ok, ok], None);
    verify(&[ok, ok, hi], Some(2));
    verify(&[ok, ok, lo], Some(2));
    verify(&[ok, hi, ok], Some(1));
    verify(&[ok, hi, hi], Some(1));
    verify(&[ok, hi, lo], None);
    verify(&[ok, lo, ok], Some(1));
    verify(&[ok, lo, hi], Some(1));
    verify(&[ok, lo, lo], Some(1));
    //
    verify(&[hi, lo, ok, ok], None);
    verify(&[hi, lo, ok, hi], Some(3));
    verify(&[hi, lo, ok, lo], Some(3));
    verify(&[hi, lo, hi, ok], Some(2));
    verify(&[hi, lo, hi, hi], Some(2));
    verify(&[hi, lo, hi, lo], None);
    verify(&[hi, lo, lo, ok], Some(2));
    verify(&[hi, lo, lo, hi], Some(2));
    verify(&[hi, lo, lo, lo], Some(2));
}
