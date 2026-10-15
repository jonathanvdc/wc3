//! Typed Warcraft III property definitions used by semantic animation bindings.
use crate::model::semantic::profiles::Mdlx;
use crate::model::semantic::AttachmentActivation;
use crate::model::semantic::NodeFeature;
use crate::model::semantic::{
    FeatureId, FeatureProperty, MaterialId, OwnerId, PropertyIdentityKey, PropertyKey,
    PropertyKeyRef, PropertyPayload, TextureId,
};
use std::any::TypeId;

/// Animated geometry properties on an attached feature.
pub mod geometry {
    use super::*;
    use super::{FeatureId, PropertyKey};
    /// Animated opacity.
    pub const OPACITY: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Geometry(GeometryField::Opacity));
    /// Animated color.
    pub const COLOR: PropertyKey<FeatureId, [f32; 3], Mdlx> =
        PropertyKey::new(Target::Geometry(GeometryField::Color));
}

/// Animated particle properties on an attached feature.
pub mod particle {
    use super::*;
    use super::{FeatureId, PropertyKey};
    /// Animated emission rate.
    pub const EMISSION_RATE: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Particle(ParticleField::EmissionRate));
    /// Animated gravity.
    pub const GRAVITY: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Particle(ParticleField::Gravity));
    /// Animated longitude.
    pub const LONGITUDE: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Particle(ParticleField::Longitude));
    /// Animated latitude.
    pub const LATITUDE: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Particle(ParticleField::Latitude));
    /// Animated life span.
    pub const LIFE_SPAN: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Particle(ParticleField::LifeSpan));
    /// Animated initial velocity.
    pub const INITIAL_VELOCITY: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Particle(ParticleField::InitialVelocity));
    /// Animated visibility.
    pub const VISIBILITY: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Particle(ParticleField::Visibility));
}

/// Animated particle2 properties on an attached feature.
pub mod particle2 {
    use super::*;
    use super::{FeatureId, PropertyKey};
    /// Animated speed.
    pub const SPEED: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Particle2(Particle2Field::Speed));
    /// Animated variation.
    pub const VARIATION: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Particle2(Particle2Field::Variation));
    /// Animated latitude.
    pub const LATITUDE: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Particle2(Particle2Field::Latitude));
    /// Animated gravity.
    pub const GRAVITY: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Particle2(Particle2Field::Gravity));
    /// Animated emission rate.
    pub const EMISSION_RATE: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Particle2(Particle2Field::EmissionRate));
    /// Animated length.
    pub const LENGTH: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Particle2(Particle2Field::Length));
    /// Animated width.
    pub const WIDTH: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Particle2(Particle2Field::Width));
    /// Animated visibility.
    pub const VISIBILITY: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Particle2(Particle2Field::Visibility));
}

/// Animated ribbon properties on an attached feature.
pub mod ribbon {
    use super::*;
    use super::{FeatureId, PropertyKey};
    /// Animated height above.
    pub const HEIGHT_ABOVE: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Ribbon(RibbonField::HeightAbove));
    /// Animated height below.
    pub const HEIGHT_BELOW: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Ribbon(RibbonField::HeightBelow));
    /// Animated opacity.
    pub const OPACITY: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Ribbon(RibbonField::Opacity));
    /// Animated color.
    pub const COLOR: PropertyKey<FeatureId, [f32; 3], Mdlx> =
        PropertyKey::new(Target::Ribbon(RibbonField::Color));
    /// Animated texture slot.
    pub const TEXTURE_SLOT: PropertyKey<FeatureId, u32, Mdlx> =
        PropertyKey::new(Target::Ribbon(RibbonField::TextureSlot));
    /// Animated visibility.
    pub const VISIBILITY: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Ribbon(RibbonField::Visibility));
}

/// Animated popcorn properties on an attached feature.
pub mod popcorn {
    use super::*;
    use super::{FeatureId, PropertyKey};
    /// Animated life span.
    pub const LIFE_SPAN: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Popcorn(PopcornField::LifeSpan));
    /// Animated emission rate.
    pub const EMISSION_RATE: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Popcorn(PopcornField::EmissionRate));
    /// Animated speed.
    pub const SPEED: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Popcorn(PopcornField::Speed));
    /// Animated color.
    pub const COLOR: PropertyKey<FeatureId, [f32; 3], Mdlx> =
        PropertyKey::new(Target::Popcorn(PopcornField::Color));
    /// Animated opacity.
    pub const OPACITY: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Popcorn(PopcornField::Opacity));
    /// Animated visibility.
    pub const VISIBILITY: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Popcorn(PopcornField::Visibility));
}

/// Animated light properties on an attached feature.
pub mod light {
    use super::*;
    use super::{FeatureId, PropertyKey};
    /// Animated color.
    pub const COLOR: PropertyKey<FeatureId, [f32; 3], Mdlx> =
        PropertyKey::new(Target::Light(LightField::Color));
    /// Animated intensity.
    pub const INTENSITY: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Light(LightField::Intensity));
    /// Animated ambient color.
    pub const AMBIENT_COLOR: PropertyKey<FeatureId, [f32; 3], Mdlx> =
        PropertyKey::new(Target::Light(LightField::AmbientColor));
    /// Animated ambient intensity.
    pub const AMBIENT_INTENSITY: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Light(LightField::AmbientIntensity));
    /// Animated visibility.
    pub const VISIBILITY: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Light(LightField::Visibility));
    /// Animated attenuation start.
    pub const ATTENUATION_START: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Light(LightField::AttenuationStart));
    /// Animated attenuation end.
    pub const ATTENUATION_END: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Light(LightField::AttenuationEnd));
    /// Animated shadow start.
    pub const SHADOW_START: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Light(LightField::ShadowStart));
    /// Animated shadow end.
    pub const SHADOW_END: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Light(LightField::ShadowEnd));
    /// Animated quadratic.
    pub const QUADRATIC: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Light(LightField::Quadratic));
    /// Animated linear.
    pub const LINEAR: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Light(LightField::Linear));
    /// Animated damping.
    pub const DAMPING: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Light(LightField::Damping));
}

/// Animated camera properties on an attached feature.
pub mod camera {
    use super::*;
    use super::{FeatureId, PropertyKey};
    /// Animated eye offset.
    pub const EYE_OFFSET: PropertyKey<FeatureId, [f32; 3], Mdlx> =
        PropertyKey::new(Target::Camera(CameraField::EyeOffset));
    /// Animated target offset.
    pub const TARGET_OFFSET: PropertyKey<FeatureId, [f32; 3], Mdlx> =
        PropertyKey::new(Target::Camera(CameraField::TargetOffset));
    /// Animated roll.
    pub const ROLL: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Camera(CameraField::Roll));
    /// Animated visibility.
    pub const VISIBILITY: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Camera(CameraField::Visibility));
    /// Animated focus distance.
    pub const FOCUS_DISTANCE: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Camera(CameraField::FocusDistance));
    /// Animated focal length.
    pub const FOCAL_LENGTH: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Camera(CameraField::FocalLength));
    /// Animated f stop.
    pub const F_STOP: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Camera(CameraField::FStop));
}

/// Animated attachment properties on an attached feature.
pub mod attachment {
    use super::*;
    use super::{FeatureId, PropertyKey};
    /// Animated visibility.
    pub const VISIBILITY: PropertyKey<FeatureId, f32, Mdlx> =
        PropertyKey::new(Target::Attachment(AttachmentField::Visibility));
}

/// Material-layer destinations selected by their logical layer index.
#[derive(Clone, Copy, Debug)]
pub struct Layer(pub usize);
impl Layer {
    /// Animated layer opacity.
    pub fn opacity(self) -> PropertyKey<MaterialId, f32, Mdlx> {
        PropertyKey::new(Target::Layer {
            index: self.0,
            field: LayerField::Opacity,
        })
    }
    /// Animated layer texture selection.
    pub fn texture(self) -> PropertyKey<MaterialId, Option<TextureId>, Mdlx> {
        PropertyKey::new(Target::Layer {
            index: self.0,
            field: LayerField::Texture,
        })
    }
    /// Animated emissive gain.
    pub fn emissive_gain(self) -> PropertyKey<MaterialId, f32, Mdlx> {
        PropertyKey::new(Target::Layer {
            index: self.0,
            field: LayerField::EmissiveGain,
        })
    }
    /// Animated Fresnel color.
    pub fn fresnel_color(self) -> PropertyKey<MaterialId, [f32; 3], Mdlx> {
        PropertyKey::new(Target::Layer {
            index: self.0,
            field: LayerField::FresnelColor,
        })
    }
    /// Animated Fresnel opacity.
    pub fn fresnel_opacity(self) -> PropertyKey<MaterialId, f32, Mdlx> {
        PropertyKey::new(Target::Layer {
            index: self.0,
            field: LayerField::FresnelOpacity,
        })
    }
    /// Animated Fresnel team-color contribution.
    pub fn fresnel_team_color(self) -> PropertyKey<MaterialId, f32, Mdlx> {
        PropertyKey::new(Target::Layer {
            index: self.0,
            field: LayerField::FresnelTeamColor,
        })
    }
    /// Animated texture selection for an additional logical sampling slot.
    pub fn texture_slot(self, slot: usize) -> PropertyKey<MaterialId, Option<TextureId>, Mdlx> {
        PropertyKey::new(Target::LayerTexture {
            layer: self.0,
            slot,
        })
    }
}

/// Structural destinations owned by this property vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    /// Geometry properties.
    Geometry(GeometryField),
    /// Initial speed shared by resource and atlas particle emitters.
    ParticleSpeed,
    /// Continuous particles per second on resource or non-burst atlas emitters.
    ParticleContinuousRate,
    /// Particle properties.
    Particle(ParticleField),
    /// Particle2 properties.
    Particle2(Particle2Field),
    /// Ribbon properties.
    Ribbon(RibbonField),
    /// Popcorn properties.
    Popcorn(PopcornField),
    /// Light properties.
    Light(LightField),
    /// Camera properties.
    Camera(CameraField),
    /// Attachment properties.
    Attachment(AttachmentField),
    /// One logical material-layer texture slot.
    LayerTexture {
        /// Logical material layer ordinal.
        layer: usize,
        /// Sampling slot within this layer.
        slot: usize,
    },
    /// Indexed layer property.
    Layer {
        /// Owner-local element ordinal.
        index: usize,
        /// Field of the selected element.
        field: LayerField,
    },
}
/// Structural geometry field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GeometryField {
    /// Animated opacity.
    Opacity,
    /// Animated color.
    Color,
}

/// Structural particle field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParticleField {
    /// Animated emission rate.
    EmissionRate,
    /// Animated gravity.
    Gravity,
    /// Animated longitude.
    Longitude,
    /// Animated latitude.
    Latitude,
    /// Animated life span.
    LifeSpan,
    /// Animated initial velocity.
    InitialVelocity,
    /// Animated visibility.
    Visibility,
}

/// Structural particle2 field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Particle2Field {
    /// Animated speed.
    Speed,
    /// Animated variation.
    Variation,
    /// Animated latitude.
    Latitude,
    /// Animated gravity.
    Gravity,
    /// Animated emission rate.
    EmissionRate,
    /// Animated length.
    Length,
    /// Animated width.
    Width,
    /// Animated visibility.
    Visibility,
}

/// Structural ribbon field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RibbonField {
    /// Animated height above.
    HeightAbove,
    /// Animated height below.
    HeightBelow,
    /// Animated opacity.
    Opacity,
    /// Animated color.
    Color,
    /// Animated texture slot.
    TextureSlot,
    /// Animated visibility.
    Visibility,
}

/// Structural popcorn field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PopcornField {
    /// Animated life span.
    LifeSpan,
    /// Animated emission rate.
    EmissionRate,
    /// Animated speed.
    Speed,
    /// Animated color.
    Color,
    /// Animated opacity.
    Opacity,
    /// Animated visibility.
    Visibility,
}

/// Structural light field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LightField {
    /// Animated color.
    Color,
    /// Animated intensity.
    Intensity,
    /// Animated ambient color.
    AmbientColor,
    /// Animated ambient intensity.
    AmbientIntensity,
    /// Animated visibility.
    Visibility,
    /// Animated attenuation start.
    AttenuationStart,
    /// Animated attenuation end.
    AttenuationEnd,
    /// Animated shadow start.
    ShadowStart,
    /// Animated shadow end.
    ShadowEnd,
    /// Animated quadratic.
    Quadratic,
    /// Animated linear.
    Linear,
    /// Animated damping.
    Damping,
}

/// Structural camera field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CameraField {
    /// Animated eye offset.
    EyeOffset,
    /// Animated target offset.
    TargetOffset,
    /// Animated roll.
    Roll,
    /// Animated visibility.
    Visibility,
    /// Animated focus distance.
    FocusDistance,
    /// Animated focal length.
    FocalLength,
    /// Animated f stop.
    FStop,
}

/// Structural attachment field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttachmentField {
    /// Animated visibility.
    Visibility,
}

/// Structural layer field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayerField {
    /// Animated opacity.
    Opacity,
    /// Animated texture.
    Texture,
    /// Animated emissive gain.
    EmissiveGain,
    /// Animated fresnel color.
    FresnelColor,
    /// Animated fresnel opacity.
    FresnelOpacity,
    /// Animated fresnel team color.
    FresnelTeamColor,
}

impl Target {
    fn shared_feature_property(&self) -> Option<&'static FeatureProperty> {
        match self {
            Self::Light(LightField::Color) => Some(&FeatureProperty::LightDiffuseColor),
            Self::Light(LightField::Intensity) => Some(&FeatureProperty::LightDiffuseIntensity),
            Self::Light(LightField::AmbientColor) => Some(&FeatureProperty::LightAmbientColor),
            Self::Light(LightField::AmbientIntensity) => {
                Some(&FeatureProperty::LightAmbientIntensity)
            }
            Self::Light(LightField::AttenuationStart) => {
                Some(&FeatureProperty::LightAttenuationStart)
            }
            Self::Light(LightField::AttenuationEnd) => Some(&FeatureProperty::LightAttenuationEnd),
            Self::ParticleContinuousRate => Some(&FeatureProperty::ParticleContinuousRate),
            Self::ParticleSpeed | Self::Particle2(Particle2Field::Speed) => {
                Some(&FeatureProperty::ParticleSpeed)
            }
            Self::Camera(CameraField::Roll) => Some(&FeatureProperty::CameraRoll),
            Self::Ribbon(RibbonField::Color) => Some(&FeatureProperty::TrailColor),
            Self::Ribbon(RibbonField::Opacity) => Some(&FeatureProperty::TrailOpacity),
            Self::Ribbon(RibbonField::HeightAbove) => Some(&FeatureProperty::TrailHeightAbove),
            Self::Ribbon(RibbonField::HeightBelow) => Some(&FeatureProperty::TrailHeightBelow),
            Self::Particle(ParticleField::EmissionRate) => {
                Some(&FeatureProperty::ParticleContinuousRate)
            }
            Self::Particle(ParticleField::LifeSpan) => Some(&FeatureProperty::ParticleLifetime),
            Self::Particle(ParticleField::InitialVelocity) => Some(&FeatureProperty::ParticleSpeed),

            _ => None,
        }
    }
}
impl PropertyPayload for Target {
    fn identity(&self) -> PropertyKeyRef<'_> {
        match self.shared_feature_property() {
            Some(value) => value.identity(),
            None => PropertyKeyRef::new(self),
        }
    }
    fn from_identity(key: &PropertyIdentityKey) -> Option<Self> {
        if let Some(value) = key.downcast_ref::<FeatureProperty>() {
            return Some(match value {
                FeatureProperty::LightDiffuseColor => Self::Light(LightField::Color),
                FeatureProperty::LightDiffuseIntensity => Self::Light(LightField::Intensity),
                FeatureProperty::LightAmbientColor => Self::Light(LightField::AmbientColor),
                FeatureProperty::LightAmbientIntensity => Self::Light(LightField::AmbientIntensity),
                FeatureProperty::LightAttenuationStart => Self::Light(LightField::AttenuationStart),
                FeatureProperty::LightAttenuationEnd => Self::Light(LightField::AttenuationEnd),
                FeatureProperty::CameraRoll => Self::Camera(CameraField::Roll),
                FeatureProperty::TrailColor => Self::Ribbon(RibbonField::Color),
                FeatureProperty::TrailOpacity => Self::Ribbon(RibbonField::Opacity),
                FeatureProperty::TrailHeightAbove => Self::Ribbon(RibbonField::HeightAbove),
                FeatureProperty::TrailHeightBelow => Self::Ribbon(RibbonField::HeightBelow),
                FeatureProperty::ParticleContinuousRate => Self::ParticleContinuousRate,
                FeatureProperty::ParticleLifetime => Self::Particle(ParticleField::LifeSpan),
                FeatureProperty::ParticleSpeed => Self::ParticleSpeed,
                _ => return None,
            });
        }
        key.downcast_ref::<Self>().cloned()
    }

    fn value_type(&self) -> Option<TypeId> {
        Some(match self {
            Target::ParticleSpeed | Target::ParticleContinuousRate => TypeId::of::<f32>(),
            Target::Geometry(GeometryField::Opacity) => TypeId::of::<f32>(),
            Target::Geometry(GeometryField::Color) => TypeId::of::<[f32; 3]>(),
            Target::Particle(ParticleField::EmissionRate) => TypeId::of::<f32>(),
            Target::Particle(ParticleField::Gravity) => TypeId::of::<f32>(),
            Target::Particle(ParticleField::Longitude) => TypeId::of::<f32>(),
            Target::Particle(ParticleField::Latitude) => TypeId::of::<f32>(),
            Target::Particle(ParticleField::LifeSpan) => TypeId::of::<f32>(),
            Target::Particle(ParticleField::InitialVelocity) => TypeId::of::<f32>(),
            Target::Particle(ParticleField::Visibility) => TypeId::of::<f32>(),
            Target::Particle2(Particle2Field::Speed) => TypeId::of::<f32>(),
            Target::Particle2(Particle2Field::Variation) => TypeId::of::<f32>(),
            Target::Particle2(Particle2Field::Latitude) => TypeId::of::<f32>(),
            Target::Particle2(Particle2Field::Gravity) => TypeId::of::<f32>(),
            Target::Particle2(Particle2Field::EmissionRate) => TypeId::of::<f32>(),
            Target::Particle2(Particle2Field::Length) => TypeId::of::<f32>(),
            Target::Particle2(Particle2Field::Width) => TypeId::of::<f32>(),
            Target::Particle2(Particle2Field::Visibility) => TypeId::of::<f32>(),
            Target::Ribbon(RibbonField::HeightAbove) => TypeId::of::<f32>(),
            Target::Ribbon(RibbonField::HeightBelow) => TypeId::of::<f32>(),
            Target::Ribbon(RibbonField::Opacity) => TypeId::of::<f32>(),
            Target::Ribbon(RibbonField::Color) => TypeId::of::<[f32; 3]>(),
            Target::Ribbon(RibbonField::TextureSlot) => TypeId::of::<u32>(),
            Target::Ribbon(RibbonField::Visibility) => TypeId::of::<f32>(),
            Target::Popcorn(PopcornField::LifeSpan) => TypeId::of::<f32>(),
            Target::Popcorn(PopcornField::EmissionRate) => TypeId::of::<f32>(),
            Target::Popcorn(PopcornField::Speed) => TypeId::of::<f32>(),
            Target::Popcorn(PopcornField::Color) => TypeId::of::<[f32; 3]>(),
            Target::Popcorn(PopcornField::Opacity) => TypeId::of::<f32>(),
            Target::Popcorn(PopcornField::Visibility) => TypeId::of::<f32>(),
            Target::Light(LightField::Color) => TypeId::of::<[f32; 3]>(),
            Target::Light(LightField::Intensity) => TypeId::of::<f32>(),
            Target::Light(LightField::AmbientColor) => TypeId::of::<[f32; 3]>(),
            Target::Light(LightField::AmbientIntensity) => TypeId::of::<f32>(),
            Target::Light(LightField::Visibility) => TypeId::of::<f32>(),
            Target::Light(LightField::AttenuationStart) => TypeId::of::<f32>(),
            Target::Light(LightField::AttenuationEnd) => TypeId::of::<f32>(),
            Target::Light(LightField::ShadowStart) => TypeId::of::<f32>(),
            Target::Light(LightField::ShadowEnd) => TypeId::of::<f32>(),
            Target::Light(LightField::Quadratic) => TypeId::of::<f32>(),
            Target::Light(LightField::Linear) => TypeId::of::<f32>(),
            Target::Light(LightField::Damping) => TypeId::of::<f32>(),
            Target::Camera(CameraField::EyeOffset) => TypeId::of::<[f32; 3]>(),
            Target::Camera(CameraField::TargetOffset) => TypeId::of::<[f32; 3]>(),
            Target::Camera(CameraField::Roll) => TypeId::of::<f32>(),
            Target::Camera(CameraField::Visibility) => TypeId::of::<f32>(),
            Target::Camera(CameraField::FocusDistance) => TypeId::of::<f32>(),
            Target::Camera(CameraField::FocalLength) => TypeId::of::<f32>(),
            Target::Camera(CameraField::FStop) => TypeId::of::<f32>(),
            Target::Attachment(AttachmentField::Visibility) => TypeId::of::<f32>(),
            Target::Layer {
                field: LayerField::Opacity,
                ..
            } => TypeId::of::<f32>(),
            Target::Layer {
                field: LayerField::Texture,
                ..
            } => TypeId::of::<Option<TextureId>>(),
            Target::Layer {
                field: LayerField::EmissiveGain,
                ..
            } => TypeId::of::<f32>(),
            Target::Layer {
                field: LayerField::FresnelColor,
                ..
            } => TypeId::of::<[f32; 3]>(),
            Target::Layer {
                field: LayerField::FresnelOpacity,
                ..
            } => TypeId::of::<f32>(),
            Target::Layer {
                field: LayerField::FresnelTeamColor,
                ..
            } => TypeId::of::<f32>(),
            Target::LayerTexture { .. } => TypeId::of::<Option<TextureId>>(),
        })
    }
    fn accepts_owner(&self, owner: OwnerId) -> bool {
        match self {
            Target::ParticleSpeed | Target::ParticleContinuousRate => {
                matches!(owner, OwnerId::Feature(_))
            }
            Target::Geometry(GeometryField::Opacity) => matches!(owner, OwnerId::Feature(_)),
            Target::Geometry(GeometryField::Color) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::EmissionRate) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::Gravity) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::Longitude) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::Latitude) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::LifeSpan) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::InitialVelocity) => {
                matches!(owner, OwnerId::Feature(_))
            }
            Target::Particle(ParticleField::Visibility) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle2(Particle2Field::Speed) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle2(Particle2Field::Variation) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle2(Particle2Field::Latitude) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle2(Particle2Field::Gravity) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle2(Particle2Field::EmissionRate) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle2(Particle2Field::Length) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle2(Particle2Field::Width) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle2(Particle2Field::Visibility) => matches!(owner, OwnerId::Feature(_)),
            Target::Ribbon(RibbonField::HeightAbove) => matches!(owner, OwnerId::Feature(_)),
            Target::Ribbon(RibbonField::HeightBelow) => matches!(owner, OwnerId::Feature(_)),
            Target::Ribbon(RibbonField::Opacity) => matches!(owner, OwnerId::Feature(_)),
            Target::Ribbon(RibbonField::Color) => matches!(owner, OwnerId::Feature(_)),
            Target::Ribbon(RibbonField::TextureSlot) => matches!(owner, OwnerId::Feature(_)),
            Target::Ribbon(RibbonField::Visibility) => matches!(owner, OwnerId::Feature(_)),
            Target::Popcorn(PopcornField::LifeSpan) => matches!(owner, OwnerId::Feature(_)),
            Target::Popcorn(PopcornField::EmissionRate) => matches!(owner, OwnerId::Feature(_)),
            Target::Popcorn(PopcornField::Speed) => matches!(owner, OwnerId::Feature(_)),
            Target::Popcorn(PopcornField::Color) => matches!(owner, OwnerId::Feature(_)),
            Target::Popcorn(PopcornField::Opacity) => matches!(owner, OwnerId::Feature(_)),
            Target::Popcorn(PopcornField::Visibility) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::Color) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::Intensity) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::AmbientColor) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::AmbientIntensity) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::Visibility) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::AttenuationStart) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::AttenuationEnd) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::ShadowStart) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::ShadowEnd) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::Quadratic) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::Linear) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::Damping) => matches!(owner, OwnerId::Feature(_)),
            Target::Camera(CameraField::EyeOffset) => matches!(owner, OwnerId::Feature(_)),
            Target::Camera(CameraField::TargetOffset) => matches!(owner, OwnerId::Feature(_)),
            Target::Camera(CameraField::Roll) => matches!(owner, OwnerId::Feature(_)),
            Target::Camera(CameraField::Visibility) => matches!(owner, OwnerId::Feature(_)),
            Target::Camera(CameraField::FocusDistance) => matches!(owner, OwnerId::Feature(_)),
            Target::Camera(CameraField::FocalLength) => matches!(owner, OwnerId::Feature(_)),
            Target::Camera(CameraField::FStop) => matches!(owner, OwnerId::Feature(_)),
            Target::Attachment(AttachmentField::Visibility) => matches!(owner, OwnerId::Feature(_)),
            Target::Layer {
                field: LayerField::Opacity,
                ..
            } => matches!(owner, OwnerId::Material(_)),
            Target::Layer {
                field: LayerField::Texture,
                ..
            } => matches!(owner, OwnerId::Material(_)),
            Target::Layer {
                field: LayerField::EmissiveGain,
                ..
            } => matches!(owner, OwnerId::Material(_)),
            Target::Layer {
                field: LayerField::FresnelColor,
                ..
            } => matches!(owner, OwnerId::Material(_)),
            Target::Layer {
                field: LayerField::FresnelOpacity,
                ..
            } => matches!(owner, OwnerId::Material(_)),
            Target::Layer {
                field: LayerField::FresnelTeamColor,
                ..
            } => matches!(owner, OwnerId::Material(_)),
            Target::LayerTexture { .. } => matches!(owner, OwnerId::Material(_)),
        }
    }
}

impl Target {
    pub(crate) fn supports_feature(&self, value: &super::Node<'_>) -> bool {
        if matches!(self, Self::ParticleContinuousRate) {
            return matches!(value, super::Node::Particle(_))
                || matches!(value, super::Node::AtlasParticles(particle) if !particle.emission.trigger);
        }
        if matches!(self, Self::ParticleSpeed) {
            return matches!(
                value,
                super::Node::Particle(_) | super::Node::AtlasParticles(_)
            );
        }
        match (self, value) {
            (
                Self::Light(LightField::ShadowStart | LightField::ShadowEnd),
                super::Node::Light(v),
            ) => return v.activation.shadow_range.is_some(),
            (
                Self::Light(LightField::Quadratic | LightField::Linear | LightField::Damping),
                super::Node::Light(v),
            ) => return v.activation.falloff.is_some(),
            (Self::Camera(CameraField::FocusDistance), super::Node::Camera(v)) => {
                return v.convention.focus_distance.is_some()
            }
            (Self::Camera(CameraField::FocalLength), super::Node::Camera(v)) => {
                return v.convention.focal_length.is_some()
            }
            (Self::Camera(CameraField::FStop), super::Node::Camera(v)) => {
                return v.convention.f_stop.is_some()
            }
            _ => {}
        }
        matches!(
            (self, value),
            (Self::Geometry(_), super::Node::GeometryAnimation(_))
                | (Self::Particle(_), super::Node::Particle(_))
                | (Self::Particle2(_), super::Node::AtlasParticles(_))
                | (Self::Ribbon(_), super::Node::Ribbon(_))
                | (Self::Popcorn(_), super::Node::ParticleProgram(_))
                | (Self::Light(_), super::Node::Light(_))
                | (Self::Camera(_), super::Node::Camera(_))
                | (Self::Attachment(_), super::Node::Attachment(_))
        )
    }
}

impl Target {
    pub(crate) fn supports_universal_feature(&self, value: &NodeFeature<'_>) -> bool {
        use crate::model::semantic::{
            Attachment, Camera, CameraConvention, Light, LightActivation, ParticleEmitter,
            RibbonEmitter, TrailAtlasSlot, TrailControls,
        };
        match (self, value) {
            (
                Self::Light(
                    LightField::Color
                    | LightField::Intensity
                    | LightField::AttenuationStart
                    | LightField::AttenuationEnd,
                ),
                NodeFeature::Light(Light::Portable(_)),
            ) => true,

            (Self::Light(field), NodeFeature::Light(Light::AmbientDiffuse(v))) => match field {
                LightField::Color
                | LightField::Intensity
                | LightField::AmbientColor
                | LightField::AmbientIntensity
                | LightField::AttenuationStart
                | LightField::AttenuationEnd => true,
                _ => match &v.activation {
                    LightActivation::Visibility(control) => match field {
                        LightField::ShadowStart | LightField::ShadowEnd => {
                            control.shadow_range.is_some()
                        }
                        LightField::Quadratic | LightField::Linear | LightField::Damping => {
                            control.falloff.is_some()
                        }
                        _ => true,
                    },
                    LightActivation::Enabled(_) => false,
                },
            },
            (Self::Camera(field), NodeFeature::Camera(Camera::Aimed(v))) => match field {
                CameraField::Roll => true,
                _ => match &v.convention {
                    CameraConvention::Vertical(control) => match field {
                        CameraField::FocusDistance => control.focus_distance.is_some(),
                        CameraField::FocalLength => control.focal_length.is_some(),
                        CameraField::FStop => control.f_stop.is_some(),
                        _ => true,
                    },
                    CameraConvention::Diagonal(_) => false,
                },
            },
            (Self::Geometry(_), NodeFeature::GeometryAnimation(_))
            | (Self::Popcorn(_), NodeFeature::ParticleProgram(_)) => true,
            (Self::Particle(field), NodeFeature::Particle(ParticleEmitter::Simulated(value))) => {
                value.supports_resource_property(field)
            }
            (Self::Particle2(field), NodeFeature::Particle(ParticleEmitter::Simulated(value))) => {
                value.supports_atlas_property(field)
            }
            (Self::Ribbon(field), NodeFeature::Ribbon(RibbonEmitter::Trail(value))) => {
                match field {
                    RibbonField::TextureSlot => {
                        matches!(value.appearance.texture_slot, TrailAtlasSlot::Wide(_))
                    }
                    RibbonField::Visibility => {
                        matches!(value.controls, TrailControls::Gravity(_))
                    }
                    _ => true,
                }
            }
            (Self::Attachment(_), NodeFeature::Attachment(Attachment::External(value))) => {
                value.point.as_ref().is_some_and(|point| {
                    matches!(point.activation, AttachmentActivation::Visibility(_))
                })
            }
            _ => false,
        }
    }
}
