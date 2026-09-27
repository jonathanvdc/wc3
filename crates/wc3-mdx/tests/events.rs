use wc3_mdx::io::{Cursor, Decodable, Encodable};
use wc3_mdx::scene::{EventObject, Node};
use wc3_mdx::Model;

#[test]
fn event_object_round_trip() {
    let node = Node::new("Sound", 2).unwrap();
    let mut event = EventObject::new(node, u32::MAX, &[100, 200]);
    event.set_global_sequence_id(3);
    event.set_frames(&[100, 200, 300]);
    let mut model = Model::<wc3_mdx::V800>::new();
    model.set_event_objects(&[event]);
    let parsed = Model::<wc3_mdx::V800>::decode(&model.encode().unwrap(), 800).unwrap();
    let event = &parsed.event_objects()[0];
    assert_eq!(event.node().name(), "Sound");
    assert_eq!(event.global_sequence_id(), 3);
    assert_eq!(event.frames(), vec![100, 200, 300]);
}

#[test]
fn adjacent_events_decode_at_their_own_boundaries() {
    let first = EventObject::new(Node::new("First", 1).unwrap(), 0, &[10, 20]);
    let second = EventObject::new(Node::new("Second", 2).unwrap(), 1, &[30]);
    let mut bytes = first.encode().unwrap();
    let first_len = bytes.len();
    bytes.extend_from_slice(&second.encode().unwrap());
    let mut cursor = Cursor::new(&bytes);
    assert_eq!(EventObject::decode_one(&mut cursor, 800).unwrap(), first);
    assert_eq!(cursor.position(), first_len);
    assert_eq!(EventObject::decode_one(&mut cursor, 800).unwrap(), second);
    cursor.finish().unwrap();
    assert!(EventObject::decode(&bytes, 800).is_err());
}

#[test]
fn local_event_objects_round_trip_when_available() {
    let Ok(directory) = std::env::var("WC3_MDX_FIXTURES") else {
        return;
    };
    let mut pending = vec![std::path::PathBuf::from(directory)];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "mdx") {
                let bytes = std::fs::read(&path).unwrap();
                let mut model = Model::<wc3_mdx::V800>::decode(&bytes, 800).unwrap();
                if model.chunk(*b"EVTS").is_some() {
                    let events = model.event_objects();
                    model.set_event_objects(&events);
                    assert_eq!(model.encode().unwrap(), bytes);
                }
            }
        }
    }
}
