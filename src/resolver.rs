use std::ffi;
use std::os::raw::{c_char, c_void};

use code_module::CodeModule;
use errors::*;
use stack_frame::StackFrame;

pub type Internal = c_void;

extern "C" {
    fn resolver_new() -> *mut Internal;
    fn resolver_delete(resolver: *mut Internal);
    fn resolver_load_symbols(
        resolver: *mut Internal,
        module: *const CodeModule,
        symbol_file: *const c_char,
    ) -> bool;
    fn resolver_fill_frame(resolver: *mut Internal, frame: *mut StackFrame);
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
    pub fn load_symbols<S: Into<Vec<u8>>>(&mut self, module: &CodeModule, symbol_file: S) -> Result<()> {
        let cstr = ffi::CString::new(symbol_file).unwrap();
        if unsafe { resolver_load_symbols(self.internal, module, cstr.as_ptr()) } {
            Ok(())
        } else {
            let err = ErrorKind::ResolverError("Could not load symbols".into());
            Err(err.into())
        }
    }

    /// Tries to locate the frame's instruction in the loaded code modules. On
    /// success, it writes all source line information to the frame. If no
    /// symbols for the referenced code module have been loaded, the frame
    /// remains untouched.
    pub fn fill_frame(&mut self, frame: &mut StackFrame) {
        unsafe { resolver_fill_frame(self.internal, frame) }
    }
}

impl Drop for Resolver {
    fn drop(&mut self) {
        unsafe { resolver_delete(self.internal) };
    }
}
