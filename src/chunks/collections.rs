//! Complete payloads for chunks containing a sequence of records.

use crate::{Cursor, Error, KnownChunk, Record};

/// A complete chunk made of consecutive records of one type.
pub trait CollectionChunk: Sized {
    /// The record type stored in this chunk.
    type Item: Record;

    /// Returns this chunk type's tag.
    fn tag() -> [u8; 4];

    /// Borrows the records in file order.
    fn records(&self) -> &[Self::Item];

    /// Builds a chunk payload from decoded records.
    fn from_records(records: Vec<Self::Item>) -> Self;
}

impl<C: CollectionChunk> Record for C {
    fn decode_one(cursor: &mut Cursor<'_>, version: u32) -> Result<Self, Error> {
        let mut records = Vec::new();
        while !cursor.remaining().is_empty() {
            let start = cursor.position();
            let record = C::Item::decode_one(cursor, version)?;
            if cursor.position() <= start {
                return Err(Error::MalformedRecord {
                    tag: C::tag(),
                    offset: start,
                });
            }
            records.push(record);
        }
        Ok(C::from_records(records))
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = Vec::new();
        for record in self.records() {
            bytes.extend_from_slice(&record.encode()?);
            if bytes.len() > u32::MAX as usize {
                return Err(Error::ChunkTooLarge {
                    tag: C::tag(),
                    size: bytes.len(),
                });
            }
        }
        Ok(bytes)
    }
}

macro_rules! record_collection {
    ($name:ident, $item:ty) => {
        #[doc = concat!("The complete `", stringify!($name), "` chunk payload.")]
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            /// Records in their original order.
            pub records: Vec<$item>,
        }

        impl $name {
            /// Creates a chunk payload from its records.
            pub fn new(records: Vec<$item>) -> Self {
                Self { records }
            }
        }

        impl CollectionChunk for $name {
            type Item = $item;
            fn tag() -> [u8; 4] {
                <$item>::TAG
            }
            fn records(&self) -> &[Self::Item] {
                &self.records
            }
            fn from_records(records: Vec<Self::Item>) -> Self {
                Self { records }
            }
        }

        impl KnownChunk for $name {
            const TAG: [u8; 4] = <$item>::TAG;
        }
    };
}

record_collection!(SequencesChunk, crate::Sequence);
record_collection!(TexturesChunk, crate::Texture);
record_collection!(GeosetsChunk, crate::Geoset);
record_collection!(GeosetAnimationsChunk, crate::GeosetAnimation);
record_collection!(MaterialsChunk, crate::Material);
record_collection!(BonesChunk, crate::Bone);
record_collection!(HelpersChunk, crate::Node);
record_collection!(AttachmentsChunk, crate::Attachment);
record_collection!(CamerasChunk, crate::Camera);
record_collection!(CollisionShapesChunk, crate::CollisionShape);
record_collection!(EventObjectsChunk, crate::EventObject);
record_collection!(FaceFxChunk, crate::FaceFx);
record_collection!(LightsChunk, crate::Light);
record_collection!(ParticleEmittersChunk, crate::ParticleEmitter);
record_collection!(ParticleEmitters2Chunk, crate::ParticleEmitter2);
record_collection!(PopcornEmittersChunk, crate::PopcornEmitter);
record_collection!(RibbonEmittersChunk, crate::RibbonEmitter);
record_collection!(TextureAnimationsChunk, crate::TextureAnimation);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chunk;

    #[test]
    fn fixed_width_collection_uses_the_whole_chunk() {
        let records = vec![
            crate::Sequence::new("Stand", [0, 100]).unwrap(),
            crate::Sequence::new("Walk", [101, 200]).unwrap(),
        ];
        let original = SequencesChunk::new(records);
        let chunk = original.encode_chunk().unwrap();
        assert_eq!(
            SequencesChunk::decode_chunk(&chunk, 1800).unwrap(),
            original
        );
        assert!(crate::Sequence::decode(&chunk.data, 800).is_err());
    }

    #[test]
    fn variable_width_collection_uses_the_whole_chunk() {
        let records = vec![
            crate::Geoset::new(1800, &[], &[], &[]).unwrap(),
            crate::Geoset::new(1800, &[], &[], &[]).unwrap(),
        ];
        let original = GeosetsChunk::new(records);
        let bytes = original.encode().unwrap();
        assert_eq!(GeosetsChunk::decode(&bytes, 1800).unwrap(), original);
    }
}
