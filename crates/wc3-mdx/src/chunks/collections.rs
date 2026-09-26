//! Complete chunks containing a sequence of records.
use crate::EncodeError;
use crate::Encoder;
use crate::{Tag, Version};

use crate::{
    Attachment, Bone, Camera, CollisionShape, EventObject, FaceFx, Geoset, GeosetAnimation, Light,
    Material, Node, ParticleEmitter, ParticleEmitter2, PopcornEmitter, RibbonEmitter, Sequence,
    Texture, TextureAnimation,
};
use crate::{Chunk, Cursor, Decodable, DecodeError, Encodable, KnownChunk, Record};

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
    ($name:ident, $item:ty) => {
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
                <$item>::TAG
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
            const TAG: Tag = <$item>::TAG;

            fn decode_payload(
                cursor: &mut Cursor<'_>,
                version: Version,
            ) -> Result<Self, DecodeError> {
                decode_records(cursor, version)
            }
        }
    };
}

record_collection!(SequencesChunk, Sequence);
record_collection!(TexturesChunk, Texture);
record_collection!(GeosetsChunk, Geoset);
record_collection!(GeosetAnimationsChunk, GeosetAnimation);
record_collection!(MaterialsChunk, Material);
record_collection!(BonesChunk, Bone);
record_collection!(HelpersChunk, Node);
record_collection!(AttachmentsChunk, Attachment);
record_collection!(CamerasChunk, Camera);
record_collection!(CollisionShapesChunk, CollisionShape);
record_collection!(EventObjectsChunk, EventObject);
record_collection!(FaceFxChunk, FaceFx);
record_collection!(LightsChunk, Light);
record_collection!(ParticleEmittersChunk, ParticleEmitter);
record_collection!(ParticleEmitters2Chunk, ParticleEmitter2);
record_collection!(PopcornEmittersChunk, PopcornEmitter);
record_collection!(RibbonEmittersChunk, RibbonEmitter);
record_collection!(TextureAnimationsChunk, TextureAnimation);

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
            Geoset::new(1800, &[], &[], &[]).unwrap(),
            Geoset::new(1800, &[], &[], &[]).unwrap(),
        ];
        let original = GeosetsChunk::new(records);
        let bytes = original.encode().unwrap();
        assert_eq!(GeosetsChunk::decode(&bytes, 1800).unwrap(), original);
    }
}
