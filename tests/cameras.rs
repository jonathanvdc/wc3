use wc3_mdx::Record;
use wc3_mdx::{AnimationTrack, Camera, Keyframe, Model};

#[test]
fn camera_fields_and_tracks_round_trip() {
    let mut camera = Camera::new("Portrait").unwrap();
    camera.set_position([1.0, 2.0, 3.0]);
    camera.set_target_position([4.0, 5.0, 6.0]);
    camera.set_field_of_view(0.7);
    camera.set_far_clip(1000.0);
    camera.set_near_clip(10.0);
    let track = AnimationTrack {
        tag: *b"KCRL",
        interpolation: 1,
        global_sequence_id: u32::MAX,
        keyframes: vec![Keyframe {
            frame: 100,
            value: vec![0.5],
            in_tangent: None,
            out_tangent: None,
        }],
    };
    camera.set_tracks(std::slice::from_ref(&track)).unwrap();
    let mut model = Model::new(1100);
    model.set_cameras(&[camera]);
    let decoded = Model::decode(&model.encode().unwrap(), 800).unwrap();
    let camera = &decoded.cameras().unwrap()[0];
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
    let mut camera = Camera::new_for_version("Portrait", 1800).unwrap();
    assert_eq!(camera.record_flags(), 3);
    camera.set_field_of_view(0.8);
    let mut model = Model::new(1800);
    model.set_cameras(&[camera]);
    let bytes = model.encode().unwrap();
    let parsed = Model::decode(&bytes, 800).unwrap();
    assert_eq!(parsed.cameras().unwrap()[0].record_flags(), 3);
    assert_eq!(parsed.encode().unwrap(), bytes);
}
