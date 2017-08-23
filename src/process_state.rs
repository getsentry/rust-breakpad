use std::{fmt, mem, slice};
use std::collections::HashSet;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::path::Path;

use call_stack::CallStack;
use code_module::CodeModule;
use errors::*;
use utils;

/// Result of processing a Minidump or Microdump file.
/// Usually included in `ProcessError` when the file cannot be processed.
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

    /// The dump processing was interrupted (not fatal).
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
/// To get source code information for `StackFrame`s, create a `Resolver` and
/// load all `CodeModules` included in one of the frames. To get a list of all
/// these modules use `referenced_modules`.
pub struct ProcessState {
    internal: *mut Internal,
}

impl ProcessState {
    /// Reads a minidump from the filesystem into memory and processes it.
    /// Returns a `ProcessState` that contains information about the crashed
    /// process.
    pub fn from_minidump<P: AsRef<Path>>(file_path: P) -> Result<ProcessState> {
        let bytes = utils::path_to_bytes(file_path.as_ref());
        let cstr = CString::new(bytes).unwrap();

        let mut result: ProcessResult = ProcessResult::Ok;
        let internal = unsafe { process_minidump(cstr.as_ptr(), &mut result) };

        if result == ProcessResult::Ok && !internal.is_null() {
            Ok(ProcessState { internal })
        } else {
            Err(ErrorKind::ProcessError(result).into())
        }
    }

    /// Returns a list of `CallStack`s in the minidump.
    pub fn threads(&self) -> &[&CallStack] {
        unsafe {
            let mut size = 0 as usize;
            let data = process_state_threads(self.internal, &mut size);
            let slice = slice::from_raw_parts(data, size);
            mem::transmute(slice)
        }
    }

    /// Returns a list of all `CodeModule`s referenced in one of the `CallStack`s.
    pub fn referenced_modules(&self) -> HashSet<&CodeModule> {
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
