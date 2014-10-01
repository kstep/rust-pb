use serialize::{Decodable, Decoder};
use objects::Push;

#[deriving(Show, PartialEq)]
enum Event {
    NopEvent,
    PushTickleEvent,
    DeviceTickleEvent,
    PushEvent(Push)
}

impl<S: Decoder<E>, E> Decodable<S, E> for Event {
    fn decode(decoder: &mut S) -> Result<Event, E> {
        decoder.read_struct("Event", 0, |d| {
            match try!(d.read_struct_field("type", 0, |d| d.read_str())).as_slice() {
                "nop" => Ok(NopEvent),
                "tickle" => match try!(d.read_struct_field("subtype", 0, |d| d.read_str())).as_slice() {
                    "push" => Ok(PushTickleEvent),
                    "device" => Ok(DeviceTickleEvent),
                    subtyp @ _ => Err(d.error(format!("Unknown tickle subtype: {}", subtyp).as_slice()))
                },
                "push" => Ok(PushEvent(try!(Decodable::decode(d)))),
                typ @ _ => Err(d.error(format!("Unknown type: {}", typ).as_slice()))
            }
        })
    }
}
