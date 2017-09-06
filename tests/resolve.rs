extern crate breakpad;
extern crate difference;

mod common;

use std::fs::File;
use std::io::prelude::*;

use breakpad::{ProcessState, Resolver};
use common::{assert_snapshot, fixture_path};

#[test]
fn create_from_file() {
    let resolver =
        Resolver::from_file(fixture_path("crash_macos_func.sym")).expect("Could not load symbols");
    assert!(!resolver.corrupt());
}

#[test]
fn create_from_buffer() {
    let mut buffer = Vec::new();
    let mut file =
        File::open(fixture_path("crash_macos_func.sym")).expect("Could not open symbols");
    file.read_to_end(&mut buffer)
        .expect("Could not read symbols");

    let resolver = Resolver::from_buffer(buffer.as_slice()).expect("Could not load symbols");
    assert!(!resolver.corrupt());
}

#[test]
fn create_corrupt_resolver() {
    let resolver =
        Resolver::from_file(fixture_path("Corrupt.sym")).expect("Could not load symbols");
    assert!(resolver.corrupt());
}

#[test]
fn resolve_stack_frame() {
    let state = ProcessState::from_minidump_file(fixture_path("crash_macos.dmp"), None).unwrap();
    let thread = state.threads().first().unwrap();
    let frame = thread.frames()[0];

    let resolver =
        Resolver::from_file(fixture_path("crash_macos_func.sym")).expect("Could not load symbols");

    let resolved_frame = resolver.resolve_frame(&frame);
    assert_snapshot("resolved_frame.txt", &resolved_frame);
}
