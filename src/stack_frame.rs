use std::{borrow, ffi, fmt};
use std::os::raw::{c_char, c_int, c_void};

use code_module::CodeModule;

#[repr(C)]
pub enum FrameTrust {
    None,        // Unknown
    Scan,        // Scanned the stack, found this
    CFIScan,     // Found while scanning stack using call frame info
    FP,          // Derived from frame pointer
    CFI,         // Derived from call frame info
    Prewalked,   // Explicitly provided by some external stack walker.
    Context,     // Given as instruction pointer in a context
}

#[repr(C)]
pub struct StackFrame(c_void);

extern "C" {
    fn stack_frame_instruction(frame: *const StackFrame) -> u64;
    fn stack_frame_module(frame: *const StackFrame) -> *const CodeModule;
    fn stack_frame_function_name(frame: *const StackFrame) -> *const c_char;
    fn stack_frame_source_file_name(frame: *const StackFrame) -> *const c_char;
    fn stack_frame_source_line(frame: *const StackFrame) -> c_int;
    fn stack_frame_trust(frame: *const StackFrame) -> FrameTrust;
}

impl StackFrame {
    pub fn instruction(&self) -> u64 {
        unsafe { stack_frame_instruction(self) }
    }

    pub fn module(&self) -> Option<&CodeModule> {
        unsafe { stack_frame_module(self).as_ref() }
    }

    pub fn function_name(&self) -> borrow::Cow<str> {
        unsafe {
            let ptr = stack_frame_function_name(self);
            ffi::CStr::from_ptr(ptr).to_string_lossy()
        }
    }

    pub fn source_file_name(&self) -> borrow::Cow<str> {
        unsafe {
            let ptr = stack_frame_source_file_name(self);
            ffi::CStr::from_ptr(ptr).to_string_lossy()
        }
    }

    pub fn source_line(&self) -> c_int {
        unsafe { stack_frame_source_line(self) }
    }

    pub fn trust(&self) -> FrameTrust {
        unsafe { stack_frame_trust(self) }
    }
}

impl fmt::Debug for StackFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StackFrame")
            .field("instruction", &self.instruction())
            .field("function_name", &self.function_name())
            .field("source_file_name", &self.source_file_name())
            .field("source_line", &self.source_line())
            .field("module", &self.module())
            .finish()
    }
}
