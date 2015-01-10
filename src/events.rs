use rustc_serialize::{Decodable, Decoder};
use objects::Push;

#[derive(Show, PartialEq)]
enum Event {
    Nop,
    PushTickle,
    DeviceTickle,
    Push(Push)
}

impl Decodable for Event {
    fn decode<S: Decoder>(decoder: &mut S) -> Result<Event, S::Error> {
        decoder.read_struct("Event", 0, |d| {
            match &*try!(d.read_struct_field("type", 0, |d| d.read_str())) {
                "nop" => Ok(Event::Nop),
                "tickle" => match &*try!(d.read_struct_field("subtype", 0, |d| d.read_str())) {
                    "push" => Ok(Event::PushTickle),
                    "device" => Ok(Event::DeviceTickle),
                    subtyp @ _ => Err(d.error(&*format!("Unknown tickle subtype: {:?}", subtyp)))
                },
                "push" => Ok(Event::Push(try!(Decodable::decode(d)))),
                typ @ _ => Err(d.error(&*format!("Unknown type: {:?}", typ)))
            }
        })
    }
}
