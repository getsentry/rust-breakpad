use std::{ffi, path};
use std::os::raw::c_char;

extern "C" {
    fn string_delete(string: *mut c_char);
}

/// Converts an owned raw pointer to characters to an owned String object.
pub fn ptr_to_string(ptr: *mut c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }

    let string = unsafe { ffi::CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();

    unsafe { string_delete(ptr) };
    string
}

#[cfg(windows)]
pub fn path_to_bytes<P: AsRef<path::Path> + ?Sized>(path: &P) -> &[u8] {
    path.as_ref().as_os_str().to_str().unwrap().as_bytes()
}

#[cfg(unix)]
pub fn path_to_bytes<P: AsRef<path::Path> + ?Sized>(path: &P) -> &[u8] {
    use std::os::unix::ffi::OsStrExt;
    path.as_ref().as_os_str().as_bytes()
}
