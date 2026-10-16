//! Fixed render state shared with the resident trail transport.
use super::{UniversalBlend, UniversalMaterial};
use crate::effects::blend::EffectBlend;
use bevy::prelude::AlphaMode;

/// Fixed render operations expressible by the shared GPU trail transport.
pub(crate) struct EffectState {
    pub blend: EffectBlend,
    pub depth_test: bool,
    pub depth_write: bool,
}
pub(crate) fn effect_state(material: &UniversalMaterial) -> Option<EffectState> {
    let state = &material.extension;
    let authored_blend = state.blend.unwrap_or(match material.base.alpha_mode {
        AlphaMode::Opaque | AlphaMode::Mask(_) | AlphaMode::AlphaToCoverage => {
            UniversalBlend::Opaque
        }
        AlphaMode::Add => UniversalBlend::AdditiveAlpha,
        AlphaMode::Multiply => UniversalBlend::Modulate,
        AlphaMode::Premultiplied => UniversalBlend::Premultiplied,
        _ => UniversalBlend::Alpha,
    });
    let blend = match authored_blend {
        UniversalBlend::Opaque if matches!(material.base.alpha_mode, AlphaMode::Mask(_)) => {
            EffectBlend::Cutout
        }
        UniversalBlend::Opaque => EffectBlend::Opaque,
        UniversalBlend::Alpha => EffectBlend::Alpha,
        UniversalBlend::Additive => EffectBlend::Additive,
        UniversalBlend::AdditiveAlpha => EffectBlend::AdditiveAlpha,
        UniversalBlend::Modulate => EffectBlend::Modulate,
        UniversalBlend::Modulate2x => EffectBlend::Modulate2x,
        UniversalBlend::Premultiplied => return None,
    };
    if state.combiner.clip.x > 0. && !matches!(blend, EffectBlend::Opaque | EffectBlend::Cutout) {
        return None;
    }
    Some(EffectState {
        blend,
        depth_test: state.depth_test,
        depth_write: state
            .depth_write
            .unwrap_or(authored_blend == UniversalBlend::Opaque),
    })
}
