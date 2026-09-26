//! Decoded, unknown, and malformed model chunks.
use crate::EncodeError;
use crate::Encoder;
use crate::{Tag, Version};

use super::*;
use crate::{Chunk, Cursor, DecodeError, KnownChunk, RawChunk};

/// A known chunk that could not be decoded. Its original bytes remain intact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MalformedChunk {
    pub(crate) raw: RawChunk,
    pub(crate) error: DecodeError,
}

impl Chunk for MalformedChunk {
    fn tag(&self) -> Tag {
        self.raw.tag
    }

    fn encode_payload_to(&self, output: &mut Encoder<'_>) -> Result<(), EncodeError> {
        self.raw.encode_payload_to(output)
    }
}

impl MalformedChunk {
    pub fn raw(&self) -> &RawChunk {
        &self.raw
    }
    pub fn error(&self) -> &DecodeError {
        &self.error
    }
}

// Keep the known variants, their tags, and their codecs in one place.
macro_rules! model_chunks {
    ($( $variant:ident($chunk:ty), )*) => {
        /// One ordered chunk in a model. Known variants contain complete decoded payloads.
        #[derive(Clone, Debug)]
        pub enum ModelChunk {
            $( $variant(Box<$chunk>), )*
            Unknown(RawChunk),
            Malformed(Box<MalformedChunk>),
        }

        $(
            impl From<$chunk> for ModelChunk {
                fn from(chunk: $chunk) -> Self {
                    Self::$variant(Box::new(chunk))
                }
            }

            impl TryFrom<ModelChunk> for $chunk {
                type Error = ModelChunk;

                fn try_from(chunk: ModelChunk) -> Result<Self, Self::Error> {
                    match chunk {
                        ModelChunk::$variant(value) => Ok(*value),
                        other => Err(other),
                    }
                }
            }

            impl<'a> TryFrom<&'a ModelChunk> for &'a $chunk {
                type Error = ();

                fn try_from(chunk: &'a ModelChunk) -> Result<Self, Self::Error> {
                    match chunk {
                        ModelChunk::$variant(value) => Ok(value.as_ref()),
                        _ => Err(()),
                    }
                }
            }
        )*

        impl Chunk for ModelChunk {
            fn tag(&self) -> Tag { ModelChunk::tag(self) }
            fn encode_payload_to(&self, output: &mut Encoder<'_>) -> Result<(), EncodeError> {
                match self {
                    $( Self::$variant(value) => value.encode_payload_to(output), )*
                    Self::Unknown(raw) => raw.encode_payload_to(output),
                    Self::Malformed(malformed) => malformed.encode_payload_to(output),
                }
            }
        }

        impl ModelChunk {
            fn decode_payload(
                tag: Tag,
                payload: &mut Cursor<'_>,
                version: Version,
            ) -> Result<Option<Self>, DecodeError> {
                let mut cursor = *payload;
                let decoded = match tag {
                    $( <$chunk>::TAG => <$chunk>::decode_payload(&mut cursor, version).map(Self::from), )*
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
                    Self::Unknown(raw) => raw.tag,
                    Self::Malformed(malformed) => malformed.raw.tag,
                }
            }
        }
    };
}

model_chunks! {
    Version(VersionChunk),
    ModelInfo(ModelInfoChunk),
    Sequences(SequencesChunk),
    GlobalSequences(GlobalSequencesChunk),
    Textures(TexturesChunk),
    Materials(MaterialsChunk),
    Geosets(GeosetsChunk),
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
    Cameras(CamerasChunk),
    Lights(LightsChunk),
    TextureAnimations(TextureAnimationsChunk),
    FaceFx(FaceFxChunk),
    PivotPoints(PivotPointsChunk),
    BindPose(BindPose),
}

impl ModelChunk {
    /// Decodes a known chunk, retaining its bytes and error if decoding fails.
    pub fn from_raw(raw: RawChunk, version: Version) -> Self {
        let decoded = Self::decode_payload(raw.tag, &mut Cursor::new(&raw.data), version);
        match decoded {
            Ok(Some(chunk)) => chunk,
            Ok(None) => Self::Unknown(raw),
            Err(error) => Self::Malformed(Box::new(MalformedChunk { raw, error })),
        }
    }

    /// Decodes directly from a bounded payload, copying bytes only when they
    /// must be retained for an unknown or malformed chunk.
    pub(crate) fn decode_from(tag: Tag, payload: &mut Cursor<'_>, version: Version) -> Self {
        match Self::decode_payload(tag, payload, version) {
            Ok(Some(chunk)) => chunk,
            Ok(None) => Self::Unknown(RawChunk::new(tag, payload.remaining().to_vec())),
            Err(error) => Self::Malformed(Box::new(MalformedChunk {
                raw: RawChunk::new(tag, payload.remaining().to_vec()),
                error,
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_chunks_convert_in_both_directions() {
        let version = VersionChunk::new(800);
        let chunk: ModelChunk = version.clone().into();
        assert!(matches!(chunk, ModelChunk::Version(_)));
        assert_eq!(<&VersionChunk>::try_from(&chunk), Ok(&version));
        assert_eq!(VersionChunk::try_from(chunk).unwrap(), version);
    }

    #[test]
    fn failed_extraction_preserves_the_chunk() {
        let chunk = ModelChunk::Unknown(RawChunk::new(*b"FUTR", vec![1, 2, 3]));
        assert!(<&VersionChunk>::try_from(&chunk).is_err());
        let original = VersionChunk::try_from(chunk).unwrap_err();
        assert!(matches!(original, ModelChunk::Unknown(raw) if raw.data == [1, 2, 3]));
    }
}
