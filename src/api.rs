use std::error;
use std::error::FromError;
use std::fmt::Show;

use hyper::client::Client;
use hyper::header::common::authorization::{Authorization, Basic};
use hyper::header::common::content_type::ContentType;
use hyper::net::HttpConnector;
use hyper::{HttpResult, HttpError};

use rustc_serialize::{Encodable, Decodable};
use rustc_serialize::json;

use objects::{Cursor, Timestamp, Error, PbObj, Iden, ToPbResult};
use messages::PbMsg;

static BASE_URL: &'static str = "https://api.pushbullet.com/v2/";

macro_rules! qs {
    [$($name:ident -> $value:expr),*] => {
        vec![$((stringify!($name), $value)),*]
    }
}

pub struct PbAPI {
    api_key: String,
    client: Client<HttpConnector>
}

#[derive(Show)]
pub enum PbError {
    Http(HttpError),
    Pb(Error),
    Js(json::DecoderError)
}

impl FromError<HttpError> for PbError {
    fn from_error(e: HttpError) -> PbError { PbError::Http(e) }
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
            PbError::Http(ref e) => e.description(),
            PbError::Pb(ref e) => e.description(),
            PbError::Js(ref e) => e.description()
        }
    }

    fn detail(&self) -> Option<String> {
        match *self {
            PbError::Http(ref e) => e.detail(),
            PbError::Pb(ref e) => e.detail(),
            PbError::Js(ref e) => e.detail()
        }
    }

    fn cause<'a>(&'a self) -> Option<&'a error::Error> {
        match *self {
            PbError::Http(ref e) => Some(e as &error::Error),
            PbError::Pb(ref e) => Some(e as &error::Error),
            PbError::Js(ref e) => Some(e as &error::Error)
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
        let url = format!("{}{}?{}", BASE_URL, path, params.iter().filter(|v| v.1 != "").map(|&(k, v)| format!("{}={}&", k, v)).fold("".to_string(), |acc, item| acc + item[]));
        self.client
            .get(url[])
            .header(Authorization(Basic { username: self.api_key.clone(), password: None }))
            .send()
            .and_then(|mut r| r.read_to_string().map_err(FromError::from_error))
    }

    fn post(&mut self, path: &str, content: &str) -> HttpResult<String> {
        let url = format!("{}{}", BASE_URL, path);
        self.client
            .post(url[])
            .header(Authorization(Basic { username: self.api_key.clone(), password: None }))
            .header(ContentType("application/json".parse().unwrap()))
            .body(content)
            .send()
            .and_then(|mut r| r.read_to_string().map_err(FromError::from_error))
    }

    fn delete(&mut self, path: &str) -> HttpResult<()> {
        let url = format!("{}{}", BASE_URL, path);
        self.client
            .delete(url[])
            .header(Authorization(Basic { username: self.api_key.clone(), password: None }))
            .send()
            .map(|_| ())
    }

    // Eventually (when RFC 195 is completed) only T: PbMsg (PbObj) type parameter will be needed,
    // and T::Obj will be used and the second type.
    pub fn send<R: PbObj, T: PbMsg>(&mut self, msg: &T) -> PbResult<R>
        where T: Encodable, R: Decodable {
        let resp = try!(self.post(PbObj::root_uri(None::<R>), json::encode(msg)[]));
        match json::decode(resp[]) {
            Ok(o) => Ok(o),
            Err(e) => Err(match json::decode::<Error>(resp[]) {
                Ok(err) => FromError::from_error(err),
                Err(_) => FromError::from_error(e)
            })
        }
    }

    pub fn remove<O: PbObj>(&mut self, iden: Iden) -> PbResult<()> {
        try!(self.delete(format!("{}/{}", PbObj::root_uri(None::<O>), iden)[]));
        Ok(())
    }

    #[inline] fn _load<R: PbObj, E>(&mut self, obj: &str, limit: Option<uint>, since: Option<Timestamp>, cursor: Option<Cursor>) -> PbResult<PbVec<R>>
        where E: Decodable, E: ToPbResult<R>, E: Show {
        let l = limit.map(|v| v.to_string()).unwrap_or("".to_string());
        let s = since.map(|v| v.to_string()).unwrap_or("".to_string());
        let c = cursor.map(|v| v.to_string()).unwrap_or("".to_string());
        let result = try!(self.get(obj, qs![limit -> l[], modified_after -> s[], cursor -> c[]][]));
        let env = try!(json::decode::<E>(result[]));
        Ok(try!(env.result().unwrap_or_else(|| Ok((Vec::new(), None)))))
    }

    pub fn load_by_iden<R: PbObj>(&mut self, iden: Iden) -> PbResult<R>
        where R: Decodable {
        let url = format!("{}/{}", PbObj::root_uri(None::<R>), iden);
        let result = try!(self.get(url[], &[]));
        Ok(try!(json::decode(result[])))
    }

    pub fn load_since<R: PbObj, E>(&mut self, since: Timestamp) -> PbResult<PbVec<R>>
        where E: Decodable, E: ToPbResult<R>, E: Show {
        self._load::<R, E>(PbObj::root_uri(None::<R>), None, Some(since), None)
    }

    pub fn load_from<R: PbObj, E>(&mut self, cursor: Cursor) -> PbResult<PbVec<R>>
        where E: Decodable, E: ToPbResult<R>, E: Show {
        self._load::<R, E>(PbObj::root_uri(None::<R>), None, None, Some(cursor))
    }

    pub fn load<R: PbObj, E>(&mut self) -> PbResult<PbVec<R>>
        where E: Decodable, E: ToPbResult<R>, E: Show {
        self._load::<R, E>(PbObj::root_uri(None::<R>), None, None, None)
    }

    pub fn loadn<R: PbObj, E>(&mut self, limit: uint) -> PbResult<PbVec<R>>
        where E: Decodable, E: ToPbResult<R>, E: Show {
        self._load::<R, E>(PbObj::root_uri(None::<R>), Some(limit), None, None)
    }

    pub fn loadn_from<R: PbObj, E>(&mut self, limit: uint, cursor: Cursor) -> PbResult<PbVec<R>>
        where E: Decodable, E: ToPbResult<R>, E: Show {
        self._load::<R, E>(PbObj::root_uri(None::<R>), Some(limit), None, Some(cursor))
    }

    pub fn loadn_since<R: PbObj, E>(&mut self, limit: uint, since: Timestamp) -> PbResult<PbVec<R>>
        where E: Decodable, E: ToPbResult<R>, E: Show {
        self._load::<R, E>(PbObj::root_uri(None::<R>), Some(limit), Some(since), None)
    }
}

#[test]
fn test_get_objects() {
    use objects::{Push, Device, Subscription, Grant, Client, Channel, Contact};
    use objects::Envelope;

    let mut api = PbAPI::new(env!("PB_API_KEY"));
    let r = api.loadn::<Push, Envelope>(10);
    panic!("{}", r);
}

//#[test]
//fn test_delete() {
    //let api = PbAPI::new(option_env!("PB_API_KEY").unwrap());
    //let result = api.remove::<Push>("123".to_string());
    //assert_eq!(result, Ok(()));
//}

