//! Texture resource paths, replacement IDs, and wrapping.
use crate::model::mdl;
use crate::model::mdl::is_zero;
use crate::model::mdx;
use crate::model::ModelVersion;
use crate::model::ValueError;
use bitfield::bitfield;

use crate::model::TexturesChunk;

use crate::model::FixedText;
use crate::model::Model;

bitfield! {
    /// Texture wrapping flags; unknown bits remain available through `bits`.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, mdx::Read, mdx::Write)]
    pub struct TextureFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Repeats the texture along its width.
    pub wrap_width, set_wrap_width: 0;
    /// Repeats the texture along its height.
    pub wrap_height, set_wrap_height: 1;
}

const PATH_SIZE: usize = 260;

/// An image resource or game-provided replaceable texture.
#[derive(Clone, Debug, Eq, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(block = "Bitmap", write_order(path, replaceable_id, flags))]
pub struct Texture {
    #[mdl(property = "ReplaceableId", default, skip_if = "is_zero")]
    /// Game-provided replacement ID; zero uses the resource path.
    pub replaceable_id: u32,
    #[mdl(property = "Image", default)]
    /// Image resource path, such as `Textures\\Armor.blp`.
    pub path: FixedText<PATH_SIZE>,
    #[mdl(flags(WrapWidth = 1, WrapHeight = 2))]
    /// Texture wrapping flags.
    pub flags: TextureFlags,
}

impl Texture {
    /// Creates a texture with a path and no flags or replacement ID.
    pub fn new(path: &str) -> Result<Self, ValueError> {
        let mut texture = Self {
            replaceable_id: 0,
            path: FixedText::default(),
            flags: TextureFlags::default(),
        };
        texture.path.set_text(path)?;
        Ok(texture)
    }
}

impl<V: ModelVersion> Model<V> {
    /// Returns owned copies of `TEXS` chunks in file order.
    pub fn textures(&self) -> Vec<Texture> {
        self.collect_chunk_records::<TexturesChunk>()
    }

    /// Writes the texture list to the first `TEXS` chunk. Additional `TEXS`
    /// chunks are removed after their records are replaced.
    pub fn set_textures(&mut self, textures: &[Texture]) {
        self.replace_chunk(TexturesChunk::new(textures.to_vec()));
    }
}
