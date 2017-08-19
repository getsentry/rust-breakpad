use std::{collections, fmt, mem, slice};
use std::os::raw::c_void;

use call_stack::CallStack;
use code_module::CodeModule;

type Internal = c_void;

extern "C" {
    fn process_state_delete(state: *mut Internal);
    fn process_state_threads(
        state: *const Internal,
        size_out: *mut usize,
    ) -> *const *const CallStack;
}

pub struct ProcessState {
    internal: *mut Internal,
}

impl ProcessState {
    pub fn new(internal: *mut Internal) -> ProcessState {
        ProcessState { internal }
    }

    pub fn threads(&self) -> &[&CallStack] {
        unsafe {
            let mut size = 0 as usize;
            let data = process_state_threads(self.internal, &mut size);
            let slice = slice::from_raw_parts(data, size);
            mem::transmute(slice)
        }
    }

    pub fn referenced_modules(&self) -> collections::HashSet<&CodeModule> {
        self.threads()
            .iter()
            .flat_map(|stack| stack.frames().iter())
            .filter_map(|frame| frame.module())
            .collect()
    }
}

impl Drop for ProcessState {
    fn drop(&mut self) {
        unsafe { process_state_delete(self.internal) };
    }
}

impl fmt::Debug for ProcessState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ProcessState")
            .field("threads", &self.threads())
            .finish()
    }
}
