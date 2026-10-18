use wc3::model::formats::mdlx::mdx::Read as _;
use wc3::model::formats::mdlx::mdx::Write as _;
use wc3::model::formats::mdlx::scene::{EventObject, Node};
use wc3::model::formats::mdlx::Document;

#[test]
fn event_object_round_trip() {
    let node = Node::new("Sound", 2).unwrap();
    let mut event = EventObject::new(node, u32::MAX, &[100, 200]);
    event.global_sequence_id = 3;
    event.set_frames(&[300, 100, 200]);
    let mut model = Document::<wc3::model::formats::mdlx::V800>::new();
    model.set_event_objects([event]);
    let parsed =
        Document::<wc3::model::formats::mdlx::V800>::decode_mdx(&model.encode_mdx().unwrap())
            .unwrap()
            .into_owned();
    let event = &parsed.event_objects().next().unwrap();
    assert_eq!(event.node.name.text(), "Sound");
    assert_eq!(event.global_sequence_id, 3);
    assert_eq!(*event.frames(), [100, 200, 300]);
}
