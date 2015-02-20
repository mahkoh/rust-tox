#![feature(plugin, collections, core, std_misc, libc, old_io, path, os)]
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
