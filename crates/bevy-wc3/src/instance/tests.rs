use super::*;
use crate::assets::model::Wc3Model;
use crate::materials::layers::{animate_layers, AnimatedLayer};

#[test]
fn repeated_instances_prepare_one_model() {
    let source = Wc3Model::decode(include_bytes!("../../tests/fixtures/quad_model.mdx")).unwrap();
    let textures = crate::assets::loader::ResolvedModelTextures::default();
    let mut app = App::new();
    app.insert_resource(Assets::<Wc3ModelAsset>::default());
    app.insert_resource(Assets::<Mesh>::default());
    app.insert_resource(Assets::<Wc3LayerMaterial>::default());
    app.insert_resource(Assets::<SkinnedMeshInverseBindposes>::default());
    app.init_resource::<PreparedModelCache>();
    app.add_systems(Update, spawn_loaded_instances);
    let handle = app
        .world_mut()
        .resource_mut::<Assets<Wc3ModelAsset>>()
        .add(Wc3ModelAsset {
            source,
            textures,
            models: default(),
        });
    let first = app
        .world_mut()
        .spawn(Wc3ModelInstance::new(handle.clone()))
        .id();
    let second = app
        .world_mut()
        .spawn(Wc3ModelInstance::new(handle.clone()))
        .id();
    app.update();
    assert!(app.world().entity(first).contains::<Wc3Animation>());
    assert!(app.world().entity(second).contains::<Wc3Animation>());
    assert_eq!(
        app.world().resource::<PreparedModelCache>().prepared.len(),
        1
    );
    let meshes = app.world().resource::<Assets<Mesh>>().len();
    let materials = app.world().resource::<Assets<Wc3LayerMaterial>>().len();
    let bindposes = app
        .world()
        .resource::<Assets<SkinnedMeshInverseBindposes>>()
        .len();
    app.world_mut().spawn(Wc3ModelInstance::new(handle));
    app.update();
    assert_eq!(app.world().resource::<Assets<Mesh>>().len(), meshes);
    assert!(app.world().resource::<Assets<Wc3LayerMaterial>>().len() > materials);
    assert_eq!(
        app.world()
            .resource::<Assets<SkinnedMeshInverseBindposes>>()
            .len(),
        bindposes
    );
}

#[test]
fn changing_bindings_updates_only_the_target_instance() {
    let mut source =
        Wc3Model::decode(include_bytes!("../../tests/fixtures/quad_model.mdx")).unwrap();
    let mut bitmaps = source.model.textures();
    bitmaps[0].path.set_text("").unwrap();
    bitmaps[0].replaceable_id = 31;
    source.model.set_textures(&bitmaps);
    let mut app = App::new();
    app.insert_resource(Assets::<Wc3ModelAsset>::default());
    app.insert_resource(Assets::<Image>::default());
    app.insert_resource(Assets::<Mesh>::default());
    app.insert_resource(Assets::<Wc3LayerMaterial>::default());
    app.insert_resource(Assets::<SkinnedMeshInverseBindposes>::default());
    app.init_resource::<PreparedModelCache>();
    app.add_systems(Update, (spawn_loaded_instances, animate_layers).chain());
    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
    let model = app
        .world_mut()
        .resource_mut::<Assets<Wc3ModelAsset>>()
        .add(Wc3ModelAsset {
            source,
            models: default(),
            textures: crate::assets::loader::ResolvedModelTextures {
                bitmaps: vec![crate::assets::loader::ResolvedTexture {
                    replaceable_id: 31,
                    ..default()
                }],
                ..default()
            },
        });
    let first = app
        .world_mut()
        .spawn(Wc3ModelInstance::new(model.clone()))
        .id();
    let second = app.world_mut().spawn(Wc3ModelInstance::new(model)).id();
    app.update();
    app.world_mut()
        .entity_mut(first)
        .get_mut::<Wc3TextureBindings>()
        .unwrap()
        .set_replaceable(31, image.clone());
    app.update();
    let mut query = app
        .world_mut()
        .query::<(&AnimatedLayer, &MeshMaterial3d<Wc3LayerMaterial>)>();
    let handles: Vec<_> = query
        .iter(app.world())
        .map(|(layer, material)| (layer.root, material.0.clone()))
        .collect();
    let materials = app.world().resource::<Assets<Wc3LayerMaterial>>();
    assert!(handles.iter().any(|(root, handle)| *root == first
        && materials.get(handle).unwrap().base.base_color_texture == Some(image.clone())));
    assert!(handles.iter().any(|(root, handle)| *root == second
        && materials
            .get(handle)
            .unwrap()
            .base
            .base_color_texture
            .is_none()));
}
