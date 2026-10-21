//! Per-view M2 sprite expansion from shared, exactly sampled live records.
mod records;
mod refraction;
pub use refraction::ParticleRefraction;
mod render;
pub(crate) use records::{
    AgeSpriteComposition, AgeSpriteData, AgeSpriteRecord, AgeSpriteStyle, AgeSpriteUniform,
    SampleBuffers,
};
pub(crate) use render::AgeSpriteRenderPlugin;

#[cfg(test)]
pub(super) use render::sort_for_benchmark;
