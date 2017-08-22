use std::fmt;
use std::os::raw::c_void;

use code_module::CodeModule;

/// Indicates how well the instruction pointer derived during
/// stack walking is trusted. Since the stack walker can resort to
/// stack scanning, it can wind up with dubious frames.
///
/// In rough order of "trust metric".
#[repr(C)]
#[derive(Debug)]
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

/// Contains information from the memorydump, especially the frame's instruction
/// pointer. Also references an optional `CodeModule` that contains the
/// instruction of this stack frame.
///
/// Use a `Resolver` to fill a stack frame with source code information. The
/// resolver needs symbols for this frame's `CodeModule` in order to provide
/// debug information.
#[repr(C)]
pub struct StackFrame(c_void);

extern "C" {
    fn stack_frame_instruction(frame: *const StackFrame) -> u64;
    fn stack_frame_module(frame: *const StackFrame) -> *const CodeModule;
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
    /// Use `trust` to obtain how trustworthy this instruction is.
    pub fn instruction(&self) -> u64 {
        unsafe { stack_frame_instruction(self) }
    }

    /// Returns the `CodeModule` that contains this frame's instruction.
    pub fn module(&self) -> Option<&CodeModule> {
        unsafe { stack_frame_module(self).as_ref() }
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
            .field("trust", &self.trust())
            .field("module", &self.module())
            .finish()
    }
}
