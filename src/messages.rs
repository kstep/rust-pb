
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
pub struct PushMsg {
    pub title: Option<String>,
    pub body: Option<String>,

    pub target: TargetIden,
    pub data: PushData,
    pub source_device_iden: Option<Iden>,
}

impl PushMsg {
    pub fn new(target: TargetIden) -> PushMsg {
        PushMsg {
            title: None,
            body: None,
            target: target,
            data: PushData::Note,
            source_device_iden: None
        }
    }

    pub fn note(target: TargetIden, title: Option<&str>, body: Option<&str>) -> PushMsg {
        PushMsg {
            title: title.map(|v| v.to_string()),
            body: body.map(|v| v.to_string()),
            target: target,
            data: PushData::Note,
            source_device_iden: None
        }
    }

    pub fn title(mut self, title: &str) -> PushMsg {
        self.title = Some(title.to_string());
        self
    }

    pub fn body(mut self, body: &str) -> PushMsg {
        self.body = Some(body.to_string());
        self
    }

    pub fn source(mut self, source: Iden) -> PushMsg {
        self.source_device_iden = Some(source);
        self
    }

    pub fn data(mut self, data: PushData) -> PushMsg {
        self.data = data;
        self
    }
}

impl PbMsg for PushMsg {
    type Obj = super::objects::Push;
}

impl Encodable for PushMsg {
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
pub struct DeviceMsg {
    pub nickname: String,
    pub typ: String,
}

impl PbMsg for DeviceMsg {
    type Obj = super::objects::Device;
}

impl Encodable for DeviceMsg {
    fn encode<S: Encoder>(&self, encoder: &mut S) -> Result<(), S::Error> {
        encoder.emit_struct("DeviceMsg", 2, |e| {
            try!(e.emit_struct_field("nickname", 0, |e| self.nickname.encode(e)));
            try!(e.emit_struct_field("type", 1, |e| self.typ.encode(e)));
            Ok(())
        })
    }
}

#[derive(PartialEq, Debug, RustcEncodable)]
pub struct ContactMsg {
    pub name: String,
    pub email: String,
}

impl PbMsg for ContactMsg {
    type Obj = super::objects::Contact;
}

#[test]
fn test_push_msg_encode() {
    let push = PushMsg {
        title: Some("Note Title".to_string()),
        body: Some("Note Body".to_string()),

        target: TargetIden::DeviceIden("udx234acsdc".to_string()),
        data: PushData::Note,
        source_device_iden: None,
    };
    assert_eq!(&*json::encode(&push).unwrap(), "{\"title\":\"Note Title\",\"body\":\"Note Body\",\"source_device_iden\":null,\"device_iden\":\"udx234acsdc\",\"type\":\"note\"}");
}

#[test]
fn test_device_msg_encode() {
    let device = DeviceMsg {
        nickname: "Nickname".to_string(),
        typ: "stream".to_string()
    };
    assert_eq!(&*json::encode(&device).unwrap(), "{\"nickname\":\"Nickname\",\"type\":\"stream\"}");
}

#[test]
fn test_build_msg_push() {
    let push = PushMsg::new(TargetIden::DeviceIden("udx111asdf".to_string()))
        .body("Hello, world").title("Title");
    assert_eq!(&*json::encode(&push).unwrap(), "{\"title\":\"Title\",\"body\":\"Hello, world\",\"source_device_iden\":null,\"device_iden\":\"udx111asdf\",\"type\":\"note\"}");
}
