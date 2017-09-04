use std::fmt;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::os::raw::{c_char, c_void};
use uuid::Uuid;

use errors::{Error, Result};
use errors::ErrorKind::ParseIdError;
use utils;

/// Unique identifier of a `CodeModule`
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CodeModuleId {
    uuid: Uuid,
    age: u32,
}

impl CodeModuleId {
    pub fn parse(input: &str) -> Result<CodeModuleId> {
        let uuid = Uuid::parse_str(&input[..32])
            .map_err(|_| Error::from(ParseIdError("Could not parse UUID".into())))?;
        let age = u32::from_str_radix(&input[32..], 16)
            .map_err(|_| Error::from(ParseIdError("Could not parse age".into())))?;
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

extern "C" {
    fn code_module_base_address(module: *const CodeModule) -> u64;
    fn code_module_size(module: *const CodeModule) -> u64;
    fn code_module_code_file(module: *const CodeModule) -> *mut c_char;
    fn code_module_code_identifier(module: *const CodeModule) -> *mut c_char;
    fn code_module_debug_file(module: *const CodeModule) -> *mut c_char;
    fn code_module_debug_identifier(module: *const CodeModule) -> *mut c_char;
}

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
    assert!(CodeModuleId::parse("DFB8E43AF2423D73A453AEB6A777EF75").is_err());
}
