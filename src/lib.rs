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

pub use objects::{Iden, Cursor, Timestamp, Envelope, Push, PushData, Account, Device, Contact, Client, Channel, ChannelInfo, Subscription, Grant, ListItem, Error};
pub use messages::{TargetIden, PushMsg, DeviceMsg, ContactMsg};
pub use api::{PbAPI, PbError, PbResult, PbVec};

pub mod objects;
pub mod events;
pub mod messages;
pub mod api;
