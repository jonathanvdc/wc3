//! Immutable GPU sprite records and rendering.
mod records;
mod render;
pub(crate) use records::{
    ParticleEmitterUniform, ParticleInstance, ParticleInstances, StagedSpriteParameters,
};
pub(crate) use render::ParticleRenderPlugin;
