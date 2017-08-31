extern crate breakpad;
extern crate difference;

mod common;

use breakpad::{ProcessState, Resolver};
use common::{assert_snapshot, fixture_path};

#[test]
fn resolve_electron_stack_frame() {
    let state = ProcessState::from_minidump(fixture_path("electron.dmp")).unwrap();
    let thread = state.threads().first().unwrap();
    let frame = thread.frames()[1];

    let resolver = Resolver::new(fixture_path("Electron Framework.sym"))
        .expect("Could not load symbols for Electron Framework.");

    let resolved_frame = resolver.resolve_frame(&frame);
    assert_snapshot("resolved_frame.txt", &resolved_frame);
}

#[test]
fn create_corrupt_resolver() {
    let resolver = Resolver::new(fixture_path("Corrupt.sym"))
        .expect("Could not load symbols for Corrupt.");
    assert!(resolver.corrupt());
}
