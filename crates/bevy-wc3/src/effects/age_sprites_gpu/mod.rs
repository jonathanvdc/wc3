//! Per-view M2 sprite expansion from shared, exactly sampled live records.
mod records;
mod render;
pub(crate) use records::{
    AgeSpriteData, AgeSpriteRecord, AgeSpriteStyle, AgeSpriteUniform, SampleBuffers,
};
pub(crate) use render::AgeSpriteRenderPlugin;
