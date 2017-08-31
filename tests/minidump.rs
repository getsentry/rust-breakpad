extern crate breakpad;
extern crate difference;

mod common;

use std::collections::BTreeSet;

use breakpad::ProcessState;
use common::{assert_snapshot, fixture_path};

/// Process a minidump file
fn process_minidump<S: AsRef<str>>(file_name: S) -> ProcessState {
    ProcessState::from_minidump(fixture_path(file_name)).unwrap()
}

#[test]
fn get_minidump_process_state() {
    let state = process_minidump("crash_macos.dmp");
    assert_snapshot("process_state.txt", &state);
}

#[test]
fn obtain_referenced_modules() {
    let state = process_minidump("crash_macos.dmp");
    let modules: BTreeSet<_> = state.referenced_modules().iter().cloned().collect();

    assert_snapshot("referenced_modules.txt", &modules);
}
