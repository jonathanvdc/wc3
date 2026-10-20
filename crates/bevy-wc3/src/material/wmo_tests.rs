use super::super::ModelMaterial;
use super::*;
use crate::material::native_fixtures::wmo as wmo_shader;
use bevy::math::{Vec2, Vec3, Vec4};
use wc3::model::ModelMetadata;
use wc3::model::{AnimationSample, ModelBuilder, Time};
#[test]
fn world_scroll_is_sampled_absolutely_and_keeps_clamped_coordinates() {
    let base =
        Affine2::from_scale_angle_translation(Vec2::new(2.0, 3.0), 0.75, Vec2::new(0.2, -0.4));
    let scrolled = scrolling_transform(base, Vec2::from_array([0.5, -0.25]), 6.0);
    assert_eq!(scrolled.matrix2, base.matrix2);
    assert_eq!(scrolled.translation, Vec2::new(3.2, -1.9));
    assert_eq!(
        scrolling_transform(base, Vec2::from_array([0.5, -0.25]), 0.0),
        base
    );
    assert_eq!(
        scrolling_transform(base, Vec2::from_array([0.0; 2]), 6.0),
        base
    );
    // Resampling an earlier pose never integrates from a previously sampled UV.
    assert_eq!(
        scrolling_transform(base, Vec2::from_array([0.5, -0.25]), 2.0).translation,
        Vec2::new(1.2, -0.9),
    );
}

#[test]
fn world_compiler_retains_independent_scroll_velocities() {
    let model = ModelBuilder::<Universal>::new(ModelMetadata::Wmo(Default::default()))
        .finish()
        .unwrap();
    let surface = wmo_shader::Material {
        shader: Some(6),
        uv_scroll: [
            Vec2::from_array([0.25, -0.5]),
            Vec2::from_array([-0.75, 1.0]),
        ],
        ..default()
    };
    let pass = compile_wmo(&model, &surface.clone().into(), &TextureImages::default()).unwrap();
    assert_eq!(pass.animation.uv_scroll, surface.uv_scroll);
    assert!(pass.animation.is_animated());
    assert!(!pass
        .diagnostics
        .iter()
        .any(|line| line.message.contains("UV scroll")));
}

#[test]
fn procedural_scroll_uses_shared_clock_without_a_clip_and_avoids_unchanged_writes() {
    let model = ModelBuilder::<Universal>::new(ModelMetadata::Wmo(Default::default()))
        .finish()
        .unwrap();
    let surface = wmo_shader::Material {
        shader: Some(6),
        uv_scroll: [Vec2::from_array([0.25, -0.5]), Vec2::from_array([0.5, 0.0])],
        ..default()
    };
    let pass = compile_wmo(&model, &surface.clone().into(), &TextureImages::default()).unwrap();
    let mut material = ModelMaterial {
        base: pass.material,
        extension: pass.state,
    };
    for (seconds, expected) in [
        (2.0, Vec2::new(0.5, -1.0)),
        (2.0, Vec2::new(0.5, -1.0)),
        (1.0, Vec2::new(0.25, -0.5)),
    ] {
        let sample = model
            .animation_sample(AnimationSample {
                shared_time: Time::seconds(seconds).unwrap(),
                ..default()
            })
            .unwrap();
        let changed = pass.animation.apply_to_material(&sample, &mut material);
        assert_eq!(material.base.uv_transform.translation, expected);
        assert_eq!(
            material.extension.combiner.uv_transform.z_axis.x,
            seconds as f32 * 0.5
        );
        assert!(!pass.animation.apply_to_material(&sample, &mut material));
        if seconds == 1.0 {
            assert!(changed);
        }
    }
}

#[test]
fn world_programmes_select_native_routes_without_sampling_unimplemented_families() {
    let model = ModelBuilder::<Universal>::new(ModelMetadata::Wmo(Default::default()))
        .finish()
        .unwrap();
    for (shader, count, secondary, third, generated) in [
        (0, 1, None, None, 0),
        (4, 1, None, None, 0),
        (16, 1, None, None, 0),
        (21, 1, None, None, 0),
        (3, 2, Some(0), None, 2),
        (5, 2, Some(0), None, 2),
        (6, 2, Some(1), None, 0),
        (9, 2, Some(1), None, 0),
        (13, 2, Some(1), None, 0),
        (15, 2, Some(1), None, 0),
        (19, 2, Some(1), None, 0),
        (18, 3, Some(1), Some(2), 0),
        (20, 3, Some(1), Some(2), 0),
        (11, 3, Some(0), Some(1), 1),
        (12, 3, Some(0), Some(1), 1),
        (17, 3, Some(0), Some(1), 1),
    ] {
        let surface = wmo_shader::Material {
            shader: Some(shader),
            ..default()
        };
        let pass = compile_wmo(&model, &surface.clone().into(), &TextureImages::default()).unwrap();
        assert_eq!(
            pass.state.world.composition.x,
            if count > 1 { shader + 1 } else { 0 }
        );
        assert_eq!(
            pass.state.world.composition.y,
            if count > 1 { count } else { 0 }
        );
        assert_eq!(pass.secondary_coordinate, secondary);
        assert_eq!(pass.third_coordinate, third);
        assert_eq!(pass.state.combiner.coordinates.y, generated);
        assert_eq!(
            pass.diagnostics
                .iter()
                .any(|line| line.message.contains("partially rendered")),
            matches!(shader, 16 | 21)
        );
        assert!(
            !pass
                .diagnostics
                .iter()
                .any(|line| line.message.contains("showing its first")),
            "{shader}"
        );
    }
    for shader in [
        None,
        Some(1),
        Some(2),
        Some(7),
        Some(8),
        Some(10),
        Some(14),
        Some(22),
        Some(23),
        Some(999),
    ] {
        let surface = wmo_shader::Material {
            shader,
            ..default()
        };
        let pass = compile_wmo(&model, &surface.clone().into(), &TextureImages::default()).unwrap();
        assert_eq!(pass.state.world.composition.x, 0);
        assert!(pass
            .diagnostics
            .iter()
            .any(|line| line.message.contains("showing its first")));
    }
}

#[test]
fn reflection_programmes_scroll_the_uv_layer_without_scrolling_generated_coordinates() {
    let model = ModelBuilder::<Universal>::new(ModelMetadata::Wmo(Default::default()))
        .finish()
        .unwrap();
    let surface = wmo_shader::Material {
        shader: Some(12),
        uv_scroll: [
            Vec2::from_array([0.25, -0.5]),
            Vec2::from_array([0.5, -0.25]),
        ],
        ..default()
    };
    let pass = compile_wmo(&model, &surface.clone().into(), &TextureImages::default()).unwrap();
    assert_eq!(pass.animation.uv_scroll[1], Vec2::from_array([0.; 2]));
    assert_eq!(
        pass.animation.third_uv_scroll,
        Vec2::from_array([0.5, -0.25])
    );
    let mut material = ModelMaterial {
        base: pass.material,
        extension: pass.state,
    };
    let sample = model
        .animation_sample(AnimationSample {
            shared_time: Time::seconds(2.).unwrap(),
            ..default()
        })
        .unwrap();
    assert!(pass.animation.apply_to_material(&sample, &mut material));
    assert_eq!(material.extension.combiner.uv_transform, Mat3::IDENTITY);
    assert_eq!(
        material
            .extension
            .combiner
            .third_transform
            .z_axis
            .truncate(),
        Vec2::new(1., -0.5)
    );
    assert!(!pass.animation.apply_to_material(&sample, &mut material));
}

#[test]
fn shared_world_equations_parse_and_validate() {
    use naga::front::wgsl::parse_str;
    use naga::valid::{Capabilities, ValidationFlags, Validator};
    let source = include_str!("../shaders/world_composition.wgsl")
        .split("fn world_color")
        .next()
        .unwrap()
        .lines()
        .filter(|line| !line.starts_with("#import"))
        .collect::<Vec<_>>()
        .join("\n")
        .replace("#{MATERIAL_BIND_GROUP}", "2");
    let module =
        parse_str(&source).unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
    Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module)
        .unwrap();
}

#[test]
fn world_illumination_is_draw_local_and_nonworld_materials_keep_their_identity() {
    use super::super::ModelMaterialExtension;
    let ambient = Vec3::new(0.2, 0.4, 0.6);
    let mut ordinary = ModelMaterialExtension::default();
    assert_eq!(ordinary.set_world_illumination(ambient, 2), [0; 4]);
    assert_eq!(ordinary.world.ambient, Vec4::ZERO);
    let model = ModelBuilder::<Universal>::new(ModelMetadata::Wmo(Default::default()))
        .finish()
        .unwrap();
    let surface = wmo_shader::Material {
        shader: Some(0),
        ..default()
    };
    let mut pass = compile_wmo(&model, &surface.clone().into(), &TextureImages::default()).unwrap();
    for lighting in 0..=2 {
        let key = pass.state.set_world_illumination(ambient, lighting);
        assert_eq!(
            key,
            [
                0.2f32.to_bits(),
                0.4f32.to_bits(),
                0.6f32.to_bits(),
                lighting
            ]
        );
        assert_eq!(pass.state.world.ambient, ambient.extend(0.));
        assert_eq!(pass.state.world.flags.z, lighting);
        assert_eq!(pass.state.world.flags.x, 1);
    }
}

#[test]
fn world_extended_blends_preserve_native_rgb_and_alpha_factors() {
    use super::super::ModelMaterialKey;
    use bevy::render::render_resource::{BlendComponent, BlendFactor, BlendOperation};
    let model = ModelBuilder::<Universal>::new(ModelMetadata::Wmo(Default::default()))
        .finish()
        .unwrap();
    for (operation, source, source_alpha) in [
        (
            WorldBlend::ModulateAdd,
            BlendFactor::Dst,
            BlendFactor::DstAlpha,
        ),
        (
            WorldBlend::InverseSourceAlphaAdd,
            BlendFactor::OneMinusSrcAlpha,
            BlendFactor::OneMinusSrcAlpha,
        ),
    ] {
        let mut surface = wmo_shader::Material {
            shader: Some(0),
            ..default()
        };
        surface.state.blend = BlendOperations::Extension(operation);
        let pass = compile_wmo(&model, &surface.clone().into(), &TextureImages::default()).unwrap();
        assert_eq!(pass.material.alpha_mode, AlphaMode::Blend);
        assert!(pass.diagnostics.is_empty());
        let key = ModelMaterialKey::from(&pass.state);
        let mut state = None;
        key.apply_blend(&mut state);
        let state = state.unwrap();
        assert_eq!(
            state.color,
            BlendComponent {
                src_factor: source,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add
            }
        );
        assert_eq!(
            state.alpha,
            BlendComponent {
                src_factor: source_alpha,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add
            }
        );
        assert_eq!(key.depth_write, Some(false));
    }
}
