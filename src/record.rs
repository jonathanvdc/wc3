//! Binary conversion for typed MDX records.

use crate::model::LATEST_VERSION;
use crate::Error;

/// A typed MDX record that can be converted to and from bytes.
pub trait Record: Sized {
    /// Parses the first record in a byte stream and returns its length.
    fn decode_one(bytes: &[u8], version: u32) -> Result<(Self, usize), Error>;

    /// Parses exactly one record, rejecting any trailing bytes.
    fn decode(bytes: &[u8], version: u32) -> Result<Self, Error> {
        let (record, consumed) = Self::decode_one(bytes, version)?;
        if consumed != bytes.len() {
            return Err(Error::TrailingRecordBytes {
                consumed,
                total: bytes.len(),
            });
        }
        Ok(record)
    }

    /// Writes a record.
    fn encode(&self) -> Result<Vec<u8>, Error>;

    /// Parses a record, assuming the model has the latest version
    /// if it has no `VERS` chunk.
    fn decode_latest(bytes: &[u8]) -> Result<Self, Error> {
        Self::decode(bytes, LATEST_VERSION)
    }
}

/// Reads a record's size word and checks that its complete payload is available.
pub(crate) fn sized_record_len(
    bytes: &[u8],
    tag: [u8; 4],
    minimum: usize,
    mask: u32,
    extra: usize,
) -> Result<usize, Error> {
    let size_bytes = bytes
        .get(..4)
        .ok_or(Error::MalformedRecord { tag, offset: 0 })?;
    let size = (u32::from_le_bytes(size_bytes.try_into().expect("size word")) & mask) as usize;
    if size < minimum {
        return Err(Error::MalformedRecord { tag, offset: 0 });
    }
    size.checked_add(extra)
        .filter(|&length| length <= bytes.len())
        .ok_or(Error::MalformedRecord { tag, offset: 0 })
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
    fn decode_one_advances_through_records_and_decode_rejects_trailing_bytes() {
        let first = Sequence::new("Stand", [0, 100]).unwrap();
        let second = Sequence::new("Walk", [101, 200]).unwrap();
        let mut bytes = first.encode().unwrap();
        bytes.extend_from_slice(&second.encode().unwrap());

        let (decoded, consumed) = Sequence::decode_one(&bytes, 800).unwrap();
        assert_eq!(decoded, first);
        assert_eq!(consumed, first.encode().unwrap().len());
        assert_eq!(
            Sequence::decode(&bytes, 800),
            Err(crate::Error::TrailingRecordBytes {
                consumed,
                total: bytes.len(),
            })
        );

        let first = Geoset::new(800, &[], &[], &[]).unwrap();
        let second = Geoset::new(800, &[], &[], &[]).unwrap();
        let mut bytes = first.encode().unwrap();
        bytes.extend_from_slice(&second.encode().unwrap());
        let (decoded, consumed) = Geoset::decode_one(&bytes, 800).unwrap();
        assert_eq!(decoded, first);
        assert_eq!(consumed, first.encode().unwrap().len());
        assert!(matches!(
            Geoset::decode(&bytes, 800),
            Err(crate::Error::TrailingRecordBytes { .. })
        ));
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
