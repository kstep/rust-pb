use http::client::RequestWriter;
use http::method::{Method, Get, Post, Delete};
use http::headers::content_type::MediaType;
use serialize::{Encodable, Decodable, Encoder, Decoder};
use serialize::base64::{ToBase64, STANDARD};
use serialize::json;
use url::Url;
use std::io;
use std::io::{standard_error, IoResult, IoError};
use std::str::from_utf8;
use objects::{Envelope, PbObj, Cursor};
use messages::PbMsg;

static BASE_URL: &'static str = "https://api.pushbullet.com/v2/";

pub struct PbAPI {
    api_key: String,
}

impl PbAPI {
    fn make_writer(&self, method: Method, url: &str) -> IoResult<RequestWriter> {
        let mut writer = try!(RequestWriter::new(method, match Url::parse(url) {
            Ok(u) => u,
            Err(e) => return Err(standard_error(io::OtherIoError))
        }));
        writer.headers.authorization = Some(format!("Basic {}", format!("{}:", self.api_key).as_bytes().to_base64(STANDARD)));
        Ok(writer)
    }

    pub fn new(api_key: &str) -> PbAPI {
        PbAPI{ api_key: api_key.to_string() }
    }

    //pub fn post<S: Encoder<E>, E, T: PbMsg + Encodable<S, E>>(&mut self, msg: &T) -> IoResult<PbObj> {
        //let url = format!("{}{}", BASE_URL, PbMsg::uri());
        //let writer = try!(self.make_writer(Post, url.as_slice()));
        //let data = json::encode(msg).into_bytes();

        //writer.headers.content_length = Some(data.len());
        //writer.headers.content_type = Some(MediaType::new("application".to_string(), "json".to_string(), Vec::new()));
        //writer.write(data.as_slice());

        //match writer.read_response() {
            //Ok(resp) => Ok(match json::decode(String::from_utf8(try!(resp.read_to_end()))) {
                //Ok(r) => r,
                //Err(e) => return Err(standard_error(io::InvalidInput))
            //}),
            //Err((req, err)) => Err(err)
        //}
    //}

    fn get(&self, path: &str, params: Vec<(&str, &str)>) -> Result<Envelope, String> {
        let url = format!("{}{}?{}", BASE_URL, path, params.iter().map(|&(k, v)| format!("{}={}&", k, v)).fold("".to_string(), |acc, item| acc + item));
        let writer = try!(self.make_writer(Get, url.as_slice()).map_err(|e| format!("{}", e)));

        match writer.read_response() {
            Ok(ref mut resp) => {
                let envelope: Envelope = match from_utf8(try!(resp.read_to_end().map_err(|e| format!("{}", e))).as_slice()).map(|v| json::decode(v)) {
                    Some(Ok(e)) => e,
                    Some(Err(err)) => return Err(format!("{}", err)),
                    None => return Err("invalid UTF-8".to_string())
                };

                //match envelope.error {
                    //Some(..) => return Err(format!("{}", envelope.error)),
                    //_ => ()
                //}

                Ok(envelope)
            },
            Err((req, err)) => Err(format!("{}", err))
        }
    }
}

macro_rules! map {
    [$($name:ident -> $value:expr),*] => {
        vec![$((stringify!($name), $value)),*]
    }
}

#[test]
fn test_get_objects() {
    let api = PbAPI::new("XXXXXXX");
    for obj in vec!["pushes", "devices"].iter() {
        let result = api.get(*obj, map![limit -> "10"]);
        match result {
            Ok(env) => {
                match env.pushes {
                    None => fail!("{} missing", obj),
                    Some(ref pushes) => {
                        assert!(pushes.len() >= 0);
                        assert!(pushes.len() <= 10);
                    }
                }
            },
            Err(e) => fail!("error: {}", e)
        }
    }
}
