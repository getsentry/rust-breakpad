use std::ffi;
use std::os::raw::{c_char, c_void};

use errors::*;
use process_state::ProcessState;

type Internal = c_void;

extern "C" {
    fn minidump_read(file_path: *const c_char) -> *mut Internal;
    fn minidump_delete(dump: *mut Internal);
    fn minidump_print(dump: *mut Internal);
    fn minidump_process(dump: *mut Internal) -> *mut Internal;
}

/// The primary interface to minidump files. Create an instance from a file
/// path and use process to obtain metadata about the dump's threads
/// containing all call stacks.
pub struct Minidump {
    internal: *mut Internal,
}

impl Minidump {
    /// Reads a minidump from the file system into memory.
    pub fn new<S: AsRef<str>>(file_path: S) -> Result<Minidump> {
        let cstr = ffi::CString::new(file_path.as_ref()).unwrap();
        let internal = unsafe { minidump_read(cstr.as_ptr()) };

        if internal.is_null() {
            let err = ErrorKind::MinidumpError("Minidump could not be read".into());
            Err(err.into())
        } else {
            Ok(Minidump { internal })
        }
    }

    /// Prints debug information of the minidump to standard output.
    pub fn print(&self) {
        unsafe { minidump_print(self.internal) }
    }

    /// Analyzes the minidump. The returned ProcessState struct contains
    /// threads with callstacks and loaded modules of the crash.
    pub fn process(&self) -> Result<ProcessState> {
        let ptr = unsafe { minidump_process(self.internal) };

        if ptr.is_null() {
            let err = ErrorKind::MinidumpError("Could not process minidump".into());
            Err(err.into())
        } else {
            Ok(ProcessState::new(ptr))
        }
    }
}

impl Drop for Minidump {
    fn drop(&mut self) {
        unsafe { minidump_delete(self.internal) };
    }
}
