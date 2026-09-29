//! Surface appearance, texture references, and animated material layers.
//!
//! A [`Material`] contains ordered [`Layer`] values. Layers choose blend modes,
//! texture bindings, and shading flags. Texture IDs refer to entries in the model
//! texture collection; coordinate IDs select a geoset UV set. Reforged versions
//! add shader choices and HD texture slots.
use crate::model::mdx;
mod layer;
pub use layer::{Layer, LayerFilterMode, LayerFresnel, LayerShadingFlags, LayerTextureSlot};
mod material;
pub use material::{Material, MaterialLayout, MaterialRenderFlags};
mod texture;
use crate::model::{Encoder, Tag};
pub use texture::{Texture, TextureFlags};

fn write_count(bytes: &mut Encoder<'_>, count: usize, tag: Tag) -> Result<(), mdx::WriteError> {
    let value = u32::try_from(count).map_err(|_| mdx::WriteError::SizeOverflow {
        field: "record count",
        tag,
        size: count,
    })?;
    bytes.write(&(value))?;
    Ok(())
}

mod shader;
pub use shader::ShaderType;
