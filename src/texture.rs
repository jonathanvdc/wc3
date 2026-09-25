//! Fixed-width texture records in `TEXS` chunks.

use std::borrow::Cow;

use crate::{Error, Model};

const TAG: [u8; 4] = *b"TEXS";
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
        let field = &self.bytes[PATH_START..PATH_START + PATH_SIZE];
        let end = field
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(PATH_SIZE);
        String::from_utf8_lossy(&field[..end])
    }

    /// Sets the path, clearing the unused part of the fixed-width field.
    pub fn set_path(&mut self, path: &str) -> Result<(), Error> {
        if path.len() >= PATH_SIZE || path.as_bytes().contains(&0) {
            return Err(Error::InvalidString {
                max_bytes: PATH_SIZE - 1,
            });
        }
        self.bytes[PATH_START..PATH_START + PATH_SIZE].fill(0);
        self.bytes[PATH_START..PATH_START + path.len()].copy_from_slice(path.as_bytes());
        Ok(())
    }

    /// Returns raw texture flags.
    pub fn flags(&self) -> u32 {
        u32::from_le_bytes(self.bytes[264..268].try_into().expect("fixed-size field"))
    }

    /// Sets raw texture flags.
    pub fn set_flags(&mut self, flags: u32) {
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
