use wc3::model::animation::{Animatable, Track};
use wc3::model::Color;

use wc3::model::animation::{GeosetAnimation, GeosetAnimationFlags, ValueKeyframe};
use wc3::model::mdl::Read as _;
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::Model;

#[test]
fn geoset_animation_fields_round_trip() {
    let mut animation = GeosetAnimation::new(2);
    animation.alpha = Animatable::Static(0.5);
    animation.flags = GeosetAnimationFlags(7);
    animation.color = Animatable::Static([0.1, 0.2, 0.3]);
    let mut model = Model::<wc3::model::V1800>::new();
    model.set_geoset_animations(&[animation]);
    let parsed = Model::<wc3::model::V1800>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    let actual = &parsed.geoset_animations()[0];
    assert_eq!(actual.geoset_id, 2);
    assert_eq!(actual.alpha, Animatable::Static(0.5));
    assert!(actual.flags.drop_shadow());
    assert!(actual.flags.color());
    assert_eq!(actual.flags.bits(), 7);
    assert_eq!(actual.color, Animatable::Static([0.1, 0.2, 0.3]));
}

#[test]
fn geoset_animation_color_track_round_trip() {
    let mut animation = GeosetAnimation::new(1);
    let track = Track::<Color>::linear(
        vec![ValueKeyframe {
            frame: 25,
            value: [0.2, 0.4, 0.8],
        }],
        None,
    )
    .unwrap();
    animation.color.set_track(track.clone());
    let parsed = GeosetAnimation::decode_mdx(&animation.encode_mdx().unwrap()).unwrap();
    assert_eq!(parsed.color.track(), Some(&track));
}

#[test]
fn static_wire_color_is_bgr_but_tracks_and_public_color_are_rgb() {
    let mut animation =
        GeosetAnimation::decode_mdl("GeosetAnim { GeosetId 0, static Color { 0.25, 0.5, 0.75 }, }")
            .unwrap();
    assert_eq!(animation.color, Animatable::Static([0.25, 0.5, 0.75]));
    let bytes = animation.encode_mdx().unwrap();
    let wire: Vec<f32> = bytes[12..24]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|bytes| f32::from_le_bytes(*bytes))
        .collect();
    assert_eq!(wire, [0.75, 0.5, 0.25]);
    assert_eq!(
        GeosetAnimation::decode_mdx(&bytes).unwrap().color,
        animation.color
    );
    animation.color.set_track(
        Track::linear(
            vec![ValueKeyframe {
                frame: 0,
                value: [1.0, 0.0, 0.0],
            }],
            None,
        )
        .unwrap(),
    );
    let bytes = animation.encode_mdx().unwrap();
    assert_eq!(
        GeosetAnimation::decode_mdx(&bytes).unwrap().color,
        animation.color
    );
    // KGAC: tag, count, interpolation, global sequence, frame, RGB value.
    assert_eq!(f32::from_le_bytes(bytes[48..52].try_into().unwrap()), 1.0);
}
