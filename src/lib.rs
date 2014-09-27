extern crate http;
extern crate websocket;
extern crate url;
extern crate serialize;

use std::io::{Buffer, Reader, Writer, IoResult, BufferedStream, standard_error};
use websocket::socket::WebSocket;
use url::Url;
use http::client::RequestWriter;
use http::method::Get;
use serialize::base64::ToBase64;
use serialize::base64::STANDARD;
use serialize::json::{Json, ToJson, decode, encode};
use serialize::{Encodable, Decodable, Encoder, Decoder};

static BASE_URL: &'static str = "https://api.pushbullet.com/v2/";

type Iden = String;
type Cursor = String;

trait PushBulletObject {
    fn uri(&self) -> Url;
    fn iden<'a>(&'a self) -> &'a Iden;
}

trait PushTarget {
    fn ident(&self); // ???
    fn create(&self, api: &mut PushBulletAPI) -> IoResult<()>;
    fn update(&self, api: &mut PushBulletAPI) -> IoResult<()>;
    fn delete(&self, api: &mut PushBulletAPI) -> IoResult<()>;
    fn push(&self, api: &mut PushBulletAPI, push: &Push) -> IoResult<()>;
}

trait PushBulletAPI {}

#[deriving(Show, PartialEq)]
struct Device {
    app_version: Option<uint>,
    created: u64,
    modified: Option<u64>,
    active: bool,
    pushable: bool,
    iden: Iden,
    push_token: Option<String>,
    fingerprint: Option<String>,
    nickname: String,
    manufacturer: Option<String>,
    model: Option<String>,
    kind: String,
    typ: String, // type
}

impl<S: Encoder<E>, E> Encodable<S, E> for Device {
    fn encode(&self, encoder: &mut S) -> Result<(), E> {
        encoder.emit_struct("Device", 0, |e| {
            try!(e.emit_struct_field("app_version", 0u, |e| self.app_version.encode(e)));
            try!(e.emit_struct_field("created", 1u, |e| self.created.encode(e)));
            try!(e.emit_struct_field("modified", 2u, |e| self.modified.encode(e)));
            try!(e.emit_struct_field("active", 3u, |e| self.active.encode(e)));
            try!(e.emit_struct_field("pushable", 4u, |e| self.pushable.encode(e)));
            try!(e.emit_struct_field("iden", 5u, |e| self.iden.encode(e)));
            try!(e.emit_struct_field("push_token", 6u, |e| self.push_token.encode(e)));
            try!(e.emit_struct_field("fingerprint", 7u, |e| self.fingerprint.encode(e)));
            try!(e.emit_struct_field("nickname", 8u, |e| self.nickname.encode(e)));
            try!(e.emit_struct_field("manufacturer", 9u, |e| self.manufacturer.encode(e)));
            try!(e.emit_struct_field("model", 10u, |e| self.model.encode(e)));
            try!(e.emit_struct_field("kind", 11u, |e| self.kind.encode(e)));
            try!(e.emit_struct_field("type", 12u, |e| self.typ.encode(e)));
            Ok(())
        })
    }
}

impl<S: Decoder<E>, E> Decodable<S, E> for Device {
    fn decode(decoder: &mut S) -> Result<Device, E> {
        decoder.read_struct("root", 0, |d| {
            Ok(Device {
                app_version: d.read_struct_field("app_version", 0, |d| Decodable::decode(d)).ok(),
                created: try!(d.read_struct_field("created", 0, |d| Decodable::decode(d))),
                modified: d.read_struct_field("modified", 0, |d| Decodable::decode(d)).ok(),
                active: try!(d.read_struct_field("active", 0, |d| Decodable::decode(d))),
                pushable: try!(d.read_struct_field("pushable", 0, |d| Decodable::decode(d))),
                iden: try!(d.read_struct_field("iden", 0, |d| Decodable::decode(d))),
                push_token: d.read_struct_field("push_token", 0, |d| Decodable::decode(d)).ok(),
                fingerprint: d.read_struct_field("fingerprint", 0, |d| Decodable::decode(d)).ok(),
                nickname: try!(d.read_struct_field("nickname", 0, |d| Decodable::decode(d))),
                manufacturer: d.read_struct_field("manufacturer", 0, |d| Decodable::decode(d)).ok(),
                model: d.read_struct_field("model", 0, |d| Decodable::decode(d)).ok(),
                kind: try!(d.read_struct_field("kind", 0, |d| Decodable::decode(d))),
                typ: try!(d.read_struct_field("type", 0, |d| Decodable::decode(d))),
            })
        })
    }
}

#[deriving(Encodable, Decodable, Show, PartialEq)]
struct Contact {
    active: bool,
    created: u64,
    modified: Option<u64>,
    email: String,
    email_normalized: String,
    iden: Iden,
    name: String,
    status: String,
}

#[deriving(Encodable, Decodable, Show, PartialEq)]
struct Grant {
    iden: Iden,
    active: bool,
    created: u64,
    modified: Option<u64>,
    client: Option<Client>,
}

#[deriving(Encodable, Decodable, Show, PartialEq)]
struct Client {
    iden: Iden,
    image_url: Url,
    name: String,
    website_url: Url,
}

#[deriving(Show, PartialEq)]
struct Push {
    iden: Iden,
    active: bool,
    dismissed: bool,
    created: u64,
    modified: Option<u64>,

    title: Option<String>,
    body: Option<String>,

    receiver_iden: Option<Iden>,
    receiver_email: Option<String>,
    receiver_email_normalized: Option<String>,

    sender_name: String,
    sender_email: Option<String>,
    sender_email_normalized: Option<String>,
    sender_iden: Option<Iden>,

    target_device_iden: Option<Iden>,

    data: PushData,
}

impl<S: Decoder<E>, E> Decodable<S, E> for Push {
    fn decode(decoder: &mut S) -> Result<Push, E> {
        decoder.read_struct("root", 0, |d| {
            Ok(Push {
                iden: try!(d.read_struct_field("iden", 0, |d| Decodable::decode(d))),
                active: try!(d.read_struct_field("active", 0, |d| Decodable::decode(d))),
                dismissed: try!(d.read_struct_field("dismissed", 0, |d| Decodable::decode(d))),
                created: try!(d.read_struct_field("created", 0, |d| Decodable::decode(d))),
                modified: d.read_struct_field("modified", 0, |d| Decodable::decode(d)).ok(),

                title: d.read_struct_field("title", 0, |d| Decodable::decode(d)).ok(),
                body: d.read_struct_field("body", 0, |d| Decodable::decode(d)).ok(),

                receiver_iden: d.read_struct_field("receiver_iden", 0, |d| Decodable::decode(d)).ok(),
                receiver_email: d.read_struct_field("receiver_email", 0, |d| Decodable::decode(d)).ok(),
                receiver_email_normalized: d.read_struct_field("receiver_email_normalized", 0, |d| Decodable::decode(d)).ok(),

                sender_name: try!(d.read_struct_field("sender_name", 0, |d| Decodable::decode(d))),
                sender_iden: d.read_struct_field("sender_iden", 0, |d| Decodable::decode(d)).ok(),
                sender_email: d.read_struct_field("sender_email", 0, |d| Decodable::decode(d)).ok(),
                sender_email_normalized: d.read_struct_field("sender_email_normalized", 0, |d| Decodable::decode(d)).ok(),

                target_device_iden: d.read_struct_field("target_device_iden", 0, |d| Decodable::decode(d)).ok(),

                data: match try!(d.read_struct_field("type", 0, |d| d.read_str())).as_slice() {
                    "note" => NotePush,
                    "url" => UrlPush(try!(d.read_struct_field("url", 0, |d| Decodable::decode(d)))),
                    "file" => FilePush(
                        try!(d.read_struct_field("file_name", 0, |d| Decodable::decode(d))),
                        try!(d.read_struct_field("file_type", 0, |d| Decodable::decode(d))),
                        try!(d.read_struct_field("file_url", 0, |d| Decodable::decode(d))),
                        d.read_struct_field("image_url", 0, |d| Decodable::decode(d)).ok(),
                        ),
                    "list" => ListPush(try!(d.read_struct_field("items", 0, |d| Decodable::decode(d)))),
                    "address" => AddressPush(try!(d.read_struct_field("address", 0, |d| Decodable::decode(d)))),
                    _ => return Err(d.error("Unknown type"))
                }
            })
        })
    }
}

impl<S: Encoder<E>, E> Encodable<S, E> for Push {
    fn encode(&self, encoder: &mut S) -> Result<(), E> {
        encoder.emit_struct("Push", 0, |e| {
            try!(e.emit_struct_field("iden", 0u, |e| self.iden.encode(e)));
            try!(e.emit_struct_field("active", 1u, |e| self.active.encode(e)));
            try!(e.emit_struct_field("dismissed", 2u, |e| self.dismissed.encode(e)));
            try!(e.emit_struct_field("created", 3u, |e| self.created.encode(e)));
            try!(e.emit_struct_field("modified", 4u, |e| self.modified.encode(e)));
            try!(e.emit_struct_field("title", 5u, |e| self.title.encode(e)));
            try!(e.emit_struct_field("body", 6u, |e| self.body.encode(e)));
            try!(e.emit_struct_field("receiver_iden", 7u, |e| self.receiver_iden.encode(e)));
            try!(e.emit_struct_field("receiver_email", 8u, |e| self.receiver_email.encode(e)));
            try!(e.emit_struct_field("receiver_email_normalized", 9u, |e| self.receiver_email_normalized.encode(e)));
            try!(e.emit_struct_field("sender_name", 10u, |e| self.sender_name.encode(e)));
            try!(e.emit_struct_field("sender_email", 11u, |e| self.sender_email.encode(e)));
            try!(e.emit_struct_field("sender_email_normalized", 12u, |e| self.sender_email_normalized.encode(e)));
            try!(e.emit_struct_field("sender_iden", 13u, |e| self.sender_iden.encode(e)));
            try!(e.emit_struct_field("target_device_iden", 14u, |e| self.target_device_iden.encode(e)));

            match self.data {
                NotePush => try!(e.emit_struct_field("type", 15u, |e| e.emit_str("note"))),
                UrlPush(ref url) => {
                    try!(e.emit_struct_field("type", 15u, |e| e.emit_str("url")));
                    try!(e.emit_struct_field("url", 16u, |e| url.encode(e)));
                },
                FilePush(ref name, ref mime, ref url, ref img) => {
                    try!(e.emit_struct_field("type", 15u, |e| e.emit_str("file")));
                    try!(e.emit_struct_field("file_name", 16u, |e| name.encode(e)));
                    try!(e.emit_struct_field("file_type", 17u, |e| mime.encode(e)));
                    try!(e.emit_struct_field("file_url", 18u, |e| url.encode(e)));
                    match *img {
                        Some(ref url) => try!(e.emit_struct_field("image_url", 19u, |e| url.encode(e))),
                        None => ()
                    }
                },
                ListPush(ref items) => {
                    try!(e.emit_struct_field("type", 15u, |e| e.emit_str("list")));
                    try!(e.emit_struct_field("items", 16u, |e| items.encode(e)));
                },
                AddressPush(ref address) => {
                    try!(e.emit_struct_field("type", 15u, |e| e.emit_str("address")));
                    try!(e.emit_struct_field("address", 16u, |e| address.encode(e)));
                },
            }

            Ok(())
        })
    }
}

impl PushBulletObject for Push {
    fn uri(&self) -> Url {
        Url::parse(format!("pushes/{}", self.iden).as_slice()).unwrap()
    }

    fn iden<'a>(&'a self) -> &'a Iden { &self.iden }
}

impl PushBulletObject for Device {
    fn uri(&self) -> Url {
        Url::parse(format!("devices/{}", self.iden).as_slice()).unwrap()
    }

    fn iden<'a>(&'a self) -> &'a Iden { &self.iden }
}

impl PushBulletObject for Contact {
    fn uri(&self) -> Url {
        Url::parse(format!("contacts/{}", self.iden).as_slice()).unwrap()
    }

    fn iden<'a>(&'a self) -> &'a Iden { &self.iden }
}

impl PushBulletObject for Grant {
    fn uri(&self) -> Url {
        Url::parse(format!("grants/{}", self.iden).as_slice()).unwrap()
    }

    fn iden<'a>(&'a self) -> &'a Iden { &self.iden }
}

#[deriving(Show, PartialEq, Decodable, Encodable)]
struct ListItem {
    checked: bool,
    text: String,
}

#[deriving(Show, PartialEq)]
enum PushData {
    NotePush,
    UrlPush(Url),
    FilePush(String, String, Url, Option<Url>),  // name, type, url, image
    ListPush(Vec<ListItem>),
    AddressPush(String),
}

#[deriving(Show, PartialEq, Decodable, Encodable)]
struct Envelope {
    //aliases: Vec<Alias>,
    //channels: Vec<Channel>,
    //clients: Vec<Client>,
    devices: Vec<Device>,
    grants: Vec<Grant>,
    pushes: Vec<Push>,
    //subscriptions: Vec<Subscription>,
    cursor: Option<Cursor>,
}

#[test]
fn it_works() {
}
