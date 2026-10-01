use super::*;
use crate::material::Wc3LayerState;
use wc3::model::animation::ValueKeyframe;
use wc3::model::materials::LayerFilterMode;

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
fn sequence_clock_loops_and_global_clock_runs_independently() {
    let animation = Wc3Animation {
        sequence: 0,
        elapsed_ms: 150.0,
        speed: 1.0,
        playing: true,
        sequences: vec![Sequence::new("Stand", [1000, 1100]).unwrap()],
        global_sequences: vec![200],
    };
    let sequence_track = Track::linear(
        vec![
            ValueKeyframe {
                frame: 1000,
                value: 0.0f32,
            },
            ValueKeyframe {
                frame: 1100,
                value: 1.0f32,
            },
        ],
        None,
    )
    .unwrap();
    let global_track = Track::linear(
        vec![
            ValueKeyframe {
                frame: 0,
                value: 0.0f32,
            },
            ValueKeyframe {
                frame: 200,
                value: 1.0f32,
            },
        ],
        Some(0),
    )
    .unwrap();
    assert_eq!(sample(&sequence_track, &animation), Some(0.5));
    assert_eq!(sample(&global_track, &animation), Some(0.75));
}

#[test]
fn color_tracks_use_sequence_and_global_clocks_with_neutral_fallback() {
    let mut animation = Wc3Animation {
        sequence: 0,
        elapsed_ms: 50.0,
        speed: 1.0,
        playing: false,
        sequences: vec![
            Sequence::new("Stand", [0, 100]).unwrap(),
            Sequence::new("Other", [200, 300]).unwrap(),
        ],
        global_sequences: vec![100],
    };
    let track = Track::linear(
        vec![
            ValueKeyframe {
                frame: 0,
                value: [1.0, 0.0, 0.0],
            },
            ValueKeyframe {
                frame: 100,
                value: [0.0, 0.0, 1.0],
            },
        ],
        None,
    )
    .unwrap();
    assert_eq!(sample(&track, &animation), Some([0.5, 0.0, 0.5]));
    animation.play(1);
    assert_eq!(sample(&track, &animation), None);
    let global = Track::linear(
        vec![
            ValueKeyframe {
                frame: 0,
                value: [1.0, 0.0, 0.0],
            },
            ValueKeyframe {
                frame: 100,
                value: [0.0, 0.0, 1.0],
            },
        ],
        Some(0),
    )
    .unwrap();
    animation.elapsed_ms = 150.0;
    assert_eq!(sample(&global, &animation), Some([0.5, 0.0, 0.5]));
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
