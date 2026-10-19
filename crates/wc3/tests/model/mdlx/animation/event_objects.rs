use mdl::{Read as _, Write as _};
use mdx::{Read as _, Write as _};
use wc3::model::formats::mdlx::scene::{EventObject, Node};
use wc3::model::formats::mdlx::{mdl, mdx};

fn event<'data>(frames: &[i32], global: u32) -> EventObject<'data> {
    EventObject::new(Node::new("Arbitrary", 0).unwrap(), global, frames)
}

#[test]
fn every_construction_and_replacement_path_retains_order_and_duplicates() {
    let mut value = event(&[30, -10, 20, 20], u32::MAX);
    assert_eq!(*value.frames(), [30, -10, 20, 20]);
    value.set_frames(&[100, 0, -5, 0]);
    assert_eq!(*value.frames(), [100, 0, -5, 0]);
    let parsed = EventObject::decode_mdl(
        "EventObject \"Arbitrary\" { ObjectId 0, EventTrack 4 { 30, -10, 20, 20, } }",
    )
    .unwrap();
    assert_eq!(*parsed.frames(), [30, -10, 20, 20]);
    // Rewrite independent wire timestamps to verify order preservation by the MDX reader.
    let mut bytes = value.encode_mdx().unwrap();
    let tail = bytes.len() - 16;
    for (chunk, frame) in bytes[tail..]
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip([30i32, -10, 20, 20])
    {
        chunk.copy_from_slice(&frame.to_le_bytes());
    }
    assert_eq!(
        *EventObject::decode_mdx(&bytes).unwrap().frames(),
        [30, -10, 20, 20]
    );
    assert_eq!(
        EventObject::decode_mdx(&parsed.encode_mdx().unwrap())
            .unwrap()
            .into_owned(),
        parsed
    );
    assert_eq!(
        EventObject::decode_mdl(&parsed.encode_mdl().unwrap()).unwrap(),
        parsed
    );
}
