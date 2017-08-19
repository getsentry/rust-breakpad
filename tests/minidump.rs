extern crate breakpad;
extern crate difference;

use difference::Changeset;
use std::{collections, fs, io, path};
use std::io::prelude::*;

use breakpad::{CodeModule, Minidump};

fn fixture_path<S: AsRef<str>>(file_name: S) -> String {
    format!("tests/fixtures/{}", file_name.as_ref())
}

fn load_file<P: AsRef<path::Path>>(path: P) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn save_file<P: AsRef<path::Path>, S: AsRef<str>>(path: P, contents: S) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(contents.as_ref().as_bytes())?;
    Ok(())
}

fn assert_snapshot<S: AsRef<str>, T: std::fmt::Debug>(snapshot_name: S, obj: &T) {
    let name = snapshot_name.as_ref();

    let output = format!("{:#?}", obj);
    save_file(format!("tests/outputs/{}.txt", name), &output).unwrap_or_default();

    let snapshot = load_file(format!("tests/snapshots/{}.txt", name)).unwrap();
    assert_eq!(snapshot, output, "{}", Changeset::new(&snapshot, &output, "\n"));
}

#[test]
fn process_minidump() {
    let dump = Minidump::new(fixture_path("minidump.dmp")).unwrap();
    let state = dump.process().unwrap();
    assert_snapshot("process_state", &state);
}

#[test]
fn obtain_referenced_modules() {
    let dump = Minidump::new(fixture_path("minidump.dmp")).unwrap();
    let state = dump.process().unwrap();
    let modules: collections::BTreeSet<&CodeModule> =
        state.referenced_modules().iter().cloned().collect();

    assert_snapshot("referenced_modules", &modules);
}
