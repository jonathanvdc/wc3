use wc3::model::animation::Track;

use wc3::model::animation::ValueKeyframe;
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::scene::{Camera, CameraVariant};
use wc3::model::Model;

#[test]
fn camera_fields_and_tracks_round_trip() {
    let mut camera = Camera::<wc3::model::V1100>::new("Portrait").unwrap();
    camera.position = [1.0, 2.0, 3.0];
    camera.target_position = [4.0, 5.0, 6.0];
    camera.field_of_view = 0.7;
    camera.far_clip = 1000.0;
    camera.near_clip = 10.0;
    let track = Track::<f32>::linear(
        vec![ValueKeyframe {
            frame: 100,
            value: 0.5,
        }],
        None,
    )
    .unwrap();
    camera.rotation = Some(track.clone());
    let mut model = Model::<wc3::model::V1100>::new();
    model.set_cameras(&[camera]);
    let decoded = Model::<wc3::model::V1100>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    let camera = &decoded.cameras()[0];
    assert_eq!(camera.name.text(), "Portrait");
    assert_eq!(camera.position, [1.0, 2.0, 3.0]);
    assert_eq!(camera.target_position, [4.0, 5.0, 6.0]);
    assert_eq!(camera.field_of_view, 0.7);
    assert_eq!(camera.far_clip, 1000.0);
    assert_eq!(camera.near_clip, 10.0);
    assert_eq!(camera.rotation.as_ref(), Some(&track));
}

#[test]
fn newer_camera_variant_round_trip() {
    let mut camera = Camera::<wc3::model::V1800>::new("Portrait").unwrap();
    assert_eq!(camera.variant, CameraVariant::Variant3);
    camera.field_of_view = 0.8;
    let mut model = Model::<wc3::model::V1800>::new();
    model.set_cameras(&[camera]);
    let bytes = model.encode_mdx().unwrap();
    let parsed = Model::<wc3::model::V1800>::decode_mdx(&bytes).unwrap();
    assert_eq!(parsed.cameras()[0].variant, CameraVariant::Variant3);
    assert_eq!(parsed.encode_mdx().unwrap(), bytes);
}

#[test]
fn camera_variants_round_trip() {
    let variants = [
        CameraVariant::Variant0,
        CameraVariant::Variant1([1; 12]),
        CameraVariant::Variant2([2; 12]),
        CameraVariant::Variant3,
        CameraVariant::Unknown(0x80),
    ];
    for variant in variants {
        let mut camera = Camera::<wc3::model::V1200>::new("Portrait").unwrap();
        camera.variant = variant;
        camera.target_position = [4.0, 5.0, 6.0];
        let mut model = Model::<wc3::model::V1200>::new();
        model.set_cameras(&[camera]);
        let bytes = model.encode_mdx().unwrap();
        let parsed = Model::<wc3::model::V1200>::decode_mdx(&bytes).unwrap();
        assert_eq!(parsed.cameras()[0].variant, variant);
        assert_eq!(parsed.cameras()[0].target_position, [4.0, 5.0, 6.0]);
        assert_eq!(parsed.encode_mdx().unwrap(), bytes);
    }
}
