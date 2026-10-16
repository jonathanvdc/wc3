//! Additional age-sprite texture composition in the shared PBR material.
use super::{particle_material, ModelMaterial, ParticleUniform};
use bevy::prelude::{Handle, Image, UVec4};
use wc3::model::values::model::ParticleBlend;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ParticleComposition {
    /// Two RGB inputs and three alpha inputs.
    TwoColor,
    /// Three RGB inputs and three alpha inputs.
    ThreeColor,
}
pub(crate) fn particle_material_layers(
    blend: ParticleBlend,
    images: [Option<Handle<Image>>; 3],
    lit: bool,
    composition: ParticleComposition,
) -> ModelMaterial {
    let [primary, secondary, tertiary] = images;
    let mut material = particle_material(blend, primary, lit);
    material.extension.secondary = secondary;
    material.extension.tertiary = tertiary;
    material.extension.particle = ParticleUniform {
        mode: UVec4::new(
            match composition {
                ParticleComposition::TwoColor => 1,
                ParticleComposition::ThreeColor => 2,
            },
            0,
            0,
            0,
        ),
    };
    material
}

#[cfg(test)]
#[path = "particle_tests.rs"]
mod tests;
