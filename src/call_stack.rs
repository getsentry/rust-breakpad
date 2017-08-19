use std::{fmt, mem, slice};
use std::os::raw::c_void;

use stack_frame::StackFrame;

#[repr(C)]
pub struct CallStack(c_void);

extern "C" {
    fn call_stack_thread_id(stack: *const CallStack) -> u32;
    fn call_stack_frames(stack: *const CallStack, size_out: *mut usize)
        -> *const *const StackFrame;
}

impl CallStack {
    pub fn thread_id(&self) -> u32 {
        unsafe { call_stack_thread_id(self) }
    }

    pub fn frames(&self) -> &[&StackFrame] {
        unsafe {
            let mut size = 0 as usize;
            let data = call_stack_frames(self, &mut size);
            let slice = slice::from_raw_parts(data, size);
            mem::transmute(slice)
        }
    }
}

impl fmt::Debug for CallStack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CallStack")
            .field("thread_id", &self.thread_id())
            .field("frames", &self.frames())
            .finish()
    }
}
