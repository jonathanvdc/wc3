//! Reforged face-animation references in `FAFX` chunks.
use crate::EncodeError;
use crate::Encoder;
use crate::ModelVersion;
use crate::ValueError;

use crate::{Cursor, FaceFxChunk};
use crate::{Decodable, Encodable, Readable, Writable};
use std::borrow::Cow;

use crate::FixedText;
use crate::{DecodeError, Model};

const NAME_SIZE: usize = 80;
const PATH_SIZE: usize = 260;

/// One fixed-size face-animation name and path pair.
#[derive(Clone, Debug, Eq, PartialEq, Readable, Writable)]
pub struct FaceFx {
    name: FixedText<NAME_SIZE>,
    path: FixedText<PATH_SIZE>,
}

impl FaceFx {
    /// Creates a face-animation reference.
    pub fn new(name: &str, path: &str) -> Result<Self, ValueError> {
        let mut entry = Self {
            name: FixedText::default(),
            path: FixedText::default(),
        };
        entry.set_name(name)?;
        entry.set_path(path)?;
        Ok(entry)
    }

    /// Returns the name up to the first NUL.
    pub fn name(&self) -> Cow<'_, str> {
        self.name.text()
    }

    /// Replaces the name.
    pub fn set_name(&mut self, name: &str) -> Result<(), ValueError> {
        self.name.set_text(name)
    }

    /// Returns the animation resource path up to the first NUL.
    pub fn path(&self) -> Cow<'_, str> {
        self.path.text()
    }

    /// Replaces the animation resource path.
    pub fn set_path(&mut self, path: &str) -> Result<(), ValueError> {
        self.path.set_text(path)
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes every `FAFX` record in file order.
    pub fn face_fx(&self) -> Vec<FaceFx> {
        self.collect_chunk_records::<FaceFxChunk>()
    }

    /// Replaces face-animation records in the first `FAFX` chunk.
    pub fn set_face_fx(&mut self, entries: &[FaceFx]) {
        self.replace_chunk(FaceFxChunk::new(entries.to_vec()));
    }
}

impl Decodable for FaceFx {
    fn decode_one(cursor: &mut Cursor<'_>, _version: u32) -> Result<Self, DecodeError> {
        cursor.read()
    }
}

impl Encodable for FaceFx {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        bytes.write(self);
        Ok(())
    }
}
