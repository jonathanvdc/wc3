//! Decoded, unknown, and malformed model chunks.
use crate::Encoder;
use crate::{Tag, Version};

use super::*;
use crate::{Chunk, Cursor, Decodable, Encodable, Error, KnownChunk, RawChunk};

/// A known chunk that could not be decoded. Its original bytes remain intact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MalformedChunk {
    pub(crate) raw: RawChunk,
    pub(crate) error: Error,
}

impl Encodable for MalformedChunk {
    fn encode_to(&self, output: &mut Encoder<'_>) -> Result<(), Error> {
        self.raw.encode_to(output)
    }
}

impl Chunk for MalformedChunk {
    fn tag(&self) -> Tag {
        self.raw.tag
    }
}

impl MalformedChunk {
    pub fn raw(&self) -> &RawChunk {
        &self.raw
    }
    pub fn error(&self) -> &Error {
        &self.error
    }
}

// Keep the known variants, their tags, and their codecs in one place.
macro_rules! model_chunks {
    ($( $variant:ident($chunk:ty), )*) => {
        /// One ordered chunk in a model. Known variants contain complete decoded payloads.
        #[derive(Clone, Debug)]
        pub enum ModelChunk {
            $( $variant($chunk), )*
            Unknown(RawChunk),
            Malformed(MalformedChunk),
        }

        impl Encodable for ModelChunk {
            fn encode_to(&self, output: &mut Encoder<'_>) -> Result<(), Error> {
                let tag = self.tag();
                output.write_bytes(&tag);
                let marker = output.begin_sized();
                match self {
                    $( Self::$variant(value) => value.encode_to(output)?, )*
                    Self::Unknown(raw) => raw.encode_to(output)?,
                    Self::Malformed(malformed) => malformed.encode_to(output)?,
                }
                output.finish_payload(marker, tag)?;
                Ok(())
            }
        }

        impl ModelChunk {
            fn decode_payload(
                tag: Tag,
                payload: &mut Cursor<'_>,
                version: Version,
            ) -> Result<Option<Self>, Error> {
                let mut cursor = *payload;
                let decoded = match tag {
                    $( <$chunk>::TAG => <$chunk>::decode_one(&mut cursor, version).map(Self::$variant), )*
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

impl Chunk for ModelChunk {
    fn tag(&self) -> Tag {
        ModelChunk::tag(self)
    }
}

impl ModelChunk {
    /// Decodes a known chunk, retaining its bytes and error if decoding fails.
    pub fn from_raw(raw: RawChunk, version: Version) -> Self {
        let decoded = Self::decode_payload(raw.tag, &mut Cursor::new(&raw.data), version);
        match decoded {
            Ok(Some(chunk)) => chunk,
            Ok(None) => Self::Unknown(raw),
            Err(error) => Self::Malformed(MalformedChunk { raw, error }),
        }
    }

    /// Decodes directly from a bounded payload, copying bytes only when they
    /// must be retained for an unknown or malformed chunk.
    pub(crate) fn decode_from(tag: Tag, payload: &mut Cursor<'_>, version: Version) -> Self {
        match Self::decode_payload(tag, payload, version) {
            Ok(Some(chunk)) => chunk,
            Ok(None) => Self::Unknown(RawChunk::new(tag, payload.remaining().to_vec())),
            Err(error) => Self::Malformed(MalformedChunk {
                raw: RawChunk::new(tag, payload.remaining().to_vec()),
                error,
            }),
        }
    }
}
