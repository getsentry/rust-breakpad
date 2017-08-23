extern crate breakpad;
extern crate difference;

mod common;

use std::collections;

use breakpad::{CodeModule, ProcessState, Resolver};
use common::{assert_snapshot, fixture_path};

/// Process a minidump file
fn process_minidump<S: AsRef<str>>(file_name: S) -> ProcessState {
    ProcessState::from_minidump(fixture_path(file_name)).unwrap()
}

#[test]
fn get_minidump_process_state() {
    let state = process_minidump("electron.dmp");
    assert_snapshot("process_state.txt", &state);
}

#[test]
fn obtain_referenced_modules() {
    let state = process_minidump("electron.dmp");
    let modules: collections::BTreeSet<&CodeModule> =
        state.referenced_modules().iter().cloned().collect();

    assert_snapshot("referenced_modules.txt", &modules);
}

#[test]
fn resolve_electron_stack_frame() {
    let state = process_minidump("electron.dmp");
    let thread = state.threads().first().unwrap();
    let frame = thread.frames()[1];
    let module = frame.module().unwrap();

    let resolver = Resolver::new()
        .expect("Could not allocate the resolver.");

    resolver.load_symbols(&module, fixture_path("Electron Framework.sym"))
        .expect("Could not load symbols for Electron Framework");

    let resolved_frame = resolver.resolve_frame(&frame);
    assert_snapshot("resolved_frame.txt", &resolved_frame);
}
