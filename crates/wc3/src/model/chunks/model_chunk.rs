//! Decoded and unknown model chunks.
use crate::model::Encoder;
use crate::model::WriteError;
use crate::model::{ModelVersion, Tag};

use super::*;
use crate::model::{Chunk, Cursor, KnownChunk, RawChunk, ReadError};
use std::marker::PhantomData;

/// An opaque chunk whose tag is not defined by this library.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownChunk<V: ModelVersion> {
    raw: RawChunk,
    version: PhantomData<V>,
}

impl<V: ModelVersion> UnknownChunk<V> {
    /// Returns `None` when the tag belongs to a known chunk.
    pub fn new(raw: RawChunk) -> Option<Self> {
        (!ModelChunk::<V>::is_known_tag(raw.tag)).then_some(Self {
            raw,
            version: PhantomData,
        })
    }

    pub fn raw(&self) -> &RawChunk {
        &self.raw
    }

    /// Edits the opaque payload without changing its checked tag.
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.raw.data
    }
}

// Keep the known variants, their tags, and their codecs in one place.
macro_rules! model_chunks {
    ($( $variant:ident($chunk:ty), )*) => {
        /// One ordered chunk in a model. Known variants contain complete decoded payloads.
        #[derive(Clone, Debug)]
        pub enum ModelChunk<V: ModelVersion> {
            $( $variant(Box<$chunk>), )*
            Unknown(UnknownChunk<V>),
        }

        $(
            impl<V: ModelVersion> From<$chunk> for ModelChunk<V> {
                fn from(chunk: $chunk) -> Self {
                    Self::$variant(Box::new(chunk))
                }
            }

            impl<V: ModelVersion> TryFrom<ModelChunk<V>> for $chunk {
                type Error = ModelChunk<V>;

                fn try_from(chunk: ModelChunk<V>) -> Result<Self, Self::Error> {
                    match chunk {
                        ModelChunk::$variant(value) => Ok(*value),
                        other => Err(other),
                    }
                }
            }

            impl<'a, V: ModelVersion> TryFrom<&'a ModelChunk<V>> for &'a $chunk {
                type Error = ();

                fn try_from(chunk: &'a ModelChunk<V>) -> Result<Self, Self::Error> {
                    match chunk {
                        ModelChunk::$variant(value) => Ok(value.as_ref()),
                        _ => Err(()),
                    }
                }
            }
        )*

        impl<V: ModelVersion> Chunk for ModelChunk<V> {
            fn tag(&self) -> Tag { ModelChunk::tag(self) }
            fn encode_payload_to(&self, output: &mut Encoder<'_>) -> Result<(), WriteError> {
                match self {
                    $( Self::$variant(value) => value.encode_payload_to(output), )*
                    Self::Unknown(unknown) => unknown.raw.encode_payload_to(output),
                }
            }
        }

        impl<V: ModelVersion> ModelChunk<V> {
            fn is_known_tag(tag: Tag) -> bool {
                matches!(tag, $( <$chunk>::TAG )|*)
            }

            fn decode_payload(
                tag: Tag,
                payload: &mut Cursor<'_>,
            ) -> Result<Option<Self>, ReadError> {
                let mut cursor = *payload;
                let decoded = match tag {
                    $( <$chunk>::TAG => <$chunk>::decode_payload(&mut cursor).map(Self::from), )*
                    _ => return Ok(None),
                };
                let chunk = decoded?;
                cursor.finish()?;
                Ok(Some(chunk))
            }

            /// Returns this chunk's tag.
            pub fn tag(&self) -> Tag {
                match self {
                    $( Self::$variant(_) => <$chunk>::TAG, )*
                    Self::Unknown(unknown) => unknown.raw.tag,
                }
            }
        }
    };
}

model_chunks! {
    Version(VersionChunk<V>),
    ModelInfo(ModelInfoChunk),
    Sequences(SequencesChunk),
    GlobalSequences(GlobalSequencesChunk),
    Textures(TexturesChunk),
    Materials(MaterialsChunk<V>),
    Geosets(GeosetsChunk<V>),
    GeosetAnimations(GeosetAnimationsChunk),
    Bones(BonesChunk),
    Helpers(HelpersChunk),
    Attachments(AttachmentsChunk),
    EventObjects(EventObjectsChunk),
    CollisionShapes(CollisionShapesChunk),
    ParticleEmitters(ParticleEmittersChunk),
    ParticleEmitters2(ParticleEmitters2Chunk),
    RibbonEmitters(RibbonEmittersChunk),
    PopcornEmitters(PopcornEmittersChunk),
    Cameras(CamerasChunk<V>),
    Lights(LightsChunk<V>),
    TextureAnimations(TextureAnimationsChunk),
    FaceFx(FaceFxChunk),
    PivotPoints(PivotPointsChunk),
    BindPose(BindPoseChunk),
    Gliders(GlidersChunk),
}

impl<V: ModelVersion> ModelChunk<V> {
    /// Decodes a known chunk or retains an unknown one.
    pub fn from_raw(raw: RawChunk) -> Result<Self, ReadError> {
        let decoded = Self::decode_payload(raw.tag, &mut Cursor::new(&raw.data));
        match decoded {
            Ok(Some(chunk)) => Ok(chunk),
            Ok(None) => Ok(Self::Unknown(UnknownChunk {
                raw,
                version: PhantomData,
            })),
            Err(error) => Err(error),
        }
    }

    /// Decodes directly from a bounded payload, copying bytes only when they
    /// must be retained for an unknown chunk.
    pub(crate) fn decode_from(tag: Tag, payload: &mut Cursor<'_>) -> Result<Self, ReadError> {
        match Self::decode_payload(tag, payload) {
            Ok(Some(chunk)) => Ok(chunk),
            Ok(None) => Ok(Self::Unknown(UnknownChunk {
                raw: RawChunk::new(tag, payload.remaining().to_vec()),
                version: PhantomData,
            })),
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_chunks_convert_in_both_directions() {
        let version = VersionChunk::<crate::model::V800>::new();
        let chunk: ModelChunk<crate::model::V800> = version.clone().into();
        assert!(matches!(chunk, ModelChunk::Version(_)));
        assert_eq!(
            <&VersionChunk<crate::model::V800>>::try_from(&chunk),
            Ok(&version)
        );
        assert_eq!(
            VersionChunk::<crate::model::V800>::try_from(chunk).unwrap(),
            version
        );
    }

    #[test]
    fn failed_extraction_preserves_the_chunk() {
        let chunk = ModelChunk::Unknown(
            UnknownChunk::<crate::model::V800>::new(RawChunk::new(*b"FUTR", vec![1, 2, 3]))
                .unwrap(),
        );
        assert!(<&VersionChunk<crate::model::V800>>::try_from(&chunk).is_err());
        let original = VersionChunk::<crate::model::V800>::try_from(chunk).unwrap_err();
        assert!(matches!(original, ModelChunk::Unknown(raw) if raw.raw().data == [1, 2, 3]));
    }
}
