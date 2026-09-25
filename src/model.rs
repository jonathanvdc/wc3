//! A model in the Warcraft III MDX format.
use crate::Encoder;
use crate::{Tag, Version};

use crate::Cursor;
use crate::{CollectionChunk, Error, ModelChunk, Record, VersionChunk};

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
    ) -> Vec<C::Item>
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
    ) -> Vec<T> {
        let mut result = Vec::new();
        for chunk in self.chunks.iter().filter(|chunk| chunk.tag() == tag) {
            if let Some(items) = typed(chunk) {
                result.extend_from_slice(items);
            } else {
                unreachable!("tag matched a different decoded chunk type");
            }
        }
        result
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

    /// Replaces `VERS` chunks with one version chunk, preserving the first
    /// decoded chunk's extension bytes.
    pub fn set_version(&mut self, version: Version) {
        let extension = self.chunks.iter().find_map(|chunk| match chunk {
            ModelChunk::Version(current) => Some(current.extension.clone()),
            _ => None,
        });
        self.replace_chunk(ModelChunk::Version(VersionChunk {
            version,
            extension: extension.unwrap_or_default(),
        }));
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

    pub(crate) fn replace_chunk(&mut self, chunk: ModelChunk) {
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
    }
}

impl Record for Model {
    fn decode_one(cursor: &mut Cursor<'_>, default_version: Version) -> Result<Self, Error> {
        let version = scan_version(*cursor)?.unwrap_or(default_version);

        let mut parse = *cursor;
        parse.read_exact(4)?;
        let mut chunks = Vec::new();
        while !parse.remaining().is_empty() {
            let (tag, _, mut payload) = read_chunk(&mut parse)?;
            chunks.push(ModelChunk::decode_from(tag, &mut payload, version));
        }
        *cursor = parse;
        Ok(Self {
            default_version,
            chunks,
        })
    }

    fn encode_to(&self, output: &mut Encoder<'_>) -> Result<(), Error> {
        output.write_bytes(&MAGIC);
        for chunk in &self.chunks {
            chunk.encode_to(output)?;
        }
        Ok(())
    }
}

fn scan_version(mut cursor: Cursor<'_>) -> Result<Option<Version>, Error> {
    if cursor.read_exact(4).ok() != Some(MAGIC.as_slice()) {
        return Err(Error::InvalidMagic);
    }
    let mut version = None;
    while !cursor.remaining().is_empty() {
        let (tag, _, mut payload) = read_chunk(&mut cursor)?;
        if tag == *b"VERS" {
            let found = payload.read_u32().map_err(|_| Error::InvalidVersionChunk)?;
            version.get_or_insert(found);
        }
    }
    Ok(version)
}

fn read_chunk<'a>(cursor: &mut Cursor<'a>) -> Result<(Tag, u32, Cursor<'a>), Error> {
    let offset = cursor.absolute_position();
    if cursor.remaining().len() < 8 {
        return Err(Error::TruncatedHeader { offset });
    }
    let tag = cursor.read_exact(4)?.try_into().expect("four-byte tag");
    let size = cursor.read_u32()?;
    let payload = cursor
        .slice(size as usize)
        .map_err(|_| Error::TruncatedChunk { tag, offset, size })?;
    Ok((tag, size, payload))
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
    fn late_version_and_malformed_chunks_round_trip() {
        let mut bytes = MAGIC.to_vec();
        for (tag, payload) in [
            (*b"FUTR", vec![1, 2]),
            (*b"TEXS", vec![0; 267]),
            (*b"VERS", 800u32.to_le_bytes().to_vec()),
            (*b"VERS", 1800u32.to_le_bytes().to_vec()),
        ] {
            bytes.extend_from_slice(&tag);
            bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&payload);
        }
        let model = Model::decode(&bytes, 1800).unwrap();
        assert_eq!(model.version(), 800);
        assert!(matches!(model.chunks()[0], ModelChunk::Unknown(_)));
        assert!(matches!(model.chunks()[1], ModelChunk::Malformed(_)));
        assert_eq!(model.encode().unwrap(), bytes);
    }

    #[test]
    fn invalid_later_version_is_rejected() {
        let mut bytes = Model::new(800).encode().unwrap();
        bytes.extend_from_slice(b"VERS");
        bytes.extend_from_slice(&3u32.to_le_bytes());
        bytes.extend_from_slice(&[1, 2, 3]);
        assert_eq!(Model::decode(&bytes, 1800), Err(Error::InvalidVersionChunk));
    }
}
