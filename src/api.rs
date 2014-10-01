
use http::client::RequestWriter;
use http::method::{Method, Get, Post, Delete};
use http::headers::content_type::MediaType;
use serialize::{Encodable, Decodable, Encoder, Decoder};
use serialize::base64::{ToBase64, STANDARD};
use serialize::json;
use url::Url;
use std::io;
use std::io::{standard_error, IoResult, IoError};
use objects::{Envelope, PbObj, Cursor};
use messages::PbMsg;

static BASE_URL: &'static str = "https://api.pushbullet.com/v2/";

pub struct PbAPI {
    api_key: String,
}

impl PbAPI {
    fn make_writer(&self, method: Method, url: &str) -> IoResult<RequestWriter> {
        let writer = try!(RequestWriter::new(method, match Url::parse(url) {
            Ok(u) => u,
            Err(e) => return Err(standard_error(io::OtherIoError))
        }));
        writer.headers.authorization = Some(format!("Basic {}", format!("{}:", self.api_key).as_bytes().to_base64(STANDARD)));
        Ok(writer)
    }

    pub fn new(api_key: &str) -> PbAPI {
        PbAPI{ api_key: api_key.to_string() }
    }

    pub fn post<S: Encoder<E>, E, T: PbMsg + Encodable<S, E>>(&mut self, msg: &T) -> IoResult<PbObj> {
        let url = format!("{}{}", BASE_URL, PbMsg::uri());
        let writer = try!(self.make_writer(Post, url.as_slice()));
        let data = json::encode(msg).into_bytes();

        writer.headers.content_length = Some(data.len());
        writer.headers.content_type = Some(MediaType::new("application".to_string(), "json".to_string(), Vec::new()));
        writer.write(data.as_slice());

        match writer.read_response() {
            Ok(resp) => Ok(match json::decode(String::from_utf8(try!(resp.read_to_end()))) {
                Ok(r) => r,
                Err(e) => return Err(standard_error(io::InvalidInput))
            }),
            Err((req, err)) => Err(err)
        }
    }

    pub fn get<O: PbObj>(&mut self, limit: uint, cursor: Option<Cursor>) -> IoResult<(Vec<O>, Option<Cursor>)> {
        let url = format!("{}{}?limit={}&cursor={}", BASE_URL, PbObj::collection_uri(), limit, cursor);
        let writer = try!(self.make_writer(Get, url.as_slice()));

        match writer.read_response() {
            Ok(resp) => {
                let envelope: Envelope = match json::decode(String::from_utf8_lossy(try!(resp.read_to_end()))) {
                    Ok(e) => e,
                    Err(err) => return Err(standard_error(io::InvalidInput))
                };

                match envelope.err() {
                    Some(..) => return Err(standard_error(io::OtherIoError)),
                    _ => ()
                }

                Ok(PbObj::unpack(&envelope))
            },
            Err((req, err)) => Err(err)
        }
    }
}

