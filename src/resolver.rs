use std::os::raw::{c_char, c_void};
use std::path::Path;

use errors::Result;
use errors::ErrorKind::ResolverError;
use resolved_stack_frame::ResolvedStackFrame;
use stack_frame::StackFrame;
use utils;

pub type Internal = c_void;

extern "C" {
    fn resolver_new(buffer: *const c_char, buffer_size: usize) -> *mut Internal;
    fn resolver_delete(resolver: *mut Internal);
    fn resolver_is_corrupt(resolver: *const Internal) -> bool;
    fn resolver_resolve_frame(
        resolver: *const Internal,
        frame: *const StackFrame,
    ) -> *mut StackFrame;
}

/// Source line resolver for stack frames. Handles Breakpad symbol files and
/// searches them for instructions.
///
/// To use this resolver, obtain a list of referenced modules from a
/// ProcessState and load all of them into the resolver. Once symbols have
/// been loaded for a `CodeModule`, the resolver can fill frames with source
/// line information.
///
/// See `ResolvedStackFrame` for all available information.
pub struct Resolver {
    internal: *mut Internal,
}

impl Resolver {
    /// Creates a new `Resolver` instance from a Breakpad symbol file in the
    /// file system
    pub fn from_file<P: AsRef<Path>>(file_path: P) -> Result<Resolver> {
        let buffer = utils::read_buffer(file_path)?;
        Self::from_buffer(buffer.as_slice())
    }

    /// Creates a new `Resolver` instance from a buffer containing Breakpad symbols
    pub fn from_buffer(buffer: &[u8]) -> Result<Resolver> {
        let internal = unsafe { resolver_new(buffer.as_ptr() as *const c_char, buffer.len()) };

        if internal.is_null() {
            Err(ResolverError("Could not load symbols".into()).into())
        } else {
            Ok(Resolver { internal })
        }
    }

    /// Returns whether this `Resolver` is corrupt or it can be used to
    /// resolve source line locations of `StackFrames`.
    pub fn corrupt(&self) -> bool {
        unsafe { resolver_is_corrupt(self.internal) }
    }

    /// Tries to locate the frame's instruction in the loaded code modules.
    /// Returns a resolved stack frame instance. If no  symbols can be found
    /// for the frame, a clone of the input is returned.
    pub fn resolve_frame(&self, frame: &StackFrame) -> ResolvedStackFrame {
        let ptr = unsafe { resolver_resolve_frame(self.internal, frame) };
        ResolvedStackFrame::from_ptr(ptr)
    }
}

impl Drop for Resolver {
    fn drop(&mut self) {
        unsafe { resolver_delete(self.internal) };
    }
}
