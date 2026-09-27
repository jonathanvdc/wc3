mod material;
pub use material::{
    Layer, LayerShadingFlags, LayerTextureSlot, Material, MaterialLayout, MaterialRenderFlags,
};
mod texture;
pub use texture::{Texture, TextureFlags};

pub use material::LayerTrack;
