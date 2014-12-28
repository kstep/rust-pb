use std::str::FromStr;
use url::Url;
use rustc_serialize::{Encodable, Decodable, Encoder, Decoder};
use std::error;

#[cfg(test)]
use rustc_serialize::json;

pub type Iden = String;
pub type Cursor = String;
pub type Timestamp = f64;

pub trait PbObj : Sized {
    //fn uri(&self) -> String { format!("{}/{}", PbObj::root_uri(None::<Self>), self.iden()) }
    fn root_uri(_: Option<Self>) -> &'static str;
}

#[deriving(Show, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Account {
    iden: Iden,
    created: Timestamp,
    modified: Timestamp,
    email: String,
    email_normalized: String,
    name: String,
    image_url: Url,
    //preferences: {
        //onboarding:{
            //app:false,
            //friends: false,
            //extension: false
        //},
        //social: false
    //},
    api_key: String
}

//impl PbObj for Account {
    //fn uri(&self) -> String { "users/me".to_string() }
    //fn iden<'a>(&'a self) -> &'a Iden { &self.iden }
//}

#[deriving(Show, PartialEq)]
pub struct Device {
    app_version: Option<uint>,
    created: Timestamp,
    modified: Timestamp,
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
        decoder.read_struct("Device", 0, |d| {
            Ok(Device {
                app_version: try!(d.read_struct_field("app_version", 0, |d| Decodable::decode(d))),
                created: try!(d.read_struct_field("created", 0, |d| Decodable::decode(d))),
                modified: try!(d.read_struct_field("modified", 0, |d| Decodable::decode(d))),
                active: try!(d.read_struct_field("active", 0, |d| Decodable::decode(d))),
                pushable: try!(d.read_struct_field("pushable", 0, |d| Decodable::decode(d))),
                iden: try!(d.read_struct_field("iden", 0, |d| Decodable::decode(d))),
                push_token: try!(d.read_struct_field("push_token", 0, |d| Decodable::decode(d))),
                fingerprint: try!(d.read_struct_field("fingerprint", 0, |d| Decodable::decode(d))),
                nickname: try!(d.read_struct_field("nickname", 0, |d| Decodable::decode(d))),
                manufacturer: try!(d.read_struct_field("manufacturer", 0, |d| Decodable::decode(d))),
                model: try!(d.read_struct_field("model", 0, |d| Decodable::decode(d))),
                kind: try!(d.read_struct_field("kind", 0, |d| Decodable::decode(d))),
                typ: try!(d.read_struct_field("type", 0, |d| Decodable::decode(d))),
            })
        })
    }
}

#[deriving(RustcEncodable, RustcDecodable, Show, PartialEq)]
pub struct Contact {
    pub active: bool,
    pub created: Timestamp,
    pub modified: Timestamp,
    pub email: String,
    pub email_normalized: String,
    pub iden: Iden,
    pub name: String,
    pub status: String,
}

#[deriving(RustcEncodable, RustcDecodable, Show, PartialEq)]
pub struct Grant {
    pub iden: Iden,
    pub active: bool,
    pub created: Timestamp,
    pub modified: Timestamp,
    pub client: Option<Client>,
}

#[deriving(RustcEncodable, RustcDecodable, Show, PartialEq)]
pub struct Client {
    pub iden: Iden,
    pub image_url: Url,
    pub name: String,
    pub website_url: Url,
}

#[deriving(Show, PartialEq)]
pub struct Push {
    pub iden: Iden,
    pub active: bool,
    pub dismissed: bool,
    pub created: Timestamp,
    pub modified: Timestamp,

    pub title: Option<String>,
    pub body: Option<String>,

    pub receiver_name: Option<String>,
    pub receiver_iden: Option<Iden>,
    pub receiver_email: Option<String>,
    pub receiver_email_normalized: Option<String>,

    pub sender_name: Option<String>,
    pub sender_email: Option<String>,
    pub sender_email_normalized: Option<String>,
    pub sender_iden: Option<Iden>,

    pub source_device_iden: Option<Iden>,
    pub target_device_iden: Option<Iden>,
    pub channel_iden: Option<Iden>,

    pub data: PushData,
}

impl<S: Decoder<E>, E> Decodable<S, E> for Push {
    fn decode(decoder: &mut S) -> Result<Push, E> {
        decoder.read_struct("Push", 0, |d| {
            Ok(Push {
                iden: try!(d.read_struct_field("iden", 0, |d| Decodable::decode(d))),
                active: try!(d.read_struct_field("active", 0, |d| Decodable::decode(d))),
                dismissed: try!(d.read_struct_field("dismissed", 0, |d| Decodable::decode(d))),
                created: try!(d.read_struct_field("created", 0, |d| Decodable::decode(d))),
                modified: try!(d.read_struct_field("modified", 0, |d| Decodable::decode(d))),

                title: try!(d.read_struct_field("title", 0, |d| Decodable::decode(d))),
                body: try!(d.read_struct_field("body", 0, |d| Decodable::decode(d))),

                receiver_name: try!(d.read_struct_field("receiver_name", 0, |d| Decodable::decode(d))),
                receiver_iden: try!(d.read_struct_field("receiver_iden", 0, |d| Decodable::decode(d))),
                receiver_email: try!(d.read_struct_field("receiver_email", 0, |d| Decodable::decode(d))),
                receiver_email_normalized: try!(d.read_struct_field("receiver_email_normalized", 0, |d| Decodable::decode(d))),

                sender_name: try!(d.read_struct_field("sender_name", 0, |d| Decodable::decode(d))),
                sender_iden: try!(d.read_struct_field("sender_iden", 0, |d| Decodable::decode(d))),
                sender_email: try!(d.read_struct_field("sender_email", 0, |d| Decodable::decode(d))),
                sender_email_normalized: try!(d.read_struct_field("sender_email_normalized", 0, |d| Decodable::decode(d))),

                source_device_iden: try!(d.read_struct_field("source_device_iden", 0, |d| Decodable::decode(d))),
                target_device_iden: try!(d.read_struct_field("target_device_iden", 0, |d| Decodable::decode(d))),
                channel_iden: try!(d.read_struct_field("channel_iden", 0, |d| Decodable::decode(d))),

                data: try!(Decodable::decode(d))
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
            try!(e.emit_struct_field("receiver_name", 7u, |e| self.receiver_name.encode(e)));
            try!(e.emit_struct_field("receiver_iden", 8u, |e| self.receiver_iden.encode(e)));
            try!(e.emit_struct_field("receiver_email", 9u, |e| self.receiver_email.encode(e)));
            try!(e.emit_struct_field("receiver_email_normalized", 10u, |e| self.receiver_email_normalized.encode(e)));
            try!(e.emit_struct_field("sender_name", 11u, |e| self.sender_name.encode(e)));
            try!(e.emit_struct_field("sender_email", 12u, |e| self.sender_email.encode(e)));
            try!(e.emit_struct_field("sender_email_normalized", 13u, |e| self.sender_email_normalized.encode(e)));
            try!(e.emit_struct_field("sender_iden", 14u, |e| self.sender_iden.encode(e)));
            try!(e.emit_struct_field("target_device_iden", 15u, |e| self.target_device_iden.encode(e)));
            try!(e.emit_struct_field("source_device_iden", 15u, |e| self.source_device_iden.encode(e)));

            try!(self.data.encode(e));

            Ok(())
        })
    }
}

impl PbObj for Push {
    fn root_uri(_: Option<Push>) -> &'static str { "pushes" }
}

impl PbObj for Device {
    fn root_uri(_: Option<Device>) -> &'static str { "devices" }
}

impl PbObj for Contact {
    fn root_uri(_: Option<Contact>) -> &'static str { "contacts" }
}

impl PbObj for Grant {
    fn root_uri(_: Option<Grant>) -> &'static str { "grants" }
}

impl PbObj for Client {
    fn root_uri(_: Option<Client>) -> &'static str { "clients" }
}

#[deriving(Show, PartialEq)]
pub struct ListItem(bool, String);

impl FromStr for ListItem {
    fn from_str(s: &str) -> Option<ListItem> {
        Some(ListItem(false, s.to_string()))
    }
}

impl ListItem {
    pub fn new(text: &str, checked: bool) -> ListItem {
        ListItem(checked, text.to_string())
    }

    pub fn checked(self) -> ListItem {
        match self {
            ListItem(_, s) => ListItem(true, s)
        }
    }
    pub fn unchecked(self) -> ListItem {
        match self {
            ListItem(_, s) => ListItem(false, s)
        }
    }
    pub fn toggled(self) -> ListItem {
        match self {
            ListItem(c, s) => ListItem(!c, s)
        }
    }
    pub fn to_string(&self) -> String {
        match *self {
            ListItem(_, ref s) => s.to_string()
        }
    }
    pub fn is_checked(&self) -> bool {
        match *self {
            ListItem(c, _) => c
        }
    }
}

impl<S: Encoder<E>, E> Encodable<S, E> for ListItem {
    fn encode(&self, encoder: &mut S) -> Result<(), E> {
        match *self {
            ListItem(checked, ref text) => encoder.emit_struct("ListItem", 0, |e| {
                try!(e.emit_struct_field("checked", 0u, |e| e.emit_bool(checked)));
                try!(e.emit_struct_field("text", 1u, |e| e.emit_str(text.as_slice())));
                Ok(())
            })
        }
    }
}

impl<S: Decoder<E>, E> Decodable<S, E> for ListItem {
    fn decode(decoder: &mut S) -> Result<ListItem, E> {
        decoder.read_struct("root", 0, |d| {
            Ok(ListItem(
                try!(d.read_struct_field("checked", 0, |d| Decodable::decode(d))),
                try!(d.read_struct_field("text", 0, |d| Decodable::decode(d)))
            ))
        })
    }
}

#[deriving(Show, PartialEq)]
pub enum PushData {
    Empty,
    Note,
    Url(Option<Url>),
    File(String, String, Url, Option<Url>),  // name, type, url, image
    List(Vec<ListItem>),
    Address(String),
    Dismissal,
    Mirror,
}

impl<S: Encoder<E>, E> Encodable<S, E> for PushData {
    fn encode(&self, encoder: &mut S) -> Result<(), E> {
        match *self {
            PushData::Empty => (),
            PushData::Mirror => try!(encoder.emit_struct_field("type", 100u, |e| e.emit_str("mirror"))),
            PushData::Dismissal => try!(encoder.emit_struct_field("type", 100u, |e| e.emit_str("dismissal"))),
            PushData::Note => try!(encoder.emit_struct_field("type", 100u, |e| e.emit_str("note"))),
            PushData::Url(ref url) => {
                try!(encoder.emit_struct_field("type", 100u, |e| e.emit_str("url")));
                try!(encoder.emit_struct_field("url", 101u, |e| url.encode(e)));
            },
            PushData::File(ref name, ref mime, ref url, ref img) => {
                try!(encoder.emit_struct_field("type", 100u, |e| e.emit_str("file")));
                try!(encoder.emit_struct_field("file_name", 101u, |e| name.encode(e)));
                try!(encoder.emit_struct_field("file_type", 102u, |e| mime.encode(e)));
                try!(encoder.emit_struct_field("file_url", 103u, |e| url.encode(e)));
                try!(encoder.emit_struct_field("image_url", 104u, |e| img.encode(e)));
            },
            PushData::List(ref items) => {
                try!(encoder.emit_struct_field("type", 100u, |e| e.emit_str("list")));
                try!(encoder.emit_struct_field("items", 101u, |e| items.encode(e)));
            },
            PushData::Address(ref address) => {
                try!(encoder.emit_struct_field("type", 100u, |e| e.emit_str("address")));
                try!(encoder.emit_struct_field("address", 101u, |e| address.encode(e)));
            },
        }
        Ok(())
    }
}

impl<S: Decoder<E>, E> Decodable<S, E> for PushData {
    fn decode(decoder: &mut S) -> Result<PushData, E> {
        let typ: Option<String> = try!(decoder.read_struct_field("type", 0, |d| d.read_option(|d, b| if b { d.read_str().map(|v| Some(v)) } else { Ok(None) })));

        Ok(match typ {
            Some(ref t) => match t.as_slice() {
                "note" => PushData::Note,
                "link" => PushData::Url(try!(decoder.read_struct_field("url", 0, |d| Decodable::decode(d)))),
                "file" => PushData::File(
                    try!(decoder.read_struct_field("file_name", 0, |d| Decodable::decode(d))),
                    try!(decoder.read_struct_field("file_type", 0, |d| Decodable::decode(d))),
                    try!(decoder.read_struct_field("file_url", 0, |d| Decodable::decode(d))),
                    try!(decoder.read_struct_field("image_url", 0, |d| Decodable::decode(d))),
                    ),
                "list" => PushData::List(try!(decoder.read_struct_field("items", 0, |d| Decodable::decode(d)))),
                "address" => PushData::Address(try!(decoder.read_struct_field("address", 0, |d| Decodable::decode(d)))),
                "mirror" => PushData::Mirror,
                "dismissal" => PushData::Dismissal,
                typ @ _ => return Err(decoder.error(format!("Unknown type: {}", typ).as_slice()))
            },
            _ => PushData::Empty
        })
    }
}

#[deriving(Show, PartialEq, RustcDecodable)]
pub struct Channel {
    pub iden: Iden,
    pub active: bool,
    pub created: Timestamp,
    pub modified: Timestamp,
    pub tag: String,
    pub name: String,
    pub description: String,
    pub image_url: Option<Url>,
    pub website_url: Option<Url>,
    pub feed_url: Option<Url>,
}

impl PbObj for Channel {
    fn root_uri(_: Option<Channel>) -> &'static str { "channels" }
}

#[deriving(Show, PartialEq, RustcDecodable)]
pub struct ChannelInfo {
    pub iden: Iden,
    pub tag: String,
    pub name: String,
    pub description: String,
    pub image_url: Option<Url>,
    pub website_url: Option<Url>,
}

#[deriving(Show, PartialEq, RustcDecodable)]
pub struct Subscription {
    pub iden: Iden,
    pub active: bool,
    pub created: Timestamp,
    pub modified: Timestamp,
    pub channel: Option<ChannelInfo>,
}

impl PbObj for Subscription {
    fn root_uri(_: Option<Subscription>) -> &'static str { "subscriptions" }
}

#[deriving(Show, PartialEq, RustcDecodable)]
pub struct Envelope {
    //aliases: Vec<Alias>,
    pub channels: Option<Vec<Channel>>,
    pub clients: Option<Vec<Client>>,
    pub devices: Option<Vec<Device>>,
    pub grants: Option<Vec<Grant>>,
    pub pushes: Option<Vec<Push>>,
    pub contacts: Option<Vec<Contact>>,
    pub subscriptions: Option<Vec<Subscription>>,
    pub cursor: Option<Cursor>,
    pub error: Option<Error>,
}

pub trait ToPbResult<R: PbObj> {
    fn result(self) -> Option<Result<(Vec<R>, Option<Cursor>), Error>> {
        None
    }
    fn ok(self) -> Option<(Vec<R>, Option<Cursor>)> {
        match self.result() {
            Some(Ok(r)) => Some(r),
            _ => None
        }
    }
    fn err(self) -> Option<Error> {
        match self.result() {
            Some(Err(e)) => Some(e),
            _ => None
        }
    }
}

macro_rules! to_pb_result_impl {
    ($(($t:ty, $f:ident)),+) => {
        $(impl ToPbResult<$t> for Envelope {
            fn result(self) -> Option<Result<(Vec<$t>, Option<Cursor>), Error>> {
                match self.$f {
                    Some(xs) => Some(Ok((xs, self.cursor))),
                    None => match self.error {
                        Some(e) => Some(Err(e)),
                        None => None
                    }
                }
            }
        })+
    }
}

to_pb_result_impl! {
    (Channel, channels),
    (Client, clients),
    (Device, devices),
    (Grant, grants),
    (Push, pushes),
    (Contact, contacts),
    (Subscription, subscriptions)
}

impl Envelope {
    pub fn new() -> Envelope {
        Envelope {
            channels: None,
            clients: None,
            devices: None,
            grants: None,
            pushes: None,
            contacts: None,
            subscriptions: None,
            cursor: None,
            error: None,
        }
    }
    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }
    pub fn is_err(&self) -> bool {
        self.error.is_some()
    }
}

#[deriving(Show, PartialEq)]
pub struct Error {
    message: String,
    typ: String,
    cat: String,
}

impl error::Error for Error {
    fn description(&self) -> &str { "PushBuller error" }
    fn detail(&self) -> Option<String> { Some(self.message.clone()) }
}

impl<S: Decoder<E>, E> Decodable<S, E> for Error {
    fn decode(decoder: &mut S) -> Result<Error, E> {
        decoder.read_struct("Error", 0, |d| {
            Ok(Error {
                message: try!(d.read_struct_field("message", 0, |d| Decodable::decode(d))),
                typ: try!(d.read_struct_field("type", 0, |d| Decodable::decode(d))),
                cat: try!(d.read_struct_field("cat", 0, |d| Decodable::decode(d))),
            })
        })
    }
}

#[test]
fn test_note_push_decode() {
    let example = "{
        \"iden\": \"ubdpj29aOK0sKG\",
        \"type\": \"note\",
        \"title\": \"Note Title\",
        \"body\": \"Note Body\",
        \"created\": 1399253701.9744401,
        \"modified\": 1399253701.9746201,
        \"active\": true,
        \"dismissed\": false,
        \"sender_iden\": \"ubd\",
        \"sender_email\": \"ryan@pushbullet.com\",
        \"sender_email_normalized\": \"ryan@pushbullet.com\",
        \"receiver_iden\": \"ubd\",
        \"receiver_email\": \"ryan@pushbullet.com\",
        \"receiver_email_normalized\": \"ryan@pushbullet.com\"
    }";
    let push: Result<Push, _> = json::decode(example);
    match push {
        Ok(ref p) => assert_eq!(*p, Push {
            iden: "ubdpj29aOK0sKG".to_string(),
            active: true,
            dismissed: false,
            created: 1399253701f64,
            modified: 1399253701f64,

            title: Some("Note Title".to_string()),
            body: Some("Note Body".to_string()),

            receiver_name: None,
            receiver_iden: Some("ubd".to_string()),
            receiver_email: Some("ryan@pushbullet.com".to_string()),
            receiver_email_normalized: Some("ryan@pushbullet.com".to_string()),

            sender_name: None,
            sender_iden: Some("ubd".to_string()),
            sender_email: Some("ryan@pushbullet.com".to_string()),
            sender_email_normalized: Some("ryan@pushbullet.com".to_string()),

            target_device_iden: None,
            source_device_iden: None,
            channel_iden: None,

            data: PushData::Note,
        }),
        Err(e) => panic!("Error: {}", e)
    }
}

#[test]
fn test_list_push_decode() {
    let example = "{
        \"iden\": \"ubdpjAkaGXvUl2\",
        \"type\": \"list\",
        \"title\": \"List Title\",
        \"items\": [{\"checked\": true, \"text\": \"Item One\"}, {\"checked\": false, \"text\": \"Item Two\"}],
        \"created\": 1411595195.1267679,
        \"modified\": 1411699878.2501802,
        \"active\": true,
        \"dismissed\": false,
        \"sender_iden\": \"ubd\",
        \"sender_email\": \"ryan@pushbullet.com\",
        \"sender_email_normalized\": \"ryan@pushbullet.com\",
        \"receiver_iden\": \"ubd\",
        \"receiver_email\": \"ryan@pushbullet.com\",
        \"receiver_email_normalized\": \"ryan@pushbullet.com\"
    }";
    let push: Result<Push, _> = json::decode(example);
    match push {
        Ok(ref p) => assert_eq!(*p, Push {
            iden: "ubdpjAkaGXvUl2".to_string(),
            active: true,
            dismissed: false,
            created: 1411595195f64,
            modified: 1411699878f64,

            title: Some("List Title".to_string()),
            body: None,

            receiver_name: None,
            receiver_iden: Some("ubd".to_string()),
            receiver_email: Some("ryan@pushbullet.com".to_string()),
            receiver_email_normalized: Some("ryan@pushbullet.com".to_string()),

            sender_name: None,
            sender_iden: Some("ubd".to_string()),
            sender_email: Some("ryan@pushbullet.com".to_string()),
            sender_email_normalized: Some("ryan@pushbullet.com".to_string()),

            source_device_iden: None,
            target_device_iden: None,
            channel_iden: None,

            data: PushData::List(vec![
                "Item One".parse::<ListItem>().unwrap().checked(),
                "Item Two".parse::<ListItem>().unwrap()
            ]),
        }),
        Err(e) => panic!("Error: {}", e)
    }
}

#[test]
fn test_account_decode() {
    let example = "{
        \"iden\": \"udx234acsdc\",
        \"created\": 1398342586.00574,
        \"modified\": 1409046718.1501,
        \"email\": \"me@kstep.me\",
        \"email_normalized\": \"me@kstep.me\",
        \"name\": \"Konstantin Stepanov\",
        \"image_url\": \"https://lh5.googleusercontent.com/photo.jpg\",
        \"preferences\": {
            \"onboarding\":{
                \"app\":false,
                \"friends\": false,
                \"extension\": false
            },
            \"social\": false
        },
        \"api_key\": \"9aau3q49898u98me3q48u\"
    }";
    let account: Result<Account, _> = json::decode(example);
    match account {
        Ok(ref a) => assert_eq!(*a, Account {
            iden: "udx234acsdc".to_string(),
            created: 1398342586f64,
            modified: 1409046718f64,
            email: "me@kstep.me".to_string(),
            email_normalized: "me@kstep.me".to_string(),
            name: "Konstantin Stepanov".to_string(),
            image_url: Url::parse("https://lh5.googleusercontent.com/photo.jpg").unwrap(),
            //preferences: Map(...),
            api_key: "9aau3q49898u98me3q48u".to_string(),
        }),
        Err(e) => panic!("Error: {}", e)
    }
}

#[test]
fn test_decode_err_result() {
    let error = "{
        \"error\": {
            \"message\": \"The resource could not be found.\",
            \"type\": \"invalid_request\",
            \"cat\": \"~(=^‥^)\"
        }
    }";
    let result: Result<Envelope, _> = json::decode(error);
    match result {
        Ok(ref env) => {
            assert_eq!(*env, Envelope {
                error: Some(Error {
                    message: "The resource could not be found.".to_string(),
                    typ: "invalid_request".to_string(),
                    cat: "~(=^‥^)".to_string(),
                }),
                devices: None,
                pushes: None,
                contacts: None,
                channels: None,
                subscriptions: None,
                clients: None,
                grants: None,
                cursor: None
            });

            assert_eq!(env.is_ok(), false);
            assert_eq!(env.is_err(), true);
            //assert_eq!(env.err(), Some(&env.error.unwrap()));
            //assert_eq!(env.ok(), None);
            //assert_eq!(env.result(), Err(&env.error.unwrap()));
            //panic!("{}", env);
        },
        err @ _ => panic!("Unexpected result: {}", err)
    }
}

#[test]
fn test_decode_ok_result() {
    let envelope = "{
        \"devices\": [],
        \"grants\": [],
        \"pushes\": [],
        \"contacts\": []
    }";
    let result: Result<Envelope, _> = json::decode(envelope);
    match result {
        Ok(ref env) => {
            assert_eq!(*env, Envelope {
                devices: Some(vec![]),
                grants: Some(vec![]),
                pushes: Some(vec![]),
                contacts: Some(vec![]),
                channels: None,
                clients: None,
                subscriptions: None,
                error: None,
                cursor: None
            });

            assert_eq!(env.is_ok(), true);
            assert_eq!(env.is_err(), false);
            //assert_eq!(env.err(), None);
            //assert_eq!(env.ok(), Some(env));
            //assert_eq!(env.result(), Ok(env));
            panic!("{}", env);
        },
        _ => ()
        //err @ _ => panic!("Unexpected result: {}", err)
    }
}

#[test]
fn test_events() {
}
