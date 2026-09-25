//! Complete payloads for chunks containing a sequence of records.

use crate::{Chunk, ChunkRecord, Error, Model, Record};

macro_rules! record_collection {
    ($name:ident, $item:ty, $accessor:ident) => {
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

        impl Record for $name {
            fn decode(bytes: &[u8], version: u32) -> Result<Self, Error> {
                let mut model = Model::new(version);
                model.push(Chunk::new(Self::TAG, bytes.to_vec()));
                Ok(Self::new(model.$accessor()?))
            }

            fn encode(&self) -> Result<Vec<u8>, Error> {
                let mut bytes = Vec::new();
                for record in &self.records {
                    bytes.extend_from_slice(&record.encode()?);
                    if bytes.len() > u32::MAX as usize {
                        return Err(Error::ChunkTooLarge {
                            tag: Self::TAG,
                            size: bytes.len(),
                        });
                    }
                }
                Ok(bytes)
            }
        }

        impl ChunkRecord for $name {
            const TAG: [u8; 4] = <$item>::TAG;
        }
    };
}

record_collection!(SequencesChunk, crate::Sequence, sequences);
record_collection!(TexturesChunk, crate::Texture, textures);
record_collection!(GeosetsChunk, crate::Geoset, geosets);
record_collection!(
    GeosetAnimationsChunk,
    crate::GeosetAnimation,
    geoset_animations
);
record_collection!(MaterialsChunk, crate::Material, materials);
record_collection!(BonesChunk, crate::Bone, bones);
record_collection!(HelpersChunk, crate::Node, helpers);
record_collection!(AttachmentsChunk, crate::Attachment, attachments);
record_collection!(CamerasChunk, crate::Camera, cameras);
record_collection!(
    CollisionShapesChunk,
    crate::CollisionShape,
    collision_shapes
);
record_collection!(EventObjectsChunk, crate::EventObject, event_objects);
record_collection!(FaceFxChunk, crate::FaceFx, face_fx);
record_collection!(LightsChunk, crate::Light, lights);
record_collection!(
    ParticleEmittersChunk,
    crate::ParticleEmitter,
    particle_emitters
);
record_collection!(
    ParticleEmitters2Chunk,
    crate::ParticleEmitter2,
    particle_emitters2
);
record_collection!(
    PopcornEmittersChunk,
    crate::PopcornEmitter,
    popcorn_emitters
);
record_collection!(RibbonEmittersChunk, crate::RibbonEmitter, ribbon_emitters);
record_collection!(
    TextureAnimationsChunk,
    crate::TextureAnimation,
    texture_animations
);

/// A complete `MODL` chunk, including bytes after the standard record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelInfoChunk {
    pub info: crate::ModelInfo,
    pub extension: Vec<u8>,
}

impl ModelInfoChunk {
    pub fn new(info: crate::ModelInfo, extension: Vec<u8>) -> Self {
        Self { info, extension }
    }
}

impl Record for ModelInfoChunk {
    fn decode(bytes: &[u8], version: u32) -> Result<Self, Error> {
        let info = crate::ModelInfo::parse(bytes)?;
        let _ = version;
        Ok(Self::new(info, bytes[372..].to_vec()))
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let size = 372usize
            .checked_add(self.extension.len())
            .ok_or(Error::ChunkTooLarge {
                tag: Self::TAG,
                size: usize::MAX,
            })?;
        if size > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: Self::TAG,
                size,
            });
        }
        let mut bytes = Vec::with_capacity(size);
        bytes.extend_from_slice(self.info.as_bytes());
        bytes.extend_from_slice(&self.extension);
        Ok(bytes)
    }
}

impl ChunkRecord for ModelInfoChunk {
    const TAG: [u8; 4] = crate::ModelInfo::TAG;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_width_collection_uses_the_whole_chunk() {
        let records = vec![
            crate::Sequence::new("Stand", [0, 100]).unwrap(),
            crate::Sequence::new("Walk", [101, 200]).unwrap(),
        ];
        let original = SequencesChunk::new(records);
        let chunk = original.encode_chunk().unwrap();
        assert_eq!(SequencesChunk::decode_chunk(&chunk).unwrap(), original);
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

    #[test]
    fn model_info_chunk_keeps_extension_bytes() {
        let original = ModelInfoChunk::new(crate::ModelInfo::default(), vec![1, 2, 3]);
        let chunk = original.encode_chunk().unwrap();
        assert_eq!(ModelInfoChunk::decode_chunk(&chunk).unwrap(), original);
    }
}
