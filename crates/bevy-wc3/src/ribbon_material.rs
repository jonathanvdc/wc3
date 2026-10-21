//! Fixed render state shared with the resident trail transport.
use super::{Blend, ModelMaterial};
use crate::effects::blend::EffectBlend;
use bevy::prelude::AlphaMode;

/// Fixed render operations expressible by the shared GPU trail transport.
pub(crate) struct EffectState {
    pub blend: EffectBlend,
    pub alpha_cutoff: f32,
    pub depth_test: bool,
    pub depth_write: bool,
}
pub(crate) fn effect_state(material: &ModelMaterial) -> Option<EffectState> {
    let state = &material.extension;
    let authored_blend = state.blend.unwrap_or(match material.base.alpha_mode {
        AlphaMode::Opaque | AlphaMode::Mask(_) | AlphaMode::AlphaToCoverage => Blend::Opaque,
        AlphaMode::Add => Blend::AdditiveAlpha,
        AlphaMode::Multiply => Blend::Modulate,
        AlphaMode::Premultiplied => Blend::Premultiplied,
        _ => Blend::Alpha,
    });
    let blend = match authored_blend {
        Blend::Opaque if matches!(material.base.alpha_mode, AlphaMode::Mask(_)) => {
            EffectBlend::Cutout
        }
        Blend::Opaque => EffectBlend::Opaque,
        Blend::Alpha => EffectBlend::Alpha,
        Blend::Additive => EffectBlend::Additive,
        Blend::AdditiveAlpha => EffectBlend::AdditiveAlpha,
        Blend::Modulate => EffectBlend::Modulate,
        Blend::Modulate2x => EffectBlend::Modulate2x,
        Blend::Premultiplied | Blend::ModulateAdd | Blend::InverseSourceAlphaAdd => return None,
    };
    let material_cutoff = match material.base.alpha_mode {
        AlphaMode::Mask(cutoff) => cutoff,
        _ => 0.,
    };
    let alpha_cutoff = material_cutoff.max(state.combiner.clip.x).max(0.);
    Some(EffectState {
        blend,
        alpha_cutoff,
        depth_test: state.depth_test,
        depth_write: state.depth_write.unwrap_or(authored_blend == Blend::Opaque),
    })
}

#[cfg(test)]
#[path = "ribbon_material_tests.rs"]
mod tests;
