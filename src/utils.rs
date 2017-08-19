use std::ffi;
use std::os::raw::c_char;

extern "C" {
    fn string_delete(string: *mut c_char);
}

pub fn ptr_to_owned_str(ptr: *mut c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        let string = unsafe { ffi::CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned();

        unsafe { string_delete(ptr) };
        string
    }
}
