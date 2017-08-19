#[macro_use]
extern crate error_chain;

mod errors;
mod minidump;

pub use errors::*;
pub use minidump::Minidump;
