extern crate breakpad;
extern crate difference;

mod common;

use std::collections::BTreeSet;
use std::fs::File;
use std::io::prelude::*;

use breakpad::{CodeModuleId, FrameInfoMap, ProcessState};
use common::{assert_snapshot, fixture_path, load_fixture};

#[test]
fn process_minidump_from_path() {
    let state = ProcessState::from_minidump_file(fixture_path("crash_macos.dmp"), None)
        .expect("Could not process minidump");

    assert_snapshot("process_state.txt", &state);
}

#[test]
fn process_minidump_from_buffer() {
    let mut buffer = Vec::new();
    let mut file = File::open(fixture_path("crash_macos.dmp")).expect("Could not open minidump");
    file.read_to_end(&mut buffer)
        .expect("Could not read minidump");

    let state = ProcessState::from_minidump_buffer(buffer.as_slice(), None)
        .expect("Could not process minidump");

    assert_snapshot("process_state.txt", &state);
}

#[test]
fn obtain_referenced_modules() {
    let state = ProcessState::from_minidump_file(fixture_path("crash_macos.dmp"), None)
        .expect("Could not process minidump");

    let modules: BTreeSet<_> = state.referenced_modules().iter().cloned().collect();
    assert_snapshot("referenced_modules.txt", &modules);
}

#[test]
fn get_minidump_process_state_cfi() {
    let module_id = CodeModuleId::parse("DFB8E43AF2423D73A453AEB6A777EF750")
        .expect("Could not parse CodeModule ID");
    let module_cfi = load_fixture("crash_macos_cfi.sym").expect("Could not load CFI symbols");

    let mut symbols = FrameInfoMap::new();
    symbols.insert(module_id, module_cfi.as_bytes());

    let state = ProcessState::from_minidump_file(fixture_path("crash_macos.dmp"), Some(&symbols))
        .expect("Could not process minidump");

    assert_snapshot("process_state_cfi.txt", &state);
}
