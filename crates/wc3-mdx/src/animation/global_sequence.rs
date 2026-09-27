//! Model accessors for global sequences.
use crate::ModelVersion;
use crate::{
    Cursor, Decodable, DecodeError, Encodable, EncodeError, Encoder, GlobalSequencesChunk, Model,
    Version,
};

/// One global sequence duration in milliseconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlobalSequence(pub u32);

impl Encodable for GlobalSequence {
    fn encode_to(&self, output: &mut Encoder<'_>) -> Result<(), EncodeError> {
        output.write(self.0);
        Ok(())
    }
}

impl Decodable for GlobalSequence {
    fn decode_one(cursor: &mut Cursor<'_>, _version: Version) -> Result<Self, DecodeError> {
        Ok(Self(cursor.read()?))
    }
}

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
