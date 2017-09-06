#[macro_use]
extern crate error_chain;
extern crate uuid;

mod call_stack;
mod code_module;
mod errors;
mod process_state;
mod resolved_stack_frame;
mod resolver;
mod stack_frame;
mod utils;

pub use call_stack::CallStack;
pub use code_module::{CodeModule, CodeModuleId};
pub use errors::*;
pub use process_state::{FrameInfoMap, ProcessResult, ProcessState};
pub use resolved_stack_frame::ResolvedStackFrame;
pub use resolver::Resolver;
pub use stack_frame::{FrameTrust, StackFrame};
