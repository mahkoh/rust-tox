#![feature(globs, macro_rules)]
#![crate_type = "lib"]
#![crate_name = "tox"]
#![allow(non_camel_case_types)]

extern crate libc;
extern crate "core" as rust_core;

pub mod core;
