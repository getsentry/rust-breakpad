use std::{borrow, ffi, fmt};
use std::os::raw::{c_char, c_int, c_void};

use code_module::CodeModule;

/// Indicates how well the instruction pointer derived during
/// stack walking is trusted. Since the stack walker can resort to
/// stack scanning, it can wind up with dubious frames.
/// In rough order of "trust metric".
#[repr(C)]
pub enum FrameTrust {
    /// Unknown trust.
    None,

    /// Scanned the stack, found this (lowest precision).
    Scan,

    /// Found while scanning stack using call frame info.
    CFIScan,

    /// Derived from frame pointer.
    FP,

    /// Derived from call frame info.
    CFI,

    /// Explicitly provided by some external stack walker.
    Prewalked,

    /// Given as instruction pointer in a context (highest precision).
    Context,
}

/// Contains information from the stackdump, especially the frame's instruction
/// pointer. After being processed by a resolver, this struct also contains
/// source code locations and code offsets.
///
/// Use a Resolver o fill a stack frame with source code information. The
/// resolver needs symbols for this frame's the code module in order to provide
/// debug information.
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
    /// Returns the program counter location as an absolute virtual address.
    ///
    /// - For the innermost called frame in a stack, this will be an exact
    ///   program counter or instruction pointer value.
    ///
    /// - For all other frames, this address is within the instruction that
    ///   caused execution to branch to this frame's callee (although it may
    ///   not point to the exact beginning of that instruction). This ensures
    ///   that, when we look up the source code location for this frame, we
    ///   get the source location of the call, not of the point at which
    ///   control will resume when the call returns, which may be on the next
    ///   line. (If the compiler knows the callee never returns, it may even
    ///   place the call instruction at the very end of the caller's machine
    ///   code, such that the "return address" (which will never be used)
    ///   immediately after the call instruction is in an entirely different
    ///   function, perhaps even from a different source file.)
    ///
    /// On some architectures, the return address as saved on the stack or in
    /// a register is fine for looking up the point of the call. On others, it
    /// requires adjustment. ReturnAddress returns the address as saved by the
    /// machine.
    ///
    /// Use stack_frame_trust to obtain how trustworthy this instruction is.
    pub fn instruction(&self) -> u64 {
        unsafe { stack_frame_instruction(self) }
    }

    /// Returns the code module that contains this frame's instruction.
    pub fn module(&self) -> Option<&CodeModule> {
        unsafe { stack_frame_module(self).as_ref() }
    }

    /// Returns the function name that contains the instruction. Can be empty
    /// before running the resolver or if debug symbols are missing.
    pub fn function_name(&self) -> borrow::Cow<str> {
        unsafe {
            let ptr = stack_frame_function_name(self);
            ffi::CStr::from_ptr(ptr).to_string_lossy()
        }
    }

    /// Returns the source code line at which the instruction was declared.
    /// Can be empty before running the resolver or if debug symbols are
    /// missing.
    pub fn source_file_name(&self) -> borrow::Cow<str> {
        unsafe {
            let ptr = stack_frame_source_file_name(self);
            ffi::CStr::from_ptr(ptr).to_string_lossy()
        }
    }

    /// Returns the source code line at which the instruction was declared. Can
    /// be empty before running the resolver or if debug symbols are missing.
    pub fn source_line(&self) -> c_int {
        unsafe { stack_frame_source_line(self) }
    }

    /// Returns how well the instruction pointer is trusted.
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
