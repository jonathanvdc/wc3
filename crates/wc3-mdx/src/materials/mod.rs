mod layer;
pub use layer::{Layer, LayerFresnel, LayerShadingFlags, LayerTextureSlot, LayerTrack};
mod material;
pub use material::{Material, MaterialLayout, MaterialRenderFlags};
mod texture;
pub use texture::{Texture, TextureFlags};

use crate::{EncodeError, Encoder, Tag};

fn write_count(bytes: &mut Encoder<'_>, count: usize, tag: Tag) -> Result<(), EncodeError> {
    let value =
        u32::try_from(count).map_err(|_| EncodeError::ChunkTooLarge { tag, size: count })?;
    bytes.write(&(value))?;
    Ok(())
}
