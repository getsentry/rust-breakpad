use std::{fmt, mem, ptr, slice};
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::path::Path;

use call_stack::CallStack;
use code_module::{CodeModule, CodeModuleId};
use errors::ErrorKind::ProcessError;
use errors::Result;
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
        let formatted = match self {
            &ProcessResult::Ok => "Dump processed successfully",
            &ProcessResult::MinidumpNotFound => "Minidump file was not found",
            &ProcessResult::NoMinidumpHeader => "Minidump file had no header",
            &ProcessResult::ErrorNoThreadList => "Minidump file has no thread list",
            &ProcessResult::ErrorGettingThread => "Error getting one thread's data",
            &ProcessResult::ErrorGettingThreadId => "Error getting a thread id",
            &ProcessResult::DuplicateRequestingThreads => {
                "There was more than one requesting thread"
            }
            &ProcessResult::SymbolSupplierInterrupted => "Processing was interrupted (not fatal)",
        };

        write!(f, "{}", formatted)
    }
}

type Internal = c_void;

/// Internal type used to transfer Breakpad symbols over FFI
#[repr(C)]
struct SymbolEntry {
    debug_identifier: *const c_char,
    symbol_size: usize,
    symbol_data: *const u8,
}

extern "C" {
    fn process_minidump(
        buffer: *const c_char,
        buffer_size: usize,
        symbols: *const SymbolEntry,
        symbol_count: usize,
        result: *mut ProcessResult,
    ) -> *mut Internal;
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

/// Contains stack frame information for `CodeModules`
///
/// This information is required by the stackwalker in case framepointers are
/// missing in the raw stacktraces. Frame information is given as plain ASCII
/// text as specified in the Breakpad symbol file specification.
pub type FrameInfoMap<'a> = BTreeMap<CodeModuleId, &'a [u8]>;

impl ProcessState {
    /// Reads a minidump from the filesystem into memory and processes it
    ///
    /// Returns a `ProcessState` that contains information about the crashed
    /// process. The parameter `frame_infos` expects a map of Breakpad symbols
    /// containing STACK CFI and STACK WIN records to allow stackwalking with
    /// omitted frame pointers.
    pub fn from_minidump_file<P: AsRef<Path>>(
        file_path: P,
        frame_infos: Option<&FrameInfoMap>,
    ) -> Result<ProcessState> {
        let buffer = utils::read_buffer(file_path)?;
        Self::from_minidump_buffer(buffer.as_slice(), frame_infos)
    }

    /// Processes a minidump supplied via raw binary data
    ///
    /// Returns a `ProcessState` that contains information about the crashed
    /// process. The parameter `frame_infos` expects a map of Breakpad symbols
    /// containing STACK CFI and STACK WIN records to allow stackwalking with
    /// omitted frame pointers.
    pub fn from_minidump_buffer(
        buffer: &[u8],
        frame_infos: Option<&FrameInfoMap>,
    ) -> Result<ProcessState> {
        let cfi_count = frame_infos.map_or(0, |s| s.len());
        let mut result: ProcessResult = ProcessResult::Ok;

        // Keep a reference to all CStrings to extend their lifetime
        let cfi_vec: Vec<_> = frame_infos.map_or(Vec::new(), |s| {
            s.iter()
                .map(|(k, v)| (CString::new(k.to_string()), v.len(), v.as_ptr()))
                .collect()
        });

        // Keep a reference to all symbol entries to extend their lifetime
        let cfi_entries: Vec<_> = cfi_vec
            .iter()
            .map(|&(ref id, size, data)| {
                SymbolEntry {
                    debug_identifier: id.as_ref().map(|i| i.as_ptr()).unwrap_or(ptr::null()),
                    symbol_size: size,
                    symbol_data: data,
                }
            })
            .collect();

        let internal = unsafe {
            process_minidump(
                buffer.as_ptr() as *const c_char,
                buffer.len(),
                cfi_entries.as_ptr(),
                cfi_count,
                &mut result,
            )
        };

        if result == ProcessResult::Ok && !internal.is_null() {
            Ok(ProcessState { internal })
        } else {
            Err(ProcessError(result).into())
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
