//! Complete payloads for chunks containing a sequence of records.

use crate::{Error, KnownChunk, Record};

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

    /// Splits one chunk payload at record boundaries.
    fn record_slices(bytes: &[u8]) -> Result<Vec<&[u8]>, Error>;
}

impl<C: CollectionChunk> Record for C {
    fn decode(bytes: &[u8], version: u32) -> Result<Self, Error> {
        let records = C::record_slices(bytes)?
            .into_iter()
            .map(|record| C::Item::decode(record, version))
            .collect::<Result<Vec<_>, _>>()?;
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

pub(crate) fn fixed_records(bytes: &[u8], tag: [u8; 4], size: usize) -> Result<Vec<&[u8]>, Error> {
    if bytes.len() % size != 0 {
        return Err(Error::MalformedChunk {
            tag,
            size: bytes.len(),
            expected: size,
        });
    }
    Ok(bytes.chunks_exact(size).collect())
}

pub(crate) fn sized_records(
    bytes: &[u8],
    tag: [u8; 4],
    minimum: usize,
    mask: u32,
    extra: usize,
) -> Result<Vec<&[u8]>, Error> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let size_bytes = bytes
            .get(offset..offset.saturating_add(4))
            .ok_or(Error::MalformedRecord { tag, offset })?;
        let size =
            (u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) & mask) as usize;
        if size < minimum {
            return Err(Error::MalformedRecord { tag, offset });
        }
        let end = offset
            .checked_add(size)
            .and_then(|end| end.checked_add(extra))
            .filter(|&end| end <= bytes.len())
            .ok_or(Error::MalformedRecord { tag, offset })?;
        records.push(&bytes[offset..end]);
        offset = end;
    }
    Ok(records)
}

pub(crate) fn records_with_end(
    bytes: &[u8],
    tag: [u8; 4],
    end_of_record: fn(&[u8], usize) -> Result<usize, Error>,
) -> Result<Vec<&[u8]>, Error> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let end = end_of_record(bytes, offset)?;
        if end <= offset || end > bytes.len() {
            return Err(Error::MalformedRecord { tag, offset });
        }
        records.push(&bytes[offset..end]);
        offset = end;
    }
    Ok(records)
}

macro_rules! record_collection {
    ($name:ident, $item:ty, |$bytes:ident : &[u8]| $split:expr) => {
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
            fn record_slices($bytes: &[u8]) -> Result<Vec<&[u8]>, Error> {
                $split
            }
        }

        impl KnownChunk for $name {
            const TAG: [u8; 4] = <$item>::TAG;
        }
    };
}

record_collection!(SequencesChunk, crate::Sequence, |b: &[u8]| fixed_records(
    b,
    crate::Sequence::TAG,
    crate::sequence::SIZE
));
record_collection!(TexturesChunk, crate::Texture, |b: &[u8]| fixed_records(
    b,
    crate::Texture::TAG,
    crate::texture::SIZE
));
record_collection!(GeosetsChunk, crate::Geoset, |b: &[u8]| sized_records(
    b,
    crate::Geoset::TAG,
    4,
    u32::MAX,
    0
));
record_collection!(
    GeosetAnimationsChunk,
    crate::GeosetAnimation,
    |b: &[u8]| sized_records(
        b,
        crate::GeosetAnimation::TAG,
        crate::geoset_animation::HEADER_SIZE,
        u32::MAX,
        0
    )
);
record_collection!(MaterialsChunk, crate::Material, |b: &[u8]| sized_records(
    b,
    crate::Material::TAG,
    4,
    u32::MAX,
    0
));
record_collection!(BonesChunk, crate::Bone, |b: &[u8]| sized_records(
    b,
    crate::Bone::TAG,
    crate::node::HEADER_SIZE,
    u32::MAX,
    8
));
record_collection!(HelpersChunk, crate::Node, |b: &[u8]| sized_records(
    b,
    crate::Node::TAG,
    crate::node::HEADER_SIZE,
    u32::MAX,
    0
));
record_collection!(AttachmentsChunk, crate::Attachment, |b: &[u8]| {
    records_with_end(b, crate::Attachment::TAG, crate::attachment::record_end)
});
record_collection!(CamerasChunk, crate::Camera, |b: &[u8]| sized_records(
    b,
    crate::Camera::TAG,
    crate::camera::HEADER_SIZE,
    0x00ff_ffff,
    0
));
record_collection!(CollisionShapesChunk, crate::CollisionShape, |b: &[u8]| {
    records_with_end(b, crate::CollisionShape::TAG, crate::collision::record_end)
});
record_collection!(EventObjectsChunk, crate::EventObject, |b: &[u8]| {
    records_with_end(b, crate::EventObject::TAG, crate::event::record_end)
});
record_collection!(FaceFxChunk, crate::FaceFx, |b: &[u8]| fixed_records(
    b,
    crate::FaceFx::TAG,
    crate::face_fx::SIZE
));
record_collection!(LightsChunk, crate::Light, |b: &[u8]| records_with_end(
    b,
    crate::Light::TAG,
    crate::light::record_end
));
record_collection!(
    ParticleEmittersChunk,
    crate::ParticleEmitter,
    |b: &[u8]| crate::sized_node::records(
        b,
        crate::ParticleEmitter::TAG,
        crate::particle::FIXED_SIZE
    )
);
record_collection!(
    ParticleEmitters2Chunk,
    crate::ParticleEmitter2,
    |b: &[u8]| crate::sized_node::records(
        b,
        crate::ParticleEmitter2::TAG,
        crate::particle2::FIXED_SIZE
    )
);
record_collection!(PopcornEmittersChunk, crate::PopcornEmitter, |b: &[u8]| {
    records_with_end(b, crate::PopcornEmitter::TAG, crate::popcorn::record_end)
});
record_collection!(RibbonEmittersChunk, crate::RibbonEmitter, |b: &[u8]| {
    crate::sized_node::records(b, crate::RibbonEmitter::TAG, crate::ribbon::FIXED_SIZE)
});
record_collection!(
    TextureAnimationsChunk,
    crate::TextureAnimation,
    |b: &[u8]| sized_records(b, crate::TextureAnimation::TAG, 4, u32::MAX, 0)
);

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
