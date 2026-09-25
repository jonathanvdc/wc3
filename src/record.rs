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
    fn cursor_decoders_stop_at_the_next_record() {
        fn check<T: Record + PartialEq + std::fmt::Debug>(first: T, second: T) {
            let mut bytes = first.encode().unwrap();
            let first_len = bytes.len();
            bytes.extend_from_slice(&second.encode().unwrap());
            assert_eq!(T::decode_one(&bytes, 800).unwrap(), (first, first_len));
            assert_eq!(T::decode_one(&bytes[first_len..], 800).unwrap().0, second);
        }

        let first = crate::Node::new("First", 1).unwrap();
        let second = crate::Node::new("Second", 2).unwrap();
        check(first.clone(), second.clone());
        check(
            crate::Bone::new(first.clone(), 1, 2),
            crate::Bone::new(second.clone(), 3, 4),
        );
        check(
            crate::Camera::new("First").unwrap(),
            crate::Camera::new("Second").unwrap(),
        );
        check(
            crate::ParticleEmitter::new(first.clone(), "first.mdx").unwrap(),
            crate::ParticleEmitter::new(second.clone(), "second.mdx").unwrap(),
        );
        check(
            crate::ParticleEmitter2::new(first.clone()).unwrap(),
            crate::ParticleEmitter2::new(second.clone()).unwrap(),
        );
        check(
            crate::RibbonEmitter::new(first).unwrap(),
            crate::RibbonEmitter::new(second).unwrap(),
        );
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
