use wc3::model::animation::ValueKeyframe;
use wc3::model::animation::{Animatable, Track};
use wc3::model::Color;

use wc3::model::emitters::RibbonEmitter;
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::scene::Node;
use wc3::model::Model;

#[test]
fn ribbon_fields_and_integer_animation_round_trip() {
    let mut emitter = RibbonEmitter::new(Node::new("Trail", 4).unwrap());
    emitter.height_above = Animatable::Static(3.0);
    emitter.color = Animatable::Static([1.0, 0.5, 0.25]);
    emitter.material_id = 9;
    let key = ValueKeyframe {
        frame: 100,
        value: 7u32,
    };
    emitter
        .texture_slot
        .set_track(Track::<u32>::linear(vec![key], None).unwrap());
    let mut model = Model::<wc3::model::V800>::new();
    model.set_ribbon_emitters(std::slice::from_ref(&emitter));
    let bytes = model.encode_mdx().unwrap();
    let parsed = Model::<wc3::model::V800>::decode_mdx(&bytes).unwrap();
    let ribbons = parsed.ribbon_emitters();
    assert_eq!(ribbons[0], emitter);
    assert_eq!(
        ribbons[0]
            .texture_slot
            .track()
            .unwrap()
            .linear_keys()
            .unwrap()[0]
            .value,
        7
    );
    assert_eq!(parsed.encode_mdx().unwrap(), bytes);
}

#[test]
fn ribbon_color_animation_round_trip() {
    let mut emitter = RibbonEmitter::new(Node::new("ColorTrail", 5).unwrap());
    let track = Track::<Color>::linear(
        vec![ValueKeyframe {
            frame: 42,
            value: [1.0, 0.5, 0.25],
        }],
        None,
    )
    .unwrap();
    emitter.color.set_track(track.clone());
    assert_eq!(
        RibbonEmitter::decode_mdx(&emitter.encode_mdx().unwrap())
            .unwrap()
            .color
            .track(),
        Some(&track)
    );
}
