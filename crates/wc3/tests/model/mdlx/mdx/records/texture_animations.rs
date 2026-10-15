use wc3::model::formats::mdlx::animation::Track;
use wc3::model::formats::mdlx::Vec3;

use wc3::model::formats::mdlx::animation::{TextureAnimation, ValueKeyframe};
use wc3::model::formats::mdlx::mdx::Read as _;
use wc3::model::formats::mdlx::mdx::Write as _;

use wc3::model::formats::mdlx::Document;

#[test]
fn texture_animation_tracks_round_trip() {
    let track = Track::<Vec3>::linear(
        [ValueKeyframe {
            frame: 100,
            value: [1.0, 2.0, 3.0],
        }],
        None,
    )
    .unwrap();
    let mut animation = TextureAnimation::new();
    animation.translation = Some(track.clone());
    let mut model = Document::<wc3::model::formats::mdlx::V1100>::new();
    model.set_texture_animations([animation]);
    let parsed =
        Document::<wc3::model::formats::mdlx::V1100>::decode_mdx(&model.encode_mdx().unwrap())
            .unwrap()
            .into_owned();
    assert_eq!(
        parsed
            .texture_animations()
            .next()
            .unwrap()
            .translation
            .as_ref(),
        Some(&track)
    );
}
