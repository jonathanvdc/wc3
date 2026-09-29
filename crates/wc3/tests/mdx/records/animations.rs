use wc3::model::animation::{Animatable, TangentKeyframe, Track, ValueKeyframe};
use wc3::model::mdx::{self, Read as _, Write as _};
use wc3::model::scene::Node;
use wc3::model::Vec3;

#[test]
fn node_transform_tracks_round_trip() {
    let track = Track::<Vec3>::hermite(
        vec![TangentKeyframe {
            frame: 100,
            value: [1.0, 2.0, 3.0],
            in_tangent: [0.0; 3],
            out_tangent: [4.0, 5.0, 6.0],
        }],
        None,
    )
    .unwrap();
    let mut node = Node::new("Animated", 4).unwrap();
    node.translation = Some(track);
    assert_eq!(Node::decode_mdx(&node.encode_mdx().unwrap()).unwrap(), node);
}

#[test]
fn track_order_is_normalized_and_name_padding_is_preserved() {
    let mut node = Node::new("N", 2).unwrap();
    let track = Track::<Vec3>::linear(
        vec![ValueKeyframe {
            frame: 7,
            value: [1.0, 2.0, 3.0],
        }],
        None,
    )
    .unwrap();
    node.translation = Some(track.clone());
    node.scaling = Some(track);
    let mut canonical = node.encode_mdx().unwrap();
    canonical[20] = 0xe1;
    let mut reversed = canonical[..96].to_vec();
    reversed.extend_from_slice(&canonical[128..]);
    reversed.extend_from_slice(&canonical[96..128]);
    let parsed: Node = Node::decode_mdx(&reversed).unwrap();
    assert_eq!(parsed.encode_mdx().unwrap(), canonical);
}

#[test]
fn standalone_integer_track_preserves_values_and_rejects_trailing_bytes() {
    let track = Track::<u32>::linear(
        vec![ValueKeyframe {
            frame: 10,
            value: u32::MAX,
        }],
        None,
    )
    .unwrap();
    let bytes = track.encode_mdx().unwrap();
    assert_eq!(Track::<u32>::decode_mdx(&bytes).unwrap(), track);
    assert!(Track::<u32>::decode_mdx(&[bytes, vec![0]].concat()).is_err());
}

#[test]
fn invalid_interpolation_and_reserved_sequence_are_rejected() {
    let track = Track::<Vec3>::step(vec![], None).unwrap();
    let mut bytes = track.encode_mdx().unwrap();
    bytes[4..8].copy_from_slice(&4u32.to_le_bytes());
    assert!(Track::<Vec3>::decode_mdx(&bytes).is_err());
    assert!(Track::<Vec3>::step(vec![], Some(u32::MAX)).is_err());
}

#[test]
fn interpolation_modes_preserve_their_key_shapes() {
    let plain = ValueKeyframe {
        frame: 1,
        value: [1.0, 2.0, 3.0],
    };
    let spline = TangentKeyframe {
        frame: 2,
        value: [1.0, 2.0, 3.0],
        in_tangent: [4.0, 5.0, 6.0],
        out_tangent: [7.0, 8.0, 9.0],
    };
    for track in [
        Track::<Vec3>::step(vec![plain.clone()], None).unwrap(),
        Track::<Vec3>::linear(vec![plain], None).unwrap(),
        Track::<Vec3>::hermite(vec![spline.clone()], None).unwrap(),
        Track::<Vec3>::bezier(vec![spline], None).unwrap(),
    ] {
        assert_eq!(
            Track::<Vec3>::decode_mdx(&track.encode_mdx().unwrap()).unwrap(),
            track
        );
    }
}

#[derive(Debug, PartialEq, mdx::Read, mdx::Write)]
#[mdx(sized(tag = *b"TEST"))]
struct Property {
    #[mdx(tag = *b"COLR")]
    color: Animatable<Vec3>,
    #[mdx(tag = *b"ALPH")]
    alpha: Animatable<f32>,
}

#[test]
fn repeated_tags_replace_tracks_and_keep_bases_and_metadata() {
    let first = Track::<Vec3>::linear(
        vec![ValueKeyframe {
            frame: 1,
            value: [1.0; 3],
        }],
        None,
    )
    .unwrap();
    let last = Track::<Vec3>::bezier(
        vec![TangentKeyframe {
            frame: -2,
            value: [2.0; 3],
            in_tangent: [3.0; 3],
            out_tangent: [4.0; 3],
        }],
        Some(7),
    )
    .unwrap();
    let mut value = Property {
        color: Animatable::Both {
            value: [0.25; 3],
            track: first,
        },
        alpha: Animatable::Static(0.5),
    };
    let mut bytes = value.encode_mdx().unwrap();
    bytes.extend_from_slice(b"COLR");
    bytes.extend_from_slice(&last.encode_mdx().unwrap());
    let size = bytes.len() as u32;
    bytes[..4].copy_from_slice(&size.to_le_bytes());
    value.color.set_track(last);
    let decoded = Property::decode_mdx(&bytes).unwrap();
    assert_eq!(decoded, value);
    assert!(decoded.encode_mdx().unwrap().len() < bytes.len());
}

#[test]
fn unknown_tags_are_rejected_and_malformed_replacements_are_not_ignored() {
    let mut node = Node::new("N", 1).unwrap();
    node.translation = Some(Track::step(vec![], None).unwrap());
    let mut bytes = node.encode_mdx().unwrap();
    bytes[96..100].copy_from_slice(b"KATV");
    assert!(<Node as mdx::Read>::decode_mdx(&bytes).is_err());
    bytes[96..100].copy_from_slice(b"KGTR");
    bytes.extend_from_slice(b"KGTR");
    let len = bytes.len() as u32;
    bytes[..4].copy_from_slice(&len.to_le_bytes());
    assert!(<Node as mdx::Read>::decode_mdx(&bytes).is_err());
}
