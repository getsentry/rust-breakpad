use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::prelude::*;
use std::os::raw::c_char;
use std::path::Path;

use errors::Result;

extern "C" {
    fn string_delete(string: *mut c_char);
}

/// Converts an owned raw pointer to characters to an owned `String`.
/// If the pointer is NULL, an empty string `""` is returned.
pub fn ptr_to_string(ptr: *mut c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }

    let string = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();

    unsafe { string_delete(ptr) };
    string
}

/// Directly converts a path to a list of bytes without allocating memory.
/// This is useful when passing paths to extern "C" functions.
#[cfg(windows)]
pub fn path_to_bytes<P: AsRef<Path> + ?Sized>(path: &P) -> &[u8] {
    path.as_ref().as_os_str().to_str().unwrap().as_bytes()
}

/// Directly converts a path to a list of bytes without allocating memory.
/// This is useful when passing paths to extern "C" functions.
#[cfg(unix)]
pub fn path_to_bytes<P: AsRef<Path> + ?Sized>(path: &P) -> &[u8] {
    use std::os::unix::ffi::OsStrExt;
    path.as_ref().as_os_str().as_bytes()
}

/// Directly converts a path to a C string without allocating memory.
pub fn path_to_str<P: AsRef<Path>>(path: P) -> CString {
    let bytes = path_to_bytes(path.as_ref());
    CString::new(bytes).unwrap()
}

/// Reads an entire file into a memory buffer
pub fn read_buffer<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
