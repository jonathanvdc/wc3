//! A model in the Warcraft III MDX format.

use crate::{Chunk, Error, Record};

/// The four bytes at the start of an MDX file.
pub const MAGIC: [u8; 4] = *b"MDLX";

/// The latest version number understood by this library.
/// This is also the default version number used when creating a new model
/// or when decoding a model that has no `VERS` chunk.
pub const LATEST_VERSION: u32 = 1800;

/// An ordered MDX model. Unknown chunks remain available and writable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    /// The version number that is used if the model has no `VERS` chunk.
    default_version: u32,

    /// The ordered list of chunks in the model.
    chunks: Vec<Chunk>,
}

impl Model {
    /// Creates a model with a `VERS` chunk for the given version.
    pub fn new(version: u32) -> Self {
        Self {
            default_version: version,
            chunks: vec![Chunk::new(*b"VERS", version.to_le_bytes().to_vec())],
        }
    }

    /// Returns the first `VERS` value, if present.
    pub fn stored_version(&self) -> Option<u32> {
        self.chunk(*b"VERS")
            .and_then(|chunk| chunk.data.get(..4))
            .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("four-byte version")))
    }

    /// Returns the model version, producing the value from the `VERS` chunk
    /// if there is such a chunk, or the default version otherwise.
    pub fn version(&self) -> u32 {
        self.stored_version().unwrap_or(self.default_version)
    }

    /// Updates the first `VERS` value, or inserts a `VERS` chunk first.
    pub fn set_version(&mut self, version: u32) {
        if let Some(chunk) = self.chunk_mut(*b"VERS") {
            if chunk.data.len() < 4 {
                chunk.data.resize(4, 0);
            }
            chunk.data[..4].copy_from_slice(&version.to_le_bytes());
        } else {
            self.chunks
                .insert(0, Chunk::new(*b"VERS", version.to_le_bytes().to_vec()));
        }
    }

    /// Returns the ordered list of chunks.
    pub fn chunks(&self) -> &[Chunk] {
        &self.chunks
    }

    /// Returns a mutable ordered list of chunks.
    pub fn chunks_mut(&mut self) -> &mut Vec<Chunk> {
        &mut self.chunks
    }

    /// Finds the first chunk with the given tag.
    pub fn chunk(&self, tag: [u8; 4]) -> Option<&Chunk> {
        self.chunks.iter().find(|chunk| chunk.tag == tag)
    }

    /// Finds the first mutable chunk with the given tag.
    pub fn chunk_mut(&mut self, tag: [u8; 4]) -> Option<&mut Chunk> {
        self.chunks.iter_mut().find(|chunk| chunk.tag == tag)
    }

    /// Appends a chunk.
    pub fn push(&mut self, chunk: Chunk) {
        self.chunks.push(chunk);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_magic_and_lengths() {
        assert_eq!(Model::decode(b"wrong", 0), Err(Error::InvalidMagic));
        assert_eq!(
            Model::decode(b"MDLXVE", 0),
            Err(Error::TruncatedHeader { offset: 4 })
        );
        let mut bytes = b"MDLXTEST".to_vec();
        bytes.extend_from_slice(&5u32.to_le_bytes());
        bytes.push(1);
        assert_eq!(
            Model::decode(&bytes, 0),
            Err(Error::TruncatedChunk {
                tag: *b"TEST",
                offset: 4,
                size: 5,
            })
        );
    }

    #[test]
    fn edits_version_without_discarding_extra_bytes() {
        let mut model = Model::new(800);
        model
            .chunk_mut(*b"VERS")
            .unwrap()
            .data
            .extend_from_slice(&[9, 8]);
        model.set_version(1200);
        assert_eq!(model.version(), 1200);
        assert_eq!(&model.chunk(*b"VERS").unwrap().data[4..], &[9, 8]);
    }
}

impl Record for Model {
    fn decode(bytes: &[u8], default_version: u32) -> Result<Self, Error> {
        if !bytes.starts_with(&MAGIC) {
            return Err(Error::InvalidMagic);
        }
        let mut chunks = Vec::new();
        let mut offset = MAGIC.len();
        while offset < bytes.len() {
            if bytes.len() - offset < 8 {
                return Err(Error::TruncatedHeader { offset });
            }
            let tag = bytes[offset..offset + 4]
                .try_into()
                .expect("four-byte slice");
            let size = u32::from_le_bytes(
                bytes[offset + 4..offset + 8]
                    .try_into()
                    .expect("four-byte slice"),
            );
            let start = offset + 8;
            let end = start
                .checked_add(size as usize)
                .filter(|&end| end <= bytes.len())
                .ok_or(Error::TruncatedChunk { tag, offset, size })?;
            chunks.push(Chunk::new(tag, bytes[start..end].to_vec()));
            offset = end;
        }
        let model = Self {
            default_version,
            chunks,
        };
        if model
            .chunks
            .iter()
            .any(|chunk| chunk.tag == *b"VERS" && chunk.data.len() < 4)
        {
            return Err(Error::InvalidVersionChunk);
        }
        Ok(model)
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let capacity = self.chunks.iter().try_fold(4usize, |total, chunk| {
            if chunk.data.len() > u32::MAX as usize {
                return Err(Error::ChunkTooLarge {
                    tag: chunk.tag,
                    size: chunk.data.len(),
                });
            }
            total
                .checked_add(8)
                .and_then(|n| n.checked_add(chunk.data.len()))
                .ok_or(Error::ChunkTooLarge {
                    tag: chunk.tag,
                    size: chunk.data.len(),
                })
        })?;
        let mut output = Vec::with_capacity(capacity);
        output.extend_from_slice(&MAGIC);
        for chunk in &self.chunks {
            output.extend_from_slice(&chunk.tag);
            output.extend_from_slice(&(chunk.data.len() as u32).to_le_bytes());
            output.extend_from_slice(&chunk.data);
        }
        Ok(output)
    }
}
