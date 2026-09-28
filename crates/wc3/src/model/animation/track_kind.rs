//! Known animation track kinds and their value types.
use crate::model::mdx;
use crate::model::{Color, Tag, Vec3, Vec4};

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
    /// Property name used by the MDL codec.
    const MDL_NAME: &'static str;
}

/// A scalar or vector value supported by MDX keyframe tracks.
pub trait TrackValue: mdx::Read + mdx::Write + Copy + PartialEq + std::fmt::Debug {
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
    ($($name:ident => ($tag:literal, $value:ty, $mdl:literal)),+ $(,)?) => {
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
                const MDL_NAME: &'static str = $mdl;
            }
        )+
    };
}

track_kinds! {
    NodeTranslation => (b"KGTR", Vec3, "Translation"),
    NodeScaling => (b"KGSC", Vec3, "Scaling"),
    CameraTranslation => (b"KCTR", Vec3, "Translation"),
    CameraTargetTranslation => (b"KTTR", Vec3, "Translation"),
    PopcornColor => (b"KPPC", Color, "Color"),
    TextureTranslation => (b"KTAT", Vec3, "Translation"),
    TextureScaling => (b"KTAS", Vec3, "Scaling"),
    GeosetColor => (b"KGAC", Color, "Color"),
    LightColor => (b"KLAC", Color, "Color"),
    LightAmbientColor => (b"KLBC", Color, "AmbColor"),
    LayerFresnelColor => (b"KFC3", Color, "FresnelColor"),
    RibbonColor => (b"KRCO", Color, "Color"),
    NodeRotation => (b"KGRT", Vec4, "Rotation"),
    TextureRotation => (b"KTAR", Vec4, "Rotation"),
    CameraVisibility => (b"KCVS", f32, "Visibility"),
    CameraFocusDistance => (b"IDUF", f32, "FocusDistanceKeys"),
    CameraFocalLength => (b"ELAF", f32, "FocalLengthKeys"),
    CameraFStop => (b"PTSF", f32, "FStopKeys"),
    CameraRotation => (b"KCRL", f32, "Rotation"),
    AttachmentVisibility => (b"KATV", f32, "Visibility"),
    PopcornAlpha => (b"KPPA", f32, "Alpha"),
    PopcornEmissionRate => (b"KPPE", f32, "EmissionRate"),
    PopcornLifespan => (b"KPPL", f32, "LifeSpan"),
    PopcornSpeed => (b"KPPS", f32, "Speed"),
    PopcornVisibility => (b"KPPV", f32, "Visibility"),
    ParticleVisibility => (b"KPEV", f32, "Visibility"),
    ParticleEmissionRate => (b"KPEE", f32, "EmissionRate"),
    ParticleGravity => (b"KPEG", f32, "Gravity"),
    ParticleLongitude => (b"KPLN", f32, "Longitude"),
    ParticleLatitude => (b"KPLT", f32, "Latitude"),
    ParticleLifespan => (b"KPEL", f32, "LifeSpan"),
    ParticleSpeed => (b"KPES", f32, "InitVelocity"),
    Particle2Visibility => (b"KP2V", f32, "Visibility"),
    Particle2EmissionRate => (b"KP2E", f32, "EmissionRate"),
    Particle2Width => (b"KP2W", f32, "Width"),
    Particle2Length => (b"KP2N", f32, "Length"),
    Particle2Speed => (b"KP2S", f32, "Speed"),
    Particle2Latitude => (b"KP2L", f32, "Latitude"),
    Particle2Gravity => (b"KP2G", f32, "Gravity"),
    Particle2Variation => (b"KP2R", f32, "Variation"),
    RibbonVisibility => (b"KRVS", f32, "Visibility"),
    RibbonHeightAbove => (b"KRHA", f32, "HeightAbove"),
    RibbonHeightBelow => (b"KRHB", f32, "HeightBelow"),
    RibbonAlpha => (b"KRAL", f32, "Alpha"),
    RibbonTextureSlot => (b"KRTX", u32, "TextureSlot"),
    GeosetAlpha => (b"KGAO", f32, "Alpha"),
    LightVisibility => (b"KLAV", f32, "Visibility"),
    LightIntensity => (b"KLAI", f32, "Intensity"),
    LightAmbientIntensity => (b"KLBI", f32, "AmbIntensity"),
    LightAttenuationStart => (b"KLAS", f32, "AttenuationStart"),
    LightAttenuationEnd => (b"KLAE", f32, "AttenuationEnd"),
    LightShadowCastingStart => (b"KLSS", f32, "ShadowCastingStart"),
    LightShadowCastingEnd => (b"KLSE", f32, "ShadowCastingEnd"),
    LightQuadraticFalloff => (b"KLQF", f32, "QuadraticFalloff"),
    LightLinearFalloff => (b"KLLF", f32, "LinearFalloff"),
    LightDamping => (b"KLDA", f32, "Damping"),
    LayerAlpha => (b"KMTA", f32, "Alpha"),
    LayerTextureId => (b"KMTF", u32, "TextureID"),
    LayerEmissiveGain => (b"KMTE", f32, "EmissiveGain"),
    LayerFresnelOpacity => (b"KFCA", f32, "FresnelOpacity"),
    LayerFresnelTeamColor => (b"KFTC", f32, "FresnelTeamColor"),
}
