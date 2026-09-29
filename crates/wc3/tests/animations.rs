use wc3::model::animation::{
    AnimationTrack, LayerTextureId, NodeScaling, NodeTranslation, TangentKeyframe, TrackKind,
    TrackTag, TrackValueKind, ValueKeyframe,
};
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;
use wc3::model::mdx::{Cursor, Encoder, ReadError};
use wc3::model::scene::{Node, NodeTrack};

fn write_track<T: wc3::model::mdx::Write>(value: &T) -> Vec<u8> {
    let mut bytes = Vec::new();
    Encoder::new(&mut bytes).write(value).unwrap();
    bytes
}

fn read_track<K: TrackKind>(bytes: &[u8]) -> Result<AnimationTrack<K>, ReadError> {
    let mut cursor = Cursor::new(bytes);
    let track = cursor.read()?;
    cursor.finish()?;
    Ok(track)
}

#[test]
fn node_transform_tracks_round_trip() {
    let track: NodeTrack = AnimationTrack::<NodeTranslation>::hermite(
        vec![TangentKeyframe {
            frame: 100,
            value: [1.0, 2.0, 3.0],
            in_tangent: [0.0, 0.0, 0.0],
            out_tangent: [4.0, 5.0, 6.0],
        }],
        None,
    )
    .unwrap()
    .into();
    let mut node = Node::new("Animated", 4).unwrap();
    node.tracks = (std::slice::from_ref(&track)).to_vec();
    let parsed = Node::decode_mdx(&node.encode_mdx().unwrap()).unwrap();
    assert_eq!(parsed.tracks.as_slice(), vec![track]);
}

#[test]
fn node_keeps_name_padding_and_track_order() {
    let mut node = Node::new("N", 2).unwrap();
    let key = ValueKeyframe {
        frame: 7,
        value: [1.0, 2.0, 3.0],
    };
    let tracks: [NodeTrack; 2] = [
        AnimationTrack::<NodeScaling>::linear(vec![key.clone()], None)
            .unwrap()
            .into(),
        AnimationTrack::<NodeTranslation>::linear(vec![key], None)
            .unwrap()
            .into(),
    ];
    node.tracks = tracks.to_vec();
    let mut bytes = node.encode_mdx().unwrap();
    bytes[20] = 0xe1;
    let parsed = Node::decode_mdx(&bytes).unwrap();
    assert_eq!(parsed.encode_mdx().unwrap(), bytes);
    assert_eq!(parsed.tracks[0].tag(), TrackTag::NodeScaling);
}

#[test]
fn standalone_integer_track_preserves_values() {
    let track = AnimationTrack::<LayerTextureId>::linear(
        vec![ValueKeyframe {
            frame: 10,
            value: u32::MAX,
        }],
        None,
    )
    .unwrap();
    assert_eq!(track.tag().component_count(), 1);
    assert_eq!(track.tag().value_kind(), TrackValueKind::Integer);
    let bytes = write_track(&track);
    let parsed = read_track::<LayerTextureId>(&bytes).unwrap();
    assert_eq!(parsed.linear_keys().unwrap()[0].value, u32::MAX);
    assert!(read_track::<LayerTextureId>(&[bytes, vec![0]].concat()).is_err());
}

#[test]
fn invalid_interpolation_is_rejected_on_decode() {
    let track = AnimationTrack::<NodeTranslation>::step(vec![], None).unwrap();
    let mut bytes = write_track(&track);
    bytes[8..12].copy_from_slice(&4u32.to_le_bytes());
    assert!(read_track::<NodeTranslation>(&bytes).is_err());
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
    let tracks = [
        AnimationTrack::<NodeTranslation>::step(vec![plain.clone()], None).unwrap(),
        AnimationTrack::<NodeTranslation>::linear(vec![plain], None).unwrap(),
        AnimationTrack::<NodeTranslation>::hermite(vec![spline.clone()], None).unwrap(),
        AnimationTrack::<NodeTranslation>::bezier(vec![spline], None).unwrap(),
    ];
    for track in tracks {
        let parsed = read_track::<NodeTranslation>(&write_track(&track)).unwrap();
        assert_eq!(parsed, track);
        assert_eq!(parsed.interpolation(), track.interpolation());
    }
}

#[test]
fn typed_decoder_rejects_wrong_tag_and_reserved_sequence_id() {
    let track = AnimationTrack::<NodeTranslation>::step(vec![], None).unwrap();
    let bytes = write_track(&track);
    assert!(read_track::<NodeScaling>(&bytes).is_err());
    assert!(AnimationTrack::<NodeTranslation>::step(vec![], Some(u32::MAX)).is_err());
}

#[test]
fn tracks_use_readable_and_writable_io() {
    let track = AnimationTrack::<NodeTranslation>::linear(
        vec![ValueKeyframe {
            frame: 12,
            value: [1.0, 2.0, 3.0],
        }],
        Some(2),
    )
    .unwrap();
    let mut bytes = Vec::new();
    Encoder::new(&mut bytes).write(&track).unwrap();
    assert_eq!(&bytes[..4], b"KGTR");
    let mut cursor = Cursor::new(&bytes);
    let parsed: AnimationTrack<NodeTranslation> = cursor.read().unwrap();
    cursor.finish().unwrap();
    assert_eq!(parsed, track);
}

#[test]
fn node_track_reader_rejects_other_record_tags_without_advancing() {
    use wc3::model::animation::AttachmentVisibility;

    let attachment = AnimationTrack::<AttachmentVisibility>::step(vec![], None).unwrap();
    let bytes = write_track(&attachment);
    let mut cursor = Cursor::new(&bytes);
    assert!(cursor.read::<NodeTrack>().is_err());
    assert_eq!(cursor.position(), 0);
}
