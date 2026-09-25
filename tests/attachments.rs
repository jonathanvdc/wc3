use wc3_mdx::Record;
use wc3_mdx::{AnimationTrack, Attachment, Keyframe, Model, Node};

#[test]
fn attachment_fields_and_visibility_round_trip() {
    let mut attachment = Attachment::new(
        Node::new("Weapon", 5).unwrap(),
        "Abilities\\Weapons\\Sword.mdx",
        2,
    )
    .unwrap();
    let track = AnimationTrack {
        tag: *b"KATV",
        interpolation: 1,
        global_sequence_id: u32::MAX,
        keyframes: vec![Keyframe {
            frame: 50,
            value: vec![1.0],
            in_tangent: None,
            out_tangent: None,
        }],
    };
    attachment.set_visibility_track(Some(&track)).unwrap();
    let mut model = Model::new(1800);
    model.set_attachments(&[attachment]).unwrap();
    let parsed = Model::decode(&model.encode().unwrap(), 800).unwrap();
    let attachment = &parsed.attachments().unwrap()[0];
    assert_eq!(attachment.node().name(), "Weapon");
    assert_eq!(attachment.path(), "Abilities\\Weapons\\Sword.mdx");
    assert_eq!(attachment.id(), 2);
    assert_eq!(attachment.visibility_track(), Some(&track));
}

#[test]
fn adjacent_attachments_decode_at_their_own_boundaries() {
    let first = Attachment::new(Node::new("First", 1).unwrap(), "first.mdx", 1).unwrap();
    let second = Attachment::new(Node::new("Second", 2).unwrap(), "second.mdx", 2).unwrap();
    let mut bytes = first.encode().unwrap();
    let first_len = bytes.len();
    bytes.extend_from_slice(&second.encode().unwrap());
    assert_eq!(
        Attachment::decode_one(&bytes, 800).unwrap(),
        (first.clone(), first_len)
    );
    assert_eq!(
        Attachment::decode_one(&bytes[first_len..], 800).unwrap().0,
        second
    );
    assert!(Attachment::decode(&bytes, 800).is_err());
}
