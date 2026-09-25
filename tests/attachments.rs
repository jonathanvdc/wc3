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
    let parsed = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
    let attachment = &parsed.attachments().unwrap()[0];
    assert_eq!(attachment.node().name(), "Weapon");
    assert_eq!(attachment.path(), "Abilities\\Weapons\\Sword.mdx");
    assert_eq!(attachment.id(), 2);
    assert_eq!(attachment.visibility_track(), Some(&track));
}
