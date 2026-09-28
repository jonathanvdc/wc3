use wc3::model::animation::TextureTranslation;
use wc3::model::animation::{AnimationTrack, TextureAnimation, ValueKeyframe};
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::Model;

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
    animation.tracks = (std::slice::from_ref(&track)).to_vec();
    let mut model = Model::<wc3::model::V1100>::new();
    model.set_texture_animations(&[animation]);
    let parsed = Model::<wc3::model::V1100>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    assert_eq!(
        parsed.texture_animations()[0].tracks.as_slice(),
        vec![track]
    );
}
