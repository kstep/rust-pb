use std::error;
use std::convert::From;
use std::fmt;
use std::io::Read;

use hyper::client::Client;
use hyper::header::{ContentType, Authorization, Basic};
use hyper::{HttpResult, HttpError};

use rustc_serialize::json;

use objects::{Cursor, Timestamp, Error, PbObj, Iden, FromEnvelope, Envelope};
use messages::PbMsg;

static BASE_URL: &'static str = "https://api.pushbullet.com/v2/";

macro_rules! qs {
    [$($name:ident -> $value:expr),*] => {
        vec![$((stringify!($name), $value)),*]
    }
}

pub struct PbAPI {
    api_key: String,
    client: Client
}

#[derive(Debug)]
pub enum PbError {
    Http(HttpError),
    Pb(Error),
    Js(json::DecoderError),
    Fmt(json::EncoderError)
}

impl From<HttpError> for PbError {
    fn from(e: HttpError) -> PbError { PbError::Http(e) }
}

impl From<Error> for PbError {
    fn from(e: Error) -> PbError { PbError::Pb(e) }
}

impl From<json::DecoderError> for PbError {
    fn from(e: json::DecoderError) -> PbError { PbError::Js(e) }
}

impl From<json::EncoderError> for PbError {
    fn from(e: json::EncoderError) -> PbError { PbError::Fmt(e) }
}

impl error::Error for PbError {
    fn description(&self) -> &str {
        match *self {
            PbError::Http(ref e) => e.description(),
            PbError::Pb(ref e) => e.description(),
            PbError::Fmt(ref e) => e.description(),
            PbError::Js(ref e) => e.description()
        }
    }

    fn cause<'a>(&'a self) -> Option<&'a error::Error> {
        match *self {
            PbError::Http(ref e) => Some(e as &error::Error),
            PbError::Pb(ref e) => Some(e as &error::Error),
            PbError::Fmt(ref e) => Some(e as &error::Error),
            PbError::Js(ref e) => Some(e as &error::Error)
        }
    }
}

impl fmt::Display for PbError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            PbError::Http(ref e) => e.fmt(fmt),
            PbError::Pb(ref e) => e.fmt(fmt),
            PbError::Fmt(ref e) => e.fmt(fmt),
            PbError::Js(ref e) => e.fmt(fmt)
        }
    }
}


pub type PbResult<R> = Result<R, PbError>;
pub type PbVec<I> = (Vec<I>, Option<Cursor>);

impl PbAPI {

    pub fn new(api_key: &str) -> PbAPI {
        PbAPI {
            api_key: api_key.to_string(),
            client: Client::new()
        }
    }

    fn get(&mut self, path: &str, params: &[(&str, &str)]) -> HttpResult<String> {
        let url = format!("{}{}?{}", BASE_URL, path, params.iter().filter(|v| v.1 != "").map(|&(k, v)| format!("{}={}&", k, v)).fold(String::new(), |acc, item| acc + &*item));
        let mut response = try!(self.client
            .get(&*url)
            .header(Authorization(Basic { username: self.api_key.clone(), password: None }))
            .send());
        let mut content = String::new();
        try!(response.read_to_string(&mut content));
        Ok(content)
    }

    fn post(&mut self, path: &str, content: &str) -> HttpResult<String> {
        let url = format!("{}{}", BASE_URL, path);
        let mut response = try!(self.client
            .post(&*url)
            .header(Authorization(Basic { username: self.api_key.clone(), password: None }))
            .header(ContentType("application/json".parse().unwrap()))
            .body(content)
            .send());
        let mut content = String::new();
        try!(response.read_to_string(&mut content));
        Ok(content)
    }

    fn delete(&mut self, path: &str) -> HttpResult<()> {
        let url = format!("{}{}", BASE_URL, path);
        self.client
            .delete(&*url)
            .header(Authorization(Basic { username: self.api_key.clone(), password: None }))
            .send()
            .map(|_| ())
    }

    // Eventually (when RFC 195 is completed) only T: PbMsg (PbObj) type parameter will be needed,
    // and T::Obj will be used and the second type.
    pub fn send<T: PbMsg>(&mut self, msg: &T) -> PbResult<T::Obj> {
        let resp = try!(self.post(T::Obj::root_uri(), &*try!(json::encode(msg))));
        match json::decode(&*resp) {
            Ok(o) => Ok(o),
            Err(e) => Err(match json::decode::<Error>(&*resp) {
                Ok(err) => From::from(err),
                Err(_) => From::from(e)
            })
        }
    }

    pub fn remove<O: PbObj>(&mut self, iden: Iden) -> PbResult<()> {
        try!(self.delete(&*format!("{}/{}", O::root_uri(), iden)));
        Ok(())
    }

    #[inline] fn _load<R: PbObj + FromEnvelope>(&mut self, obj: &str, limit: Option<usize>, since: Option<Timestamp>, cursor: Option<Cursor>) -> PbResult<PbVec<R>> {
        let l = limit.map(|v| v.to_string()).unwrap_or("".to_string());
        let s = since.map(|v| v.to_string()).unwrap_or("".to_string());
        let c = cursor.map(|v| v.to_string()).unwrap_or("".to_string());
        let result = try!(self.get(obj, &*qs![limit -> &*l, modified_after -> &*s, cursor -> &*c]));
        let env = try!(json::decode::<Envelope>(&*result));
        env.get::<R>().map_err(From::from)
    }

    pub fn load_by_iden<R: PbObj>(&mut self, iden: Iden) -> PbResult<R> {
        let url = format!("{}/{}", R::root_uri(), iden);
        let result = try!(self.get(&*url, &[]));
        Ok(try!(json::decode(&*result)))
    }

    pub fn load_since<R: PbObj + FromEnvelope>(&mut self, since: Timestamp) -> PbResult<PbVec<R>> {
        self._load::<R>(R::root_uri(), None, Some(since), None)
    }

    pub fn load_from<R: PbObj + FromEnvelope>(&mut self, cursor: Cursor) -> PbResult<PbVec<R>> {
        self._load::<R>(R::root_uri(), None, None, Some(cursor))
    }

    pub fn load<R: PbObj + FromEnvelope>(&mut self) -> PbResult<PbVec<R>> {
        self._load::<R>(R::root_uri(), None, None, None)
    }

    pub fn loadn<R: PbObj + FromEnvelope>(&mut self, limit: usize) -> PbResult<PbVec<R>> {
        self._load::<R>(R::root_uri(), Some(limit), None, None)
    }

    pub fn loadn_from<R: PbObj + FromEnvelope>(&mut self, limit: usize, cursor: Cursor) -> PbResult<PbVec<R>> {
        self._load::<R>(R::root_uri(), Some(limit), None, Some(cursor))
    }

    pub fn loadn_since<R: PbObj + FromEnvelope>(&mut self, limit: usize, since: Timestamp) -> PbResult<PbVec<R>> {
        self._load::<R>(R::root_uri(), Some(limit), Some(since), None)
    }
}

//#[test]
//#[allow(unused_imports)]
//fn test_get_objects() {
//    use objects::{Push, Device, Subscription, Grant, Client, Channel, Contact};
//    use objects::Envelope;
//
//    let mut api = PbAPI::new(option_env!("PB_API_KEY").unwrap());
//    let r = api.loadn::<Push>(10);
//    r.unwrap();
//    //panic!("{:?}", r);
//}

//#[test]
//fn test_delete() {
    //let api = PbAPI::new(option_env!("PB_API_KEY").unwrap());
    //let result = api.remove::<Push>("123".to_string());
    //assert_eq!(result, Ok(()));
//}

