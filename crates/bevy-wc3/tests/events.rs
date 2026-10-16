use bevy::asset::{AssetApp, AssetPlugin};
use bevy::camera::visibility::VisibilityPlugin;
use bevy::ecs::world::CommandQueue;
use bevy::image::ImagePlugin;
use bevy::mesh::{skinning::SkinnedMeshInverseBindposes, MeshPlugin};
use bevy::prelude::*;
use bevy::shader::Shader;
use bevy::time::TimeUpdateStrategy;
use bevy_wc3::{
    compile_model, spawn_model, ModelEvent, ModelMaterial, Playback, Wc3BevyPlugin, Wc3Systems,
};
use std::time::Duration;
use wc3::model::{conversion::ImportOptions, io, profiles::Universal};

#[derive(Resource, Default)]
struct Received(Vec<ModelEvent>);

fn receive(mut events: MessageReader<ModelEvent>, mut received: ResMut<Received>) {
    received.0.extend(events.read().cloned());
}

#[test]
fn plugin_dispatches_public_messages_to_ordered_application_consumers() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TransformPlugin,
        VisibilityPlugin,
        MeshPlugin,
        ImagePlugin::default(),
    ));
    app.init_asset::<Shader>();
    app.add_plugins(Wc3BevyPlugin);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app.init_resource::<Received>();
    app.add_systems(PostUpdate, receive.after(Wc3Systems::DispatchEvents));
    let source = io::mdlx::decode_mdl(
        include_str!("fixtures/events.mdl"),
        &ImportOptions::default(),
    )
    .unwrap()
    .value
    .widen::<Universal>();
    let mut meshes = Assets::<Mesh>::default();
    let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
    let mut materials = Assets::<ModelMaterial>::default();
    let compiled = compile_model(&source, |_| None).unwrap();
    let prepared = compiled.prepare(&mut meshes, &mut bindposes);
    let mut queue = CommandQueue::default();
    let root = spawn_model(
        &mut Commands::new(&mut queue, app.world()),
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::default(),
    );
    queue.apply(app.world_mut());
    app.world_mut()
        .get_mut::<Playback>(root)
        .unwrap()
        .play_immediately(prepared.clips()[0].0);
    app.update();
    assert_eq!(app.world().resource::<Received>().0.len(), 1);
    assert_eq!(app.world().resource::<Received>().0[0].root, root);
    app.world_mut().resource_mut::<Received>().0.clear();
    app.world_mut()
        .get_mut::<Playback>(root)
        .unwrap()
        .seek(0.2)
        .unwrap();
    app.update();
    assert!(app.world().resource::<Received>().0.is_empty());
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    app.update();
    let received = &app.world().resource::<Received>().0;
    assert_eq!(received.len(), 3);
    assert_eq!(received[0].name.as_ref(), "Custom notification");
    assert_eq!(received[1].name.as_ref(), "Custom notification");
    assert_eq!(received[2].name.as_ref(), "SNDxUnresolved");
    assert!(received.iter().all(|event| event.occurrence_time == 0.25));
    assert!((app.world().get::<Playback>(root).unwrap().timeline_time - 0.3).abs() < 1e-12);
}
