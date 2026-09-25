//! Decoded, unknown, and malformed model chunks.
use crate::Encoder;
use crate::{Tag, Version};

use super::*;
use crate::{Chunk, Error, KnownChunk, RawChunk, Record};

/// A known chunk that could not be decoded. Its original bytes remain intact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MalformedChunk {
    pub(crate) raw: RawChunk,
    pub(crate) error: Error,
}

impl MalformedChunk {
    pub fn raw(&self) -> &RawChunk {
        &self.raw
    }
    pub fn error(&self) -> &Error {
        &self.error
    }
}

/// One ordered chunk in a model. Known variants contain complete decoded payloads.
#[derive(Clone, Debug)]
pub enum ModelChunk {
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
    Unknown(RawChunk),
    Malformed(MalformedChunk),
}

impl PartialEq for ModelChunk {
    fn eq(&self, other: &Self) -> bool {
        self.to_raw() == other.to_raw()
    }
}
impl Eq for ModelChunk {}

impl Chunk for ModelChunk {
    fn tag(&self) -> Tag {
        ModelChunk::tag(self)
    }
    fn encode_chunk(&self) -> Result<RawChunk, Error> {
        self.to_raw()
    }
}

impl ModelChunk {
    pub(crate) fn encode_to(&self, output: &mut Encoder<'_>) -> Result<(), Error> {
        let tag = self.tag();
        output.write_bytes(&tag);
        let marker = output.begin_sized();
        match self {
            Self::Version(value) => value.encode_to(output)?,
            Self::ModelInfo(value) => value.encode_to(output)?,
            Self::Sequences(value) => value.encode_to(output)?,
            Self::GlobalSequences(value) => value.encode_to(output)?,
            Self::Textures(value) => value.encode_to(output)?,
            Self::Materials(value) => value.encode_to(output)?,
            Self::Geosets(value) => value.encode_to(output)?,
            Self::GeosetAnimations(value) => value.encode_to(output)?,
            Self::Bones(value) => value.encode_to(output)?,
            Self::Helpers(value) => value.encode_to(output)?,
            Self::Attachments(value) => value.encode_to(output)?,
            Self::EventObjects(value) => value.encode_to(output)?,
            Self::CollisionShapes(value) => value.encode_to(output)?,
            Self::ParticleEmitters(value) => value.encode_to(output)?,
            Self::ParticleEmitters2(value) => value.encode_to(output)?,
            Self::RibbonEmitters(value) => value.encode_to(output)?,
            Self::PopcornEmitters(value) => value.encode_to(output)?,
            Self::Cameras(value) => value.encode_to(output)?,
            Self::Lights(value) => value.encode_to(output)?,
            Self::TextureAnimations(value) => value.encode_to(output)?,
            Self::FaceFx(value) => value.encode_to(output)?,
            Self::PivotPoints(value) => value.encode_to(output)?,
            Self::BindPose(value) => value.encode_to(output)?,
            Self::Unknown(raw) => output.write_bytes(&raw.data),
            Self::Malformed(malformed) => output.write_bytes(&malformed.raw.data),
        }
        output.finish_payload(marker, tag)?;
        Ok(())
    }

    /// Decodes a known chunk, retaining its bytes and error if decoding fails.
    pub fn from_raw(raw: RawChunk, version: Version) -> Self {
        let decoded = match raw.tag {
            VersionChunk::TAG => VersionChunk::decode_chunk(&raw, version).map(Self::Version),
            ModelInfoChunk::TAG => ModelInfoChunk::decode_chunk(&raw, version).map(Self::ModelInfo),
            SequencesChunk::TAG => SequencesChunk::decode_chunk(&raw, version).map(Self::Sequences),
            GlobalSequencesChunk::TAG => {
                GlobalSequencesChunk::decode_chunk(&raw, version).map(Self::GlobalSequences)
            }
            TexturesChunk::TAG => TexturesChunk::decode_chunk(&raw, version).map(Self::Textures),
            MaterialsChunk::TAG => MaterialsChunk::decode_chunk(&raw, version).map(Self::Materials),
            GeosetsChunk::TAG => GeosetsChunk::decode_chunk(&raw, version).map(Self::Geosets),
            GeosetAnimationsChunk::TAG => {
                GeosetAnimationsChunk::decode_chunk(&raw, version).map(Self::GeosetAnimations)
            }
            BonesChunk::TAG => BonesChunk::decode_chunk(&raw, version).map(Self::Bones),
            HelpersChunk::TAG => HelpersChunk::decode_chunk(&raw, version).map(Self::Helpers),
            AttachmentsChunk::TAG => {
                AttachmentsChunk::decode_chunk(&raw, version).map(Self::Attachments)
            }
            EventObjectsChunk::TAG => {
                EventObjectsChunk::decode_chunk(&raw, version).map(Self::EventObjects)
            }
            CollisionShapesChunk::TAG => {
                CollisionShapesChunk::decode_chunk(&raw, version).map(Self::CollisionShapes)
            }
            ParticleEmittersChunk::TAG => {
                ParticleEmittersChunk::decode_chunk(&raw, version).map(Self::ParticleEmitters)
            }
            ParticleEmitters2Chunk::TAG => {
                ParticleEmitters2Chunk::decode_chunk(&raw, version).map(Self::ParticleEmitters2)
            }
            RibbonEmittersChunk::TAG => {
                RibbonEmittersChunk::decode_chunk(&raw, version).map(Self::RibbonEmitters)
            }
            PopcornEmittersChunk::TAG => {
                PopcornEmittersChunk::decode_chunk(&raw, version).map(Self::PopcornEmitters)
            }
            CamerasChunk::TAG => CamerasChunk::decode_chunk(&raw, version).map(Self::Cameras),
            LightsChunk::TAG => LightsChunk::decode_chunk(&raw, version).map(Self::Lights),
            TextureAnimationsChunk::TAG => {
                TextureAnimationsChunk::decode_chunk(&raw, version).map(Self::TextureAnimations)
            }
            FaceFxChunk::TAG => FaceFxChunk::decode_chunk(&raw, version).map(Self::FaceFx),
            PivotPointsChunk::TAG => {
                PivotPointsChunk::decode_chunk(&raw, version).map(Self::PivotPoints)
            }
            BindPose::TAG => BindPose::decode_chunk(&raw, version).map(Self::BindPose),
            _ => return Self::Unknown(raw),
        };
        decoded.unwrap_or_else(|error| Self::Malformed(MalformedChunk { raw, error }))
    }

    /// Returns this chunk's tag.
    pub fn tag(&self) -> Tag {
        match self {
            Self::Version(_) => VersionChunk::TAG,
            Self::ModelInfo(_) => ModelInfoChunk::TAG,
            Self::Sequences(_) => SequencesChunk::TAG,
            Self::GlobalSequences(_) => GlobalSequencesChunk::TAG,
            Self::Textures(_) => TexturesChunk::TAG,
            Self::Materials(_) => MaterialsChunk::TAG,
            Self::Geosets(_) => GeosetsChunk::TAG,
            Self::GeosetAnimations(_) => GeosetAnimationsChunk::TAG,
            Self::Bones(_) => BonesChunk::TAG,
            Self::Helpers(_) => HelpersChunk::TAG,
            Self::Attachments(_) => AttachmentsChunk::TAG,
            Self::EventObjects(_) => EventObjectsChunk::TAG,
            Self::CollisionShapes(_) => CollisionShapesChunk::TAG,
            Self::ParticleEmitters(_) => ParticleEmittersChunk::TAG,
            Self::ParticleEmitters2(_) => ParticleEmitters2Chunk::TAG,
            Self::RibbonEmitters(_) => RibbonEmittersChunk::TAG,
            Self::PopcornEmitters(_) => PopcornEmittersChunk::TAG,
            Self::Cameras(_) => CamerasChunk::TAG,
            Self::Lights(_) => LightsChunk::TAG,
            Self::TextureAnimations(_) => TextureAnimationsChunk::TAG,
            Self::FaceFx(_) => FaceFxChunk::TAG,
            Self::PivotPoints(_) => PivotPointsChunk::TAG,
            Self::BindPose(_) => BindPose::TAG,
            Self::Unknown(raw) => raw.tag,
            Self::Malformed(malformed) => malformed.raw.tag,
        }
    }

    /// Encodes a known chunk or returns the stored raw bytes.
    pub fn to_raw(&self) -> Result<RawChunk, Error> {
        match self {
            Self::Version(value) => value.encode_chunk(),
            Self::ModelInfo(value) => value.encode_chunk(),
            Self::Sequences(value) => value.encode_chunk(),
            Self::GlobalSequences(value) => value.encode_chunk(),
            Self::Textures(value) => value.encode_chunk(),
            Self::Materials(value) => value.encode_chunk(),
            Self::Geosets(value) => value.encode_chunk(),
            Self::GeosetAnimations(value) => value.encode_chunk(),
            Self::Bones(value) => value.encode_chunk(),
            Self::Helpers(value) => value.encode_chunk(),
            Self::Attachments(value) => value.encode_chunk(),
            Self::EventObjects(value) => value.encode_chunk(),
            Self::CollisionShapes(value) => value.encode_chunk(),
            Self::ParticleEmitters(value) => value.encode_chunk(),
            Self::ParticleEmitters2(value) => value.encode_chunk(),
            Self::RibbonEmitters(value) => value.encode_chunk(),
            Self::PopcornEmitters(value) => value.encode_chunk(),
            Self::Cameras(value) => value.encode_chunk(),
            Self::Lights(value) => value.encode_chunk(),
            Self::TextureAnimations(value) => value.encode_chunk(),
            Self::FaceFx(value) => value.encode_chunk(),
            Self::PivotPoints(value) => value.encode_chunk(),
            Self::BindPose(value) => value.encode_chunk(),
            Self::Unknown(raw) => Ok(raw.clone()),
            Self::Malformed(malformed) => Ok(malformed.raw.clone()),
        }
    }
}
