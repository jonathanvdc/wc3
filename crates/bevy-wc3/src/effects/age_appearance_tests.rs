use super::*;
fn appearance() -> AgeAppearance {
    AgeAppearance {
        twinkle: [4., 0.5, 0.25, 1.25],
        table: Some(Arc::new(from_fn(|index| index as f32 / 128.))),
        textures: Some(TextureMotion {
            scales: [2., 0.5],
            middle: [Vec2::new(0.5, -1.), Vec2::new(-0.25, 0.75)],
            range: [Vec2::new(0.25, 0.5), Vec2::ZERO],
        }),
    }
}
#[test]
fn twinkle_changes_on_frequency_boundaries_and_repeats_after_128_ticks() {
    let appearance = appearance();
    let birth = AgeParticleAppearance {
        phase: 63,
        origins: [Vec2::ZERO; 2],
        velocities: [Vec2::ZERO; 2],
    };
    assert!(appearance.sample(&birth, 0.).visible);
    assert!(appearance.sample(&birth, 0.249).visible);
    assert!(appearance.sample(&birth, 0.25).visible);
    assert!(!appearance.sample(&birth, 0.5).visible);
    assert_eq!(
        appearance.sample(&birth, 0.),
        appearance.sample(&birth, 32.)
    );
    let style = appearance.sample(&birth, 0.25);
    assert_eq!(style.scale, 0.75);
}
#[test]
fn birth_uv_randomness_is_stable_and_variation_is_correlated_per_layer() {
    let appearance = appearance();
    assert!(appearance.needs_particle_state());
    assert_eq!(appearance.texture_scales(), [2., 0.5]);
    let birth = appearance.birth(123);
    assert_eq!(birth, appearance.birth(123));
    assert_ne!(birth, appearance.birth(124));
    let delta = birth.velocities[0] - Vec2::new(0.5, -1.);
    assert!((delta.x / 0.25 - delta.y / 0.5).abs() < 1e-6);
    assert_eq!(birth.velocities[1], Vec2::new(-0.25, 0.75));
    assert!(birth
        .origins
        .iter()
        .all(|uv| uv.min_element() >= 0. && uv.max_element() < 1.));
}
#[test]
fn scrolling_is_analytic_wraps_negative_velocity_and_does_not_scale_atlas_uvs() {
    let appearance = appearance();
    let birth = AgeParticleAppearance {
        phase: 0,
        origins: [Vec2::new(0.25, 0.75); 2],
        velocities: [Vec2::new(-1., 0.5), Vec2::new(2., -0.25)],
    };
    let style = appearance.sample(&birth, 1.);
    assert_eq!(style.offsets, [Vec2::new(0.25, 0.25), Vec2::new(0.25, 0.5)]);
    let mut appearance = appearance;
    appearance.textures = None;
    appearance.twinkle = [0., 1., 1., 1.];
    assert!(!appearance.needs_particle_state());
    assert_eq!(appearance.texture_scales(), [1.; 2]);
    assert_eq!(appearance.sample(&birth, 0.).scale, 1.);
}

#[test]
fn primary_only_fallback_keeps_twinkle_and_omits_extra_uv_motion() {
    let appearance = appearance();
    let birth = appearance.birth(12);
    let expected = appearance.sample(&birth, 0.5);
    let primary = appearance.primary_only();
    let actual = primary.sample(&birth, 0.5);
    assert_eq!(actual.visible, expected.visible);
    assert_eq!(actual.scale, expected.scale);
    assert_eq!(actual.offsets, [Vec2::ZERO; 2]);
    assert_eq!(primary.texture_scales(), [1.; 2]);
}
