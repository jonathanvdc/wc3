use wc3_mdx::animation::{AnimationTrack, Keyframe, TextureAnimation, TrackTag};
use wc3_mdx::io::{Decodable, Encodable};
use wc3_mdx::Model;

#[test]
fn texture_animation_tracks_round_trip() {
    let track = AnimationTrack {
        tag: TrackTag::TextureTranslation,
        interpolation: 1,
        global_sequence_id: u32::MAX,
        keyframes: vec![Keyframe {
            frame: 100,
            value: vec![1.0, 2.0, 3.0],
            in_tangent: None,
            out_tangent: None,
        }],
    };
    let mut animation = TextureAnimation::new();
    animation.set_tracks(std::slice::from_ref(&track)).unwrap();
    let mut model = Model::new(1100);
    model.set_texture_animations(&[animation]);
    let parsed = Model::decode(&model.encode().unwrap(), 800).unwrap();
    assert_eq!(parsed.texture_animations()[0].tracks(), vec![track]);
}
