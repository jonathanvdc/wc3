//! Fixed-width texture records in `TEXS` chunks.

use std::borrow::Cow;

use crate::utils::field;
use crate::{Error, Model};

const TAG: [u8; 4] = *b"TEXS";

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
const SIZE: usize = 268;
const PATH_START: usize = 4;
const PATH_SIZE: usize = 256;

/// A texture reference with its original reserved bytes intact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Texture {
    bytes: [u8; SIZE],
}

impl Texture {
    /// Creates a texture with a path and no flags or replacement ID.
    pub fn new(path: &str) -> Result<Self, Error> {
        let mut texture = Self { bytes: [0; SIZE] };
        texture.set_path(path)?;
        Ok(texture)
    }

    fn parse(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.try_into().expect("fixed-size record"),
        }
    }

    /// Returns the original 268-byte record.
    pub fn as_bytes(&self) -> &[u8; SIZE] {
        &self.bytes
    }

    /// Returns the replaceable texture ID.
    pub fn replaceable_id(&self) -> u32 {
        u32::from_le_bytes(self.bytes[..4].try_into().expect("fixed-size field"))
    }

    /// Sets the replaceable texture ID.
    pub fn set_replaceable_id(&mut self, id: u32) {
        self.bytes[..4].copy_from_slice(&id.to_le_bytes());
    }

    /// Returns the path up to the first NUL, replacing invalid UTF-8.
    pub fn path(&self) -> Cow<'_, str> {
        field::text(&self.bytes[PATH_START..PATH_START + PATH_SIZE])
    }

    /// Sets the path, clearing the unused part of the fixed-width field.
    pub fn set_path(&mut self, path: &str) -> Result<(), Error> {
        field::set_text(&mut self.bytes[PATH_START..PATH_START + PATH_SIZE], path)
    }

    /// Returns decoded texture wrapping flags.
    pub fn flags(&self) -> TextureFlags {
        TextureFlags::from_bits(self.raw_flags())
    }

    /// Returns exact raw texture flag bits.
    pub fn raw_flags(&self) -> u32 {
        u32::from_le_bytes(self.bytes[264..268].try_into().expect("fixed-size field"))
    }

    /// Sets decoded texture wrapping flags.
    pub fn set_flags(&mut self, flags: TextureFlags) {
        self.set_raw_flags(flags.bits());
    }

    /// Sets exact raw texture flag bits.
    pub fn set_raw_flags(&mut self, flags: u32) {
        self.bytes[264..268].copy_from_slice(&flags.to_le_bytes());
    }
}

impl Model {
    /// Decodes all `TEXS` chunks in file order.
    pub fn textures(&self) -> Result<Vec<Texture>, Error> {
        let mut textures = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            if chunk.data.len() % SIZE != 0 {
                return Err(Error::MalformedChunk {
                    tag: TAG,
                    size: chunk.data.len(),
                    expected: SIZE,
                });
            }
            textures.extend(chunk.data.chunks_exact(SIZE).map(Texture::parse));
        }
        Ok(textures)
    }

    /// Writes the texture list to the first `TEXS` chunk. Additional `TEXS`
    /// chunks are removed after their records are replaced.
    pub fn set_textures(&mut self, textures: &[Texture]) -> Result<(), Error> {
        let size = textures
            .len()
            .checked_mul(SIZE)
            .ok_or(Error::ChunkTooLarge {
                tag: TAG,
                size: usize::MAX,
            })?;
        if size > u32::MAX as usize {
            return Err(Error::ChunkTooLarge { tag: TAG, size });
        }
        let mut data = Vec::with_capacity(size);
        for texture in textures {
            data.extend_from_slice(texture.as_bytes());
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
