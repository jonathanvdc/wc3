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
    model.set_cameras(&[camera]).unwrap();
    let decoded = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
    let camera = &decoded.cameras().unwrap()[0];
    assert_eq!(camera.name(), "Portrait");
    assert_eq!(camera.position(), [1.0, 2.0, 3.0]);
    assert_eq!(camera.target_position(), [4.0, 5.0, 6.0]);
    assert_eq!(camera.field_of_view(), 0.7);
    assert_eq!(camera.far_clip(), 1000.0);
    assert_eq!(camera.near_clip(), 10.0);
    assert_eq!(camera.tracks().unwrap(), vec![track]);
}
