//! Reforged facial-animation resource references.
use crate::model::FaceFxChunk;
use crate::model::FixedText;
use crate::model::Model;
use crate::model::ModelDialect;
use crate::model::SupportsReforgedChunks;
use crate::model::ValueError;
use crate::model::{mdl, mdx};

const NAME_SIZE: usize = 80;
const PATH_SIZE: usize = 260;

/// A named facial-animation resource for a Reforged model.
#[derive(Clone, Debug, Eq, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
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

impl<'data, D: ModelDialect> Model<'data, D> {
    /// Borrows records in chunk and record order.
    pub fn try_face_fx<'s>(
        &'s self,
    ) -> Result<impl Iterator<Item = &'s FaceFx> + use<'data, 's, D>, ValueError> {
        self.check_chunk_version(*b"FAFX")?;
        Ok(self.records::<FaceFxChunk>())
    }

    /// Returns an error if this model version does not support `FAFX`.
    pub fn try_set_face_fx(&mut self, entries: &[FaceFx]) -> Result<(), ValueError> {
        self.check_chunk_version(*b"FAFX")?;
        self.replace_chunk(FaceFxChunk::new(entries.to_vec()));
        Ok(())
    }
}

impl<'data, D: ModelDialect> Model<'data, D>
where
    D::Version: SupportsReforgedChunks,
{
    /// Borrows records in chunk and record order.
    pub fn face_fx<'s>(&'s self) -> impl Iterator<Item = &'s FaceFx> + use<'data, 's, D> {
        self.records::<FaceFxChunk>()
    }

    /// Replaces `FAFX` records.
    pub fn set_face_fx(&mut self, entries: &[FaceFx]) {
        self.replace_chunk(FaceFxChunk::new(entries.to_vec()));
    }
}
