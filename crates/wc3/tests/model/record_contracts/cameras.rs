use super::*;

#[test]
fn camera_dof_scalar_keywords_produce_correct_keyed_wire_tags() {
    for (source, tag, keyword, value) in [
        (
            "DOFDistance 180.0,",
            *b"IDUF",
            "FocusDistanceKeys",
            180.0f32,
        ),
        ("FocalLength 50.0,", *b"ELAF", "FocalLengthKeys", 50.0),
        ("FStop 2.8,", *b"PTSF", "FStopKeys", 2.8),
    ] {
        let track = CameraTrack::decode_mdl(source).unwrap();
        let mut expected = tag.to_vec();
        expected.extend_from_slice(&1u32.to_le_bytes()); // count
        expected.extend_from_slice(&0u32.to_le_bytes()); // stepped
        expected.extend_from_slice(&u32::MAX.to_le_bytes()); // no global sequence
        expected.extend_from_slice(&0i32.to_le_bytes()); // frame zero
        expected.extend_from_slice(&value.to_le_bytes());
        assert_eq!(track.encode_mdx().unwrap(), expected);
        assert_eq!(CameraTrack::decode_mdx(&expected).unwrap(), track);
        let text = track.encode_mdl().unwrap();
        assert!(text.starts_with(keyword));
        assert_eq!(CameraTrack::decode_mdl(&text).unwrap(), track);
    }
    assert!(CameraTrack::decode_mdl("FocusDistance 5.0,").is_err());
    assert!(CameraTrack::decode_mdl("FStop 2.8").is_err());
}

#[test]
fn camera_extended_tracks_survive_model_io_and_conversion() {
    let visibility = AnimationTrack::<CameraVisibility>::linear(
        vec![ValueKeyframe {
            frame: -100,
            value: 0.5,
        }],
        Some(0),
    )
    .unwrap();
    let tracks = vec![
        CameraTrack::Visibility(visibility),
        CameraTrack::focus_distance(180.0),
        CameraTrack::focal_length(50.0),
        CameraTrack::f_stop(2.8),
    ];
    let mut camera = Camera::<V1800>::new("Portrait").unwrap();
    camera.tracks = tracks.to_vec();
    let mut model = Model::<V1800>::new();
    model.set_cameras(&[camera]);
    let bytes = model.encode_mdx().unwrap();
    let parsed = Model::<V1800>::decode_mdx(&bytes).unwrap();
    assert_eq!(parsed.cameras()[0].tracks.as_slice(), tracks);
    assert_eq!(parsed.encode_mdx().unwrap(), bytes);
    // These track tags are recognized by the new client without a layout gate.
    let converted = model
        .convert::<V800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert_eq!(converted.cameras()[0].tracks.as_slice(), tracks);
    assert_eq!(
        Model::<V800>::decode_mdx(&converted.encode_mdx().unwrap())
            .unwrap()
            .cameras()[0]
            .tracks
            .as_slice(),
        tracks
    );
}
