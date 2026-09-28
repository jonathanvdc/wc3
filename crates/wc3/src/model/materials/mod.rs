mod layer;
pub use layer::{
    Layer, LayerFilterMode, LayerFresnel, LayerShadingFlags, LayerTextureSlot, LayerTrack,
};
mod material;
pub use material::{Material, MaterialLayout, MaterialRenderFlags};
mod texture;
pub use texture::{Texture, TextureFlags};

use crate::model::{Encoder, Tag, WriteError};

fn write_count(bytes: &mut Encoder<'_>, count: usize, tag: Tag) -> Result<(), WriteError> {
    let value = u32::try_from(count).map_err(|_| WriteError::ChunkTooLarge { tag, size: count })?;
    bytes.write(&(value))?;
    Ok(())
}

mod shader;
pub use shader::ShaderType;
