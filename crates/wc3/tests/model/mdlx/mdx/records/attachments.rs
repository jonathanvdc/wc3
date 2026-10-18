use wc3::model::formats::mdlx::animation::Track;

use wc3::model::formats::mdlx::animation::ValueKeyframe;
use wc3::model::formats::mdlx::mdx::Read as _;
use wc3::model::formats::mdlx::mdx::Write as _;
use wc3::model::formats::mdlx::scene::{Attachment, Node};
use wc3::model::formats::mdlx::Document;

#[test]
fn attachment_fields_and_visibility_round_trip() {
    let mut attachment = Attachment::new(
        Node::new("Weapon", 5).unwrap(),
        "Abilities\\Weapons\\Sword.mdx",
        2,
    )
    .unwrap();
    let track = Track::<f32>::linear(
        [ValueKeyframe {
            frame: 50,
            value: 1.0,
        }],
        None,
    )
    .unwrap();
    attachment.visibility = (Some(&track)).cloned();
    let mut model = Document::<wc3::model::formats::mdlx::V1800>::new();
    model.set_attachments([attachment]);
    let parsed =
        Document::<wc3::model::formats::mdlx::V1800>::decode_mdx(&model.encode_mdx().unwrap())
            .unwrap()
            .into_owned();
    let attachment = &parsed.attachments().next().unwrap();
    assert_eq!(attachment.node.name.text(), "Weapon");
    assert_eq!(attachment.path.text(), "Abilities\\Weapons\\Sword.mdx");
    assert_eq!(attachment.id, 2);
    assert_eq!(attachment.visibility.as_ref(), Some(&track));
}
