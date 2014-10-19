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

#[cfg(test)]
use objects::{Push, Device, Subscription, Grant, Client, Channel, Contact};

static BASE_URL: &'static str = "https://api.pushbullet.com/v2/";

macro_rules! qs {
    [$($name:ident -> $value:expr),*] => {
        vec![$((stringify!($name), $value)),*]
    }
}

pub struct PbAPI {
    api_key: String,
}

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

    fn get(&self, path: &str, params: &[(&str, &str)]) -> IoResult<String> {
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

    fn delete(&self, path: &str) -> IoResult<()> {
        let url = format!("{}{}", BASE_URL, path);
        let writer = try!(self.make_writer(Delete, url.as_slice()));
        let mut resp = try!(writer.read_response().map_err(|(_, e)| e));
        match from_utf8(try!(resp.read_to_end()).as_slice()).map(|v| json::decode::<Error>(v)) {
            Some(Ok(_)) => Err(standard_error(io::InvalidInput)),
            _ => Ok(())
        }
    }

    // Eventually (when RFC 195 is completed) only T: PbMsg (PbObj) type parameter will be needed,
    // and T::Obj will be used and the second type.
    pub fn save<'a, R: PbObj, T: PbMsg>(&self, msg: &T) -> IoResult<R>
        where T: Encodable<json::Encoder<'a>, IoError>, R: Decodable<json::Decoder, json::DecoderError> {
        self.post(PbObj::root_uri(None::<R>), json::encode(msg).as_slice()).map(|v| json::decode(v.as_slice()).unwrap())
    }

    pub fn remove<O: PbObj>(&self, iden: Iden) -> IoResult<()> {
        self.delete(format!("{}/{}", PbObj::root_uri(None::<O>), iden).as_slice())
    }

    #[inline] fn _load(&self, obj: &str, limit: Option<uint>, since: Option<Timestamp>, cursor: Option<Cursor>) -> IoResult<Envelope> {
        let l = limit.map(|v| v.to_string()).unwrap_or("".to_string());
        let s = since.map(|v| v.to_string()).unwrap_or("".to_string());
        let c = cursor.map(|v| v.to_string()).unwrap_or("".to_string());
        let result = try!(self.get(obj, qs![limit -> l[], modified_after -> s[], cursor -> c[]][]));
        match json::decode::<Envelope>(result.as_slice()) {
            Ok(env) => match env.error {
                None => Ok(env),
                Some(e) => {println!("{}", e); Err(standard_error(io::InvalidInput))}
            },
            Err(e) => {println!("{}: {} {}", obj, e,result ); Err(standard_error(io::InvalidInput))}
        }
    }

    pub fn load_by_iden<R: PbObj>(&self, iden: Iden) -> IoResult<R>
        where R: Decodable<json::Decoder, json::DecoderError> {
        self.get(format!("{}/{}", PbObj::root_uri(None::<R>), iden).as_slice(), &[]).map(|v| json::decode(v.as_slice()).unwrap())
    }

    pub fn load_since<R: PbObj>(&self, since: Timestamp) -> IoResult<Envelope> { self._load(PbObj::root_uri(None::<R>), None, Some(since), None) }
    pub fn load_from<R: PbObj>(&self, cursor: Cursor) -> IoResult<Envelope> { self._load(PbObj::root_uri(None::<R>), None, None, Some(cursor)) }
    pub fn load<R: PbObj>(&self) -> IoResult<Envelope> { self._load(PbObj::root_uri(None::<R>), None, None, None) }

    pub fn loadn<R: PbObj>(&self, limit: uint) -> IoResult<Envelope> { self._load(PbObj::root_uri(None::<R>), Some(limit), None, None) }
    pub fn loadn_from<R: PbObj>(&self, limit: uint, cursor: Cursor) -> IoResult<Envelope> { self._load(PbObj::root_uri(None::<R>), Some(limit), None, Some(cursor)) }
    pub fn loadn_since<R: PbObj>(&self, limit: uint, since: Timestamp) -> IoResult<Envelope> { self._load(PbObj::root_uri(None::<R>), Some(limit), Some(since), None) }
}

#[test]
fn test_get_objects() {
    let api = PbAPI::new(env!("PB_API_KEY"));
    let result = api.loadn::<Push>(10);
    match result {
        Ok(env) => {
            match env.pushes {
                None => fail!("push missing"),
                Some(ref pushes) => {
                    assert!(pushes.len() <= 10);
                }
            }
        },
        Err(e) => fail!("error: {}", e)
    }
}

#[test]
fn test_delete() {
    let api = PbAPI::new(env!("PB_API_KEY"));
    let result = api.remove::<Push>("123".to_string());
    assert_eq!(result, Ok(()));
}

