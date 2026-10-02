use super::*;
use wc3::model::animation::Sequence;

fn animation() -> Wc3Animation {
    Wc3Animation {
        sequence: 0,
        elapsed_ms: 0.0,
        speed: 1.0,
        playing: true,
        sequences: vec![
            Sequence::new("Stand", [0, 1000]).unwrap(),
            Sequence::new("Walk", [2000, 3000]).unwrap(),
        ],
        global_sequences: vec![],
        event_playback: Default::default(),
        pose_playback: PosePlayback {
            blend_time: Duration::from_millis(1000),
            ..Default::default()
        },
    }
}

fn cached_animation() -> (Wc3Animation, Entity, Transform) {
    let mut animation = animation();
    let entity = World::new().spawn_empty().id();
    let source = Transform {
        translation: Vec3::new(2.0, 3.0, 4.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    animation.pose_playback.current = Arc::new(HashMap::from([(entity, source)]));
    (animation, entity, source)
}

#[test]
fn default_blend_endpoints_midpoint_and_shortest_rotation() {
    let (mut animation, entity, source) = cached_animation();
    animation.play(1);
    let destination = Transform {
        translation: Vec3::new(6.0, 7.0, 8.0),
        rotation: -Quat::from_rotation_z(1.0),
        scale: Vec3::splat(3.0),
    };
    assert_eq!(animation.blend_transform(entity, destination), source);
    let halfway = animation
        .sample_at_elapsed(500.0)
        .blend_transform(entity, destination);
    assert!(halfway
        .translation
        .abs_diff_eq(Vec3::new(4.0, 5.0, 6.0), 1e-5));
    assert!(halfway.scale.abs_diff_eq(Vec3::splat(2.0), 1e-5));
    assert!(halfway.rotation.angle_between(Quat::from_rotation_z(0.5)) < 1e-5);
    assert_eq!(
        animation
            .sample_at_elapsed(1000.0)
            .blend_transform(entity, destination),
        destination
    );
    assert_eq!(
        animation
            .sample_at_elapsed(1200.0)
            .blend_transform(entity, destination),
        destination
    );
}

#[test]
fn interrupted_blend_uses_latest_evaluated_pose_and_last_request_wins() {
    let (mut animation, entity, _) = cached_animation();
    animation.play(1);
    let destination = Transform::from_xyz(10.0, 0.0, 0.0);
    animation = animation.sample_at_elapsed(400.0);
    let current = animation.blend_transform(entity, destination);
    animation.pose_playback.current = Arc::new(HashMap::from([(entity, current)]));
    animation.play(0);
    animation.play_with_blend(1, Duration::from_millis(200));
    assert_eq!(animation.sequence(), 1);
    assert_eq!(animation.blend_transform(entity, destination), current);
    assert_eq!(
        animation
            .sample_at_elapsed(200.0)
            .blend_transform(entity, destination),
        destination
    );
}

#[test]
fn immediate_first_play_seek_restart_and_invalid_requests() {
    let (mut animation, entity, _) = cached_animation();
    let destination = Transform::from_xyz(10.0, 0.0, 0.0);
    animation.play(1);
    let revision = animation.event_playback.revision;
    animation.play_immediately(99);
    assert_eq!(animation.sequence(), 1);
    assert_eq!(animation.event_playback.revision, revision);
    assert!(animation.preserves_particles());
    assert!(!animation.seek(f64::NAN));
    assert!(animation.preserves_particles());
    animation.play_immediately(0);
    assert_eq!(animation.blend_transform(entity, destination), destination);
    animation.play(1);
    animation.seek(123.0);
    assert!(!animation.preserves_particles());
    assert!(animation.pose_playback.current.is_empty());
    animation.play(0);
    assert_eq!(animation.blend_transform(entity, destination), destination);
    animation.pose_playback.current = Arc::new(HashMap::from([(entity, destination)]));
    animation.play(1);
    animation.restart();
    assert!(!animation.preserves_particles());
    assert!(animation.pose_playback.current.is_empty());
}

#[test]
fn transition_clock_pauses_scales_and_does_not_reverse() {
    let (animation, entity, source) = cached_animation();
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.add_systems(Update, advance_animation);
    let root = app.world_mut().spawn(animation).id();
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .play(1);
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(250));
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .playing = false;
    app.update();
    let destination = Transform::from_xyz(10.0, 0.0, 0.0);
    assert_eq!(
        app.world()
            .get::<Wc3Animation>(root)
            .unwrap()
            .blend_transform(entity, destination),
        source
    );
    {
        let mut animation = app.world_mut().get_mut::<Wc3Animation>(root).unwrap();
        animation.playing = true;
        animation.speed = 2.0;
    }
    app.update();
    let animation = app.world().get::<Wc3Animation>(root).unwrap();
    assert_eq!(animation.elapsed_ms(), 500.0);
    let half = animation.blend_transform(entity, destination);
    assert!(half
        .translation
        .abs_diff_eq(source.translation.lerp(destination.translation, 0.5), 1e-5));
    // Birth sampling uses its earlier weight even after the visible fade completes.
    app.update();
    let animation = app.world().get::<Wc3Animation>(root).unwrap();
    assert_eq!(animation.blend_transform(entity, destination), destination);
    assert_eq!(
        animation
            .sample_at_elapsed(500.0)
            .blend_transform(entity, destination),
        half
    );
    app.world_mut().get_mut::<Wc3Animation>(root).unwrap().speed = -1.0;
    app.update();
    assert_eq!(
        app.world()
            .get::<Wc3Animation>(root)
            .unwrap()
            .blend_transform(entity, destination),
        destination
    );
}
