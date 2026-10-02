use super::super::Wc3LayerState;
use super::*;
use wc3::model::animation::{Animatable, Sequence, Track, ValueKeyframe};
#[test]
fn geoset_alpha_hides_decay_geometry_and_combines_with_layer_alpha() {
    let mut app = App::new();
    app.insert_resource(Assets::<Wc3LayerMaterial>::default());
    app.add_systems(Update, animate_layers);
    let root = app
        .world_mut()
        .spawn(Wc3Animation {
            sequence: 0,
            elapsed_ms: 0.0,
            speed: 1.0,
            playing: false,
            sequences: vec![Sequence::new("Stand", [0, 101]).unwrap()],
            global_sequences: vec![],
        })
        .id();
    let material = app
        .world_mut()
        .resource_mut::<Assets<Wc3LayerMaterial>>()
        .add(Wc3LayerMaterial {
            base: StandardMaterial::default(),
            extension: Wc3LayerState {
                filter: LayerFilterMode::None,
                no_depth_test: false,
                no_depth_set: false,
                ..default()
            },
        });
    let entity = app
        .world_mut()
        .spawn((
            AnimatedLayer {
                root,
                alpha: Animatable::Static(0.5),
                geoset_alpha: Some(Animatable::Animated(
                    Track::linear(
                        vec![
                            ValueKeyframe {
                                frame: 0,
                                value: 0.0,
                            },
                            ValueKeyframe {
                                frame: 100,
                                value: 1.0,
                            },
                        ],
                        None,
                    )
                    .unwrap(),
                )),
                geoset_color: None,
                texture_id: Animatable::Static(0),
            },
            MeshMaterial3d(material.clone()),
            Visibility::Inherited,
        ))
        .id();
    app.update();
    assert_eq!(
        *app.world().entity(entity).get::<Visibility>().unwrap(),
        Visibility::Hidden
    );
    app.world_mut()
        .entity_mut(root)
        .get_mut::<Wc3Animation>()
        .unwrap()
        .elapsed_ms = 50.0;
    app.update();
    assert_eq!(
        *app.world().entity(entity).get::<Visibility>().unwrap(),
        Visibility::Inherited
    );
    assert_eq!(
        app.world()
            .resource::<Assets<Wc3LayerMaterial>>()
            .get(&material)
            .unwrap()
            .base
            .base_color
            .to_srgba()
            .alpha,
        0.25,
    );
    assert_eq!(
        app.world()
            .resource::<Assets<Wc3LayerMaterial>>()
            .get(&material)
            .unwrap()
            .base
            .alpha_mode,
        AlphaMode::AlphaToCoverage,
    );
    app.world_mut()
        .entity_mut(root)
        .get_mut::<Wc3Animation>()
        .unwrap()
        .elapsed_ms = 100.0;
    app.update();
    let material = app
        .world()
        .resource::<Assets<Wc3LayerMaterial>>()
        .get(&material)
        .unwrap();
    assert_eq!(material.base.alpha_mode, AlphaMode::Opaque);
    app.world_mut()
        .entity_mut(entity)
        .get_mut::<AnimatedLayer>()
        .unwrap()
        .alpha = Animatable::Static(0.0);
    app.update();
    assert_eq!(
        *app.world().entity(entity).get::<Visibility>().unwrap(),
        Visibility::Hidden
    );
}

#[test]
fn missing_color_keys_reset_to_white_after_sequence_change() {
    let mut app = App::new();
    app.insert_resource(Assets::<Wc3LayerMaterial>::default());
    app.add_systems(Update, animate_layers);
    let root = app
        .world_mut()
        .spawn(Wc3Animation {
            sequence: 0,
            elapsed_ms: 0.0,
            speed: 1.0,
            playing: false,
            sequences: vec![
                Sequence::new("Stand", [0, 100]).unwrap(),
                Sequence::new("Other", [200, 300]).unwrap(),
            ],
            global_sequences: vec![],
        })
        .id();
    let handle = app
        .world_mut()
        .resource_mut::<Assets<Wc3LayerMaterial>>()
        .add(Wc3LayerMaterial {
            base: StandardMaterial::default(),
            extension: Wc3LayerState {
                filter: LayerFilterMode::Blend,
                no_depth_test: false,
                no_depth_set: false,
                ..default()
            },
        });
    app.world_mut().spawn((
        AnimatedLayer {
            root,
            alpha: Animatable::Static(0.5),
            geoset_alpha: None,
            geoset_color: Some(Animatable::Animated(
                Track::linear(
                    vec![ValueKeyframe {
                        frame: 0,
                        value: [1.0, 0.0, 0.0],
                    }],
                    None,
                )
                .unwrap(),
            )),
            texture_id: Animatable::Static(0),
        },
        MeshMaterial3d(handle.clone()),
        Visibility::Inherited,
    ));
    app.update();
    assert_eq!(
        app.world()
            .resource::<Assets<Wc3LayerMaterial>>()
            .get(&handle)
            .unwrap()
            .base
            .base_color
            .to_linear(),
        LinearRgba::new(1.0, 0.0, 0.0, 0.5)
    );
    app.world_mut()
        .entity_mut(root)
        .get_mut::<Wc3Animation>()
        .unwrap()
        .play(1);
    app.update();
    assert_eq!(
        app.world()
            .resource::<Assets<Wc3LayerMaterial>>()
            .get(&handle)
            .unwrap()
            .base
            .base_color
            .to_linear(),
        LinearRgba::new(1.0, 1.0, 1.0, 0.5)
    );
}
