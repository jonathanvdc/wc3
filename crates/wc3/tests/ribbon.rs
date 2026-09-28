use wc3::model::animation::{AnimationTrack, ValueKeyframe};
use wc3::model::animation::{RibbonColor, RibbonTextureSlot};
use wc3::model::emitters::{RibbonEmitter, RibbonTrack};
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::scene::Node;
use wc3::model::Model;

#[test]
fn ribbon_fields_and_integer_animation_round_trip() {
    let mut emitter = RibbonEmitter::new(Node::new("Trail", 4).unwrap());
    emitter.height_above = 3.0;
    emitter.color = [1.0, 0.5, 0.25];
    emitter.material_id = 9;
    let key = ValueKeyframe {
        frame: 100,
        value: 7u32,
    };
    emitter.set_tracks(
        &[AnimationTrack::<RibbonTextureSlot>::linear(vec![key], None)
            .unwrap()
            .into()],
    );
    let mut model = Model::<wc3::model::V800>::new();
    model.set_ribbon_emitters(std::slice::from_ref(&emitter));
    let bytes = model.encode_mdx().unwrap();
    let parsed = Model::<wc3::model::V800>::decode_mdx(&bytes).unwrap();
    let ribbons = parsed.ribbon_emitters();
    assert_eq!(ribbons[0], emitter);
    assert_eq!(
        ribbons[0].tracks()[0],
        RibbonTrack::TextureSlot(
            AnimationTrack::<RibbonTextureSlot>::linear(
                vec![ValueKeyframe {
                    frame: 100,
                    value: 7
                }],
                None
            )
            .unwrap()
        )
    );
    assert_eq!(parsed.encode_mdx().unwrap(), bytes);
}

#[test]
fn ribbon_color_animation_round_trip() {
    let mut emitter = RibbonEmitter::new(Node::new("ColorTrail", 5).unwrap());
    let track = AnimationTrack::<RibbonColor>::linear(
        vec![ValueKeyframe {
            frame: 42,
            value: [1.0, 0.5, 0.25],
        }],
        None,
    )
    .unwrap()
    .into();
    emitter.set_tracks(std::slice::from_ref(&track));
    assert_eq!(
        RibbonEmitter::decode_mdx(&emitter.encode_mdx().unwrap())
            .unwrap()
            .tracks(),
        vec![track]
    );
}
