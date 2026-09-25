//! Reforged face-animation references in `FAFX` chunks.

use crate::Record;
use std::borrow::Cow;

use crate::utils::field;
use crate::{Error, Model};

pub(crate) const SIZE: usize = 340;
const NAME_SIZE: usize = 80;
const PATH_SIZE: usize = 260;

/// One fixed-size face-animation name and path pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FaceFx {
    bytes: [u8; SIZE],
}

impl FaceFx {
    /// Creates a face-animation reference.
    pub fn new(name: &str, path: &str) -> Result<Self, Error> {
        let mut entry = Self { bytes: [0; SIZE] };
        entry.set_name(name)?;
        entry.set_path(path)?;
        Ok(entry)
    }

    /// Returns the original record bytes.
    pub fn as_bytes(&self) -> &[u8; SIZE] {
        &self.bytes
    }

    /// Returns the name up to the first NUL.
    pub fn name(&self) -> Cow<'_, str> {
        field::text(&self.bytes[..NAME_SIZE])
    }

    /// Replaces the name.
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        field::set_text(&mut self.bytes[..NAME_SIZE], name)
    }

    /// Returns the animation resource path up to the first NUL.
    pub fn path(&self) -> Cow<'_, str> {
        field::text(&self.bytes[NAME_SIZE..])
    }

    /// Replaces the animation resource path.
    pub fn set_path(&mut self, path: &str) -> Result<(), Error> {
        field::set_text(&mut self.bytes[NAME_SIZE..NAME_SIZE + PATH_SIZE], path)
    }
}

impl Model {
    /// Decodes every `FAFX` record in file order.
    pub fn face_fx(&self) -> Result<Vec<FaceFx>, Error> {
        self.collect_chunk_records::<crate::FaceFxChunk>(|chunk| match chunk {
            crate::ModelChunk::FaceFx(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces face-animation records in the first `FAFX` chunk.
    pub fn set_face_fx(&mut self, entries: &[FaceFx]) -> Result<(), Error> {
        let size = entries
            .len()
            .checked_mul(SIZE)
            .filter(|&size| size <= u32::MAX as usize)
            .ok_or(Error::ChunkTooLarge {
                tag: FaceFx::TAG,
                size: usize::MAX,
            })?;
        let mut data = Vec::with_capacity(size);
        for entry in entries {
            data.extend_from_slice(entry.as_bytes());
        }
        self.replace_chunks(FaceFx::TAG, data)?;
        Ok(())
    }
}

impl Record for FaceFx {
    fn decode(bytes: &[u8], _version: u32) -> Result<Self, Error> {
        let bytes: [u8; SIZE] = bytes.try_into().map_err(|_| Error::MalformedChunk {
            tag: FaceFx::TAG,
            size: bytes.len(),
            expected: SIZE,
        })?;
        Ok(Self { bytes })
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        Ok(self.bytes.to_vec())
    }
}

impl FaceFx {
    /// The tag of the chunk containing this record.
    pub const TAG: [u8; 4] = *b"FAFX";
}
