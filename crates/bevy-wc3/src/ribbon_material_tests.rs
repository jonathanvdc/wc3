use super::*;
use bevy::prelude::StandardMaterial;

#[test]
fn scalar_cutoff_is_independent_of_every_supported_trail_blend_and_depth_policy() {
    for (blend, expected) in [
        (Blend::Opaque, EffectBlend::Opaque),
        (Blend::Alpha, EffectBlend::Alpha),
        (Blend::Additive, EffectBlend::Additive),
        (Blend::AdditiveAlpha, EffectBlend::AdditiveAlpha),
        (Blend::Modulate, EffectBlend::Modulate),
        (Blend::Modulate2x, EffectBlend::Modulate2x),
    ] {
        for cutoff in [0., 1. / 255., 0.125, 0.75] {
            for depth_test in [false, true] {
                for depth_write in [false, true] {
                    let mut material = ModelMaterial::default();
                    material.extension.blend = Some(blend);
                    material.extension.combiner.clip.x = cutoff;
                    material.extension.depth_test = depth_test;
                    material.extension.depth_write = Some(depth_write);
                    let state = effect_state(&material)
                        .expect("scalar rejection does not require another blend equation");
                    assert_eq!(state.blend, expected);
                    assert_eq!(state.alpha_cutoff, cutoff);
                    assert_eq!(state.depth_test, depth_test);
                    assert_eq!(state.depth_write, depth_write);
                }
            }
        }
    }
}

#[test]
fn trail_mask_and_combiner_cutoffs_retain_the_stricter_material_test() {
    let mut material = ModelMaterial {
        base: StandardMaterial {
            alpha_mode: AlphaMode::Mask(0.25),
            ..Default::default()
        },
        ..Default::default()
    };
    let state = effect_state(&material).unwrap();
    assert_eq!(state.blend, EffectBlend::Cutout);
    assert_eq!(state.alpha_cutoff, 0.25);
    material.extension.combiner.clip.x = 0.75;
    assert_eq!(effect_state(&material).unwrap().alpha_cutoff, 0.75);
    // Changing the sampled material does not leave a fixed cutout pipeline.
    material.base.alpha_mode = AlphaMode::Blend;
    material.extension.blend = Some(Blend::Alpha);
    material.extension.combiner.clip.x = 0.125;
    let faded = effect_state(&material).unwrap();
    assert_eq!(faded.blend, EffectBlend::Alpha);
    assert_eq!(faded.alpha_cutoff, 0.125);
}
