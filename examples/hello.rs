extern crate pb;

use std::env;

fn main() {
    let msg = pb::PushMsg {
        title: Some("Hello, world!".to_string()),
        body: Some("This a push test".to_string()),
        target: pb::TargetIden::CurrentUser,
        data: pb::PushData::Note,
        source_device_iden: None,
    };

    let mut api = pb::PbAPI::new(&*env::var("PB_API_KEY").ok().expect("missing PB_API_KEY environment variable"));
    api.send(&msg).unwrap();
}
