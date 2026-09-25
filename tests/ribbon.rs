use wc3_mdx::Record;
use wc3_mdx::{AnimationTrack, Keyframe, Model, Node, RibbonEmitter};

#[test]
fn ribbon_fields_and_integer_animation_round_trip() {
    let mut emitter = RibbonEmitter::new(Node::new("Trail", 4).unwrap());
    let mut fields = emitter.fields();
    fields.height_above = 3.0;
    fields.color = [1.0, 0.5, 0.25];
    fields.material_id = 9;
    emitter.set_fields(&fields);
    let mut key = Keyframe {
        frame: 100,
        value: vec![0.0],
        in_tangent: None,
        out_tangent: None,
    };
    assert!(key.set_integer_value(7));
    emitter
        .set_tracks(&[AnimationTrack {
            tag: *b"KRTX",
            interpolation: 1,
            global_sequence_id: u32::MAX,
            keyframes: vec![key],
        }])
        .unwrap();
    let mut model = Model::new(800);
    model.set_ribbon_emitters(&[emitter]);
    let bytes = model.encode().unwrap();
    let parsed = Model::decode(&bytes, 800).unwrap();
    let ribbons = parsed.ribbon_emitters();
    assert_eq!(ribbons[0].fields(), fields);
    assert_eq!(ribbons[0].tracks()[0].keyframes[0].integer_value(), Some(7));
    assert_eq!(parsed.encode().unwrap(), bytes);
}

#[test]
fn ribbon_color_animation_round_trip() {
    let mut emitter = RibbonEmitter::new(Node::new("ColorTrail", 5).unwrap());
    let track = AnimationTrack {
        tag: *b"KRCO",
        interpolation: 1,
        global_sequence_id: u32::MAX,
        keyframes: vec![Keyframe {
            frame: 42,
            value: vec![1.0, 0.5, 0.25],
            in_tangent: None,
            out_tangent: None,
        }],
    };
    emitter.set_tracks(std::slice::from_ref(&track)).unwrap();
    assert_eq!(
        RibbonEmitter::decode(&emitter.encode().unwrap(), 800)
            .unwrap()
            .tracks(),
        vec![track]
    );
}
