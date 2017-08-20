use std::{borrow, ffi, fmt, ops};
use std::os::raw::{c_char, c_int};

use stack_frame::StackFrame;

extern "C" {
    fn stack_frame_function_name(frame: *const StackFrame) -> *const c_char;
    fn stack_frame_source_file_name(frame: *const StackFrame) -> *const c_char;
    fn stack_frame_source_line(frame: *const StackFrame) -> c_int;
    fn stack_frame_delete(frame: *mut StackFrame);
}

/// A resolved version of StackFrame. Contains source code locations and code
/// offsets, if the resolver was able to locate symbols for this frame.
/// Otherwise, the additional attributes are empty.
///
/// ResolvedStackFrame implements Deref for StackFrame, so that it can be used
/// interchangibly. See StackFrame for additional accessors.
pub struct ResolvedStackFrame {
    internal: *mut StackFrame,
}

impl ResolvedStackFrame {
    /// Creates a ResolvedStackFrame instance from a raw stack frame pointer.
    /// The pointer is assumed to be owned, and the underlying memory will be
    /// freed when this struct is dropped.
    pub(crate) fn from_ptr(internal: *mut StackFrame) -> ResolvedStackFrame {
        ResolvedStackFrame { internal }
    }

    /// Returns the function name that contains the instruction. Can be empty
    /// before running the resolver or if debug symbols are missing.
    pub fn function_name(&self) -> borrow::Cow<str> {
        unsafe {
            let ptr = stack_frame_function_name(self.internal);
            ffi::CStr::from_ptr(ptr).to_string_lossy()
        }
    }

    /// Returns the source code line at which the instruction was declared.
    /// Can be empty before running the resolver or if debug symbols are
    /// missing.
    pub fn source_file_name(&self) -> borrow::Cow<str> {
        unsafe {
            let ptr = stack_frame_source_file_name(self.internal);
            ffi::CStr::from_ptr(ptr).to_string_lossy()
        }
    }

    /// Returns the source code line at which the instruction was declared. Can
    /// be empty before running the resolver or if debug symbols are missing.
    pub fn source_line(&self) -> c_int {
        unsafe { stack_frame_source_line(self.internal) }
    }
}

impl ops::Deref for ResolvedStackFrame {
    type Target = StackFrame;

    fn deref(&self) -> &StackFrame {
        unsafe { &*self.internal }
    }
}

impl Drop for ResolvedStackFrame {
    fn drop(&mut self) {
        unsafe { stack_frame_delete(self.internal) };
    }
}

impl fmt::Debug for ResolvedStackFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ResolvedStackFrame")
            .field("instruction", &self.instruction())
            .field("function_name", &self.function_name())
            .field("source_file_name", &self.source_file_name())
            .field("source_line", &self.source_line())
            .field("trust", &self.trust())
            .field("module", &self.module())
            .finish()
    }
}
