extern crate breakpad;
extern crate difference;

mod common;

use breakpad::{ProcessState, Resolver};
use common::{assert_snapshot, fixture_path};

#[test]
fn resolve_stack_frame() {
    let state = ProcessState::from_minidump_path(fixture_path("crash_macos.dmp"), None).unwrap();
    let thread = state.threads().first().unwrap();
    let frame = thread.frames()[0];

    let resolver =
        Resolver::new(fixture_path("crash_macos_func.sym")).expect("Could not load symbols");

    let resolved_frame = resolver.resolve_frame(&frame);
    assert_snapshot("resolved_frame.txt", &resolved_frame);
}

#[test]
fn create_corrupt_resolver() {
    let resolver = Resolver::new(fixture_path("Corrupt.sym")).expect("Could not load symbols");
    assert!(resolver.corrupt());
}
