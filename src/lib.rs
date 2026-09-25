//! Lossless Warcraft III MDX container parsing and writing.
//!
//! MDX files start with `MDLX`, followed by tagged chunks. Each chunk has a
//! four-byte identifier, a little-endian payload length, and its payload.
//! Keeping payloads as bytes preserves chunks whose internal layout changes
//! between Classic and Reforged versions. Use [`Model::version`] and
//! [`Model::set_version`] for the `VERS` chunk, and [`Model::chunks`] or
//! [`Model::chunk_mut`] for other chunks.
//!
//! ```
//! use wc3_mdx::{Chunk, Model};
//! let mut model = Model::new(800);
//! model.push(Chunk::new(*b"TEST", vec![1, 2, 3]));
//! let bytes = model.to_bytes().unwrap();
//! assert_eq!(Model::from_bytes(&bytes).unwrap(), model);
//! ```

use std::fmt;

mod animation;
pub use animation::{AnimationTrack, Keyframe};
mod attachment;
pub use attachment::Attachment;
mod bind_pose;
pub use bind_pose::BindPose;
mod collision;
pub use collision::{CollisionKind, CollisionShape};
mod camera;
pub use camera::Camera;
mod event;
pub use event::EventObject;
mod face_fx;
pub use face_fx::FaceFx;
mod geoset;
pub use geoset::{Geoset, GeosetExtent};
mod geoset_animation;
pub use geoset_animation::{GeosetAnimation, GeosetAnimationFlags};
mod light;
pub use light::Light;
mod material;
pub use material::{Layer, LayerShadingFlags, LayerTextureSlot, Material, MaterialRenderFlags};
mod model_info;
pub use model_info::ModelInfo;
mod node;
pub use node::{Bone, Node, NodeFlags};
mod popcorn;
pub use popcorn::PopcornEmitter;
mod particle;
pub use particle::ParticleEmitter;
mod particle2;
pub use particle2::{Particle2Fields, ParticleEmitter2};
mod ribbon;
pub use ribbon::{RibbonEmitter, RibbonFields};
mod sequence;
pub use sequence::{Sequence, SequenceFlags};
mod simple_chunks;
mod sized_node;
mod texture;
pub use texture::{Texture, TextureFlags};
mod texture_animation;
pub use texture_animation::TextureAnimation;

/// The four bytes at the start of an MDX file.
pub const MAGIC: [u8; 4] = *b"MDLX";

/// A tagged top-level MDX chunk. The payload is stored without interpretation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chunk {
    /// Four-byte binary identifier, such as `VERS`, `MODL`, or `GEOS`.
    pub tag: [u8; 4],
    /// Original payload bytes, preserved on round-trip.
    pub data: Vec<u8>,
}

impl Chunk {
    /// Creates a chunk from its tag and payload.
    pub fn new(tag: [u8; 4], data: Vec<u8>) -> Self {
        Self { tag, data }
    }
}

/// An ordered MDX model. Unknown chunks remain available and writable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    chunks: Vec<Chunk>,
}

/// Errors caused by malformed input or a payload too large for the MDX format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// The input does not begin with `MDLX`.
    InvalidMagic,
    /// The chunk header is incomplete.
    TruncatedHeader { offset: usize },
    /// The declared chunk payload exceeds the input.
    TruncatedChunk {
        tag: [u8; 4],
        offset: usize,
        size: u32,
    },
    /// `VERS` has no four-byte version number.
    InvalidVersionChunk,
    /// The payload cannot be represented by a 32-bit MDX chunk size.
    ChunkTooLarge { tag: [u8; 4], size: usize },
    /// A known chunk is too short for its fixed layout.
    MalformedChunk {
        tag: [u8; 4],
        size: usize,
        expected: usize,
    },
    /// A fixed-width string is too long or contains a NUL.
    InvalidString { max_bytes: usize },
    /// A size-bounded record or section is malformed at the given offset.
    MalformedRecord { tag: [u8; 4], offset: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(f, "expected MDLX magic"),
            Self::TruncatedHeader { offset } => {
                write!(f, "truncated chunk header at byte {offset}")
            }
            Self::TruncatedChunk { tag, offset, size } => write!(
                f,
                "truncated {:?} chunk at byte {offset} (declared size {size})",
                String::from_utf8_lossy(tag)
            ),
            Self::InvalidVersionChunk => write!(f, "VERS chunk has fewer than four bytes"),
            Self::ChunkTooLarge { tag, size } => write!(
                f,
                "{:?} chunk size {size} exceeds u32",
                String::from_utf8_lossy(tag)
            ),
            Self::MalformedChunk {
                tag,
                size,
                expected,
            } => write!(
                f,
                "{:?} chunk has {size} bytes; expected at least {expected}",
                String::from_utf8_lossy(tag)
            ),
            Self::InvalidString { max_bytes } => {
                write!(f, "string must fit in {max_bytes} bytes and contain no NUL")
            }
            Self::MalformedRecord { tag, offset } => write!(
                f,
                "malformed {:?} record at byte {offset}",
                String::from_utf8_lossy(tag)
            ),
        }
    }
}

impl std::error::Error for Error {}

impl Model {
    /// Creates a model with a `VERS` chunk for the given version.
    pub fn new(version: u32) -> Self {
        Self {
            chunks: vec![Chunk::new(*b"VERS", version.to_le_bytes().to_vec())],
        }
    }

    /// Parses an MDX file and keeps every chunk in its original order.
    pub fn from_bytes(input: &[u8]) -> Result<Self, Error> {
        if !input.starts_with(&MAGIC) {
            return Err(Error::InvalidMagic);
        }
        let mut chunks = Vec::new();
        let mut offset = MAGIC.len();
        while offset < input.len() {
            if input.len() - offset < 8 {
                return Err(Error::TruncatedHeader { offset });
            }
            let tag = input[offset..offset + 4]
                .try_into()
                .expect("four-byte slice");
            let size = u32::from_le_bytes(
                input[offset + 4..offset + 8]
                    .try_into()
                    .expect("four-byte slice"),
            );
            let start = offset + 8;
            let end = start
                .checked_add(size as usize)
                .filter(|&end| end <= input.len())
                .ok_or(Error::TruncatedChunk { tag, offset, size })?;
            chunks.push(Chunk::new(tag, input[start..end].to_vec()));
            offset = end;
        }
        let model = Self { chunks };
        if model
            .chunk(*b"VERS")
            .is_some_and(|chunk| chunk.data.len() < 4)
        {
            return Err(Error::InvalidVersionChunk);
        }
        Ok(model)
    }

    /// Serializes all chunks in their current order.
    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
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

    /// Returns the first `VERS` value, if present.
    pub fn version(&self) -> Option<u32> {
        self.chunk(*b"VERS").map(|chunk| {
            u32::from_le_bytes(chunk.data[..4].try_into().expect("validated VERS chunk"))
        })
    }

    /// Updates the first `VERS` value, or inserts a `VERS` chunk first.
    pub fn set_version(&mut self, version: u32) {
        if let Some(chunk) = self.chunk_mut(*b"VERS") {
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
        assert_eq!(Model::from_bytes(b"wrong"), Err(Error::InvalidMagic));
        assert_eq!(
            Model::from_bytes(b"MDLXVE"),
            Err(Error::TruncatedHeader { offset: 4 })
        );
        let mut bytes = b"MDLXTEST".to_vec();
        bytes.extend_from_slice(&5u32.to_le_bytes());
        bytes.push(1);
        assert_eq!(
            Model::from_bytes(&bytes),
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
        assert_eq!(model.version(), Some(1200));
        assert_eq!(&model.chunk(*b"VERS").unwrap().data[4..], &[9, 8]);
    }
}
