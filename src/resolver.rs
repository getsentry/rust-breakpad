use std::{ffi, path};
use std::os::raw::{c_char, c_void};

use code_module::CodeModule;
use errors::*;
use resolved_stack_frame::ResolvedStackFrame;
use stack_frame::StackFrame;
use utils::path_to_bytes;

pub type Internal = c_void;

extern "C" {
    fn resolver_new() -> *mut Internal;
    fn resolver_delete(resolver: *mut Internal);
    fn resolver_load_symbols(
        resolver: *mut Internal,
        module: *const CodeModule,
        symbol_file: *const c_char,
    ) -> bool;
    fn resolver_resolve_frame(resolver: *mut Internal, frame: *const StackFrame)
        -> *mut StackFrame;
}

/// Source line resolver for stack frames. Handles Breakpad symbol files and
/// searches them for instructions.
///
/// To use this resolver, obtain a list of referenced modules from a
/// ProcessState and load all of them into the resolver. Once symbols have
/// been loaded for a code module, the resolver can fill frames with source
/// line information.
///
/// See StackFrame for all available information.
pub struct Resolver {
    internal: *mut Internal,
}

impl Resolver {
    /// Creates a new resolver.
    pub fn new() -> Result<Resolver> {
        let internal = unsafe { resolver_new() };

        if internal.is_null() {
            let err = ErrorKind::ResolverError("Could not create resolver".into());
            Err(err.into())
        } else {
            Ok(Resolver { internal })
        }
    }

    /// Adds symbols for the given code module from a Breakpad symbol file in
    /// the file system.
    pub fn load_symbols<P: AsRef<path::Path>>(&self, module: &CodeModule, file_path: P) -> Result<()> {
        let bytes = path_to_bytes(file_path.as_ref());
        let cstr = ffi::CString::new(bytes).unwrap();

        if unsafe { resolver_load_symbols(self.internal, module, cstr.as_ptr()) } {
            Ok(())
        } else {
            let err = ErrorKind::ResolverError("Could not load symbols".into());
            Err(err.into())
        }
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
