#![feature(slicing_syntax, plugin, collections, io, core, std_misc, path, libc)]
#![crate_type = "lib"]
#![crate_name = "tox"]
#![allow(non_camel_case_types)]
#![plugin(rest_easy)]

extern crate libc;
extern crate "core" as rust_core;
extern crate comm;

pub mod core;
pub mod av;
pub mod util;
