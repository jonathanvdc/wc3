//! Typed keyframe tracks.
use crate::{Color, Cursor, DecodeError, Encoder, Readable, Tag, ValueError, Vec3, Vec4, Writable};
use std::marker::PhantomData;

/// Binary value type stored in a keyframe track.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackValueKind {
    Float,
    Integer,
}

mod sealed {
    pub trait Sealed {}
}

/// A known kind of MDX animation track.
pub trait TrackKind: sealed::Sealed {
    type Value: TrackValue;
    const TAG: Tag;
    const TAG_KIND: TrackTag;
}

/// A scalar or vector value supported by MDX keyframe tracks.
pub trait TrackValue: Readable + Writable + Copy + PartialEq + std::fmt::Debug {
    const COMPONENTS: usize;
    const KIND: TrackValueKind;
}

impl TrackValue for f32 {
    const COMPONENTS: usize = 1;
    const KIND: TrackValueKind = TrackValueKind::Float;
}
impl TrackValue for u32 {
    const COMPONENTS: usize = 1;
    const KIND: TrackValueKind = TrackValueKind::Integer;
}
impl TrackValue for Vec3 {
    const COMPONENTS: usize = 3;
    const KIND: TrackValueKind = TrackValueKind::Float;
}
impl TrackValue for Vec4 {
    const COMPONENTS: usize = 4;
    const KIND: TrackValueKind = TrackValueKind::Float;
}

macro_rules! track_kinds {
    ($($name:ident => ($tag:literal, $value:ty)),+ $(,)?) => {
        /// A known MDX animation track identifier.
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
        pub enum TrackTag { $($name),+ }
        impl TrackTag {
            /// Returns the four bytes stored in an MDX track header.
            pub const fn bytes(self) -> Tag {
                match self { $(Self::$name => *$tag),+ }
            }
            /// Parses a recognized track identifier.
            pub const fn from_bytes(tag: Tag) -> Option<Self> {
                match &tag { $($tag => Some(Self::$name),)+ _ => None }
            }
            /// Number of scalar components in each keyframe value.
            pub const fn component_count(self) -> usize {
                match self { $(Self::$name => <$value as TrackValue>::COMPONENTS),+ }
            }
            /// Binary value type stored by this track kind.
            pub const fn value_kind(self) -> TrackValueKind {
                match self { $(Self::$name => <$value as TrackValue>::KIND),+ }
            }
        }
        $(
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub struct $name;
            impl sealed::Sealed for $name {}
            impl TrackKind for $name {
                type Value = $value;
                const TAG: Tag = *$tag;
                const TAG_KIND: TrackTag = TrackTag::$name;
            }
        )+
    };
}

track_kinds! {
    NodeTranslation => (b"KGTR", Vec3),
    NodeScaling => (b"KGSC", Vec3),
    CameraTranslation => (b"KCTR", Vec3),
    CameraTargetTranslation => (b"KTTR", Vec3),
    PopcornColor => (b"KPPC", Color),
    TextureTranslation => (b"KTAT", Vec3),
    TextureScaling => (b"KTAS", Vec3),
    GeosetColor => (b"KGAC", Color),
    LightColor => (b"KLAC", Color),
    LightAmbientColor => (b"KLBC", Color),
    LayerFresnelColor => (b"KFC3", Color),
    RibbonColor => (b"KRCO", Color),
    NodeRotation => (b"KGRT", Vec4),
    TextureRotation => (b"KTAR", Vec4),
    CameraRotation => (b"KCRL", f32),
    AttachmentVisibility => (b"KATV", f32),
    PopcornAlpha => (b"KPPA", f32),
    PopcornEmissionRate => (b"KPPE", f32),
    PopcornLifespan => (b"KPPL", f32),
    PopcornSpeed => (b"KPPS", f32),
    PopcornVisibility => (b"KPPV", f32),
    ParticleVisibility => (b"KPEV", f32),
    ParticleEmissionRate => (b"KPEE", f32),
    ParticleGravity => (b"KPEG", f32),
    ParticleLongitude => (b"KPLN", f32),
    ParticleLatitude => (b"KPLT", f32),
    ParticleLifespan => (b"KPEL", f32),
    ParticleSpeed => (b"KPES", f32),
    Particle2Visibility => (b"KP2V", f32),
    Particle2EmissionRate => (b"KP2E", f32),
    Particle2Width => (b"KP2W", f32),
    Particle2Length => (b"KP2N", f32),
    Particle2Speed => (b"KP2S", f32),
    Particle2Latitude => (b"KP2L", f32),
    Particle2Gravity => (b"KP2G", f32),
    Particle2Variation => (b"KP2R", f32),
    RibbonVisibility => (b"KRVS", f32),
    RibbonHeightAbove => (b"KRHA", f32),
    RibbonHeightBelow => (b"KRHB", f32),
    RibbonAlpha => (b"KRAL", f32),
    RibbonTextureSlot => (b"KRTX", u32),
    GeosetAlpha => (b"KGAO", f32),
    LightVisibility => (b"KLAV", f32),
    LightIntensity => (b"KLAI", f32),
    LightAmbientIntensity => (b"KLBI", f32),
    LightAttenuationStart => (b"KLAS", f32),
    LightAttenuationEnd => (b"KLAE", f32),
    LayerAlpha => (b"KMTA", f32),
    LayerTextureId => (b"KMTF", u32),
    LayerEmissiveGain => (b"KMTE", f32),
    LayerFresnelOpacity => (b"KFCA", f32),
    LayerFresnelTeamColor => (b"KFTC", f32),
}

/// An interpolation mode shared by all keys in a track.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Interpolation {
    Step,
    Linear,
    Hermite,
    Bezier,
}

/// A keyframe without interpolation tangents.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueKeyframe<T> {
    /// Frame time in milliseconds.
    pub frame: u32,
    /// The value at this frame.
    pub value: T,
}

/// A keyframe with both interpolation tangents.
#[derive(Clone, Debug, PartialEq)]
pub struct TangentKeyframe<T> {
    /// Frame time in milliseconds.
    pub frame: u32,
    /// The value at this frame.
    pub value: T,
    /// Incoming tangent.
    pub in_tangent: T,
    /// Outgoing tangent.
    pub out_tangent: T,
}

impl<T: TrackValue> Readable for ValueKeyframe<T> {
    fn read_from(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            frame: cursor.read()?,
            value: cursor.read()?,
        })
    }
}
impl<T: TrackValue> Writable for &ValueKeyframe<T> {
    fn write_to(self, encoder: &mut Encoder<'_>) {
        encoder.write(self.frame);
        encoder.write(self.value);
    }
}
impl<T: TrackValue> Readable for TangentKeyframe<T> {
    fn read_from(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            frame: cursor.read()?,
            value: cursor.read()?,
            in_tangent: cursor.read()?,
            out_tangent: cursor.read()?,
        })
    }
}
impl<T: TrackValue> Writable for &TangentKeyframe<T> {
    fn write_to(self, encoder: &mut Encoder<'_>) {
        encoder.write(self.frame);
        encoder.write(self.value);
        encoder.write(self.in_tangent);
        encoder.write(self.out_tangent);
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Keyframes<T> {
    Step(Vec<ValueKeyframe<T>>),
    Linear(Vec<ValueKeyframe<T>>),
    Hermite(Vec<TangentKeyframe<T>>),
    Bezier(Vec<TangentKeyframe<T>>),
}

/// An animation track whose tag, value type, and tangent shape are linked.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationTrack<K: TrackKind> {
    global_sequence_id: Option<u32>,
    keyframes: Keyframes<K::Value>,
    kind: PhantomData<K>,
}

impl<K: TrackKind> AnimationTrack<K> {
    /// Creates a track with no interpolation or tangents.
    pub fn step(
        keys: Vec<ValueKeyframe<K::Value>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Step(keys), global_sequence_id)
    }
    /// Creates a track with linear interpolation and no tangents.
    pub fn linear(
        keys: Vec<ValueKeyframe<K::Value>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Linear(keys), global_sequence_id)
    }
    /// Creates a Hermite track whose keys each have two tangents.
    pub fn hermite(
        keys: Vec<TangentKeyframe<K::Value>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Hermite(keys), global_sequence_id)
    }
    /// Creates a Bezier track whose keys each have two tangents.
    pub fn bezier(
        keys: Vec<TangentKeyframe<K::Value>>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        Self::new(Keyframes::Bezier(keys), global_sequence_id)
    }
    fn new(
        keyframes: Keyframes<K::Value>,
        global_sequence_id: Option<u32>,
    ) -> Result<Self, ValueError> {
        if keyframes.len() > u32::MAX as usize {
            return Err(ValueError::CountTooLarge {
                tag: K::TAG,
                count: keyframes.len(),
            });
        }
        if global_sequence_id == Some(u32::MAX) {
            return Err(ValueError::InvalidGlobalSequenceId { id: u32::MAX });
        }
        Ok(Self {
            global_sequence_id,
            keyframes,
            kind: PhantomData,
        })
    }
    /// Returns the optional global sequence index.
    pub fn global_sequence_id(&self) -> Option<u32> {
        self.global_sequence_id
    }
    /// Returns the interpolation mode shared by all keys.
    pub fn interpolation(&self) -> Interpolation {
        match self.keyframes {
            Keyframes::Step(_) => Interpolation::Step,
            Keyframes::Linear(_) => Interpolation::Linear,
            Keyframes::Hermite(_) => Interpolation::Hermite,
            Keyframes::Bezier(_) => Interpolation::Bezier,
        }
    }
    pub fn step_keys(&self) -> Option<&[ValueKeyframe<K::Value>]> {
        match &self.keyframes {
            Keyframes::Step(keys) => Some(keys),
            _ => None,
        }
    }
    pub fn linear_keys(&self) -> Option<&[ValueKeyframe<K::Value>]> {
        match &self.keyframes {
            Keyframes::Linear(keys) => Some(keys),
            _ => None,
        }
    }
    pub fn hermite_keys(&self) -> Option<&[TangentKeyframe<K::Value>]> {
        match &self.keyframes {
            Keyframes::Hermite(keys) => Some(keys),
            _ => None,
        }
    }
    pub fn bezier_keys(&self) -> Option<&[TangentKeyframe<K::Value>]> {
        match &self.keyframes {
            Keyframes::Bezier(keys) => Some(keys),
            _ => None,
        }
    }
    /// Returns this track's tag.
    pub fn tag(&self) -> TrackTag {
        K::TAG_KIND
    }
}
impl<T> Keyframes<T> {
    fn len(&self) -> usize {
        match self {
            Self::Step(v) | Self::Linear(v) => v.len(),
            Self::Hermite(v) | Self::Bezier(v) => v.len(),
        }
    }
    fn interpolation(&self) -> u32 {
        match self {
            Self::Step(_) => 0,
            Self::Linear(_) => 1,
            Self::Hermite(_) => 2,
            Self::Bezier(_) => 3,
        }
    }
}
impl<K: TrackKind> Readable for AnimationTrack<K> {
    fn read_from(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        let mut next = *cursor;
        let offset = next.absolute_position();
        let tag: Tag = next.read_exact(4)?.try_into().expect("four-byte tag");
        let malformed = || DecodeError::MalformedRecord { tag, offset };
        if tag != K::TAG {
            return Err(malformed());
        }
        let count = next.read::<u32>().map_err(|_| malformed())? as usize;
        let interpolation = next.read::<u32>().map_err(|_| malformed())?;
        if interpolation > 3 {
            return Err(malformed());
        }
        let sequence = next.read::<u32>().map_err(|_| malformed())?;
        let components = K::Value::COMPONENTS;
        let vector_count = if interpolation >= 2 { 3usize } else { 1usize };
        let key_size = components
            .checked_mul(vector_count)
            .and_then(|n| n.checked_mul(4))
            .and_then(|n| n.checked_add(4))
            .ok_or_else(malformed)?;
        let body_size = count.checked_mul(key_size).ok_or_else(malformed)?;
        let mut body = next.slice(body_size).map_err(|_| malformed())?;
        let keyframes = match interpolation {
            0 | 1 => {
                let mut keys = Vec::with_capacity(count);
                for _ in 0..count {
                    keys.push(
                        body.read::<ValueKeyframe<K::Value>>()
                            .map_err(|_| malformed())?,
                    );
                }
                if interpolation == 0 {
                    Keyframes::Step(keys)
                } else {
                    Keyframes::Linear(keys)
                }
            }
            2 | 3 => {
                let mut keys = Vec::with_capacity(count);
                for _ in 0..count {
                    keys.push(
                        body.read::<TangentKeyframe<K::Value>>()
                            .map_err(|_| malformed())?,
                    );
                }
                if interpolation == 2 {
                    Keyframes::Hermite(keys)
                } else {
                    Keyframes::Bezier(keys)
                }
            }
            _ => unreachable!(),
        };
        *cursor = next;
        Ok(Self {
            global_sequence_id: (sequence != u32::MAX).then_some(sequence),
            keyframes,
            kind: PhantomData,
        })
    }
}
impl<K: TrackKind> Writable for &AnimationTrack<K> {
    fn write_to(self, encoder: &mut Encoder<'_>) {
        encoder.write_bytes(&K::TAG);
        encoder.write(self.keyframes.len() as u32);
        encoder.write(self.keyframes.interpolation());
        encoder.write(self.global_sequence_id.unwrap_or(u32::MAX));
        match &self.keyframes {
            Keyframes::Step(keys) | Keyframes::Linear(keys) => {
                for key in keys {
                    encoder.write(key);
                }
            }
            Keyframes::Hermite(keys) | Keyframes::Bezier(keys) => {
                for key in keys {
                    encoder.write(key);
                }
            }
        }
    }
}
macro_rules! track_group {
    ($vis:vis enum $group:ident { $($variant:ident : $kind:ident),+ $(,)? }) => {
        #[derive(Clone, Debug, PartialEq)]
        $vis enum $group {
            $( $variant($crate::animation::AnimationTrack<$crate::animation::$kind>), )+
        }
        impl $group {
            pub fn tag(&self) -> $crate::animation::TrackTag {
                match self { $(Self::$variant(_) => <$crate::animation::$kind as $crate::animation::TrackKind>::TAG_KIND,)+ }
            }
            pub fn accepts_bytes(tag: $crate::Tag) -> bool {
                false $(|| tag == <$crate::animation::$kind as $crate::animation::TrackKind>::TAG)+
            }
        }
        impl $crate::Readable for $group {
            fn read_from(cursor: &mut $crate::Cursor<'_>) -> Result<Self, $crate::DecodeError> {
                let offset = cursor.absolute_position();
                let tag: $crate::Tag = cursor.peek_exact(4)?.try_into().expect("four-byte tag");
                $(if tag == <$crate::animation::$kind as $crate::animation::TrackKind>::TAG {
                    return Ok(Self::$variant(cursor.read::<$crate::animation::AnimationTrack<$crate::animation::$kind>>()?));
                })+
                Err($crate::DecodeError::MalformedRecord { tag, offset })
            }
        }
        impl $crate::Writable for &$group {
            fn write_to(self, encoder: &mut $crate::Encoder<'_>) {
                match self { $( $group::$variant(track) => encoder.write(track), )+ }
            }
        }
        $(impl From<$crate::animation::AnimationTrack<$crate::animation::$kind>> for $group {
            fn from(track: $crate::animation::AnimationTrack<$crate::animation::$kind>) -> Self { Self::$variant(track) }
        })+
    };
}
pub(crate) use track_group;
