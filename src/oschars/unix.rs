use std::ffi::{OsStr, OsString};

#[cfg(not(unix))]
fn osstr_to_bytes(_s: &OsStr) -> Vec<u8> {
    unimplemented!("osstr_to_bytes() is only available on Unix")
}

#[cfg(not(unix))]
fn osstring_from_bytes(_b: &[u8]) -> OsString {
    unimplemented!("osstring_from_bytes is only available on Unix")
}

#[cfg(unix)]
fn osstr_to_bytes(s: &OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStringExt;
    OsString::into_vec(s.to_owned())
}

#[cfg(unix)]
fn osstring_from_bytes(b: &[u8]) -> OsString {
    use std::os::unix::ffi::OsStringExt;
    OsString::from_vec(b.to_owned())
}

/// Return an example of a badly encoded OsString
pub fn badly_encoded() -> OsString {
    osstring_from_bytes(b"\x80BAD")
}

/// Split the OsString into the prefix that is UTF-16 valid, and the tail that isn't.
pub fn split_valid(os: &OsStr) -> (String, OsString) {
    let bytes = osstr_to_bytes(os);
    match String::from_utf8(bytes) {
        Ok(s) => (s, "".into()),
        Err(e) => {
            let idx = e.utf8_error().valid_up_to();
            let head = String::from_utf8(e.as_bytes()[..idx].to_owned()).unwrap();
            let tail = osstring_from_bytes(&e.as_bytes()[idx..]);
            (head, tail)
        }
    }
}
