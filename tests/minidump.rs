extern crate breakpad;
extern crate difference;

use difference::Changeset;
use std::{collections, fs, io, path};
use std::io::prelude::*;

use breakpad::{CodeModule, ProcessState, Resolver};

/// Resolves the full path to a fixture file.
fn fixture_path<S: AsRef<str>>(file_name: S) -> path::PathBuf {
    path::Path::new("tests").join("fixtures").join(file_name.as_ref())
}

/// Loads the file at the given location and returns its contents as string.
fn load_file<P: AsRef<path::Path>>(path: P) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Saves the given string into the specified file path.
fn save_file<P: AsRef<path::Path>, S: AsRef<str>>(path: P, contents: S) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(contents.as_ref().as_bytes())?;
    Ok(())
}

/// Assets that the given object matches the snapshot saved in the snapshot
/// file. The object is serialized using the Debug trait.
///
/// If the value differs from the snapshot, the assertion fails and prints
/// a colored diff output.
fn assert_snapshot<S: AsRef<str>, T: std::fmt::Debug>(snapshot_name: S, val: &T) {
    let name = snapshot_name.as_ref();

    let output = format!("{:#?}", val);
    save_file(path::Path::new("tests").join("outputs").join(name), &output).unwrap_or_default();

    let snapshot = load_file(path::Path::new("tests").join("snapshots").join(name)).unwrap();
    assert_eq!(snapshot, output, "{}", Changeset::new(&snapshot, &output, "\n"));
}

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
