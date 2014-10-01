
use serialize::{Encodable, Encoder};
use objects::{Iden, PushData};
use url::Url;

#[cfg(test)]
use serialize::json;

#[cfg(test)]
use objects::{Push, NotePush};

pub trait PbMsg {
    fn uri() -> &'static str;
}

#[deriving(PartialEq, Show)]
pub enum TargetIden {
    ContactEmail(String),
    DeviceIden(Iden)
}

#[deriving(PartialEq, Show)]
pub struct PushMsg {
    pub title: Option<String>,
    pub body: Option<String>,

    pub target: TargetIden,
    pub data: PushData,
    pub source_device_iden: Option<Iden>,
}

impl PbMsg for PushMsg {
    fn uri() -> &'static str { "pushes" }
}

impl<S: Encoder<E>, E> Encodable<S, E> for PushMsg {
    fn encode(&self, encoder: &mut S) -> Result<(), E> {
        encoder.emit_struct("PushMsg", 0, |e| {
            try!(e.emit_struct_field("title", 0u, |e| self.title.encode(e)));
            try!(e.emit_struct_field("body", 1u, |e| self.body.encode(e)));
            try!(e.emit_struct_field("source_device_iden", 2u, |e| self.source_device_iden.encode(e)));
            match self.target {
                ContactEmail(ref email) => try!(e.emit_struct_field("email", 3u, |e| e.emit_str(email.as_slice()))),
                DeviceIden(ref iden) => try!(e.emit_struct_field("device_iden", 3u, |e| e.emit_str(iden.as_slice())))
            }
            try!(self.data.encode(e));
            Ok(())
        })
    }
}

#[deriving(PartialEq, Show)]
pub struct DeviceMsg {
    pub nickname: String,
    pub typ: String,
}

impl PbMsg for DeviceMsg {
    fn uri() -> &'static str { "devices" }
}

impl<S:Encoder<E>, E> Encodable<S, E> for DeviceMsg {
    fn encode(&self, encoder: &mut S) -> Result<(), E> {
        encoder.emit_struct("DeviceMsg", 0, |e| {
            try!(e.emit_struct_field("nickname", 0u, |e| self.nickname.encode(e)));
            try!(e.emit_struct_field("type", 1u, |e| self.typ.encode(e)));
            Ok(())
        })
    }
}

#[deriving(PartialEq, Show, Encodable)]
pub struct ContactMsg {
    pub name: String,
    pub email: String,
}

impl PbMsg for ContactMsg {
    fn uri() -> &'static str { "contacts" }
}

#[test]
fn test_push_msg_encode() {
    let push = PushMsg {
        title: Some("Note Title".to_string()),
        body: Some("Note Body".to_string()),

        target: DeviceIden("udx234acsdc".to_string()),
        data: NotePush,
        source_device_iden: None,
    };
    assert_eq!(json::encode(&push).as_slice(), "{\"title\":\"Note Title\",\"body\":\"Note Body\",\"source_device_iden\":null,\"device_iden\":\"udx234acsdc\",\"type\":\"note\"}");
}

#[test]
fn test_device_msg_encode() {
    let device = DeviceMsg {
        nickname: "Nickname".to_string(),
        typ: "stream".to_string()
    };
    assert_eq!(json::encode(&device).as_slice(), "{\"nickname\":\"Nickname\",\"type\":\"stream\"}");
}
