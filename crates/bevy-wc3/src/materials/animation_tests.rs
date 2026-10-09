use super::*;
use crate::assets::loader::{ResolvedModelTextures, ResolvedTexture};
use crate::materials::Wc3LayerState;
use bevy::render::render_resource::TextureFormat;
use wc3::model::animation::{Sequence, Track, ValueKeyframe};

fn animation() -> Wc3Animation {
    Wc3Animation {
        sequence: 0,
        elapsed_ms: 50.0,
        speed: 1.0,
        playing: false,
        sequences: vec![Sequence::new("Stand", [0, 100]).unwrap()],
        global_sequences: vec![100],
        event_playback: Default::default(),
        pose_playback: Default::default(),
    }
}

#[test]
fn uv_animation_rotates_and_scales_around_center() {
    let definition = TextureAnimation {
        translation: Some(
            Track::linear(
                vec![
                    ValueKeyframe {
                        frame: 0,
                        value: [0.0; 3],
                    },
                    ValueKeyframe {
                        frame: 100,
                        value: [0.4, 0.0, 0.0],
                    },
                ],
                None,
            )
            .unwrap(),
        ),
        scaling: Some(
            Track::linear(
                vec![ValueKeyframe {
                    frame: 0,
                    value: [2.0, 3.0, 1.0],
                }],
                None,
            )
            .unwrap(),
        ),
        rotation: Some(
            Track::linear(
                vec![ValueKeyframe {
                    frame: 0,
                    value: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array(),
                }],
                None,
            )
            .unwrap(),
        ),
    };
    let transform =
        texture_transform_at_time(Some(&definition), SamplingTime::live(animation().time()));
    assert!(transform
        .transform_point2(Vec2::splat(0.5))
        .abs_diff_eq(Vec2::new(0.7, 0.5), 1e-5));
    assert!(transform
        .transform_point2(Vec2::new(1.0, 0.5))
        .abs_diff_eq(Vec2::new(0.7, 1.5), 1e-5));
}

#[test]
fn hd_controls_and_slots_follow_animation_and_overrides() {
    let mut app = App::new();
    app.insert_resource(Assets::<Image>::default());
    app.insert_resource(Assets::<Wc3LayerMaterial>::default());
    app.add_message::<AssetEvent<Image>>();
    app.add_systems(Update, animate_surface);
    let handles: Vec<_> = (0..6)
        .map(|_| {
            app.world_mut().resource_mut::<Assets<Image>>().add({
                let mut image = Image::default();
                image.texture_descriptor.format = TextureFormat::Rgba8Unorm;
                image
            })
        })
        .collect();
    let bindings = Wc3TextureBindings::default().with_defaults(ResolvedModelTextures {
        bitmaps: handles
            .iter()
            .map(|handle| ResolvedTexture {
                default: Some(handle.clone()),
                ..default()
            })
            .collect(),
        ..default()
    });
    let root = app.world_mut().spawn((animation(), bindings)).id();
    let material = app
        .world_mut()
        .resource_mut::<Assets<Wc3LayerMaterial>>()
        .add(Wc3LayerMaterial {
            base: StandardMaterial::default(),
            extension: Wc3LayerState::default(),
        });
    let mut layer = Layer::<V1800>::new();
    layer.set_shader_type(ShaderType::HD_DEFAULT_UNIT);
    layer.set_texture_slots(
        &(0..6)
            .map(|role| wc3::model::materials::LayerTextureSlot {
                texture_type: role,
                texture_id: Animatable::Static(role),
            })
            .collect::<Vec<_>>(),
    );
    layer.set_emissive_gain(Animatable::Animated(
        Track::linear(
            vec![
                ValueKeyframe {
                    frame: 0,
                    value: 0.0,
                },
                ValueKeyframe {
                    frame: 100,
                    value: 4.0,
                },
            ],
            Some(0),
        )
        .unwrap(),
    ));
    let mut slots = layer.texture_slots().to_vec();
    slots[1].texture_id = Animatable::Animated(
        Track::step(
            vec![
                ValueKeyframe { frame: 0, value: 1 },
                ValueKeyframe {
                    frame: 75,
                    value: 5,
                },
            ],
            None,
        )
        .unwrap(),
    );
    layer.set_texture_slots(&slots);
    layer.set_fresnel_opacity(Animatable::Animated(
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
    ));
    layer.set_fresnel_color(Animatable::Static([0.2, 0.4, 0.8]));
    layer.set_fresnel_team_color(Animatable::Static(0.3));
    let mut surface = AnimatedSurface::new(&layer, None, 3);
    surface.root = root;
    let other_bindings = app.world().get::<Wc3TextureBindings>(root).unwrap().clone();
    let other_root = app.world_mut().spawn((animation(), other_bindings)).id();
    let other_material = app
        .world_mut()
        .resource_mut::<Assets<Wc3LayerMaterial>>()
        .add(Wc3LayerMaterial {
            base: StandardMaterial::default(),
            extension: Wc3LayerState::default(),
        });
    let mut other_surface = surface.clone();
    other_surface.root = other_root;
    app.world_mut()
        .spawn((other_surface, MeshMaterial3d(other_material.clone())));
    app.world_mut()
        .spawn((surface, MeshMaterial3d(material.clone())));
    app.update();
    let assets = app.world().resource::<Assets<Wc3LayerMaterial>>();
    let value = assets.get(&material).unwrap();
    assert_eq!(value.base.normal_map_texture, Some(handles[1].clone()));
    assert_eq!(value.extension.orm, Some(handles[2].clone()));
    assert_eq!(value.base.emissive.red, 2.0);
    assert_eq!(value.extension.hd.maps, UVec4::ONE);
    assert_eq!(value.base.depth_bias, 3.0);
    assert_eq!(value.extension.hd.fresnel, Vec4::new(0.5, 0.3, 0.0, 0.0));
    assert_eq!(
        value.extension.hd.fresnel_color,
        Vec4::new(0.2, 0.4, 0.8, 1.0)
    );
    app.world_mut()
        .entity_mut(root)
        .get_mut::<Wc3TextureBindings>()
        .unwrap()
        .set_slot(
            crate::materials::textures::Wc3TextureSlot::Bitmap(2),
            handles[5].clone(),
        );
    app.update();
    assert_eq!(
        app.world()
            .resource::<Assets<Wc3LayerMaterial>>()
            .get(&material)
            .unwrap()
            .extension
            .orm,
        Some(handles[5].clone())
    );
    assert_eq!(
        app.world()
            .resource::<Assets<Wc3LayerMaterial>>()
            .get(&other_material)
            .unwrap()
            .extension
            .orm,
        Some(handles[2].clone())
    );
    app.world_mut()
        .entity_mut(root)
        .get_mut::<Wc3Animation>()
        .unwrap()
        .elapsed_ms = 80.0;
    app.update();
    assert_eq!(
        app.world()
            .resource::<Assets<Wc3LayerMaterial>>()
            .get(&material)
            .unwrap()
            .base
            .normal_map_texture,
        Some(handles[5].clone())
    );
    assert_eq!(
        app.world()
            .resource::<Assets<Wc3LayerMaterial>>()
            .get(&other_material)
            .unwrap()
            .base
            .normal_map_texture,
        Some(handles[1].clone())
    );
    // Seeking back resamples every role rather than retaining the previous map.
    app.world_mut()
        .entity_mut(root)
        .get_mut::<Wc3Animation>()
        .unwrap()
        .elapsed_ms = 0.0;
    app.update();
    assert_eq!(
        app.world()
            .resource::<Assets<Wc3LayerMaterial>>()
            .get(&material)
            .unwrap()
            .base
            .normal_map_texture,
        Some(handles[1].clone())
    );
}
