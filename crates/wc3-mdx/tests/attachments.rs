use wc3_mdx::animation::AttachmentVisibility;
use wc3_mdx::animation::{AnimationTrack, ValueKeyframe};
use wc3_mdx::io::{Cursor, Encodable, Readable};
use wc3_mdx::scene::{Attachment, Node};
use wc3_mdx::Model;

#[test]
fn attachment_fields_and_visibility_round_trip() {
    let mut attachment = Attachment::new(
        Node::new("Weapon", 5).unwrap(),
        "Abilities\\Weapons\\Sword.mdx",
        2,
    )
    .unwrap();
    let track = AnimationTrack::<AttachmentVisibility>::linear(
        vec![ValueKeyframe {
            frame: 50,
            value: 1.0,
        }],
        None,
    )
    .unwrap();
    attachment.set_visibility_track(Some(&track));
    let mut model = Model::<wc3_mdx::V1800>::new();
    model.set_attachments(&[attachment]);
    let parsed = Model::<wc3_mdx::V1800>::decode(&model.encode().unwrap()).unwrap();
    let attachment = &parsed.attachments()[0];
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
    let mut cursor = Cursor::new(&bytes);
    assert_eq!(cursor.read::<Attachment>().unwrap(), first);
    assert_eq!(cursor.position(), first_len);
    assert_eq!(cursor.read::<Attachment>().unwrap(), second);
    cursor.finish().unwrap();
    assert!(Attachment::decode(&bytes).is_err());
}
