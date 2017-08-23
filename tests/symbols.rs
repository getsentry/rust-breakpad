extern crate breakpad;
extern crate difference;

mod common;

use breakpad::{convert_symbols};
use common::{assert_snapshot, fixture_path};

#[test]
#[cfg(target_os = "macos")]
fn convert_macos_symbols() {
    let primary = fixture_path("hello_macos");
    let secondary = fixture_path("hello_macos.dSYM");
    let symbols = convert_symbols(primary, Some(secondary))
        .unwrap_or("None".into());

    assert_snapshot("symbols_macos.txt", &symbols);
}
