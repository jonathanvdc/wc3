use super::*;
use wc3::model::animation::Track;

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
        let camera = Camera::<V1800>::decode_mdl(&format!(
            "Camera \"C\" {{ FieldOfView 1, FarClip 2, {source} }}"
        ))
        .unwrap();
        let mut expected = tag.to_vec();
        expected.extend_from_slice(&1u32.to_le_bytes());
        expected.extend_from_slice(&0u32.to_le_bytes());
        expected.extend_from_slice(&u32::MAX.to_le_bytes());
        expected.extend_from_slice(&0i32.to_le_bytes());
        expected.extend_from_slice(&value.to_le_bytes());
        let binary = camera.encode_mdx().unwrap();
        assert_eq!(&binary[binary.len() - expected.len()..], expected);
        let text = camera.encode_mdl().unwrap();
        assert!(text.contains(keyword));
        assert_eq!(Camera::<V1800>::decode_mdl(&text).unwrap(), camera);
    }
    assert!(Camera::<V800>::decode_mdl(
        "Camera \"C\" { FieldOfView 1, FarClip 2, FocusDistance 5.0, }"
    )
    .is_err());
    assert!(
        Camera::<V800>::decode_mdl("Camera \"C\" { FieldOfView 1, FarClip 2, FStop 2.8 }").is_err()
    );
}

#[test]
fn camera_extended_tracks_survive_model_io_and_conversion() {
    let visibility = Track::<f32>::linear(
        vec![ValueKeyframe {
            frame: -100,
            value: 0.5,
        }],
        Some(0),
    )
    .unwrap();
    let mut camera = Camera::<V1800>::new("Portrait").unwrap();
    camera.visibility = Some(visibility);
    camera.focus_distance = Some(Track::constant(180.0));
    camera.focal_length = Some(Track::constant(50.0));
    camera.f_stop = Some(Track::constant(2.8));
    let mut model = Model::<V1800>::new();
    model.set_cameras(&[camera.clone()]);
    let bytes = model.encode_mdx().unwrap();
    let parsed = Model::<V1800>::decode_mdx(&bytes).unwrap();
    assert_eq!(parsed.cameras()[0], camera);
    assert_eq!(parsed.encode_mdx().unwrap(), bytes);
    // These track tags are recognized by the new client without a layout gate.
    let converted = model
        .convert::<V800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert_eq!(converted.cameras()[0].visibility, camera.visibility);
    assert_eq!(converted.cameras()[0].focus_distance, camera.focus_distance);
    assert_eq!(converted.cameras()[0].focal_length, camera.focal_length);
    assert_eq!(converted.cameras()[0].f_stop, camera.f_stop);
    assert_eq!(
        Model::<V800>::decode_mdx(&converted.encode_mdx().unwrap())
            .unwrap()
            .cameras()[0]
            .focus_distance,
        camera.focus_distance
    );
}
