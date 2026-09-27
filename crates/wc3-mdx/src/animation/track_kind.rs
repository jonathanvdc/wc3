//! Known animation track kinds and their value types.
use crate::{Color, Readable, Tag, Vec3, Vec4, Writable};

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
    LightShadowCastingStart => (b"KLSS", f32),
    LightShadowCastingEnd => (b"KLSE", f32),
    LightQuadraticFalloff => (b"KLQF", f32),
    LightLinearFalloff => (b"KLLF", f32),
    LightDamping => (b"KLDA", f32),
    LayerAlpha => (b"KMTA", f32),
    LayerTextureId => (b"KMTF", u32),
    LayerEmissiveGain => (b"KMTE", f32),
    LayerFresnelOpacity => (b"KFCA", f32),
    LayerFresnelTeamColor => (b"KFTC", f32),
}
