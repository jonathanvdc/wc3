//! Fixed-width texture records in `TEXS` chunks.
use crate::model::mdl;
use crate::model::mdl::is_zero;
use crate::model::mdx;
use crate::model::ModelVersion;
use crate::model::ValueError;
use bitfield::bitfield;

use crate::model::TexturesChunk;

use std::borrow::Cow;

use crate::model::FixedText;
use crate::model::Model;

bitfield! {
    /// Texture wrapping flags; unknown bits remain available through `bits`.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, mdx::Read, mdx::Write)]
    pub struct TextureFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the `WRAP_WIDTH` bit.
    pub wrap_width, set_wrap_width: 0;
    /// Returns or changes the `WRAP_HEIGHT` bit.
    pub wrap_height, set_wrap_height: 1;
}

const PATH_SIZE: usize = 260;

/// A texture reference with a 260-byte path and lossless wrapping flags.
#[derive(Clone, Debug, Eq, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(block = "Bitmap", write_order(path, replaceable_id, flags))]
pub struct Texture {
    #[mdl(property = "ReplaceableId", default, skip_if = "is_zero")]
    replaceable_id: u32,
    #[mdl(property = "Image", default)]
    path: FixedText<PATH_SIZE>,
    #[mdl(flags(WrapWidth = 1, WrapHeight = 2))]
    flags: TextureFlags,
}

impl Texture {
    /// Creates a texture with a path and no flags or replacement ID.
    pub fn new(path: &str) -> Result<Self, ValueError> {
        let mut texture = Self {
            replaceable_id: 0,
            path: FixedText::default(),
            flags: TextureFlags::default(),
        };
        texture.set_path(path)?;
        Ok(texture)
    }

    /// Returns the replaceable texture ID.
    pub fn replaceable_id(&self) -> u32 {
        self.replaceable_id
    }

    /// Sets the replaceable texture ID.
    pub fn set_replaceable_id(&mut self, id: u32) {
        self.replaceable_id = id;
    }

    /// Returns the path up to the first NUL, replacing invalid UTF-8.
    pub fn path(&self) -> Cow<'_, str> {
        self.path.text()
    }

    /// Sets the path, clearing the unused part of the fixed-width field.
    pub fn set_path(&mut self, path: &str) -> Result<(), ValueError> {
        self.path.set_text(path)
    }

    /// Returns decoded texture wrapping flags.
    pub fn flags(&self) -> TextureFlags {
        self.flags
    }

    /// Sets decoded texture wrapping flags.
    pub fn set_flags(&mut self, flags: TextureFlags) {
        self.flags = flags;
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes all `TEXS` chunks in file order.
    pub fn textures(&self) -> Vec<Texture> {
        self.collect_chunk_records::<TexturesChunk>()
    }

    /// Writes the texture list to the first `TEXS` chunk. Additional `TEXS`
    /// chunks are removed after their records are replaced.
    pub fn set_textures(&mut self, textures: &[Texture]) {
        self.replace_chunk(TexturesChunk::new(textures.to_vec()));
    }
}
