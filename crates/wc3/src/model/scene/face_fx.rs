//! Reforged face-animation references in `FAFX` chunks.
use crate::model::ValueError;
use crate::model::{mdl, mdx};
use crate::model::{ModelVersion, SupportsReforgedChunks};

use crate::model::FaceFxChunk;

use std::borrow::Cow;

use crate::model::FixedText;
use crate::model::Model;

const NAME_SIZE: usize = 80;
const PATH_SIZE: usize = 260;

/// One fixed-size face-animation name and path pair.
#[derive(Clone, Debug, Eq, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(block = "FaceFX")]
pub struct FaceFx {
    #[mdl(header)]
    name: FixedText<NAME_SIZE>,
    #[mdl(property = "Path", default)]
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
    /// Returns an error if this model version does not support `FAFX`.
    pub fn try_face_fx(&self) -> Result<Vec<FaceFx>, ValueError> {
        self.check_chunk_version(*b"FAFX")?;
        Ok(self.collect_chunk_records::<FaceFxChunk>())
    }

    /// Returns an error if this model version does not support `FAFX`.
    pub fn try_set_face_fx(&mut self, entries: &[FaceFx]) -> Result<(), ValueError> {
        self.check_chunk_version(*b"FAFX")?;
        self.replace_chunk(FaceFxChunk::new(entries.to_vec()));
        Ok(())
    }
}

impl<V: SupportsReforgedChunks> Model<V> {
    /// Returns decoded `FAFX` records.
    pub fn face_fx(&self) -> Vec<FaceFx> {
        self.collect_chunk_records::<FaceFxChunk>()
    }

    /// Replaces `FAFX` records.
    pub fn set_face_fx(&mut self, entries: &[FaceFx]) {
        self.replace_chunk(FaceFxChunk::new(entries.to_vec()));
    }
}
