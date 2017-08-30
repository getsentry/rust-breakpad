extern crate breakpad;

use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

use breakpad::convert_symbols;
use breakpad::ProcessState;
use breakpad::Resolver;

#[test]
fn test_example() {
    let state = ProcessState::from_minidump(Path::new("examples/target/example.dmp"))
        .expect("Couldn't process minidump");

    println!("{:#?}", state);

    let symbols = convert_symbols(
        Path::new("examples/target/crash_macos"),
        Some(Path::new("examples/target/crash_macos.dSYM")),
    ).expect("Couldn't convert symbols");

    let sym_path = Path::new("examples/target/crash_macos.sym");
    let mut file = File::create(sym_path).expect("Couldn't create sym");
    file.write_all(symbols.as_bytes()).expect("Couldn't write sym");
    file.sync_all().expect("Couldn't sync sym");

    let resolver = Resolver::new(sym_path)
        .expect("Couldn't create resolver");

    let frames: Vec<_> = state
        .threads()
        .iter()
        .flat_map(|stack| stack.frames().iter())
        .filter(|frame| frame.module().map_or(false, |m| m.debug_file() == "crash_macos"))
        .map(|frame| resolver.resolve_frame(frame))
        .collect();

    println!("{:#?}", frames);
}
