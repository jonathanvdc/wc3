//! Complete chunks containing a sequence of records.
use crate::EncodeError;
use crate::Encoder;
use crate::{ModelVersion, Tag, Version};
use std::marker::PhantomData;

use crate::{
    Attachment, Bone, Camera, CollisionShape, EventObject, FaceFx, Geoset, GeosetAnimation, Light,
    Material, Node, ParticleEmitter, ParticleEmitter2, PopcornEmitter, RibbonEmitter, Sequence,
    Texture, TextureAnimation,
};
use crate::{Chunk, Cursor, Decodable, DecodeError, Encodable, KnownChunk, Record};
use crate::{GlobalSequence, PivotPoint};

/// A complete chunk made of consecutive records of one type.
pub trait CollectionChunk: Sized {
    /// The record type stored in this chunk.
    type Item: Record;

    /// Returns this chunk type's tag.
    fn tag() -> Tag;

    /// Borrows the records in file order.
    fn records(&self) -> &[Self::Item];

    /// Builds a chunk from decoded records.
    fn from_records(records: Vec<Self::Item>) -> Self;
}

fn decode_records<C: CollectionChunk>(
    cursor: &mut Cursor<'_>,
    version: Version,
) -> Result<C, DecodeError> {
    let mut records = Vec::new();
    while !cursor.remaining().is_empty() {
        let start = cursor.position();
        let record = C::Item::decode_one(cursor, version)?;
        if cursor.position() <= start {
            return Err(DecodeError::MalformedRecord {
                tag: C::tag(),
                offset: start,
            });
        }
        records.push(record);
    }
    Ok(C::from_records(records))
}

fn encode_records<C: CollectionChunk>(
    chunk: &C,
    bytes: &mut Encoder<'_>,
) -> Result<(), EncodeError> {
    let start = bytes.position();
    for record in chunk.records() {
        record.encode_to(bytes)?;
        if bytes.position() - start > u32::MAX as usize {
            return Err(EncodeError::ChunkTooLarge {
                tag: C::tag(),
                size: bytes.position() - start,
            });
        }
    }
    Ok(())
}

macro_rules! record_collection {
    ($name:ident, $item:ty, $tag:expr) => {
        #[doc = concat!("The complete `", stringify!($name), "` chunk.")]
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
            fn tag() -> Tag {
                $tag
            }
            fn records(&self) -> &[Self::Item] {
                &self.records
            }
            fn from_records(records: Vec<Self::Item>) -> Self {
                Self { records }
            }
        }

        impl Chunk for $name {
            fn tag(&self) -> Tag {
                Self::TAG
            }

            fn encode_payload_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
                encode_records(self, bytes)
            }
        }

        impl KnownChunk for $name {
            const TAG: Tag = $tag;

            fn decode_payload(
                cursor: &mut Cursor<'_>,
                version: Version,
            ) -> Result<Self, DecodeError> {
                decode_records(cursor, version)
            }
        }
    };
}

macro_rules! versioned_record_collection {
    ($name:ident, $item:ident, $tag:expr) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name<V: ModelVersion> {
            pub records: Vec<$item<V>>,
            version: PhantomData<V>,
        }

        impl<V: ModelVersion> $name<V> {
            pub fn new(records: Vec<$item<V>>) -> Self {
                Self {
                    records,
                    version: PhantomData,
                }
            }
        }

        impl<V: ModelVersion> CollectionChunk for $name<V> {
            type Item = $item<V>;
            fn tag() -> Tag {
                $tag
            }
            fn records(&self) -> &[Self::Item] {
                &self.records
            }
            fn from_records(records: Vec<Self::Item>) -> Self {
                Self::new(records)
            }
        }

        impl<V: ModelVersion> Chunk for $name<V> {
            fn tag(&self) -> Tag {
                Self::TAG
            }
            fn encode_payload_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
                encode_records(self, bytes)
            }
        }

        impl<V: ModelVersion> KnownChunk for $name<V> {
            const TAG: Tag = $tag;
            fn decode_payload(
                cursor: &mut Cursor<'_>,
                _version: Version,
            ) -> Result<Self, DecodeError> {
                decode_records(cursor, V::NUMBER)
            }
        }
    };
}

record_collection!(SequencesChunk, Sequence, *b"SEQS");
record_collection!(TexturesChunk, Texture, *b"TEXS");
versioned_record_collection!(GeosetsChunk, Geoset, *b"GEOS");
record_collection!(GeosetAnimationsChunk, GeosetAnimation, *b"GEOA");
versioned_record_collection!(MaterialsChunk, Material, *b"MTLS");
record_collection!(BonesChunk, Bone, *b"BONE");
record_collection!(HelpersChunk, Node, *b"HELP");
record_collection!(AttachmentsChunk, Attachment, *b"ATCH");
versioned_record_collection!(CamerasChunk, Camera, *b"CAMS");
record_collection!(CollisionShapesChunk, CollisionShape, *b"CLID");
record_collection!(EventObjectsChunk, EventObject, *b"EVTS");
record_collection!(FaceFxChunk, FaceFx, *b"FAFX");
versioned_record_collection!(LightsChunk, Light, *b"LITE");
record_collection!(ParticleEmittersChunk, ParticleEmitter, *b"PREM");
record_collection!(ParticleEmitters2Chunk, ParticleEmitter2, *b"PRE2");
record_collection!(PopcornEmittersChunk, PopcornEmitter, *b"CORN");
record_collection!(RibbonEmittersChunk, RibbonEmitter, *b"RIBB");
record_collection!(TextureAnimationsChunk, TextureAnimation, *b"TXAN");
record_collection!(GlobalSequencesChunk, GlobalSequence, *b"GLBS");
record_collection!(PivotPointsChunk, PivotPoint, *b"PIVT");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_width_collection_uses_the_whole_chunk() {
        let records = vec![
            Sequence::new("Stand", [0, 100]).unwrap(),
            Sequence::new("Walk", [101, 200]).unwrap(),
        ];
        let original = SequencesChunk::new(records);
        let payload = original.encode().unwrap();
        assert_eq!(SequencesChunk::decode(&payload, 1800).unwrap(), original);
        assert!(Sequence::decode(&payload, 800).is_err());
    }

    #[test]
    fn variable_width_collection_uses_the_whole_chunk() {
        let records = vec![
            Geoset::<crate::V1800>::new(&[], &[], &[]).unwrap(),
            Geoset::<crate::V1800>::new(&[], &[], &[]).unwrap(),
        ];
        let original = GeosetsChunk::new(records);
        let bytes = original.encode().unwrap();
        assert_eq!(
            GeosetsChunk::<crate::V1800>::decode(&bytes, 1800).unwrap(),
            original
        );
    }

    #[test]
    fn fixed_width_values_use_collection_codec() {
        let durations = GlobalSequencesChunk::new(vec![
            GlobalSequence(100),
            GlobalSequence(200),
            GlobalSequence(300),
        ]);
        let points = PivotPointsChunk::new(vec![
            PivotPoint([1.0, 2.0, 3.0]),
            PivotPoint([4.0, 5.0, 6.0]),
        ]);
        assert_eq!(
            GlobalSequencesChunk::decode(&durations.encode().unwrap(), 800).unwrap(),
            durations
        );
        assert_eq!(
            PivotPointsChunk::decode(&points.encode().unwrap(), 800).unwrap(),
            points
        );
        assert!(GlobalSequencesChunk::decode(
            &[b"GLBS".as_slice(), &1u32.to_le_bytes(), &[1]].concat(),
            800
        )
        .is_err());
        assert!(PivotPointsChunk::decode(
            &[b"PIVT".as_slice(), &1u32.to_le_bytes(), &[1]].concat(),
            800
        )
        .is_err());
    }
}
