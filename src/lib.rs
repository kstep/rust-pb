#![crate_name = "pb"]

#![crate_type = "dylib"]
#![crate_type = "rlib"]

#![feature(slicing_syntax)]
#![feature(old_orphan_check)]
#![allow(unstable)]

extern crate hyper;
//extern crate websocket;
extern crate url;
extern crate "rustc-serialize" as rustc_serialize;

pub mod objects;
pub mod events;
pub mod messages;
pub mod api;
