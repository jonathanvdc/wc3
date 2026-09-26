//! Keyframe tracks for node translation, rotation, and scaling.
use crate::EncodeError;
use crate::Encoder;
use crate::Tag;

use crate::Cursor;
use crate::DecodeError;
use crate::{Decodable, Encodable};

/// Binary value type stored in a keyframe track.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackValueKind {
    Float,
    Integer,
}

macro_rules! track_tags {
    ($($name:ident => ($tag:literal, $components:literal)),+ $(,)?) => {
        /// Known animation track identifiers.
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
        pub enum TrackTag { $($name),+ }

        impl TrackTag {
            /// Returns the four bytes stored in an MDX track header.
            pub const fn bytes(self) -> Tag {
                match self { $(Self::$name => *$tag),+ }
            }

            /// Parses a known track identifier.
            pub const fn from_bytes(tag: Tag) -> Option<Self> {
                match &tag { $($tag => Some(Self::$name),)+ _ => None }
            }

            /// Number of scalar components in each keyframe value.
            pub const fn component_count(self) -> usize {
                match self { $(Self::$name => $components),+ }
            }
        }
    };
}

track_tags! {
    NodeTranslation => (b"KGTR", 3),
    NodeScaling => (b"KGSC", 3),
    CameraTranslation => (b"KCTR", 3),
    CameraTargetTranslation => (b"KTTR", 3),
    PopcornColor => (b"KPPC", 3),
    TextureTranslation => (b"KTAT", 3),
    TextureScaling => (b"KTAS", 3),
    GeosetColor => (b"KGAC", 3),
    LightColor => (b"KLAC", 3),
    LightAmbientColor => (b"KLBC", 3),
    LayerFresnelColor => (b"KFC3", 3),
    RibbonColor => (b"KRCO", 3),
    NodeRotation => (b"KGRT", 4),
    TextureRotation => (b"KTAR", 4),
    CameraRoll => (b"KCRL", 1),
    AttachmentVisibility => (b"KATV", 1),
    PopcornAlpha => (b"KPPA", 1),
    PopcornEmissionRate => (b"KPPE", 1),
    PopcornLifespan => (b"KPPL", 1),
    PopcornSpeed => (b"KPPS", 1),
    PopcornVisibility => (b"KPPV", 1),
    ParticleVisibility => (b"KPEV", 1),
    ParticleEmissionRate => (b"KPEE", 1),
    ParticleGravity => (b"KPEG", 1),
    ParticleLongitude => (b"KPLN", 1),
    ParticleLatitude => (b"KPLT", 1),
    ParticleLifespan => (b"KPEL", 1),
    ParticleSpeed => (b"KPES", 1),
    Particle2Visibility => (b"KP2V", 1),
    Particle2EmissionRate => (b"KP2E", 1),
    Particle2Width => (b"KP2W", 1),
    Particle2Length => (b"KP2N", 1),
    Particle2Speed => (b"KP2S", 1),
    Particle2Latitude => (b"KP2L", 1),
    Particle2Gravity => (b"KP2G", 1),
    Particle2Variation => (b"KP2R", 1),
    RibbonVisibility => (b"KRVS", 1),
    RibbonHeightAbove => (b"KRHA", 1),
    RibbonHeightBelow => (b"KRHB", 1),
    RibbonAlpha => (b"KRAL", 1),
    RibbonTextureSlot => (b"KRTX", 1),
    GeosetAlpha => (b"KGAO", 1),
    LightVisibility => (b"KLAV", 1),
    LightIntensity => (b"KLAI", 1),
    LightAmbientIntensity => (b"KLBI", 1),
    LightAttenuationStart => (b"KLAS", 1),
    LightAttenuationEnd => (b"KLAE", 1),
    LayerAlpha => (b"KMTA", 1),
    LayerTextureId => (b"KMTF", 1),
    LayerEmissiveGain => (b"KMTE", 1),
    LayerFresnelOpacity => (b"KFCA", 1),
    LayerFresnelTeamColor => (b"KFTC", 1),
}

/// One decoded keyframe. Tangents are present for Hermite and Bezier tracks.
#[derive(Clone, Debug, PartialEq)]
pub struct Keyframe {
    /// Frame time in milliseconds.
    pub frame: u32,
    /// One, three, or four component values, depending on the track tag.
    pub value: Vec<f32>,
    /// Incoming tangent for interpolation modes 2 and 3.
    pub in_tangent: Option<Vec<f32>>,
    /// Outgoing tangent for interpolation modes 2 and 3.
    pub out_tangent: Option<Vec<f32>>,
}

impl Keyframe {
    /// Interprets a scalar integer track value, such as a ribbon texture slot.
    /// Integer tracks store their value in the same four bytes as a float track.
    pub fn integer_value(&self) -> Option<u32> {
        (self.value.len() == 1).then(|| self.value[0].to_bits())
    }

    /// Replaces the scalar value with an integer track value, preserving its bits.
    pub fn set_integer_value(&mut self, value: u32) -> bool {
        if self.value.len() != 1 {
            return false;
        }
        self.value[0] = f32::from_bits(value);
        true
    }
}

/// A decoded animation track.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationTrack {
    /// Track identifier, such as `KGTR` for node translation or `KCTR` for a camera.
    pub tag: TrackTag,
    /// 0 = none, 1 = linear, 2 = Hermite, 3 = Bezier.
    pub interpolation: u32,
    /// Global sequence index, or `u32::MAX` when absent.
    pub global_sequence_id: u32,
    /// Keyframes in source order.
    pub keyframes: Vec<Keyframe>,
}

impl AnimationTrack {
    /// Returns the number of scalar components in each value.
    pub fn component_count(&self) -> Option<usize> {
        Some(self.tag.component_count())
    }

    /// Returns the binary value type.
    pub fn value_kind(&self) -> Option<TrackValueKind> {
        Some(
            if matches!(
                self.tag,
                TrackTag::LayerTextureId | TrackTag::RibbonTextureSlot
            ) {
                TrackValueKind::Integer
            } else {
                TrackValueKind::Float
            },
        )
    }
}

impl Decodable for AnimationTrack {
    fn decode_one(cursor: &mut Cursor<'_>, _version: u32) -> Result<Self, DecodeError> {
        let mut next = *cursor;
        let offset = next.absolute_position();
        let tag: Tag = next
            .read_exact(4)
            .map_err(|_| DecodeError::MalformedRecord {
                tag: *b"KGTR",
                offset,
            })?
            .try_into()
            .expect("four-byte tag");
        let track_tag =
            TrackTag::from_bytes(tag).ok_or(DecodeError::MalformedRecord { tag, offset })?;
        let components = track_tag.component_count();
        let malformed = || DecodeError::MalformedRecord { tag, offset };
        let count = next.read::<u32>().map_err(|_| malformed())? as usize;
        let interpolation = next.read::<u32>().map_err(|_| malformed())?;
        if interpolation > 3 {
            return Err(malformed());
        }
        let global_sequence_id = next.read::<u32>().map_err(|_| malformed())?;
        let vector_count = if interpolation >= 2 { 3 } else { 1 };
        let key_size = components
            .checked_mul(vector_count)
            .and_then(|n| n.checked_mul(4))
            .and_then(|n| n.checked_add(4))
            .ok_or_else(malformed)?;
        let body_size = count.checked_mul(key_size).ok_or_else(malformed)?;
        let mut body = next.slice(body_size).map_err(|_| malformed())?;
        let mut keyframes = Vec::with_capacity(count);
        for _ in 0..count {
            let frame = body.read::<u32>().map_err(|_| malformed())?;
            let read_vector = |body: &mut Cursor<'_>| -> Result<Vec<f32>, DecodeError> {
                (0..components)
                    .map(|_| body.read::<f32>().map_err(|_| malformed()))
                    .collect()
            };
            let value = read_vector(&mut body)?;
            let in_tangent = if interpolation >= 2 {
                Some(read_vector(&mut body)?)
            } else {
                None
            };
            let out_tangent = if interpolation >= 2 {
                Some(read_vector(&mut body)?)
            } else {
                None
            };
            keyframes.push(Keyframe {
                frame,
                value,
                in_tangent,
                out_tangent,
            });
        }
        *cursor = next;
        Ok(Self {
            tag: track_tag,
            interpolation,
            global_sequence_id,
            keyframes,
        })
    }
}

impl Encodable for AnimationTrack {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let components = self.tag.component_count();
        if self.interpolation > 3 || self.keyframes.len() > u32::MAX as usize {
            return Err(EncodeError::MalformedRecord {
                tag: self.tag.bytes(),
                offset: 0,
            });
        }
        let tangents = self.interpolation >= 2;
        bytes.write_bytes(&self.tag.bytes());
        bytes.write(self.keyframes.len() as u32);
        bytes.write(self.interpolation);
        bytes.write(self.global_sequence_id);
        for (index, key) in self.keyframes.iter().enumerate() {
            if key.value.len() != components
                || key.in_tangent.is_some() != tangents
                || key.out_tangent.is_some() != tangents
                || key
                    .in_tangent
                    .as_ref()
                    .is_some_and(|v| v.len() != components)
                || key
                    .out_tangent
                    .as_ref()
                    .is_some_and(|v| v.len() != components)
            {
                return Err(EncodeError::MalformedRecord {
                    tag: self.tag.bytes(),
                    offset: index,
                });
            }
            bytes.write(key.frame);
            for vector in [
                Some(&key.value),
                key.in_tangent.as_ref(),
                key.out_tangent.as_ref(),
            ]
            .into_iter()
            .flatten()
            {
                bytes.write(vector.as_slice());
            }
        }
        Ok(())
    }
}
