use std::time::SystemTime;
use win_binder::{Event, EventType, Key};

fn main() {
    let event = Event {
        event_type: EventType::KeyPress(Key::KeyS),
        time: SystemTime::now(),
        name: Some(String::from("S")),
    };

    let serialized = serde_json::to_string(&event).unwrap();

    let deserialized: Event = serde_json::from_str(&serialized).unwrap();

    println!("Serialized event {:?}", serialized);
    println!("Deserialized event {:?}", deserialized);
    assert_eq!(event, deserialized);
}
