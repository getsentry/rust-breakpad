use std::ffi;
use std::os::raw::c_char;

extern "C" {
    fn string_delete(string: *mut c_char);
}

/// Converts an owned raw pointer to characters to an owned String object.
pub fn ptr_to_owned_str(ptr: *mut c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }

    let string = unsafe { ffi::CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();

    unsafe { string_delete(ptr) };
    string
}
