use std::{fmt, mem, ptr, slice};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::ffi::CString;
use std::hash::{Hash, Hasher};
use std::os::raw::{c_char, c_void};
use std::path::Path;
use uuid::Uuid;

use errors::ErrorKind::{ParseIdError, ProcessError};
use errors::Result;
use utils;

extern "C" {
    fn code_module_base_address(module: *const CodeModule) -> u64;
    fn code_module_size(module: *const CodeModule) -> u64;
    fn code_module_code_file(module: *const CodeModule) -> *mut c_char;
    fn code_module_code_identifier(module: *const CodeModule) -> *mut c_char;
    fn code_module_debug_file(module: *const CodeModule) -> *mut c_char;
    fn code_module_debug_identifier(module: *const CodeModule) -> *mut c_char;

    fn stack_frame_instruction(frame: *const StackFrame) -> u64;
    fn stack_frame_module(frame: *const StackFrame) -> *const CodeModule;
    fn stack_frame_trust(frame: *const StackFrame) -> FrameTrust;

    fn call_stack_thread_id(stack: *const CallStack) -> u32;
    fn call_stack_frames(stack: *const CallStack, size_out: *mut usize)
        -> *const *const StackFrame;

    fn process_minidump(
        buffer: *const c_char,
        buffer_size: usize,
        symbols: *const SymbolEntry,
        symbol_count: usize,
        result: *mut ProcessResult,
    ) -> *mut IProcessState;
    fn process_state_delete(state: *mut IProcessState);
    fn process_state_threads(
        state: *const IProcessState,
        size_out: *mut usize,
    ) -> *const *const CallStack;
}

/// Unique identifier of a `CodeModule`
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub struct CodeModuleId {
    uuid: Uuid,
    age: u32,
}

impl CodeModuleId {
    /// Parses a CodeModuleId from a 33 character `String`
    pub fn parse(input: &str) -> Result<CodeModuleId> {
        if input.len() != 33 {
            return Err(ParseIdError("Invalid input string length".into()).into());
        }

        let uuid = Uuid::parse_str(&input[..32])?;
        let age = u32::from_str_radix(&input[32..], 16)?;
        Ok(CodeModuleId { uuid, age })
    }

    /// Returns the UUID part of the code module's debug_identifier
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    /// Returns the age part of the code module's debug identifier
    ///
    /// On Windows, this is an incrementing counter to identify the build.
    /// On all other platforms, this value will always be zero.
    pub fn age(&self) -> u32 {
        self.age
    }
}

impl fmt::Display for CodeModuleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let uuid = self.uuid.simple().to_string().to_uppercase();
        write!(f, "{}{:X}", uuid, self.age)
    }
}

impl Into<String> for CodeModuleId {
    fn into(self) -> String {
        self.to_string()
    }
}

/// Carries information about a code module loaded into the process during the
/// crash. The `debug_identifier` uniquely identifies this module.
#[repr(C)]
pub struct CodeModule(c_void);

impl CodeModule {
    /// Returns the unique identifier of this `CodeModule`.
    pub fn id(&self) -> CodeModuleId {
        CodeModuleId::parse(&self.debug_identifier()).unwrap()
    }

    /// Returns the base address of this code module as it was loaded by the
    /// process. (uint64_t)-1 on error.
    pub fn base_address(&self) -> u64 {
        unsafe { code_module_base_address(self) }
    }

    /// The size of the code module. 0 on error.
    pub fn size(&self) -> u64 {
        unsafe { code_module_size(self) }
    }

    // Returns the path or file name that the code module was loaded from.
    pub fn code_file(&self) -> String {
        unsafe {
            let ptr = code_module_code_file(self);
            utils::ptr_to_string(ptr)
        }
    }

    // An identifying string used to discriminate between multiple versions and
    // builds of the same code module.  This may contain a UUID, timestamp,
    // version number, or any combination of this or other information, in an
    // implementation-defined format.
    pub fn code_identifier(&self) -> String {
        unsafe {
            let ptr = code_module_code_identifier(self);
            utils::ptr_to_string(ptr)
        }
    }

    /// Returns the filename containing debugging information of this code
    /// module.  If debugging information is stored in a file separate from the
    /// code module itself (as is the case when .pdb or .dSYM files are used),
    /// this will be different from `code_file`.  If debugging information is
    /// stored in the code module itself (possibly prior to stripping), this
    /// will be the same as code_file.
    pub fn debug_file(&self) -> String {
        unsafe {
            let ptr = code_module_debug_file(self);
            utils::ptr_to_string(ptr)
        }
    }

    /// Returns a string identifying the specific version and build of the
    /// associated debug file.  This may be the same as `code_identifier` when
    /// the `debug_file` and `code_file` are identical or when the same identifier
    /// is used to identify distinct debug and code files.
    ///
    /// It usually comprises the library's UUID and an age field. On Windows, the
    /// age field is a generation counter, on all other platforms it is mostly
    /// zero.
    pub fn debug_identifier(&self) -> String {
        unsafe {
            let ptr = code_module_debug_identifier(self);
            utils::ptr_to_string(ptr)
        }
    }
}

impl Eq for CodeModule {}

impl PartialEq for CodeModule {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Hash for CodeModule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state)
    }
}

impl Ord for CodeModule {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id().cmp(&other.id())
    }
}

impl PartialOrd for CodeModule {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Debug for CodeModule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CodeModule")
            .field("id", &self.id())
            .field("base_address", &self.base_address())
            .field("size", &self.size())
            .field("code_file", &self.code_file())
            .field("code_identifier", &self.code_identifier())
            .field("debug_file", &self.debug_file())
            .field("debug_identifier", &self.debug_identifier())
            .finish()
    }
}

#[test]
fn test_parse() {
    assert_eq!(
        CodeModuleId::parse("DFB8E43AF2423D73A453AEB6A777EF75A").unwrap(),
        CodeModuleId {
            uuid: Uuid::parse_str("DFB8E43AF2423D73A453AEB6A777EF75").unwrap(),
            age: 10,
        }
    );
}

#[test]
fn test_to_string() {
    let id = CodeModuleId {
        uuid: Uuid::parse_str("DFB8E43AF2423D73A453AEB6A777EF75").unwrap(),
        age: 10,
    };

    assert_eq!(id.to_string(), "DFB8E43AF2423D73A453AEB6A777EF75A");
}

#[test]
fn test_parse_error() {
    assert!(CodeModuleId::parse("DFB8E43AF2423D73A").is_err());
}

/// Indicates how well the instruction pointer derived during
/// stack walking is trusted. Since the stack walker can resort to
/// stack scanning, it can wind up with dubious frames.
///
/// In rough order of "trust metric".
#[repr(C)]
#[derive(Debug)]
pub enum FrameTrust {
    /// Unknown trust.
    None,

    /// Scanned the stack, found this (lowest precision).
    Scan,

    /// Found while scanning stack using call frame info.
    CFIScan,

    /// Derived from frame pointer.
    FP,

    /// Derived from call frame info.
    CFI,

    /// Explicitly provided by some external stack walker.
    Prewalked,

    /// Given as instruction pointer in a context (highest precision).
    Context,
}

/// Contains information from the memorydump, especially the frame's instruction
/// pointer. Also references an optional `CodeModule` that contains the
/// instruction of this stack frame.
///
/// Use a `Resolver` to fill a stack frame with source code information. The
/// resolver needs symbols for this frame's `CodeModule` in order to provide
/// debug information.
#[repr(C)]
pub struct StackFrame(c_void);

impl StackFrame {
    /// Returns the program counter location as an absolute virtual address.
    ///
    /// - For the innermost called frame in a stack, this will be an exact
    ///   program counter or instruction pointer value.
    ///
    /// - For all other frames, this address is within the instruction that
    ///   caused execution to branch to this frame's callee (although it may
    ///   not point to the exact beginning of that instruction). This ensures
    ///   that, when we look up the source code location for this frame, we
    ///   get the source location of the call, not of the point at which
    ///   control will resume when the call returns, which may be on the next
    ///   line. (If the compiler knows the callee never returns, it may even
    ///   place the call instruction at the very end of the caller's machine
    ///   code, such that the "return address" (which will never be used)
    ///   immediately after the call instruction is in an entirely different
    ///   function, perhaps even from a different source file.)
    ///
    /// On some architectures, the return address as saved on the stack or in
    /// a register is fine for looking up the point of the call. On others, it
    /// requires adjustment. ReturnAddress returns the address as saved by the
    /// machine.
    ///
    /// Use `trust` to obtain how trustworthy this instruction is.
    pub fn instruction(&self) -> u64 {
        unsafe { stack_frame_instruction(self) }
    }

    /// Returns the `CodeModule` that contains this frame's instruction.
    pub fn module(&self) -> Option<&CodeModule> {
        unsafe { stack_frame_module(self).as_ref() }
    }

    /// Returns how well the instruction pointer is trusted.
    pub fn trust(&self) -> FrameTrust {
        unsafe { stack_frame_trust(self) }
    }
}

impl fmt::Debug for StackFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StackFrame")
            .field("instruction", &self.instruction())
            .field("trust", &self.trust())
            .field("module", &self.module())
            .finish()
    }
}

/// Represents a thread of the `ProcessState` which holds a list of `StackFrame`s.
#[repr(C)]
pub struct CallStack(c_void);

impl CallStack {
    /// Returns the thread identifier of this callstack.
    pub fn thread_id(&self) -> u32 {
        unsafe { call_stack_thread_id(self) }
    }

    /// Returns the list of `StackFrame`s in the call stack.
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

/// Internal type used to transfer Breakpad symbols over FFI
#[repr(C)]
struct SymbolEntry {
    debug_identifier: *const c_char,
    symbol_size: usize,
    symbol_data: *const u8,
}

type IProcessState = c_void;

/// Snapshot of the state of a processes during its crash. The object can be
/// obtained by processing Minidump or Microdump files.
///
/// To get source code information for `StackFrame`s, create a `Resolver` and
/// load all `CodeModules` included in one of the frames. To get a list of all
/// these modules use `referenced_modules`.
pub struct ProcessState {
    internal: *mut IProcessState,
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
