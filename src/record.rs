//! Binary conversion for typed MDX records.
use crate::Version;

use crate::model::LATEST_VERSION;
use crate::{Cursor, Encoder, Error};

/// A typed MDX record that can be converted to and from bytes.
pub trait Record: Sized {
    /// Parses one record and advances the cursor past it.
    fn decode_one(cursor: &mut Cursor<'_>, version: Version) -> Result<Self, Error>;

    /// Parses exactly one record, rejecting any trailing bytes.
    fn decode(bytes: &[u8], version: Version) -> Result<Self, Error> {
        let mut cursor = Cursor::new(bytes);
        let record = Self::decode_one(&mut cursor, version)?;
        cursor.finish()?;
        Ok(record)
    }

    /// Appends a record to an existing byte buffer.
    fn encode_to(&self, output: &mut Encoder<'_>) -> Result<(), Error>;

    /// Writes a record.
    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut output = Vec::new();
        self.encode_to(&mut Encoder::new(&mut output))?;
        Ok(output)
    }

    /// Parses a record, assuming the model has the latest version
    /// if it has no `VERS` chunk.
    fn decode_latest(bytes: &[u8]) -> Result<Self, Error> {
        Self::decode(bytes, LATEST_VERSION)
    }
}

#[cfg(test)]
mod tests {
    use super::Record;
    use crate::Cursor;
    use crate::KnownChunk;
    use crate::{
        AttachmentsChunk, BindPose, Bone, BonesChunk, Camera, CamerasChunk, CollisionShapesChunk,
        Error, EventObjectsChunk, FaceFxChunk, Geoset, GeosetAnimationsChunk, GeosetsChunk,
        GlobalSequencesChunk, HelpersChunk, LightsChunk, MaterialsChunk, Model, ModelInfoChunk,
        Node, ParticleEmitter, ParticleEmitter2, ParticleEmitters2Chunk, ParticleEmittersChunk,
        PivotPointsChunk, PopcornEmittersChunk, RibbonEmitter, RibbonEmittersChunk, Sequence,
        SequencesChunk, TextureAnimationsChunk, TexturesChunk, VersionChunk,
    };

    fn round_trip<T: Record + PartialEq + std::fmt::Debug>(value: &T, version: u32) {
        let bytes = value.encode().unwrap();
        let mut appended = vec![0xaa, 0xbb];
        value
            .encode_to(&mut crate::Encoder::new(&mut appended))
            .unwrap();
        assert_eq!(&appended[..2], &[0xaa, 0xbb]);
        assert_eq!(&appended[2..], bytes);
        assert_eq!(T::decode(&bytes, version).unwrap(), *value);
    }

    #[test]
    fn one_api_handles_model_fixed_and_versioned_records() {
        let model = Model::new(800);
        let bytes = model.encode().unwrap();
        assert_eq!(Model::decode(&bytes, 800).unwrap().encode().unwrap(), bytes);
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

        let mut cursor = Cursor::new(&bytes);
        let decoded = Sequence::decode_one(&mut cursor, 800).unwrap();
        assert_eq!(decoded, first);
        let consumed = cursor.position();
        assert_eq!(consumed, first.encode().unwrap().len());
        assert_eq!(
            Sequence::decode(&bytes, 800),
            Err(Error::TrailingRecordBytes {
                consumed,
                total: bytes.len(),
            })
        );

        let first = Geoset::new(800, &[], &[], &[]).unwrap();
        let second = Geoset::new(800, &[], &[], &[]).unwrap();
        let mut bytes = first.encode().unwrap();
        bytes.extend_from_slice(&second.encode().unwrap());
        let mut cursor = Cursor::new(&bytes);
        let decoded = Geoset::decode_one(&mut cursor, 800).unwrap();
        assert_eq!(decoded, first);
        assert_eq!(cursor.position(), first.encode().unwrap().len());
        assert!(matches!(
            Geoset::decode(&bytes, 800),
            Err(Error::TrailingRecordBytes { .. })
        ));
    }

    #[test]
    fn cursor_decoders_stop_at_the_next_record() {
        fn check<T: Record + PartialEq + std::fmt::Debug>(first: T, second: T) {
            let mut bytes = first.encode().unwrap();
            let first_len = bytes.len();
            bytes.extend_from_slice(&second.encode().unwrap());
            let mut cursor = Cursor::new(&bytes);
            assert_eq!(T::decode_one(&mut cursor, 800).unwrap(), first);
            assert_eq!(cursor.position(), first_len);
            assert_eq!(T::decode_one(&mut cursor, 800).unwrap(), second);
            cursor.finish().unwrap();
        }

        let first = Node::new("First", 1).unwrap();
        let second = Node::new("Second", 2).unwrap();
        check(first.clone(), second.clone());
        check(
            Bone::new(first.clone(), 1, 2),
            Bone::new(second.clone(), 3, 4),
        );
        check(
            Camera::new("First").unwrap(),
            Camera::new("Second").unwrap(),
        );
        check(
            ParticleEmitter::new(first.clone(), "first.mdx").unwrap(),
            ParticleEmitter::new(second.clone(), "second.mdx").unwrap(),
        );
        check(
            ParticleEmitter2::new(first.clone()),
            ParticleEmitter2::new(second.clone()),
        );
        check(RibbonEmitter::new(first), RibbonEmitter::new(second));
    }

    #[test]
    fn known_top_level_chunks_have_distinct_chunk_records() {
        let tags = [
            VersionChunk::TAG,
            ModelInfoChunk::TAG,
            SequencesChunk::TAG,
            GlobalSequencesChunk::TAG,
            TexturesChunk::TAG,
            MaterialsChunk::TAG,
            GeosetsChunk::TAG,
            GeosetAnimationsChunk::TAG,
            BonesChunk::TAG,
            HelpersChunk::TAG,
            AttachmentsChunk::TAG,
            EventObjectsChunk::TAG,
            CollisionShapesChunk::TAG,
            ParticleEmittersChunk::TAG,
            ParticleEmitters2Chunk::TAG,
            RibbonEmittersChunk::TAG,
            PopcornEmittersChunk::TAG,
            CamerasChunk::TAG,
            LightsChunk::TAG,
            TextureAnimationsChunk::TAG,
            FaceFxChunk::TAG,
            PivotPointsChunk::TAG,
            BindPose::TAG,
        ];
        let mut unique = tags.to_vec();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), tags.len());
    }
}
