#![feature(slicing_syntax, plugin)]
#![crate_type = "lib"]
#![crate_name = "tox"]
#![allow(non_camel_case_types, unstable)]

extern crate libc;
extern crate "core" as rust_core;

#[plugin] #[no_link] extern crate rest_easy;

pub mod core;
pub mod av;
pub mod util;
