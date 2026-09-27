use wc3_mdx::animation::CameraRotation;
use wc3_mdx::animation::{AnimationTrack, ValueKeyframe};
use wc3_mdx::io::{Decodable, Encodable};
use wc3_mdx::scene::Camera;
use wc3_mdx::Model;

#[test]
fn camera_fields_and_tracks_round_trip() {
    let mut camera = Camera::<wc3_mdx::V1100>::new("Portrait").unwrap();
    camera.set_position([1.0, 2.0, 3.0]);
    camera.set_target_position([4.0, 5.0, 6.0]);
    camera.set_field_of_view(0.7);
    camera.set_far_clip(1000.0);
    camera.set_near_clip(10.0);
    let track = AnimationTrack::<CameraRotation>::linear(
        vec![ValueKeyframe {
            frame: 100,
            value: 0.5,
        }],
        None,
    )
    .unwrap()
    .into();
    camera.set_tracks(std::slice::from_ref(&track));
    let mut model = Model::<wc3_mdx::V1100>::new();
    model.set_cameras(&[camera]);
    let decoded = Model::<wc3_mdx::V1100>::decode(&model.encode().unwrap()).unwrap();
    let camera = &decoded.cameras()[0];
    assert_eq!(camera.name(), "Portrait");
    assert_eq!(camera.position(), [1.0, 2.0, 3.0]);
    assert_eq!(camera.target_position(), [4.0, 5.0, 6.0]);
    assert_eq!(camera.field_of_view(), 0.7);
    assert_eq!(camera.far_clip(), 1000.0);
    assert_eq!(camera.near_clip(), 10.0);
    assert_eq!(camera.tracks(), vec![track]);
}

#[test]
fn newer_camera_size_flags_round_trip() {
    let mut camera = Camera::<wc3_mdx::V1800>::new("Portrait").unwrap();
    assert_eq!(camera.record_flags(), 3);
    camera.set_field_of_view(0.8);
    let mut model = Model::<wc3_mdx::V1800>::new();
    model.set_cameras(&[camera]);
    let bytes = model.encode().unwrap();
    let parsed = Model::<wc3_mdx::V1800>::decode(&bytes).unwrap();
    assert_eq!(parsed.cameras()[0].record_flags(), 3);
    assert_eq!(parsed.encode().unwrap(), bytes);
}
