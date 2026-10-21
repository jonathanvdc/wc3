use super::*;

fn material(mode: AlphaMode, blend: Blend) -> (ModelMaterialExtension, StandardMaterial) {
    let mut state = ModelMaterialExtension {
        blend: Some(blend),
        depth_test: true,
        depth_write: Some(true),
        ..Default::default()
    };
    state.m2_opacity = Some(M2Opacity::new(mode, state.blend));
    (
        state,
        StandardMaterial {
            alpha_mode: mode,
            ..Default::default()
        },
    )
}

#[test]
fn fades_restore_authored_phase_and_blend_without_changing_depth() {
    for authored in [AlphaMode::Opaque, AlphaMode::Mask(128. / 255.)] {
        let (mut state, mut base) = material(authored, Blend::Opaque);
        for alpha in [1., 0.25, 0., 1.] {
            base.base_color.set_alpha(alpha);
            state.apply_opacity(&mut base);
            assert_eq!(
                base.alpha_mode,
                if alpha < 0.99999 {
                    AlphaMode::Blend
                } else {
                    authored
                }
            );
            assert_eq!(
                state.blend,
                Some(if alpha < 0.99999 {
                    Blend::Alpha
                } else {
                    Blend::Opaque
                })
            );
            assert!(state.depth_test);
            assert_eq!(state.depth_write, Some(true));
            assert!(
                !state.apply_opacity(&mut base),
                "unchanged samples must not dirty assets"
            );
        }
    }
}

#[test]
fn visibility_threshold_and_blend_add_use_the_combined_surface_factor() {
    let (mut state, mut base) = material(AlphaMode::Opaque, Blend::Opaque);
    for (alpha, visible) in [
        (0., false),
        (0.00009, false),
        (0.0001, true),
        (0.99998, true),
        (0.99999, true),
    ] {
        base.base_color.set_alpha(alpha);
        state.apply_opacity(&mut base);
        assert_eq!(state.has_visible_opacity(alpha), visible);
        assert_eq!(state.combiner.clip.x == f32::MAX, !visible);
        assert_eq!(base.alpha_mode == AlphaMode::Opaque, alpha >= 0.99999);
    }
    let (mut state, mut base) = material(AlphaMode::Premultiplied, Blend::Premultiplied);
    base.base_color.set_alpha(0.);
    state.apply_opacity(&mut base);
    assert!(state.has_visible_opacity(0.));
    assert_eq!(base.alpha_mode, AlphaMode::Premultiplied);
    assert_eq!(state.blend, Some(Blend::Premultiplied));
    assert_ne!(state.combiner.clip.x, f32::MAX);
}

#[test]
fn faded_cutout_retains_texture_coverage_while_its_color_blends() {
    let threshold = 128. / 255.;
    let (mut state, mut base) = material(AlphaMode::Mask(threshold), Blend::Opaque);
    base.base_color.set_alpha(0.25);
    state.apply_opacity(&mut base);
    assert_eq!(state.combiner.clip.x, threshold * 0.25);
    assert_eq!(base.alpha_mode, AlphaMode::Blend);
    for texture_alpha in [0., 0.25, 0.5, 0.75, 1.] {
        assert_eq!(
            texture_alpha * 0.25 < state.combiner.clip.x,
            texture_alpha < threshold
        );
    }
}

#[test]
fn authored_blends_keep_their_fragment_coverage_rules() {
    for (mode, blend, cutoff) in [
        (AlphaMode::Opaque, Blend::Opaque, 0.),
        (AlphaMode::Premultiplied, Blend::Premultiplied, 0.),
        (AlphaMode::Blend, Blend::Alpha, 1. / 255.),
        (AlphaMode::Blend, Blend::Additive, 1. / 255.),
        (AlphaMode::Blend, Blend::AdditiveAlpha, 1. / 255.),
        (AlphaMode::Blend, Blend::Modulate, 1. / 255.),
    ] {
        let (mut state, mut base) = material(mode, blend);
        base.base_color.set_alpha(0.25);
        state.apply_opacity(&mut base);
        assert_eq!(state.combiner.clip.x, cutoff);
    }
    let (mut state, mut base) = material(AlphaMode::Mask(128. / 255.), Blend::Opaque);
    for alpha in [0.999995, 1.2] {
        base.base_color.set_alpha(alpha);
        state.apply_opacity(&mut base);
        assert_eq!(base.alpha_mode, AlphaMode::Mask(128. / 255. * alpha));
    }
}
