//! Fixed-width texture records in `TEXS` chunks.
use crate::EncodeError;
use crate::Encoder;
use crate::ModelVersion;
use crate::Tag;
use crate::ValueError;

use crate::{Cursor, TexturesChunk};
use crate::{Decodable, Encodable, Readable, Writable};
use std::borrow::Cow;

use crate::FixedText;
use crate::{DecodeError, Model};

/// Texture wrapping flags; unknown bits remain available through `bits`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextureFlags(u32);

impl TextureFlags {
    pub const WRAP_WIDTH: Self = Self(1);
    pub const WRAP_HEIGHT: Self = Self(2);
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
    pub const fn bits(self) -> u32 {
        self.0
    }
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    pub fn set(&mut self, other: Self, enabled: bool) {
        if enabled {
            self.0 |= other.0;
        } else {
            self.0 &= !other.0;
        }
    }
}
const PATH_SIZE: usize = 256;

/// A texture reference with its original reserved bytes intact.
#[derive(Clone, Debug, Eq, PartialEq, Readable, Writable)]
pub struct Texture {
    replaceable_id: u32,
    path: FixedText<PATH_SIZE>,
    reserved: Tag,
    flags: u32,
}

impl Texture {
    /// Creates a texture with a path and no flags or replacement ID.
    pub fn new(path: &str) -> Result<Self, ValueError> {
        let mut texture = Self {
            replaceable_id: 0,
            path: FixedText::default(),
            reserved: [0; 4],
            flags: 0,
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
        TextureFlags::from_bits(self.raw_flags())
    }

    /// Returns exact raw texture flag bits.
    pub fn raw_flags(&self) -> u32 {
        self.flags
    }

    /// Sets decoded texture wrapping flags.
    pub fn set_flags(&mut self, flags: TextureFlags) {
        self.set_raw_flags(flags.bits());
    }

    /// Sets exact raw texture flag bits.
    pub fn set_raw_flags(&mut self, flags: u32) {
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

impl Decodable for Texture {
    fn decode_one(cursor: &mut Cursor<'_>, _version: u32) -> Result<Self, DecodeError> {
        cursor.read()
    }
}

impl Encodable for Texture {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        bytes.write(self);
        Ok(())
    }
}
