//! A model in the Warcraft III MDX format.
use crate::Encoder;
use crate::{Tag, Version};

use crate::Cursor;
use crate::{CollectionChunk, Error, ModelChunk, RawChunk, Record, VersionChunk};

/// The four bytes at the start of an MDX file.
pub const MAGIC: Tag = *b"MDLX";

/// The latest version number understood by this library.
/// This is also the default version number used when creating a new model
/// or when decoding a model that has no `VERS` chunk.
pub const LATEST_VERSION: Version = 1800;

/// An ordered MDX model. Unknown chunks remain available and writable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    /// The version number that is used if the model has no `VERS` chunk.
    default_version: Version,

    /// The ordered list of chunks in the model.
    chunks: Vec<ModelChunk>,
}

impl Model {
    pub(crate) fn collect_chunk_records<C: CollectionChunk + 'static>(
        &self,
        typed: for<'a> fn(&'a ModelChunk) -> Option<&'a C>,
    ) -> Result<Vec<C::Item>, Error>
    where
        C::Item: Clone,
    {
        self.collect_chunk_items(C::tag(), |chunk| {
            typed(chunk).map(|collection| collection.records())
        })
    }

    pub(crate) fn collect_chunk_items<T: Clone>(
        &self,
        tag: Tag,
        typed: impl Fn(&ModelChunk) -> Option<&[T]>,
    ) -> Result<Vec<T>, Error> {
        let mut result = Vec::new();
        for chunk in self.chunks.iter().filter(|chunk| chunk.tag() == tag) {
            if let ModelChunk::Malformed(malformed) = chunk {
                return Err(malformed.error.clone());
            }
            if let Some(items) = typed(chunk) {
                result.extend_from_slice(items);
            } else {
                unreachable!("tag matched a different decoded chunk type");
            }
        }
        Ok(result)
    }

    /// Creates a model with a `VERS` chunk for the given version.
    pub fn new(version: Version) -> Self {
        Self {
            default_version: version,
            chunks: vec![ModelChunk::Version(VersionChunk {
                version,
                extension: Vec::new(),
            })],
        }
    }

    /// Returns the first `VERS` value, if present.
    pub fn stored_version(&self) -> Option<u32> {
        self.chunks.iter().find_map(|chunk| match chunk {
            ModelChunk::Version(version) => Some(version.version),
            _ => None,
        })
    }

    /// Returns the model version, producing the value from the `VERS` chunk
    /// if there is such a chunk, or the default version otherwise.
    pub fn version(&self) -> Version {
        self.stored_version().unwrap_or(self.default_version)
    }

    /// Updates the first `VERS` value, or inserts a `VERS` chunk first.
    pub fn set_version(&mut self, version: Version) -> Result<(), Error> {
        match self.chunk_mut(*b"VERS") {
            Some(ModelChunk::Version(current)) => current.version = version,
            Some(_) => return Err(Error::InvalidVersionChunk),
            None => self.chunks.insert(
                0,
                ModelChunk::Version(VersionChunk {
                    version,
                    extension: Vec::new(),
                }),
            ),
        }
        Ok(())
    }

    /// Returns the ordered list of chunks.
    pub fn chunks(&self) -> &[ModelChunk] {
        &self.chunks
    }

    /// Returns a mutable ordered list of chunks.
    pub fn chunks_mut(&mut self) -> &mut Vec<ModelChunk> {
        &mut self.chunks
    }

    /// Finds the first chunk with the given tag.
    pub fn chunk(&self, tag: Tag) -> Option<&ModelChunk> {
        self.chunks.iter().find(|chunk| chunk.tag() == tag)
    }

    /// Finds the first mutable chunk with the given tag.
    pub fn chunk_mut(&mut self, tag: Tag) -> Option<&mut ModelChunk> {
        self.chunks.iter_mut().find(|chunk| chunk.tag() == tag)
    }

    /// Appends a chunk.
    pub fn push(&mut self, chunk: ModelChunk) {
        self.chunks.push(chunk);
    }

    pub(crate) fn replace_chunk(&mut self, chunk: ModelChunk) -> Result<(), Error> {
        let tag = chunk.tag();
        if let Some(index) = self
            .chunks()
            .iter()
            .position(|existing| existing.tag() == tag)
        {
            self.chunks_mut()[index] = chunk;
            let mut seen = false;
            self.chunks_mut().retain(|existing| {
                if existing.tag() != tag {
                    return true;
                }
                if seen {
                    false
                } else {
                    seen = true;
                    true
                }
            });
        } else {
            self.push(chunk);
        }
        Ok(())
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
}

impl Record for Model {
    fn decode_one(cursor: &mut Cursor<'_>, default_version: Version) -> Result<Self, Error> {
        let bytes = cursor.remaining();
        let value = {
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
                chunks.push(RawChunk::new(tag, bytes[start..end].to_vec()));
                offset = end;
            }
            if chunks
                .iter()
                .any(|chunk| chunk.tag == *b"VERS" && chunk.data.len() < 4)
            {
                return Err(Error::InvalidVersionChunk);
            }
            let version = chunks
                .iter()
                .find(|chunk| chunk.tag == *b"VERS")
                .map(|chunk| {
                    u32::from_le_bytes(chunk.data[..4].try_into().expect("four-byte version"))
                })
                .unwrap_or(default_version);
            Ok(Self {
                default_version,
                chunks: chunks
                    .into_iter()
                    .map(|chunk| ModelChunk::from_raw(chunk, version))
                    .collect(),
            })
        }?;
        cursor.read_exact(bytes.len())?;
        Ok(value)
    }

    fn encode_to(&self, output: &mut Encoder<'_>) -> Result<(), Error> {
        output.write_bytes(&MAGIC);
        for chunk in &self.chunks {
            chunk.encode_to(output)?;
        }
        Ok(())
    }
}
