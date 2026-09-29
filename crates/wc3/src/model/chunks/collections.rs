//! Complete chunks containing a sequence of records.
use crate::model::mdx;
use crate::model::Encoder;
use crate::model::{
    Attachment, Bone, Camera, CollisionShape, EventObject, FaceFx, Geoset, GeosetAnimation, Glider,
    Light, Material, Node, ParticleEmitter, ParticleEmitter2, PopcornEmitter, RibbonEmitter,
    Sequence, Texture, TextureAnimation,
};
use crate::model::{Chunk, Cursor, KnownChunk};
use crate::model::{GlobalSequence, PivotPoint};
use crate::model::{ModelVersion, Tag};
use std::marker::PhantomData;

/// A complete chunk made of consecutive records of one type.
pub trait CollectionChunk: Sized {
    /// The record type stored in this chunk.
    type Item;

    /// Returns this chunk type's tag.
    fn tag() -> Tag;

    /// Borrows the records in file order.
    fn records(&self) -> &[Self::Item];

    /// Builds a chunk from decoded records.
    fn from_records(records: Vec<Self::Item>) -> Self;
}

fn decode_records<C: CollectionChunk>(cursor: &mut Cursor<'_>) -> Result<C, mdx::ReadError>
where
    C::Item: mdx::Read,
{
    let mut records = Vec::new();
    while !cursor.remaining().is_empty() {
        let start = cursor.absolute_position();
        let record = cursor.read()?;
        if cursor.absolute_position() <= start {
            return Err(
                mdx::ReadError::new(start, mdx::ReadErrorKind::NoProgress).with_tag(C::tag())
            );
        }
        records.push(record);
    }
    Ok(C::from_records(records))
}

fn encode_records<C: CollectionChunk>(
    chunk: &C,
    bytes: &mut Encoder<'_>,
) -> Result<(), mdx::WriteError>
where
    C::Item: mdx::Write,
{
    let start = bytes.position();
    for record in chunk.records() {
        bytes.write(record)?;
        if bytes.position() - start > u32::MAX as usize {
            return Err(mdx::WriteError::SizeOverflow {
                field: "encoded size",
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

            fn encode_payload_to(&self, bytes: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
                encode_records(self, bytes)
            }
        }

        impl KnownChunk for $name {
            const TAG: Tag = $tag;

            fn decode_payload(cursor: &mut Cursor<'_>) -> Result<Self, mdx::ReadError> {
                decode_records(cursor)
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
            fn encode_payload_to(&self, bytes: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
                encode_records(self, bytes)
            }
        }

        impl<V: ModelVersion> KnownChunk for $name<V> {
            const TAG: Tag = $tag;
            fn decode_payload(cursor: &mut Cursor<'_>) -> Result<Self, mdx::ReadError> {
                decode_records(cursor)
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
record_collection!(GlidersChunk, Glider, *b"DILG");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::V1800;
    use crate::model::{mdx::Read, mdx::Write};

    #[test]
    fn fixed_width_collection_uses_the_whole_chunk() {
        let records = vec![
            Sequence::new("Stand", [0, 100]).unwrap(),
            Sequence::new("Walk", [101, 200]).unwrap(),
        ];
        let original = SequencesChunk::new(records);
        let payload = original.encode_mdx().unwrap();
        assert_eq!(SequencesChunk::decode_mdx(&payload).unwrap(), original);
        assert!(Sequence::decode_mdx(&payload).is_err());
    }

    #[test]
    fn variable_width_collection_uses_the_whole_chunk() {
        let records = vec![
            Geoset::<V1800>::new(&[], &[], &[]).unwrap(),
            Geoset::<V1800>::new(&[], &[], &[]).unwrap(),
        ];
        let original = GeosetsChunk::new(records);
        let bytes = original.encode_mdx().unwrap();
        assert_eq!(GeosetsChunk::<V1800>::decode_mdx(&bytes).unwrap(), original);
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
            GlobalSequencesChunk::decode_mdx(&durations.encode_mdx().unwrap()).unwrap(),
            durations
        );
        assert_eq!(
            PivotPointsChunk::decode_mdx(&points.encode_mdx().unwrap()).unwrap(),
            points
        );
        assert!(GlobalSequencesChunk::decode_mdx(
            &[b"GLBS".as_slice(), &1u32.to_le_bytes(), &[1]].concat()
        )
        .is_err());
        assert!(PivotPointsChunk::decode_mdx(
            &[b"PIVT".as_slice(), &1u32.to_le_bytes(), &[1]].concat()
        )
        .is_err());
    }
}
