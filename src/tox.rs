#![feature(slicing_syntax)]
#![crate_type = "lib"]
#![crate_name = "tox"]
#![allow(non_camel_case_types, unstable)]

extern crate libc;
extern crate "core" as rust_core;

pub mod core;
pub mod av;
pub mod util;
