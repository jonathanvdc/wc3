//! Reforged face-animation references in `FAFX` chunks.

use std::borrow::Cow;

use crate::{Error, Model};

const TAG: [u8; 4] = *b"FAFX";
const SIZE: usize = 340;
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

    /// Wraps one fixed-size record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let bytes: [u8; SIZE] = bytes.try_into().map_err(|_| Error::MalformedChunk {
            tag: TAG,
            size: bytes.len(),
            expected: SIZE,
        })?;
        Ok(Self { bytes })
    }

    /// Returns the original record bytes.
    pub fn as_bytes(&self) -> &[u8; SIZE] {
        &self.bytes
    }

    /// Returns the name up to the first NUL.
    pub fn name(&self) -> Cow<'_, str> {
        text(&self.bytes[..NAME_SIZE])
    }

    /// Replaces the name.
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        set_text(&mut self.bytes[..NAME_SIZE], name)
    }

    /// Returns the animation resource path up to the first NUL.
    pub fn path(&self) -> Cow<'_, str> {
        text(&self.bytes[NAME_SIZE..])
    }

    /// Replaces the animation resource path.
    pub fn set_path(&mut self, path: &str) -> Result<(), Error> {
        set_text(&mut self.bytes[NAME_SIZE..NAME_SIZE + PATH_SIZE], path)
    }
}

fn text(bytes: &[u8]) -> Cow<'_, str> {
    let end = bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end])
}

fn set_text(field: &mut [u8], value: &str) -> Result<(), Error> {
    if value.len() >= field.len() || value.as_bytes().contains(&0) {
        return Err(Error::InvalidString {
            max_bytes: field.len() - 1,
        });
    }
    field.fill(0);
    field[..value.len()].copy_from_slice(value.as_bytes());
    Ok(())
}

impl Model {
    /// Decodes every `FAFX` record in file order.
    pub fn face_fx(&self) -> Result<Vec<FaceFx>, Error> {
        let mut entries = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            if chunk.data.len() % SIZE != 0 {
                return Err(Error::MalformedChunk {
                    tag: TAG,
                    size: chunk.data.len(),
                    expected: SIZE,
                });
            }
            entries.extend(
                chunk
                    .data
                    .chunks_exact(SIZE)
                    .map(FaceFx::from_bytes)
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        Ok(entries)
    }

    /// Replaces face-animation records in the first `FAFX` chunk.
    pub fn set_face_fx(&mut self, entries: &[FaceFx]) -> Result<(), Error> {
        let size = entries
            .len()
            .checked_mul(SIZE)
            .filter(|&size| size <= u32::MAX as usize)
            .ok_or(Error::ChunkTooLarge {
                tag: TAG,
                size: usize::MAX,
            })?;
        let mut data = Vec::with_capacity(size);
        for entry in entries {
            data.extend_from_slice(entry.as_bytes());
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
