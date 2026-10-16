use super::super::ModelMaterial;
use super::*;
use wc3::model::{AnimationSample, ModelBuilder, Time};

#[test]
fn world_scroll_is_sampled_absolutely_and_keeps_clamped_coordinates() {
    let base =
        Affine2::from_scale_angle_translation(Vec2::new(2.0, 3.0), 0.75, Vec2::new(0.2, -0.4));
    let scrolled = scrolling_transform(base, [0.5, -0.25], 6.0);
    assert_eq!(scrolled.matrix2, base.matrix2);
    assert_eq!(scrolled.translation, Vec2::new(3.2, -1.9));
    assert_eq!(scrolling_transform(base, [0.5, -0.25], 0.0), base);
    assert_eq!(scrolling_transform(base, [0.0; 2], 6.0), base);
    // Resampling an earlier pose never integrates from a previously sampled UV.
    assert_eq!(
        scrolling_transform(base, [0.5, -0.25], 2.0).translation,
        Vec2::new(1.2, -0.9),
    );
}

#[test]
fn world_compiler_retains_independent_scroll_velocities() {
    let model = ModelBuilder::<Universal>::new("world scrolling")
        .finish()
        .unwrap();
    let surface = world_surface::Material {
        shader: Some(0),
        uv_scroll: [[0.25, -0.5], [-0.75, 1.0]],
        ..default()
    };
    let pass = compile_world(&model, &surface, &TextureImages::default()).unwrap();
    assert_eq!(pass.animation.uv_scroll, surface.uv_scroll);
    assert!(pass.animation.is_animated());
    assert!(!pass
        .diagnostics
        .iter()
        .any(|line| line.contains("UV scroll")));
}

#[test]
fn procedural_scroll_uses_shared_clock_without_a_clip_and_avoids_unchanged_writes() {
    let model = ModelBuilder::<Universal>::new("clockless world")
        .finish()
        .unwrap();
    let surface = world_surface::Material {
        shader: Some(0),
        uv_scroll: [[0.25, -0.5], [0.5, 0.0]],
        ..default()
    };
    let pass = compile_world(&model, &surface, &TextureImages::default()).unwrap();
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
