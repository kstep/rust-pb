use http::client::RequestWriter;
use http::method::{Method, Get, Post, Delete};
use http::headers::content_type::MediaType;
use rustc_serialize::base64::{ToBase64, STANDARD};
use rustc_serialize::json;
use url::Url;
use std::io;
use std::io::{standard_error, IoResult, IoError};
use std::str::from_utf8;
use std::error;
use std::error::FromError;
use std::fmt::Show;
use objects::{Cursor, Timestamp, Error, PbObj, Iden, ToPbResult};
use messages::PbMsg;
use rustc_serialize::{Encodable, Decodable, Encoder, Decoder};

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

#[deriving(Show)]
pub enum PbError {
    Io(IoError),
    Pb(Error),
    Js(json::DecoderError)
}

impl FromError<IoError> for PbError {
    fn from_error(e: IoError) -> PbError { PbError::Io(e) }
}

impl FromError<Error> for PbError {
    fn from_error(e: Error) -> PbError { PbError::Pb(e) }
}

impl FromError<json::DecoderError> for PbError {
    fn from_error(e: json::DecoderError) -> PbError { PbError::Js(e) }
}

impl error::Error for PbError {
    fn description(&self) -> &str {
        match *self {
            PbError::Io(ref e) => e.description(),
            PbError::Pb(ref e) => e.description(),
            PbError::Js(ref e) => e.description()
        }
    }

    fn detail(&self) -> Option<String> {
        match *self {
            PbError::Io(ref e) => e.detail(),
            PbError::Pb(ref e) => e.detail(),
            PbError::Js(ref e) => e.detail()
        }
    }

    fn cause<'a>(&'a self) -> Option<&'a error::Error> {
        match *self {
            PbError::Io(ref e) => Some(e as &error::Error),
            PbError::Pb(ref e) => Some(e as &error::Error),
            PbError::Js(ref e) => Some(e as &error::Error)
        }
    }
}

pub type PbResult<R> = Result<R, PbError>;
pub type PbVec<I> = (Vec<I>, Option<Cursor>);

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
        let url = format!("{}{}?{}", BASE_URL, path, params.iter().filter(|v| v.1 != "").map(|&(k, v)| format!("{}={}&", k, v)).fold("".to_string(), |acc, item| acc + item[]));
        let writer = try!(self.make_writer(Get, url[]));

        match writer.read_response() {
            Ok(ref mut resp) => {
                match from_utf8(try!(resp.read_to_end())[]) {
                    Ok(ref r) => Ok(r.to_string()),
                    _ => return Err(standard_error(io::InvalidInput))
                }
            },
            Err((_, err)) => Err(err)
        }
    }

    fn post(&self, path: &str, content: &str) -> IoResult<String> {
        let url = format!("{}{}", BASE_URL, path);
        let mut writer = try!(self.make_writer(Post, url[]));

        writer.headers.content_length = Some(content.len());
        writer.headers.content_type = Some(MediaType::new("application".to_string(), "json".to_string(), Vec::new()));
        try!(writer.write(content.as_bytes()));

        match writer.read_response() {
            Ok(ref mut resp) => match from_utf8(try!(resp.read_to_end())[]) {
                Ok(ref r) => Ok(r.to_string()),
                _ => return Err(standard_error(io::InvalidInput))
            },
            Err((_, err)) => Err(err)
        }
    }

    fn delete(&self, path: &str) -> IoResult<()> {
        let url = format!("{}{}", BASE_URL, path);
        let writer = try!(self.make_writer(Delete, url[]));
        let mut resp = try!(writer.read_response().map_err(|(_, e)| e));
        match from_utf8(try!(resp.read_to_end())[]).map(|v| json::decode::<Error>(v)) {
            Ok(Ok(_)) => Err(standard_error(io::InvalidInput)),
            _ => Ok(())
        }
    }

    // Eventually (when RFC 195 is completed) only T: PbMsg (PbObj) type parameter will be needed,
    // and T::Obj will be used and the second type.
    pub fn send<'a, R: PbObj, T: PbMsg>(&self, msg: &T) -> PbResult<R>
        where T: Encodable<json::Encoder<'a>, IoError>, R: Decodable<json::Decoder, json::DecoderError> {
        let resp = try!(self.post(PbObj::root_uri(None::<R>), json::encode(msg)[]));
        match json::decode(resp[]) {
            Ok(o) => Ok(o),
            Err(e) => Err(match json::decode::<Error>(resp[]) {
                Ok(err) => FromError::from_error(err),
                Err(_) => FromError::from_error(e)
            })
        }
    }

    pub fn remove<O: PbObj>(&self, iden: Iden) -> IoResult<()> {
        self.delete(format!("{}/{}", PbObj::root_uri(None::<O>), iden)[])
    }

    #[inline] fn _load<R: PbObj, E>(&self, obj: &str, limit: Option<uint>, since: Option<Timestamp>, cursor: Option<Cursor>) -> PbResult<PbVec<R>>
        where E: Decodable<json::Decoder, json::DecoderError>, E: ToPbResult<R>, E: Show {
        let l = limit.map(|v| v.to_string()).unwrap_or("".to_string());
        let s = since.map(|v| v.to_string()).unwrap_or("".to_string());
        let c = cursor.map(|v| v.to_string()).unwrap_or("".to_string());
        let result = try!(self.get(obj, qs![limit -> l[], modified_after -> s[], cursor -> c[]][]));
        let env = try!(json::decode::<E>(result[]));
        println!("{}", env);
        Ok(try!(env.result().unwrap_or_else(|| Ok((Vec::new(), None)))))
    }

    pub fn load_by_iden<R: PbObj>(&self, iden: Iden) -> PbResult<R>
        where R: Decodable<json::Decoder, json::DecoderError> {
        let url = format!("{}/{}", PbObj::root_uri(None::<R>), iden);
        let result = try!(self.get(url[], &[]));
        Ok(try!(json::decode(result[])))
    }

    pub fn load_since<R: PbObj, E>(&self, since: Timestamp) -> PbResult<PbVec<R>>
        where E: Decodable<json::Decoder, json::DecoderError>, E: ToPbResult<R>, E: Show {
        self._load(PbObj::root_uri(None::<R>), None, Some(since), None)
    }

    pub fn load_from<R: PbObj, E>(&self, cursor: Cursor) -> PbResult<PbVec<R>>
        where E: Decodable<json::Decoder, json::DecoderError>, E: ToPbResult<R>, E: Show {
        self._load(PbObj::root_uri(None::<R>), None, None, Some(cursor))
    }

    pub fn load<R: PbObj, E>(&self) -> PbResult<PbVec<R>>
        where E: Decodable<json::Decoder, json::DecoderError>, E: ToPbResult<R>, E: Show {
        self._load(PbObj::root_uri(None::<R>), None, None, None)
    }

    pub fn loadn<R: PbObj, E>(&self, limit: uint) -> PbResult<PbVec<R>>
        where E: Decodable<json::Decoder, json::DecoderError>, E: ToPbResult<R>, E: Show {
        self._load(PbObj::root_uri(None::<R>), Some(limit), None, None)
    }

    pub fn loadn_from<R: PbObj, E>(&self, limit: uint, cursor: Cursor) -> PbResult<PbVec<R>>
        where E: Decodable<json::Decoder, json::DecoderError>, E: ToPbResult<R>, E: Show {
        self._load(PbObj::root_uri(None::<R>), Some(limit), None, Some(cursor))
    }

    pub fn loadn_since<R: PbObj, E>(&self, limit: uint, since: Timestamp) -> PbResult<PbVec<R>>
        where E: Decodable<json::Decoder, json::DecoderError>, E: ToPbResult<R>, E: Show {
        self._load(PbObj::root_uri(None::<R>), Some(limit), Some(since), None)
    }
}

#[test]
fn test_get_objects() {
    use objects::Envelope;
    let api = PbAPI::new(option_env!("PB_API_KEY").unwrap());
    let r = api.loadn(10);
    println!("{}", r);
    let result: Vec<Push> = r.ok().unwrap().0;
    panic!("{}", result);
}

//#[test]
//fn test_delete() {
    //let api = PbAPI::new(option_env!("PB_API_KEY").unwrap());
    //let result = api.remove::<Push>("123".to_string());
    //assert_eq!(result, Ok(()));
//}

