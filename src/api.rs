use http::client::RequestWriter;
use http::method::{Method, Get, Post, Delete};
use http::headers::content_type::MediaType;
use serialize::base64::{ToBase64, STANDARD};
use serialize::json;
use url::Url;
use std::io;
use std::io::{standard_error, IoResult, IoError};
use std::str::from_utf8;
use objects::{Envelope, Cursor, Timestamp, Error, PbObj, Iden};
use messages::PbMsg;
use serialize::{Encodable, Decodable, Encoder, Decoder};

static BASE_URL: &'static str = "https://api.pushbullet.com/v2/";

macro_rules! qs {
    [$($name:ident -> $value:expr),*] => {
        vec![$((stringify!($name), $value)),*]
    }
}

pub struct PbAPI {
    api_key: String,
}

pub trait JsonEncodable<'a> : Encodable<json::Encoder<'a>, IoError> {}
pub trait JsonDecodable : Decodable<json::Decoder, json::DecoderError> {}

impl PbAPI {
    fn make_writer(&self, method: Method, url: &str) -> IoResult<RequestWriter> {
        let mut writer = try!(RequestWriter::new(method, match Url::parse(url) {
            Ok(u) => u,
            Err(_) => return Err(standard_error(io::OtherIoError))
        }));
        writer.headers.authorization = Some(format!("Basic {}", format!("{}:", self.api_key).as_bytes().to_base64(STANDARD)));
        Ok(writer)
    }

    pub fn new(api_key: &str) -> PbAPI {
        PbAPI{ api_key: api_key.to_string() }
    }

    fn post(&self, path: &str, content: &str) -> IoResult<String> {
        let url = format!("{}{}", BASE_URL, path);
        let mut writer = try!(self.make_writer(Post, url.as_slice()));

        writer.headers.content_length = Some(content.len());
        writer.headers.content_type = Some(MediaType::new("application".to_string(), "json".to_string(), Vec::new()));
        try!(writer.write(content.as_bytes()));

        match writer.read_response() {
            Ok(ref mut resp) => match from_utf8(try!(resp.read_to_end()).as_slice()) {
                Some(ref r) => Ok(r.to_string()),
                _ => return Err(standard_error(io::InvalidInput))
            },
            Err((_, err)) => Err(err)
        }
    }

    pub fn save<'a, T: PbMsg + JsonEncodable<'a>, R: PbObj + JsonDecodable>(&self, obj: &str, msg: &T) -> IoResult<R> {
        self.post(obj, json::encode(msg).as_slice()).map(|v| json::decode(v.as_slice()).unwrap())
    }

    fn delete(&self, path: &str) -> IoResult<()> {
        let url = format!("{}{}", BASE_URL, path);
        let writer = try!(self.make_writer(Delete, url.as_slice()));
        let mut resp = try!(writer.read_response().map_err(|(_, e)| e));
        match from_utf8(try!(resp.read_to_end()).as_slice()).map(|v| json::decode::<Error>(v)) {
            Some(Ok(_)) => Err(standard_error(io::InvalidInput)),
            _ => Ok(())
        }
    }

    pub fn remove(&self, obj: &str, iden: Iden) -> IoResult<()> {
        self.delete(format!("{}/{}", obj, iden).as_slice())
    }

    fn get(&self, path: &str, params: Vec<(&str, &str)>) -> IoResult<String> {
        let url = format!("{}{}?{}", BASE_URL, path, params.iter().filter(|v| *v.ref1() != "").map(|&(k, v)| format!("{}={}&", k, v)).fold("".to_string(), |acc, item| acc + item));
        let writer = try!(self.make_writer(Get, url.as_slice()));

        match writer.read_response() {
            Ok(ref mut resp) => {
                match from_utf8(try!(resp.read_to_end()).as_slice()) {
                    Some(ref r) => Ok(r.to_string()),
                    _ => return Err(standard_error(io::InvalidInput))
                }
            },
            Err((_, err)) => Err(err)
        }
    }

    #[inline] fn _load(&self, obj: &str, limit: Option<uint>, since: Option<Timestamp>, cursor: Option<Cursor>) -> IoResult<Envelope> {
        let l = limit.map(|v| v.to_string()).unwrap_or("".to_string());
        let s = since.map(|v| v.to_string()).unwrap_or("".to_string());
        let c = cursor.map(|v| v.to_string()).unwrap_or("".to_string());
        let result = try!(self.get(obj, qs![limit -> l[], modified_after -> s[], cursor -> c[]]));
        match json::decode::<Envelope>(result.as_slice()) {
            Ok(env) => match env.error {
                None => Ok(env),
                Some(e) => {println!("{}", e); Err(standard_error(io::InvalidInput))}
            },
            Err(e) => {println!("{}: {} {}", obj, e,result ); Err(standard_error(io::InvalidInput))}
        }
    }

    pub fn load_by_iden<R: PbObj + JsonDecodable>(&self, obj: &str, iden: Iden) -> IoResult<R> {
        self.get(format!("{}/{}", obj, iden).as_slice(), qs![]).map(|v| json::decode(v.as_slice()).unwrap())
    }

    pub fn load_since(&self, obj: &str, since: Timestamp) -> IoResult<Envelope> { self._load(obj, None, Some(since), None) }
    pub fn load_from(&self, obj: &str, cursor: Cursor) -> IoResult<Envelope> { self._load(obj, None, None, Some(cursor)) }
    pub fn load(&self, obj: &str) -> IoResult<Envelope> { self._load(obj, None, None, None) }

    pub fn loadn(&self, obj: &str, limit: uint) -> IoResult<Envelope> { self._load(obj, Some(limit), None, None) }
    pub fn loadn_from(&self, obj: &str, limit: uint, cursor: Cursor) -> IoResult<Envelope> { self._load(obj, Some(limit), None, Some(cursor)) }
    pub fn loadn_since(&self, obj: &str, limit: uint, since: Timestamp) -> IoResult<Envelope> { self._load(obj, Some(limit), Some(since), None) }
}

#[test]
fn test_get_objects() {
    let api = PbAPI::new("XXXXXXX");
    for obj in vec!["pushes", "devices", "contacts", "channels", "clients", "grants", "subscriptions"].iter() {
        let result = api.loadn(*obj, 10);
        match result {
            Ok(env) => {
                match env.pushes {
                    None => fail!("{} missing", obj),
                    Some(ref pushes) => {
                        assert!(pushes.len() <= 10);
                    }
                }
            },
            Err(e) => fail!("error for {}: {}", obj, e)
        }
    }
}

#[test]
fn test_error() {
    let api = PbAPI::new("XXXXXXX");
    let result = api.load("invalid_object");
    match result {
        Ok(env) => fail!("expected error, got {}", env),
        Err(_) => ()
    }
}

#[test]
fn test_delete() {
    let api = PbAPI::new("XXXXXXX");
    let result = api.delete("pushes/123");
    assert_eq!(result, Ok(()));
}

#[test]
fn test_load_by_iden() {
    let api = PbAPI::new("XXXXXXX");
    let result = api.load_by_iden("pushes", "123".to_string());
    fail!("{}", result);
}
