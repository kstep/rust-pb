#![crate_name = "pb"]

#![comment = "WebSocket client"]
#![license = "MIT/ASL2"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]

extern crate http;
extern crate websocket;
extern crate url;
extern crate serialize;

pub mod objects;
pub mod events;
pub mod messages;
pub mod api;
