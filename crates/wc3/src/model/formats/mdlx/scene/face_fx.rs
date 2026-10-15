//! Reforged facial-animation resource references.
use crate::model::formats::mdlx::Dialect;
use crate::model::formats::mdlx::Document;
use crate::model::formats::mdlx::FaceFxChunk;
use crate::model::formats::mdlx::FixedText;
use crate::model::formats::mdlx::SupportsReforgedChunks;
use crate::model::formats::mdlx::ValueError;
use crate::model::formats::mdlx::{mdl, mdx};

const NAME_SIZE: usize = 80;
const PATH_SIZE: usize = 260;

/// A named facial-animation resource for a Reforged model.
#[derive(
    Clone, Debug, Eq, PartialEq, mdx::Read, mdx::ReadOwned, mdx::Write, mdl::Read, mdl::Write,
)]
#[mdl(block = "FaceFX")]
pub struct FaceFx {
    #[mdl(header)]
    /// Name identifying the facial-animation target.
    pub name: FixedText<NAME_SIZE>,
    #[mdl(property = "Path", default)]
    /// Path to the facial-animation resource.
    pub path: FixedText<PATH_SIZE>,
}

impl FaceFx {
    /// Creates a face-animation reference.
    pub fn new(name: &str, path: &str) -> Result<Self, ValueError> {
        let mut entry = Self {
            name: FixedText::default(),
            path: FixedText::default(),
        };
        entry.name.set_text(name)?;
        entry.path.set_text(path)?;
        Ok(entry)
    }
}

impl<'data, D: Dialect> Document<'data, D> {
    /// Borrows records in chunk and record order.
    pub fn try_face_fx<'s>(
        &'s self,
    ) -> Result<impl Iterator<Item = &'s FaceFx> + use<'data, 's, D>, ValueError> {
        self.check_chunk_version(*b"FAFX")?;
        Ok(self.records::<FaceFxChunk>())
    }

    /// Mutably borrows records in file order without replacing their chunks.
    pub fn try_face_fx_mut<'s>(
        &'s mut self,
    ) -> Result<impl Iterator<Item = &'s mut FaceFx> + use<'data, 's, D>, ValueError> {
        self.check_chunk_version(*b"FAFX")?;
        Ok(self.records_mut::<FaceFxChunk>())
    }

    /// Returns an error if this model version does not support `FAFX`.
    pub fn try_set_face_fx(
        &mut self,
        entries: impl IntoIterator<Item = FaceFx>,
    ) -> Result<(), ValueError> {
        self.check_chunk_version(*b"FAFX")?;
        self.replace_chunk(FaceFxChunk::new(entries.into_iter().collect()));
        Ok(())
    }
}

impl<'data, D: Dialect> Document<'data, D>
where
    D::Version: SupportsReforgedChunks,
{
    /// Borrows records in chunk and record order.
    pub fn face_fx<'s>(&'s self) -> impl Iterator<Item = &'s FaceFx> + use<'data, 's, D> {
        self.records::<FaceFxChunk>()
    }

    /// Mutably borrows records in file order without replacing their chunks.
    pub fn face_fx_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut FaceFx> + use<'data, 's, D> {
        self.records_mut::<FaceFxChunk>()
    }

    /// Replaces `FAFX` records.
    pub fn set_face_fx(&mut self, entries: impl IntoIterator<Item = FaceFx>) {
        self.replace_chunk(FaceFxChunk::new(entries.into_iter().collect()));
    }
}
