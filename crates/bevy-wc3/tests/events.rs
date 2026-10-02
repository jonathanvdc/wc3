use bevy::asset::{AssetApp, AssetPlugin};
use bevy::camera::visibility::VisibilityPlugin;
use bevy::ecs::world::CommandQueue;
use bevy::image::ImagePlugin;
use bevy::mesh::{skinning::SkinnedMeshInverseBindposes, MeshPlugin};
use bevy::prelude::*;
use bevy::shader::Shader;
use bevy::time::TimeUpdateStrategy;
use bevy_wc3::{
    prepare_model, spawn_prepared_model, Wc3Animation, Wc3BevyPlugin, Wc3LayerMaterial, Wc3Model,
    Wc3ModelEvent, Wc3Systems,
};
use std::time::Duration;

#[derive(Resource, Default)]
struct Received(Vec<Wc3ModelEvent>);

fn receive(mut events: MessageReader<Wc3ModelEvent>, mut received: ResMut<Received>) {
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
    let source = Wc3Model::decode_mdl(include_str!("fixtures/events.mdl")).unwrap();
    let mut meshes = Assets::<Mesh>::default();
    let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
    let mut materials = Assets::<Wc3LayerMaterial>::default();
    let prepared = prepare_model(&mut meshes, &mut bindposes, &source, |_| None).unwrap();
    let mut queue = CommandQueue::default();
    let root = spawn_prepared_model(
        &mut Commands::new(&mut queue, app.world()),
        &mut meshes,
        &mut materials,
        &prepared,
    );
    queue.apply(app.world_mut());
    app.update();
    assert_eq!(app.world().resource::<Received>().0.len(), 1);
    assert_eq!(app.world().resource::<Received>().0[0].root, root);
    app.world_mut().resource_mut::<Received>().0.clear();
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .seek(200.0);
    app.update();
    assert!(app.world().resource::<Received>().0.is_empty());
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    app.update();
    let received = &app.world().resource::<Received>().0;
    assert_eq!(received.len(), 3);
    assert_eq!(received[0].name, "Custom notification");
    assert_eq!(received[1].name, "Custom notification");
    assert_eq!(received[2].name, "SNDxUnresolved");
    assert!(received.iter().all(|event| event.elapsed_ms == 250.0));
    assert_eq!(
        app.world().get::<Wc3Animation>(root).unwrap().elapsed_ms(),
        300.0
    );
}
