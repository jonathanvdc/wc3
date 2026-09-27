use wc3_mdx::animation::{AnimationTrack, ValueKeyframe};
use wc3_mdx::animation::{RibbonColor, RibbonTextureSlot};
use wc3_mdx::emitters::{RibbonEmitter, RibbonTrack};
use wc3_mdx::io::{Encodable, Readable};
use wc3_mdx::scene::Node;
use wc3_mdx::Model;

#[test]
fn ribbon_fields_and_integer_animation_round_trip() {
    let mut emitter = RibbonEmitter::new(Node::new("Trail", 4).unwrap());
    let mut fields = emitter.fields();
    fields.height_above = 3.0;
    fields.color = [1.0, 0.5, 0.25];
    fields.material_id = 9;
    emitter.set_fields(&fields);
    let key = ValueKeyframe {
        frame: 100,
        value: 7u32,
    };
    emitter.set_tracks(
        &[AnimationTrack::<RibbonTextureSlot>::linear(vec![key], None)
            .unwrap()
            .into()],
    );
    let mut model = Model::<wc3_mdx::V800>::new();
    model.set_ribbon_emitters(&[emitter]);
    let bytes = model.encode().unwrap();
    let parsed = Model::<wc3_mdx::V800>::decode(&bytes).unwrap();
    let ribbons = parsed.ribbon_emitters();
    assert_eq!(ribbons[0].fields(), fields);
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
    assert_eq!(parsed.encode().unwrap(), bytes);
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
        RibbonEmitter::decode(&emitter.encode().unwrap())
            .unwrap()
            .tracks(),
        vec![track]
    );
}
