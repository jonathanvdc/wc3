//! Model accessors for global sequences.
use crate::mdl::{ReadError as MdlError, MdlRead, MdlWrite, MdlWriter, Parser, WriteError};
use crate::ModelVersion;
use crate::{GlobalSequencesChunk, Model, Readable, Writable};
use std::io::Write;

/// One global sequence duration in milliseconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Readable, Writable)]
pub struct GlobalSequence(pub u32);

impl<V: ModelVersion> Model<V> {
    /// Returns durations from every `GLBS` chunk in file order.
    pub fn global_sequences(&self) -> Vec<u32> {
        self.collect_chunk_records::<GlobalSequencesChunk>()
            .into_iter()
            .map(|sequence| sequence.0)
            .collect()
    }

    /// Writes global sequence durations to a `GLBS` chunk.
    pub fn set_global_sequences(&mut self, durations: &[u32]) {
        self.replace_chunk(GlobalSequencesChunk::new(
            durations.iter().copied().map(GlobalSequence).collect(),
        ));
    }
}

impl MdlRead for GlobalSequence {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, MdlError> {
        parser.expect_ident("Duration")?;
        Ok(Self(parser.read_property()?))
    }
}
impl MdlWrite for GlobalSequence {
    fn write_mdl<W: Write>(&self, writer: &mut MdlWriter<W>) -> Result<(), WriteError> {
        writer.property("Duration", &self.0)
    }
}
