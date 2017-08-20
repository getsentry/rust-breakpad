use std::{cmp, fmt, hash};
use std::os::raw::{c_char, c_void};

use utils::ptr_to_owned_str;

/// Carries information about a code module loaded into the process during the
/// crash. The debug_identifier uniquely identifies this module.
#[repr(C)]
pub struct CodeModule(c_void);

extern "C" {
    fn code_module_debug_file(module: *const CodeModule) -> *mut c_char;
    fn code_module_debug_identifier(module: *const CodeModule) -> *mut c_char;
}

impl CodeModule {
    /// Returns the unique identifier of the code module. Usually, it consist
    /// of the library's UUID and an age field. On Windows, the age field is a
    /// generation counter, on all other platforms it always zero.
    pub fn debug_file(&self) -> String {
        unsafe {
            let ptr = code_module_debug_file(self);
            ptr_to_owned_str(ptr)
        }
    }

    /// Returns the name of the library or framework.
    pub fn debug_identifier(&self) -> String {
        unsafe {
            let ptr = code_module_debug_identifier(self);
            ptr_to_owned_str(ptr)
        }
    }
}

impl Eq for CodeModule {}

impl PartialEq for CodeModule {
    fn eq(&self, other: &Self) -> bool {
        self.debug_identifier() == other.debug_identifier()
    }
}

impl hash::Hash for CodeModule {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.debug_identifier().hash(state)
    }
}

impl Ord for CodeModule {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.debug_identifier().cmp(&other.debug_identifier())
    }
}

impl PartialOrd for CodeModule {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Debug for CodeModule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CodeModule")
            .field("debug_file", &self.debug_file())
            .field("debug_identifier", &self.debug_identifier())
            .finish()
    }
}
