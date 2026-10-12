use super::*;
use crate::animation::Wc3Animation;
use crate::materials::layers::{animate_layers, AnimatedLayer};
use crate::materials::textures::Wc3TextureBindings;
use bevy::asset::{AssetEvent, AssetPlugin};
use bevy::ecs::message::MessageCursor;
use wc3::model::animation::{Animatable, Sequence};

#[test]
fn constant_materials_share_by_template_and_state_and_detach_on_change() {
    let mut app = App::new();
    app.insert_resource(Assets::<Wc3LayerMaterial>::default());
    app.add_systems(Update, share_constant_materials);
    let template = MaterialTemplateId::new();
    let mut parts = Vec::new();
    for _ in 0..3 {
        let material = app
            .world_mut()
            .resource_mut::<Assets<Wc3LayerMaterial>>()
            .add(Wc3LayerMaterial::default());
        parts.push(
            app.world_mut()
                .spawn((ConstantMaterial(template), MeshMaterial3d(material)))
                .id(),
        );
    }
    app.world_mut()
        .entity_mut(parts[2])
        .insert(Wc3IndependentMaterial);
    app.update();
    let handle = |app: &App, entity| {
        app.world()
            .get::<MeshMaterial3d<Wc3LayerMaterial>>(entity)
            .unwrap()
            .0
            .clone()
    };
    let shared = handle(&app, parts[0]);
    assert_eq!(shared, handle(&app, parts[1]));
    assert_ne!(shared, handle(&app, parts[2]));
    let mut candidate = app
        .world()
        .resource::<Assets<Wc3LayerMaterial>>()
        .get(&shared)
        .unwrap()
        .clone();
    candidate.base.base_color = Color::linear_rgba(0.2, 0.4, 0.6, 1.0);
    let mut part = MeshMaterial3d(shared.clone());
    apply_material(
        &mut app.world_mut().resource_mut::<Assets<Wc3LayerMaterial>>(),
        &mut part,
        candidate,
        true,
    );
    app.world_mut().entity_mut(parts[0]).insert(part);
    app.update();
    assert_ne!(handle(&app, parts[0]), handle(&app, parts[1]));
    assert_eq!(handle(&app, parts[1]), shared);
    assert_eq!(
        app.world()
            .resource::<Assets<Wc3LayerMaterial>>()
            .get(&shared)
            .unwrap()
            .base
            .base_color,
        Color::WHITE
    );
    // Rejoining an identical state reuses the existing material.
    let mut candidate = app
        .world()
        .resource::<Assets<Wc3LayerMaterial>>()
        .get(&shared)
        .unwrap()
        .clone();
    candidate.base.base_color = Color::WHITE;
    let mut part = MeshMaterial3d(handle(&app, parts[0]));
    apply_material(
        &mut app.world_mut().resource_mut::<Assets<Wc3LayerMaterial>>(),
        &mut part,
        candidate,
        true,
    );
    app.world_mut().entity_mut(parts[0]).insert(part);
    app.update();
    assert_eq!(handle(&app, parts[0]), shared);
}

#[test]
fn sampling_unchanged_layers_does_not_emit_material_modifications() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Wc3LayerMaterial>();
    app.add_systems(Update, animate_layers);
    let root = app
        .world_mut()
        .spawn((
            Wc3Animation {
                sequence: 0,
                elapsed_ms: 0.,
                speed: 1.,
                playing: false,
                sequences: vec![Sequence::new("Stand", [0, 100]).unwrap()],
                global_sequences: vec![],
                event_playback: Default::default(),
                pose_playback: Default::default(),
            },
            Wc3TextureBindings::default(),
        ))
        .id();
    let material = app
        .world_mut()
        .resource_mut::<Assets<Wc3LayerMaterial>>()
        .add(Wc3LayerMaterial::default());
    app.world_mut().spawn((
        AnimatedLayer {
            root,
            alpha: Animatable::Static(1.).into(),
            geoset_alpha: None,
            geoset_color: None,
            texture_id: Animatable::Static(0).into(),
        },
        MeshMaterial3d(material),
        Visibility::Inherited,
    ));
    app.update();
    app.update();
    let mut cursor = MessageCursor::<AssetEvent<Wc3LayerMaterial>>::default();
    cursor.read(app.world().resource()).for_each(drop);
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .seek(50.);
    app.update();
    assert!(!cursor
        .read(app.world().resource())
        .any(|event| matches!(event, AssetEvent::Modified { .. })));
}
