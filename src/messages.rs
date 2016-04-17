
use std::borrow::Cow;
use rustc_serialize::{Encodable, Encoder};
use objects::{Iden, PushData};

#[cfg(test)]
use rustc_serialize::json;

pub trait PbMsg : Encodable {
    type Obj: super::objects::PbObj;
}

#[derive(PartialEq, Debug)]
pub enum TargetIden {
    CurrentUser,
    DeviceIden(Iden),
    ContactEmail(String),
    ChannelTag(String),
    ClientIden(Iden),
}

#[derive(PartialEq, Debug)]
pub struct PushMsg<'a> {
    pub title: Option<Cow<'a, str>>,
    pub body: Option<Cow<'a, str>>,

    pub target: TargetIden,
    pub data: PushData,
    pub source_device_iden: Option<Iden>,
}

impl<'a> PushMsg<'a> {
    pub fn new(target: TargetIden) -> PushMsg<'a> {
        PushMsg {
            title: None,
            body: None,
            target: target,
            data: PushData::Note,
            source_device_iden: None
        }
    }

    pub fn note<T: Into<Cow<'a, str>>, B: Into<Cow<'a, str>>>(target: TargetIden, title: Option<T>, body: Option<B>) -> PushMsg<'a> {
        PushMsg {
            title: title.map(Into::into),
            body: body.map(Into::into),
            target: target,
            data: PushData::Note,
            source_device_iden: None
        }
    }

    pub fn title<T: Into<Cow<'a, str>>>(mut self, title: T) -> PushMsg<'a> {
        self.title = Some(title.into());
        self
    }

    pub fn body<T: Into<Cow<'a, str>>>(mut self, body: T) -> PushMsg<'a> {
        self.body = Some(body.into());
        self
    }

    pub fn source(mut self, source: Iden) -> PushMsg<'a> {
        self.source_device_iden = Some(source);
        self
    }

    pub fn data(mut self, data: PushData) -> PushMsg<'a> {
        self.data = data;
        self
    }
}

impl<'a> PbMsg for PushMsg<'a> {
    type Obj = super::objects::Push;
}

impl<'a> Encodable for PushMsg<'a> {
    fn encode<S: Encoder>(&self, encoder: &mut S) -> Result<(), S::Error> {
        encoder.emit_struct("PushMsg", 5, |e| {
            try!(e.emit_struct_field("title", 0, |e| self.title.encode(e)));
            try!(e.emit_struct_field("body", 1, |e| self.body.encode(e)));
            try!(e.emit_struct_field("source_device_iden", 2, |e| self.source_device_iden.encode(e)));
            try!(match self.target {
                TargetIden::CurrentUser => Ok(()),
                TargetIden::DeviceIden(ref iden) => e.emit_struct_field("device_iden", 3, |e| e.emit_str(&**iden)),
                TargetIden::ContactEmail(ref email) => e.emit_struct_field("email", 3, |e| e.emit_str(&**email)),
                TargetIden::ChannelTag(ref tag) => e.emit_struct_field("channel_tag", 3, |e| e.emit_str(&**tag)),
                TargetIden::ClientIden(ref iden) => e.emit_struct_field("client_iden", 3, |e| e.emit_str(&**iden)),
            });
            try!(self.data.encode(e));
            Ok(())
        })
    }
}

#[derive(PartialEq, Debug)]
pub struct DeviceMsg<'a> {
    pub nickname: Cow<'a, str>,
    pub typ: Cow<'a, str>,
}

impl<'a> PbMsg for DeviceMsg<'a> {
    type Obj = super::objects::Device;
}

impl<'a> Encodable for DeviceMsg<'a> {
    fn encode<S: Encoder>(&self, encoder: &mut S) -> Result<(), S::Error> {
        encoder.emit_struct("DeviceMsg", 2, |e| {
            try!(e.emit_struct_field("nickname", 0, |e| self.nickname.encode(e)));
            try!(e.emit_struct_field("type", 1, |e| self.typ.encode(e)));
            Ok(())
        })
    }
}

#[derive(PartialEq, Debug, RustcEncodable)]
pub struct ContactMsg<'a> {
    pub name: Cow<'a, str>,
    pub email: Cow<'a, str>,
}

impl<'a> PbMsg for ContactMsg<'a> {
    type Obj = super::objects::Contact;
}

#[test]
fn test_push_msg_encode() {
    let push = PushMsg {
        title: Some("Note Title".into()),
        body: Some("Note Body".into()),

        target: TargetIden::DeviceIden("udx234acsdc".to_string()),
        data: PushData::Note,
        source_device_iden: None,
    };
    assert_eq!(&*json::encode(&push).unwrap(), "{\"title\":\"Note Title\",\"body\":\"Note Body\",\"source_device_iden\":null,\"device_iden\":\"udx234acsdc\",\"type\":\"note\"}");
}

#[test]
fn test_device_msg_encode() {
    let device = DeviceMsg {
        nickname: "Nickname".into(),
        typ: "stream".into()
    };
    assert_eq!(&*json::encode(&device).unwrap(), "{\"nickname\":\"Nickname\",\"type\":\"stream\"}");
}

#[test]
fn test_build_msg_push() {
    let push = PushMsg::new(TargetIden::DeviceIden("udx111asdf".to_string()))
        .body("Hello, world").title("Title");
    assert_eq!(&*json::encode(&push).unwrap(), "{\"title\":\"Title\",\"body\":\"Hello, world\",\"source_device_iden\":null,\"device_iden\":\"udx111asdf\",\"type\":\"note\"}");
}
