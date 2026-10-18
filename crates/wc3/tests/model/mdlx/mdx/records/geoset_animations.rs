use wc3::model::formats::mdlx::animation::{Animatable, Track};

use wc3::model::formats::mdlx::animation::{GeosetAnimation, ValueKeyframe};
use wc3::model::formats::mdlx::mdl::Read as _;
use wc3::model::formats::mdlx::mdx::Read as _;
use wc3::model::formats::mdlx::mdx::Write as _;

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
            [ValueKeyframe {
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
