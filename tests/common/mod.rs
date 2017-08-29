#![allow(dead_code)]

use difference::Changeset;
use std::{fmt, io};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::prelude::*;

/// Loads the file at the given location and returns its contents as string.
fn load_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Saves the given string into the specified file path.
fn save_file<P: AsRef<Path>, S: AsRef<str>>(path: P, contents: S) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(contents.as_ref().as_bytes())?;
    Ok(())
}

/// Resolves the full path to a fixture file.
pub fn fixture_path<S: AsRef<str>>(file_name: S) -> PathBuf {
    Path::new("tests")
        .join("fixtures")
        .join(file_name.as_ref())
}

/// Assets that the given object matches the snapshot saved in the snapshot
/// file. The object is serialized using the Debug trait.
///
/// If the value differs from the snapshot, the assertion fails and prints
/// a colored diff output.
pub fn assert_snapshot<S: AsRef<str>, T: fmt::Debug>(snapshot_name: S, val: &T) {
    assert_snapshot_plain(snapshot_name, &format!("{:#?}", val));
}

/// Assets that the given string matches the snapshot saved in the snapshot
/// file. The given string will be used as plain output and directly compared
/// with the stored snapshot.
///
/// If the value differs from the snapshot, the assertion fails and prints
/// a colored diff output.
pub fn assert_snapshot_plain<S: AsRef<str>>(snapshot_name: S, output: &str) {
    let name = snapshot_name.as_ref();

    let output_path = Path::new("tests").join("outputs").join(name);
    save_file(output_path, &output).unwrap_or_default();

    let snapshot_path = Path::new("tests").join("snapshots").join(name);
    let snapshot = load_file(snapshot_path).unwrap_or("".into());
    assert!(
        snapshot == output,
        "Value does not match stored snapshot {}:\n\n{}",
        name,
        Changeset::new(&snapshot, &output, "\n")
    );
}
