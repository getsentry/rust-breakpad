use std::{collections, ffi, fmt, mem, path, slice};
use std::os::raw::{c_char, c_void};

use call_stack::CallStack;
use errors::*;
use code_module::CodeModule;
use utils::path_to_bytes;

/// Return type for Minidump or Microdump processors
#[repr(C)]
#[derive(Debug, Eq, PartialEq)]
pub enum ProcessResult {
    /// The dump was processed successfully.
    Ok,

    /// The minidump file was not found.
    MinidumpNotFound,

    /// The minidump file had no header.
    NoMinidumpHeader,

    /// The minidump file has no thread list.
    ErrorNoThreadList,

    /// There was an error getting one thread's data from the dump.
    ErrorGettingThread,

    /// There was an error getting a thread id from the thread's data.
    ErrorGettingThreadId,

    /// There was more than one requesting thread.
    DuplicateRequestingThreads,

    /// The dump processing was interrupted by the SymbolSupplier(not fatal).
    SymbolSupplierInterrupted,
}

impl fmt::Display for ProcessResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            &ProcessResult::Ok => "Dump processed successfully",
            &ProcessResult::MinidumpNotFound => "Minidump file was not found",
            &ProcessResult::NoMinidumpHeader => "Minidump file had no header",
            &ProcessResult::ErrorNoThreadList => "Minidump file has no thread list",
            &ProcessResult::ErrorGettingThread => "Error getting one thread's data",
            &ProcessResult::ErrorGettingThreadId => "Error getting a thread id",
            &ProcessResult::DuplicateRequestingThreads => "There was more than one requesting thread",
            &ProcessResult::SymbolSupplierInterrupted => "Processing was interrupted (not fatal)",
        })
    }
}

type Internal = c_void;

extern "C" {
    fn process_minidump(file_path: *const c_char, result: *mut ProcessResult) -> *mut Internal;
    fn process_state_delete(state: *mut Internal);
    fn process_state_threads(
        state: *const Internal,
        size_out: *mut usize,
    ) -> *const *const CallStack;
}

/// Snapshot of the state of a processes during its crash. The object can be
/// obtained by processing Minidump or Microdump files.
///
/// To get source code information for stack frames, create a Resolver and
/// load all referenced modules.
pub struct ProcessState {
    internal: *mut Internal,
}

impl ProcessState {
    /// Reads a minidump from the filesystem into memory and processes it.
    /// Returns a ProcessState that contains information about the crashed
    /// process.
    pub fn from_minidump<P: AsRef<path::Path>>(file_path: P) -> Result<ProcessState> {
        let bytes = path_to_bytes(file_path.as_ref());
        let cstr = ffi::CString::new(bytes).unwrap();

        let mut result: ProcessResult = ProcessResult::Ok;
        let internal = unsafe { process_minidump(cstr.as_ptr(), &mut result) };

        if result == ProcessResult::Ok && !internal.is_null() {
            Ok(ProcessState { internal })
        } else {
            Err(ErrorKind::ProcessError(result).into())
        }
    }

    /// Returns a list of threads in the minidump.
    pub fn threads(&self) -> &[&CallStack] {
        unsafe {
            let mut size = 0 as usize;
            let data = process_state_threads(self.internal, &mut size);
            let slice = slice::from_raw_parts(data, size);
            mem::transmute(slice)
        }
    }

    /// Returns a list of all modules referenced in one of the call stacks.
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
