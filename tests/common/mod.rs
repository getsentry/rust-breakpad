use difference::Changeset;
use std::{fmt, fs, io, path};
use std::io::prelude::*;

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

/// Resolves the full path to a fixture file.
pub fn fixture_path<S: AsRef<str>>(file_name: S) -> path::PathBuf {
    path::Path::new("tests").join("fixtures").join(file_name.as_ref())
}

/// Assets that the given object matches the snapshot saved in the snapshot
/// file. The object is serialized using the Debug trait.
///
/// If the value differs from the snapshot, the assertion fails and prints
/// a colored diff output.
pub fn assert_snapshot<S: AsRef<str>, T: fmt::Debug>(snapshot_name: S, val: &T) {
    let name = snapshot_name.as_ref();

    let output = format!("{:#?}", val);
    save_file(path::Path::new("tests").join("outputs").join(name), &output).unwrap_or_default();

    let snapshot = load_file(path::Path::new("tests").join("snapshots").join(name)).unwrap();
    assert_eq!(snapshot, output, "{}", Changeset::new(&snapshot, &output, "\n"));
}
