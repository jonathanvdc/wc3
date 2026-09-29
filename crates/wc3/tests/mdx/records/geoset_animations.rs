use wc3::model::animation::GeosetColor;
use wc3::model::animation::{AnimationTrack, GeosetAnimation, GeosetAnimationFlags, ValueKeyframe};
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::Model;

#[test]
fn geoset_animation_fields_round_trip() {
    let mut animation = GeosetAnimation::new(2);
    animation.alpha = 0.5;
    animation.flags = GeosetAnimationFlags(7);
    animation.color = [0.1, 0.2, 0.3];
    let mut model = Model::<wc3::model::V1800>::new();
    model.set_geoset_animations(&[animation]);
    let parsed = Model::<wc3::model::V1800>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    let actual = &parsed.geoset_animations()[0];
    assert_eq!(actual.geoset_id, 2);
    assert_eq!(actual.alpha, 0.5);
    assert!(actual.flags.drop_shadow());
    assert!(actual.flags.color());
    assert_eq!(actual.flags.bits(), 7);
    assert_eq!(actual.color, [0.1, 0.2, 0.3]);
}

#[test]
fn geoset_animation_color_track_round_trip() {
    let mut animation = GeosetAnimation::new(1);
    let track = AnimationTrack::<GeosetColor>::linear(
        vec![ValueKeyframe {
            frame: 25,
            value: [0.2, 0.4, 0.8],
        }],
        None,
    )
    .unwrap()
    .into();
    animation.tracks = (std::slice::from_ref(&track)).to_vec();
    let parsed = GeosetAnimation::decode_mdx(&animation.encode_mdx().unwrap()).unwrap();
    assert_eq!(parsed.tracks.as_slice(), vec![track]);
}
