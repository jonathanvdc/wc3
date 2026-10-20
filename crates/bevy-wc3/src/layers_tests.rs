use super::*;
use crate::{compile_model, spawn_model, ModelMaterial};
use bevy::ecs::world::CommandQueue;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use wc3::model::{conversion::ImportOptions, io, profiles::Universal};

fn model(app: &mut App) -> Entity {
    let source = io::mdlx::decode_mdl(
        include_str!("../tests/fixtures/particle_capture.mdl"),
        &ImportOptions::default(),
    )
    .unwrap()
    .value
    .widen::<Universal>();
    let compiled = compile_model(&source, |_| None).unwrap();
    let mut meshes = Assets::<Mesh>::default();
    let prepared = compiled.prepare(
        &mut meshes,
        &mut Assets::<SkinnedMeshInverseBindposes>::default(),
    );
    let mut queue = CommandQueue::default();
    let root = spawn_model(
        &mut Commands::new(&mut queue, app.world()),
        &mut meshes,
        &mut Assets::<ModelMaterial>::default(),
        &prepared,
        Transform::default(),
    );
    queue.apply(app.world_mut());
    root
}

#[test]
fn unrelated_application_hierarchy_churn_does_not_revisit_render_entities() {
    let mut app = App::new();
    app.init_resource::<LayerWork>();
    app.add_systems(Update, synchronize_model_layers);
    let root = model(&mut app);
    app.world_mut()
        .entity_mut(root)
        .insert(RenderLayers::layer(1));
    app.update();
    let first_work = app.world().resource::<LayerWork>().render_entities;
    assert!(first_work > 0);
    app.update();
    assert_eq!(
        app.world().resource::<LayerWork>().render_entities,
        first_work
    );
    let parent = app.world_mut().spawn_empty().id();
    let child = app.world_mut().spawn(ChildOf(parent)).id();
    app.update();
    assert_eq!(
        app.world().resource::<LayerWork>().render_entities,
        first_work
    );
    app.world_mut().entity_mut(child).remove::<ChildOf>();
    app.update();
    assert_eq!(
        app.world().resource::<LayerWork>().render_entities,
        first_work
    );
    // Reparenting an application container that actually carries a model must
    // still reconcile the owned draws, including when the container detaches.
    app.world_mut().entity_mut(root).remove::<RenderLayers>();
    app.update();
    let inherited_work = app.world().resource::<LayerWork>().render_entities;
    let container = app.world_mut().spawn_empty().id();
    app.world_mut().entity_mut(root).insert(ChildOf(container));
    app.update();
    let attached_work = app.world().resource::<LayerWork>().render_entities;
    assert!(attached_work > inherited_work);
    app.world_mut()
        .entity_mut(container)
        .insert(ChildOf(parent));
    app.update();
    let reparented_work = app.world().resource::<LayerWork>().render_entities;
    assert!(reparented_work > attached_work);
    app.world_mut().entity_mut(container).remove::<ChildOf>();
    app.update();
    assert!(app.world().resource::<LayerWork>().render_entities > reparented_work);
}
