//! Reforged face-animation references in `FAFX` chunks.
use crate::Encoder;
use crate::Tag;

use crate::Record;
use crate::{Cursor, FaceFxChunk, ModelChunk};
use std::borrow::Cow;

use crate::utils::field;
use crate::{Error, Model};

pub(crate) const SIZE: usize = 340;
const NAME_SIZE: usize = 80;
const PATH_SIZE: usize = 260;

/// One fixed-size face-animation name and path pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FaceFx {
    name: [u8; NAME_SIZE],
    path: [u8; PATH_SIZE],
}

impl FaceFx {
    /// Creates a face-animation reference.
    pub fn new(name: &str, path: &str) -> Result<Self, Error> {
        let mut entry = Self {
            name: [0; NAME_SIZE],
            path: [0; PATH_SIZE],
        };
        entry.set_name(name)?;
        entry.set_path(path)?;
        Ok(entry)
    }

    /// Returns the original record bytes.
    pub fn as_bytes(&self) -> [u8; SIZE] {
        self.encode()
            .expect("fixed-size record")
            .try_into()
            .expect("fixed-size record")
    }

    /// Returns the name up to the first NUL.
    pub fn name(&self) -> Cow<'_, str> {
        field::text(&self.name)
    }

    /// Replaces the name.
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        field::set_text(&mut self.name, name)
    }

    /// Returns the animation resource path up to the first NUL.
    pub fn path(&self) -> Cow<'_, str> {
        field::text(&self.path)
    }

    /// Replaces the animation resource path.
    pub fn set_path(&mut self, path: &str) -> Result<(), Error> {
        field::set_text(&mut self.path, path)
    }
}

impl Model {
    /// Decodes every `FAFX` record in file order.
    pub fn face_fx(&self) -> Result<Vec<FaceFx>, Error> {
        self.collect_chunk_records::<FaceFxChunk>(|chunk| match chunk {
            ModelChunk::FaceFx(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces face-animation records in the first `FAFX` chunk.
    pub fn set_face_fx(&mut self, entries: &[FaceFx]) -> Result<(), Error> {
        self.replace_chunk(ModelChunk::FaceFx(FaceFxChunk::new(entries.to_vec())))
    }
}

impl Record for FaceFx {
    fn decode_one(cursor: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let size = cursor.remaining().len();
        if size < SIZE {
            return Err(Error::MalformedChunk {
                tag: Self::TAG,
                size,
                expected: SIZE,
            });
        }
        let name = cursor
            .read_exact(NAME_SIZE)?
            .try_into()
            .expect("fixed-width name");
        let path = cursor
            .read_exact(PATH_SIZE)?
            .try_into()
            .expect("fixed-width path");
        Ok(Self { name, path })
    }

    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), Error> {
        bytes.write_bytes(&self.name);
        bytes.write_bytes(&self.path);
        Ok(())
    }
}

impl FaceFx {
    /// The tag of the chunk containing this record.
    pub const TAG: Tag = *b"FAFX";
}
