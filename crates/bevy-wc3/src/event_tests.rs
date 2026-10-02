use super::*;
use crate::animation::advance_animation;
use crate::animation::pose::animate_nodes;
use crate::assets::Wc3Model;
use crate::instance::spawn::spawn_prepared_model;
use crate::instance::spawn::Wc3NodeEntities;
use crate::materials::Wc3LayerMaterial;
use crate::preparation::prepare_model;
use crate::schedule::{configure, Wc3Systems};
use bevy::ecs::world::CommandQueue;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;

fn scene() -> (App, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin));
    configure(&mut app);
    app.add_message::<Wc3ModelEvent>();
    app.add_systems(
        PostUpdate,
        animate_nodes.in_set(Wc3Systems::EvaluateNodePoses),
    );
    app.add_systems(
        PostUpdate,
        dispatch_events.in_set(Wc3Systems::DispatchEvents),
    );
    let source = Wc3Model::decode_mdl(include_str!("../tests/fixtures/events.mdl")).unwrap();
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<Wc3LayerMaterial>::default();
    let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = prepare_model(&mut meshes, &mut bindposes, &source, |_| None).unwrap();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, app.world());
    let first = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    let second = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
    commands
        .entity(first)
        .insert(Transform::from_xyz(100.0, 0.0, 0.0));
    queue.apply(app.world_mut());
    app.world_mut()
        .get_mut::<Wc3Animation>(second)
        .unwrap()
        .playing = false;
    (app, first, second)
}

fn drain(app: &mut App) -> Vec<Wc3ModelEvent> {
    app.world_mut()
        .resource_mut::<Messages<Wc3ModelEvent>>()
        .drain()
        .collect()
}

fn advance(app: &mut App, root: Entity, elapsed_ms: f64) -> Vec<Wc3ModelEvent> {
    // Advance rather than seek, preserving the dispatch interval under test.
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .elapsed_ms = elapsed_ms;
    app.update();
    drain(app)
}

#[test]
fn dispatches_all_keys_in_order_with_occurrence_poses_and_instance_identity() {
    let (mut app, root, other) = scene();
    app.update();
    let start = drain(&mut app);
    assert_eq!(start.len(), 1);
    assert_eq!(start[0].frame, 1000);
    assert_eq!(start[0].root, root);
    assert_eq!(start[0].name, "Custom notification");
    assert_eq!(start[0].object_id, 1);
    assert_eq!(start[0].sequence, Some(0));
    assert_eq!(start[0].global_sequence_id, None);
    assert_eq!(start[0].transform.translation(), Vec3::new(103.0, 0.0, 0.0));
    assert!(drain(&mut app).is_empty());
    let events = advance(&mut app, root, 300.0);
    assert_eq!(
        events
            .iter()
            .map(|e| (e.event_index, e.key_index, e.elapsed_ms))
            .collect::<Vec<_>>(),
        [(2, 0, 125.0), (0, 1, 250.0), (0, 2, 250.0), (1, 0, 250.0)]
    );
    assert_eq!(events[0].global_sequence_id, Some(0));
    assert!(events[0]
        .transform
        .translation()
        .abs_diff_eq(Vec3::new(101.25, 1.25, 0.0), 1e-5));
    assert_eq!(
        events[1].transform.translation(),
        Vec3::new(105.5, 0.0, 0.0)
    );
    assert_eq!(events[3].name, "SNDxUnresolved");
    assert_eq!(
        app.world().get::<ChildOf>(events[1].node).unwrap().parent(),
        app.world()
            .get::<Wc3NodeEntities>(root)
            .unwrap()
            .get(0)
            .unwrap()
    );
    app.world_mut()
        .get_mut::<Wc3Animation>(other)
        .unwrap()
        .playing = true;
    let independent = advance(&mut app, root, 300.0);
    assert_eq!(independent.len(), 1);
    assert_eq!(independent[0].root, other);
    assert_eq!(independent[0].transform.translation().x, 3.0);
    // Definitions are shared, but cursor and node entities are instance-local.
    assert!(Arc::ptr_eq(
        &app.world().get::<EventState>(root).unwrap().definitions,
        &app.world().get::<EventState>(other).unwrap().definitions
    ));
    assert_ne!(independent[0].node, start[0].node);
}

#[test]
fn loops_dispatch_every_occurrence_and_keep_distinct_start_and_end_poses() {
    let (mut app, root, _) = scene();
    let events = advance(&mut app, root, 2250.0);
    let main: Vec<_> = events.iter().filter(|e| e.event_index == 0).collect();
    assert_eq!(
        main.iter()
            .map(|e| (e.frame, e.elapsed_ms))
            .collect::<Vec<_>>(),
        [
            (1000, 0.0),
            (1250, 250.0),
            (1250, 250.0),
            (1000, 1000.0),
            (2000, 1000.0),
            (1250, 1250.0),
            (1250, 1250.0),
            (1000, 2000.0),
            (2000, 2000.0),
            (1250, 2250.0),
            (1250, 2250.0)
        ]
    );
    assert_eq!(main[3].transform.translation().x, 103.0);
    assert_eq!(main[4].transform.translation().x, 113.0);
    assert_eq!(events.iter().filter(|e| e.event_index == 2).count(), 5);
    assert!(advance(&mut app, root, 2250.0).is_empty());
}

#[test]
fn seeks_skip_events_and_same_sequence_restart_fires_start_once() {
    let (mut app, root, _) = scene();
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .seek(200.0);
    assert!(advance(&mut app, root, 200.0).is_empty());
    assert_eq!(advance(&mut app, root, 250.0).len(), 3);
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .seek(1100.0);
    // Playback between the seek and dispatch still crosses the later global key.
    let events = advance(&mut app, root, 1200.0);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].elapsed_ms, 1125.0);
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .seek(0.0);
    assert!(advance(&mut app, root, 0.0).is_empty());
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .play(0);
    assert_eq!(advance(&mut app, root, 0.0).len(), 1);
    assert!(advance(&mut app, root, 0.0).is_empty());
    let mut animation = app.world_mut().get_mut::<Wc3Animation>(root).unwrap();
    assert!(!animation.seek(f64::NAN));
    assert_eq!(animation.elapsed_ms(), 0.0);
    animation.play(usize::MAX);
    assert_eq!(animation.sequence(), 0);
    assert!(advance(&mut app, root, 0.0).is_empty());
}

#[test]
fn pause_reverse_and_nonlooping_playback_have_explicit_dispatch_behavior() {
    let (mut app, root, _) = scene();
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .playing = false;
    assert!(advance(&mut app, root, 0.0).is_empty());
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .playing = true;
    assert_eq!(advance(&mut app, root, 0.0).len(), 1);
    app.world_mut().get_mut::<Wc3Animation>(root).unwrap().speed = -1.0;
    assert!(advance(&mut app, root, -100.0).is_empty());
    app.world_mut().get_mut::<Wc3Animation>(root).unwrap().speed = 1.0;
    assert_eq!(advance(&mut app, root, 0.0).len(), 1);
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .play(1);
    let events = advance(&mut app, root, 1500.0);
    let frames: Vec<_> = events
        .iter()
        .filter(|e| e.event_index == 0)
        .map(|e| e.frame)
        .collect();
    assert_eq!(frames, [3000, 3250, 4000]);
    assert_eq!(
        events
            .iter()
            .find(|e| e.frame == 4000)
            .unwrap()
            .transform
            .translation()
            .x,
        113.0
    );
    // Global events continue after a nonlooping model sequence reaches its end.
    let later = advance(&mut app, root, 1800.0);
    assert_eq!(later.len(), 1);
    assert_eq!(later[0].event_index, 2);
}

#[test]
fn ordinary_animation_advance_drives_dispatch_in_the_same_update() {
    let (mut app, root, _) = scene();
    app.add_systems(
        Update,
        advance_animation.in_set(Wc3Systems::AdvanceAnimation),
    );
    app.insert_resource(Time::<()>::default());
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(300));
    // Run the schedules directly so MinimalPlugins does not replace the injected delta.
    app.world_mut().run_schedule(Update);
    app.world_mut().run_schedule(PostUpdate);
    assert_eq!(
        app.world().get::<Wc3Animation>(root).unwrap().elapsed_ms(),
        300.0
    );
    assert_eq!(drain(&mut app).len(), 5);
}

#[test]
fn global_boundary_keys_sample_distinct_start_and_end_poses() {
    let (mut app, root, _) = scene();
    let mut state = app.world_mut().get_mut::<EventState>(root).unwrap();
    Arc::make_mut(&mut state.definitions)[2].set_frames(&[0, 500]);
    let events = advance(&mut app, root, 500.0);
    let boundary: Vec<_> = events
        .iter()
        .filter(|event| event.event_index == 2 && event.elapsed_ms == 500.0)
        .collect();
    assert_eq!(boundary.len(), 2);
    assert_eq!(boundary[0].frame, 0);
    assert_eq!(
        boundary[0].transform.translation(),
        Vec3::new(105.0, 0.0, 0.0)
    );
    assert_eq!(boundary[1].frame, 500);
    assert_eq!(
        boundary[1].transform.translation(),
        Vec3::new(105.0, 5.0, 0.0)
    );
}

#[test]
fn global_only_models_dispatch_and_restart_without_a_sequence() {
    let (mut app, root, _) = scene();
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .sequences
        .clear();
    let events = advance(&mut app, root, 700.0);
    assert_eq!(events.len(), 2);
    assert!(events
        .iter()
        .all(|event| event.sequence.is_none() && event.event_index == 2));
    assert_eq!(
        events[0].transform.translation(),
        Vec3::new(100.0, 1.25, 0.0)
    );
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .restart();
    assert_eq!(advance(&mut app, root, 125.0).len(), 1);
    assert!(advance(&mut app, root, 125.0).is_empty());
}

#[test]
fn blended_switch_dispatches_only_destination_events_at_blended_occurrence_poses() {
    use std::time::Duration;

    let (mut app, root, _) = scene();
    advance(&mut app, root, 500.0);
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .play_with_blend(1, Duration::from_millis(1000));
    let sampled = app
        .world()
        .get::<Wc3Animation>(root)
        .unwrap()
        .sample_at_elapsed(500.0);
    *app.world_mut().get_mut::<Wc3Animation>(root).unwrap() = sampled;
    app.update();
    let events = drain(&mut app);
    let local: Vec<_> = events
        .iter()
        .filter(|event| event.global_sequence_id.is_none())
        .collect();
    assert_eq!(local.len(), 2);
    assert_eq!(
        local.iter().map(|event| event.frame).collect::<Vec<_>>(),
        [3000, 3250]
    );
    assert!(local
        .iter()
        .all(|event| event.sequence == Some(1) && event.event_index == 0));
    // Frozen parent source is x=5. Destination is x=0 at its start and
    // x=2.5 at 250ms, with transition weights 0 and 0.25 respectively.
    assert!(local[0]
        .transform
        .translation()
        .abs_diff_eq(Vec3::new(108.0, 0.0, 0.0), 1e-5));
    assert!(local[1]
        .transform
        .translation()
        .abs_diff_eq(Vec3::new(107.375, 0.0, 0.0), 1e-5));
    app.update();
    assert!(drain(&mut app).is_empty());
}
