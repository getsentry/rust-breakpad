use std::process::Command;

fn main() {
    let mut child = Command::new("make").spawn().expect("Could not run make");
    let ecode = child.wait().expect("Could not wait for make to finish");
    assert!(ecode.success());
}
