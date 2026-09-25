use wc3_mdx::{EventObject, Model, Node};

#[test]
fn event_object_round_trip() {
    let node = Node::new("Sound", 2).unwrap();
    let mut event = EventObject::new(node, u32::MAX, &[100, 200]).unwrap();
    event.set_global_sequence_id(3);
    event.set_frames(&[100, 200, 300]).unwrap();
    let mut model = Model::new(800);
    model.set_event_objects(&[event]).unwrap();
    let parsed = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
    let event = &parsed.event_objects().unwrap()[0];
    assert_eq!(event.node().name(), "Sound");
    assert_eq!(event.global_sequence_id(), 3);
    assert_eq!(event.frames(), vec![100, 200, 300]);
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
                let mut model = Model::from_bytes(&bytes).unwrap();
                if model.chunk(*b"EVTS").is_some() {
                    let events = model.event_objects().unwrap();
                    model.set_event_objects(&events).unwrap();
                    assert_eq!(model.to_bytes().unwrap(), bytes);
                }
            }
        }
    }
}
