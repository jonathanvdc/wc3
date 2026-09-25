//! Binary conversion for typed MDX records.

use crate::model::LATEST_VERSION;
use crate::Error;

/// A typed MDX record that can be converted to and from bytes.
pub trait Record: Sized {
    /// Parses a record.
    fn decode(bytes: &[u8], version: u32) -> Result<Self, Error>;

    /// Writes a record.
    fn encode(&self) -> Result<Vec<u8>, Error>;

    /// Parses a record, assuming the model has the latest version
    /// if it has no `VERS` chunk.
    fn decode_latest(bytes: &[u8]) -> Result<Self, Error> {
        Self::decode(bytes, LATEST_VERSION)
    }
}

#[cfg(test)]
mod tests {
    use super::Record;
    use crate::KnownChunk;
    use crate::{Geoset, Model, Sequence};

    fn round_trip<T: Record + PartialEq + std::fmt::Debug>(value: &T, version: u32) {
        let bytes = value.encode().unwrap();
        assert_eq!(T::decode(&bytes, version).unwrap(), *value);
    }

    #[test]
    fn one_api_handles_model_fixed_and_versioned_records() {
        round_trip(&Model::new(800), 800);
        round_trip(&Sequence::new("Stand", [0, 100]).unwrap(), 800);
        round_trip(&Geoset::new(1800, &[], &[], &[]).unwrap(), 1800);
        assert!(Sequence::decode(&[0; 131], 800).is_err());
    }

    #[test]
    fn known_top_level_chunks_have_distinct_chunk_records() {
        let tags = [
            crate::VersionChunk::TAG,
            crate::ModelInfoChunk::TAG,
            crate::SequencesChunk::TAG,
            crate::GlobalSequencesChunk::TAG,
            crate::TexturesChunk::TAG,
            crate::MaterialsChunk::TAG,
            crate::GeosetsChunk::TAG,
            crate::GeosetAnimationsChunk::TAG,
            crate::BonesChunk::TAG,
            crate::HelpersChunk::TAG,
            crate::AttachmentsChunk::TAG,
            crate::EventObjectsChunk::TAG,
            crate::CollisionShapesChunk::TAG,
            crate::ParticleEmittersChunk::TAG,
            crate::ParticleEmitters2Chunk::TAG,
            crate::RibbonEmittersChunk::TAG,
            crate::PopcornEmittersChunk::TAG,
            crate::CamerasChunk::TAG,
            crate::LightsChunk::TAG,
            crate::TextureAnimationsChunk::TAG,
            crate::FaceFxChunk::TAG,
            crate::PivotPointsChunk::TAG,
            crate::BindPose::TAG,
        ];
        let mut unique = tags.to_vec();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), tags.len());
    }
}
