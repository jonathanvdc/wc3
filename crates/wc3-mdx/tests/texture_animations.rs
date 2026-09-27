use wc3_mdx::animation::TextureTranslation;
use wc3_mdx::animation::{AnimationTrack, TextureAnimation, ValueKeyframe};
use wc3_mdx::io::{Readable, Writable};
use wc3_mdx::Model;

#[test]
fn texture_animation_tracks_round_trip() {
    let track = AnimationTrack::<TextureTranslation>::linear(
        vec![ValueKeyframe {
            frame: 100,
            value: [1.0, 2.0, 3.0],
        }],
        None,
    )
    .unwrap()
    .into();
    let mut animation = TextureAnimation::new();
    animation.set_tracks(std::slice::from_ref(&track));
    let mut model = Model::<wc3_mdx::V1100>::new();
    model.set_texture_animations(&[animation]);
    let parsed = Model::<wc3_mdx::V1100>::decode(&model.encode().unwrap()).unwrap();
    assert_eq!(parsed.texture_animations()[0].tracks(), vec![track]);
}
