#![crate_name = "pb"]

#![comment = "WebSocket client"]
#![license = "MIT/ASL2"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]

#![feature(macro_rules)]

extern crate http;
extern crate websocket;
extern crate url;
extern crate serialize;

pub mod objects;
pub mod events;
pub mod messages;
pub mod api;
