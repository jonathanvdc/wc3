//! Fixed-width texture records in `TEXS` chunks.
use crate::EncodeError;
use crate::Encoder;
use crate::Tag;
use crate::ValueError;

use crate::{Cursor, TexturesChunk};
use crate::{Decodable, Encodable};
use std::borrow::Cow;

use crate::utils::field;
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
pub(crate) const SIZE: usize = 268;
const PATH_SIZE: usize = 256;

/// A texture reference with its original reserved bytes intact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Texture {
    replaceable_id: u32,
    path: [u8; PATH_SIZE],
    reserved: Tag,
    flags: u32,
}

impl Texture {
    /// Creates a texture with a path and no flags or replacement ID.
    pub fn new(path: &str) -> Result<Self, ValueError> {
        let mut texture = Self {
            replaceable_id: 0,
            path: [0; PATH_SIZE],
            reserved: [0; 4],
            flags: 0,
        };
        texture.set_path(path)?;
        Ok(texture)
    }

    /// Returns the original 268-byte record.
    pub fn as_bytes(&self) -> [u8; SIZE] {
        self.encode()
            .expect("fixed-size record")
            .try_into()
            .expect("fixed-size record")
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
        field::text(&self.path)
    }

    /// Sets the path, clearing the unused part of the fixed-width field.
    pub fn set_path(&mut self, path: &str) -> Result<(), ValueError> {
        field::set_text(&mut self.path, path)
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

impl Model {
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
        let size = cursor.remaining().len();
        if size < SIZE {
            return Err(DecodeError::MalformedChunk {
                tag: Self::TAG,
                size,
                expected: SIZE,
            });
        }
        let replaceable_id = cursor.read()?;
        let path = cursor
            .read_exact(PATH_SIZE)?
            .try_into()
            .expect("fixed-width path");
        let reserved = cursor.read_exact(4)?.try_into().expect("fixed-width field");
        let flags = cursor.read()?;
        Ok(Self {
            replaceable_id,
            path,
            reserved,
            flags,
        })
    }
}

impl Encodable for Texture {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        bytes.write(self.replaceable_id);
        bytes.write_bytes(&self.path);
        bytes.write_bytes(&self.reserved);
        bytes.write(self.flags);
        Ok(())
    }
}

impl Texture {
    /// The tag of the chunk containing this record.
    pub const TAG: Tag = *b"TEXS";
}
