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

pub struct Resolver {
    internal: *mut Internal,
}

impl Resolver {
    pub fn new() -> Result<Resolver> {
        let internal = unsafe { resolver_new() };

        if internal.is_null() {
            let err = ErrorKind::ResolverError("Could not create resolver".into());
            Err(err.into())
        } else {
            Ok(Resolver { internal })
        }
    }

    pub fn load_symbols<S: Into<Vec<u8>>>(&mut self, module: &CodeModule, symbol_file: S) -> bool {
        let cstr = ffi::CString::new(symbol_file).unwrap();
        unsafe { resolver_load_symbols(self.internal, module, cstr.as_ptr()) }
    }

    pub fn fill_frame(&mut self, frame: &mut StackFrame) {
        unsafe { resolver_fill_frame(self.internal, frame) }
    }
}

impl Drop for Resolver {
    fn drop(&mut self) {
        unsafe { resolver_delete(self.internal) };
    }
}
