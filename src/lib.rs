#![crate_name = "pb"]

#![crate_type = "dylib"]
#![crate_type = "rlib"]

#![feature(macro_rules)]
#![feature(associated_types)]

extern crate http;
//extern crate websocket;
extern crate url;
extern crate "rustc-serialize" as serialize;

pub mod objects;
pub mod events;
pub mod messages;
pub mod api;
