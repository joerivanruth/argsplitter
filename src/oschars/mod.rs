#![allow(dead_code)]
// Allow dead_code because on Unix, the Windows versions are not used
// and vice versa.

// Import both the Unix- and the Windows versions.
// Most of the Windows code can be tested even on Unix.
mod unix;
mod windows;

#[cfg(test)]
use std::ffi::OsString;

#[cfg(unix)]
pub use unix::badly_encoded;

#[cfg(unix)]
pub use unix::split_valid;

#[cfg(windows)]
pub use windows::badly_encoded;

#[cfg(windows)]
pub use windows::split_valid;

#[test]
fn test_split_valid() {
    let good = String::from("GOOD");
    let bad = badly_encoded();
    let mut input: OsString = good.clone().into();
    input.push(&bad);

    let (head, tail) = split_valid(&input);
    assert_eq!(head, good);
    assert_eq!(tail, bad);
}
