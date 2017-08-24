use std::os::raw::c_char;
use std::path::Path;
use std::ptr;

use errors::*;
use utils;

extern "C" {
    fn create_symbols(file_path: *const c_char, secondary_path: *const c_char) -> *mut c_char;
}

/// Converts debug symbols to platform independent Breakpad symbol file in
/// ASCII format. On some systems, debug symbols are extracted into a
/// secondary file (e.g. dSYM on Darwin). In this case, specify this file
/// in `secondary_path`.
pub fn convert_symbols<P: AsRef<Path>>(file_path: P, secondary_path: Option<P>) -> Result<String> {
    let file_cstr = utils::path_to_str(file_path);
    let secondary_cstr = secondary_path.map(|p| utils::path_to_str(p));

    unsafe {
        let ptr = create_symbols(file_cstr.as_ptr(), secondary_cstr.as_ref().map_or(ptr::null(), |s| s.as_ptr()));
        if ptr.is_null() {
            let err = ErrorKind::ConversionError("Files not found or invalid".into());
            Err(err.into())
        } else {
            Ok(utils::ptr_to_string(ptr))
        }
    }
}
