use std::ffi;
use std::os::raw::{c_char, c_void};

use errors::*;

type Internal = c_void;

extern "C" {
    fn minidump_read(file_path: *const c_char) -> *mut Internal;
    fn minidump_delete(dump: *mut Internal);
    fn minidump_print(dump: *mut Internal);
}

pub struct Minidump {
    internal: *mut Internal,
}

impl Minidump {
    pub fn new<S: Into<Vec<u8>>>(file_path: S) -> Result<Minidump> {
        let cstr = ffi::CString::new(file_path).unwrap();
        let internal = unsafe { minidump_read(cstr.as_ptr()) };

        if internal.is_null() {
            let err = ErrorKind::MinidumpError("Minidump could not be read".into());
            Err(err.into())
        } else {
            Ok(Minidump { internal })
        }
    }

    pub fn print(&self) {
        unsafe { minidump_print(self.internal) }
    }
}

impl Drop for Minidump {
    fn drop(&mut self) {
        unsafe { minidump_delete(self.internal) };
    }
}
