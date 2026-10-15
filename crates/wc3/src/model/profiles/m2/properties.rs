//! Typed World of Warcraft destinations, independent of native track descriptors.
use crate::model::profiles::M2;
use crate::model::AttachmentActivation;
use crate::model::NodeFeature;
use crate::model::{
    FeatureId, FeatureProperty, MaterialId, OwnerId, PropertyIdentityKey, PropertyKey,
    PropertyKeyRef, PropertyPayload, Quaternion,
};
use std::any::TypeId;
/// Camera destinations on an attached camera feature.
pub mod camera {
    use super::*;
    use super::{FeatureId, PropertyKey};
    /// Animated camera position.
    pub const POSITION: PropertyKey<FeatureId, [f32; 3], M2> =
        PropertyKey::new(Target::Camera(CameraField::Position));
    /// Animated camera target.
    pub const TARGET: PropertyKey<FeatureId, [f32; 3], M2> =
        PropertyKey::new(Target::Camera(CameraField::Target));
    /// Animated camera roll.
    pub const ROLL: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Camera(CameraField::Roll));
    /// Animated field of view.
    pub const FIELD_OF_VIEW: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Camera(CameraField::FieldOfView));
}
/// Animated material color.
pub const COLOR: PropertyKey<MaterialId, [f32; 3], M2> = PropertyKey::new(Target::Color);
/// Animated material opacity.
pub const OPACITY: PropertyKey<MaterialId, f32, M2> = PropertyKey::new(Target::Opacity);

/// Animated attachment properties on an attached feature.
pub mod attachment {
    use super::*;
    use super::{FeatureId, PropertyKey};
    /// Animated enabled.
    pub const ENABLED: PropertyKey<FeatureId, bool, M2> =
        PropertyKey::new(Target::Attachment(AttachmentField::Enabled));
}

/// Animated light properties on an attached feature.
pub mod light {
    use super::*;
    use super::{FeatureId, PropertyKey};
    /// Animated ambient color.
    pub const AMBIENT_COLOR: PropertyKey<FeatureId, [f32; 3], M2> =
        PropertyKey::new(Target::Light(LightField::AmbientColor));
    /// Animated ambient intensity.
    pub const AMBIENT_INTENSITY: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Light(LightField::AmbientIntensity));
    /// Animated diffuse color.
    pub const DIFFUSE_COLOR: PropertyKey<FeatureId, [f32; 3], M2> =
        PropertyKey::new(Target::Light(LightField::DiffuseColor));
    /// Animated diffuse intensity.
    pub const DIFFUSE_INTENSITY: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Light(LightField::DiffuseIntensity));
    /// Animated attenuation start.
    pub const ATTENUATION_START: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Light(LightField::AttenuationStart));
    /// Animated attenuation end.
    pub const ATTENUATION_END: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Light(LightField::AttenuationEnd));
    /// Animated enabled.
    pub const ENABLED: PropertyKey<FeatureId, bool, M2> =
        PropertyKey::new(Target::Light(LightField::Enabled));
}

/// Animated ribbon properties on an attached feature.
pub mod ribbon {
    use super::*;
    use super::{FeatureId, PropertyKey};
    /// Animated color.
    pub const COLOR: PropertyKey<FeatureId, [f32; 3], M2> =
        PropertyKey::new(Target::Ribbon(RibbonField::Color));
    /// Animated alpha.
    pub const ALPHA: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Ribbon(RibbonField::Alpha));
    /// Animated height above.
    pub const HEIGHT_ABOVE: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Ribbon(RibbonField::HeightAbove));
    /// Animated height below.
    pub const HEIGHT_BELOW: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Ribbon(RibbonField::HeightBelow));
    /// Animated texture slot.
    pub const TEXTURE_SLOT: PropertyKey<FeatureId, u16, M2> =
        PropertyKey::new(Target::Ribbon(RibbonField::TextureSlot));
    /// Animated enabled.
    pub const ENABLED: PropertyKey<FeatureId, bool, M2> =
        PropertyKey::new(Target::Ribbon(RibbonField::Enabled));
}

/// Animated particle properties on an attached feature.
pub mod particle {
    use super::*;
    use super::{FeatureId, PropertyKey};
    /// Animated speed.
    pub const SPEED: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Particle(ParticleField::Speed));
    /// Animated speed variation.
    pub const SPEED_VARIATION: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Particle(ParticleField::SpeedVariation));
    /// Animated vertical range.
    pub const VERTICAL_RANGE: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Particle(ParticleField::VerticalRange));
    /// Animated horizontal range.
    pub const HORIZONTAL_RANGE: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Particle(ParticleField::HorizontalRange));
    /// Animated lifespan.
    pub const LIFESPAN: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Particle(ParticleField::Lifespan));
    /// Animated rate.
    pub const RATE: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Particle(ParticleField::Rate));
    /// Animated area width.
    pub const AREA_WIDTH: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Particle(ParticleField::AreaWidth));
    /// Animated area length.
    pub const AREA_LENGTH: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Particle(ParticleField::AreaLength));
    /// Animated z source.
    pub const Z_SOURCE: PropertyKey<FeatureId, f32, M2> =
        PropertyKey::new(Target::Particle(ParticleField::ZSource));
    /// Animated enabled.
    pub const ENABLED: PropertyKey<FeatureId, bool, M2> =
        PropertyKey::new(Target::Particle(ParticleField::Enabled));
    /// Animated gravity.
    pub const GRAVITY: PropertyKey<FeatureId, [f32; 3], M2> =
        PropertyKey::new(Target::Particle(ParticleField::Gravity));
}

/// Material texture-unit destinations selected by logical slot.
#[derive(Clone, Copy, Debug)]
pub struct TextureUnit(pub usize);
impl TextureUnit {
    /// Animated sampling weight.
    pub fn weight(self) -> PropertyKey<MaterialId, f32, M2> {
        PropertyKey::new(Target::TextureUnit {
            index: self.0,
            field: TextureUnitField::Weight,
        })
    }
    /// Animated coordinate translation.
    pub fn translation(self) -> PropertyKey<MaterialId, [f32; 3], M2> {
        PropertyKey::new(Target::TextureUnit {
            index: self.0,
            field: TextureUnitField::Translation,
        })
    }
    /// Animated coordinate rotation.
    pub fn rotation(self) -> PropertyKey<MaterialId, Quaternion, M2> {
        PropertyKey::new(Target::TextureUnit {
            index: self.0,
            field: TextureUnitField::Rotation,
        })
    }
    /// Animated coordinate scale.
    pub fn scale(self) -> PropertyKey<MaterialId, [f32; 3], M2> {
        PropertyKey::new(Target::TextureUnit {
            index: self.0,
            field: TextureUnitField::Scale,
        })
    }
}

/// Structural destinations owned by this property vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    /// Camera properties.
    Camera(CameraField),
    /// Attachment properties.
    Attachment(AttachmentField),
    /// Light properties.
    Light(LightField),
    /// Ribbon properties.
    Ribbon(RibbonField),
    /// Particle properties.
    Particle(ParticleField),
    /// Animated color.
    Color,
    /// Animated opacity.
    Opacity,
    /// Indexed textureunit property.
    TextureUnit {
        /// Owner-local element ordinal.
        index: usize,
        /// Field of the selected element.
        field: TextureUnitField,
    },
}
/// Structural camera field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CameraField {
    /// Animated position.
    Position,
    /// Animated target.
    Target,
    /// Animated roll.
    Roll,
    /// Animated field of view.
    FieldOfView,
}

/// Structural attachment field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttachmentField {
    /// Animated enabled.
    Enabled,
}

/// Structural light field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LightField {
    /// Animated ambient color.
    AmbientColor,
    /// Animated ambient intensity.
    AmbientIntensity,
    /// Animated diffuse color.
    DiffuseColor,
    /// Animated diffuse intensity.
    DiffuseIntensity,
    /// Animated attenuation start.
    AttenuationStart,
    /// Animated attenuation end.
    AttenuationEnd,
    /// Animated enabled.
    Enabled,
}

/// Structural ribbon field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RibbonField {
    /// Animated color.
    Color,
    /// Animated alpha.
    Alpha,
    /// Animated height above.
    HeightAbove,
    /// Animated height below.
    HeightBelow,
    /// Animated texture slot.
    TextureSlot,
    /// Animated enabled.
    Enabled,
}

/// Structural particle field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParticleField {
    /// Animated speed.
    Speed,
    /// Animated speed variation.
    SpeedVariation,
    /// Animated vertical range.
    VerticalRange,
    /// Animated horizontal range.
    HorizontalRange,
    /// Animated lifespan.
    Lifespan,
    /// Animated rate.
    Rate,
    /// Animated area width.
    AreaWidth,
    /// Animated area length.
    AreaLength,
    /// Animated z source.
    ZSource,
    /// Animated enabled.
    Enabled,
    /// Animated gravity.
    Gravity,
}

/// Structural textureunit field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextureUnitField {
    /// Animated weight.
    Weight,
    /// Animated translation.
    Translation,
    /// Animated rotation.
    Rotation,
    /// Animated scale.
    Scale,
}

impl Target {
    fn shared_feature_property(&self) -> Option<&'static FeatureProperty> {
        match self {
            Self::Light(LightField::DiffuseColor) => Some(&FeatureProperty::LightDiffuseColor),
            Self::Light(LightField::DiffuseIntensity) => {
                Some(&FeatureProperty::LightDiffuseIntensity)
            }
            Self::Light(LightField::AmbientColor) => Some(&FeatureProperty::LightAmbientColor),
            Self::Light(LightField::AmbientIntensity) => {
                Some(&FeatureProperty::LightAmbientIntensity)
            }
            Self::Light(LightField::AttenuationStart) => {
                Some(&FeatureProperty::LightAttenuationStart)
            }
            Self::Light(LightField::AttenuationEnd) => Some(&FeatureProperty::LightAttenuationEnd),
            Self::Camera(CameraField::Roll) => Some(&FeatureProperty::CameraRoll),
            Self::Ribbon(RibbonField::Color) => Some(&FeatureProperty::TrailColor),
            Self::Ribbon(RibbonField::Alpha) => Some(&FeatureProperty::TrailOpacity),
            Self::Ribbon(RibbonField::HeightAbove) => Some(&FeatureProperty::TrailHeightAbove),
            Self::Ribbon(RibbonField::HeightBelow) => Some(&FeatureProperty::TrailHeightBelow),
            Self::Particle(ParticleField::Rate) => Some(&FeatureProperty::ParticleContinuousRate),
            Self::Particle(ParticleField::Lifespan) => Some(&FeatureProperty::ParticleLifetime),
            Self::Particle(ParticleField::Speed) => Some(&FeatureProperty::ParticleSpeed),

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
                FeatureProperty::LightDiffuseColor => Self::Light(LightField::DiffuseColor),
                FeatureProperty::LightDiffuseIntensity => Self::Light(LightField::DiffuseIntensity),
                FeatureProperty::LightAmbientColor => Self::Light(LightField::AmbientColor),
                FeatureProperty::LightAmbientIntensity => Self::Light(LightField::AmbientIntensity),
                FeatureProperty::LightAttenuationStart => Self::Light(LightField::AttenuationStart),
                FeatureProperty::LightAttenuationEnd => Self::Light(LightField::AttenuationEnd),
                FeatureProperty::CameraRoll => Self::Camera(CameraField::Roll),
                FeatureProperty::TrailColor => Self::Ribbon(RibbonField::Color),
                FeatureProperty::TrailOpacity => Self::Ribbon(RibbonField::Alpha),
                FeatureProperty::TrailHeightAbove => Self::Ribbon(RibbonField::HeightAbove),
                FeatureProperty::TrailHeightBelow => Self::Ribbon(RibbonField::HeightBelow),
                FeatureProperty::ParticleContinuousRate => Self::Particle(ParticleField::Rate),
                FeatureProperty::ParticleLifetime => Self::Particle(ParticleField::Lifespan),
                FeatureProperty::ParticleSpeed => Self::Particle(ParticleField::Speed),
                _ => return None,
            });
        }
        key.downcast_ref::<Self>().cloned()
    }

    fn value_type(&self) -> Option<TypeId> {
        Some(match self {
            Target::Camera(CameraField::Position) => TypeId::of::<[f32; 3]>(),
            Target::Camera(CameraField::Target) => TypeId::of::<[f32; 3]>(),
            Target::Camera(CameraField::Roll) => TypeId::of::<f32>(),
            Target::Camera(CameraField::FieldOfView) => TypeId::of::<f32>(),
            Target::Attachment(AttachmentField::Enabled) => TypeId::of::<bool>(),
            Target::Light(LightField::AmbientColor) => TypeId::of::<[f32; 3]>(),
            Target::Light(LightField::AmbientIntensity) => TypeId::of::<f32>(),
            Target::Light(LightField::DiffuseColor) => TypeId::of::<[f32; 3]>(),
            Target::Light(LightField::DiffuseIntensity) => TypeId::of::<f32>(),
            Target::Light(LightField::AttenuationStart) => TypeId::of::<f32>(),
            Target::Light(LightField::AttenuationEnd) => TypeId::of::<f32>(),
            Target::Light(LightField::Enabled) => TypeId::of::<bool>(),
            Target::Ribbon(RibbonField::Color) => TypeId::of::<[f32; 3]>(),
            Target::Ribbon(RibbonField::Alpha) => TypeId::of::<f32>(),
            Target::Ribbon(RibbonField::HeightAbove) => TypeId::of::<f32>(),
            Target::Ribbon(RibbonField::HeightBelow) => TypeId::of::<f32>(),
            Target::Ribbon(RibbonField::TextureSlot) => TypeId::of::<u16>(),
            Target::Ribbon(RibbonField::Enabled) => TypeId::of::<bool>(),
            Target::Particle(ParticleField::Speed) => TypeId::of::<f32>(),
            Target::Particle(ParticleField::SpeedVariation) => TypeId::of::<f32>(),
            Target::Particle(ParticleField::VerticalRange) => TypeId::of::<f32>(),
            Target::Particle(ParticleField::HorizontalRange) => TypeId::of::<f32>(),
            Target::Particle(ParticleField::Lifespan) => TypeId::of::<f32>(),
            Target::Particle(ParticleField::Rate) => TypeId::of::<f32>(),
            Target::Particle(ParticleField::AreaWidth) => TypeId::of::<f32>(),
            Target::Particle(ParticleField::AreaLength) => TypeId::of::<f32>(),
            Target::Particle(ParticleField::ZSource) => TypeId::of::<f32>(),
            Target::Particle(ParticleField::Enabled) => TypeId::of::<bool>(),
            Target::Particle(ParticleField::Gravity) => TypeId::of::<[f32; 3]>(),
            Target::Color => TypeId::of::<[f32; 3]>(),
            Target::Opacity => TypeId::of::<f32>(),
            Target::TextureUnit {
                field: TextureUnitField::Weight,
                ..
            } => TypeId::of::<f32>(),
            Target::TextureUnit {
                field: TextureUnitField::Translation,
                ..
            } => TypeId::of::<[f32; 3]>(),
            Target::TextureUnit {
                field: TextureUnitField::Rotation,
                ..
            } => TypeId::of::<Quaternion>(),
            Target::TextureUnit {
                field: TextureUnitField::Scale,
                ..
            } => TypeId::of::<[f32; 3]>(),
        })
    }
    fn accepts_owner(&self, owner: OwnerId) -> bool {
        match self {
            Target::Camera(CameraField::Position) => matches!(owner, OwnerId::Feature(_)),
            Target::Camera(CameraField::Target) => matches!(owner, OwnerId::Feature(_)),
            Target::Camera(CameraField::Roll) => matches!(owner, OwnerId::Feature(_)),
            Target::Camera(CameraField::FieldOfView) => matches!(owner, OwnerId::Feature(_)),
            Target::Attachment(AttachmentField::Enabled) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::AmbientColor) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::AmbientIntensity) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::DiffuseColor) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::DiffuseIntensity) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::AttenuationStart) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::AttenuationEnd) => matches!(owner, OwnerId::Feature(_)),
            Target::Light(LightField::Enabled) => matches!(owner, OwnerId::Feature(_)),
            Target::Ribbon(RibbonField::Color) => matches!(owner, OwnerId::Feature(_)),
            Target::Ribbon(RibbonField::Alpha) => matches!(owner, OwnerId::Feature(_)),
            Target::Ribbon(RibbonField::HeightAbove) => matches!(owner, OwnerId::Feature(_)),
            Target::Ribbon(RibbonField::HeightBelow) => matches!(owner, OwnerId::Feature(_)),
            Target::Ribbon(RibbonField::TextureSlot) => matches!(owner, OwnerId::Feature(_)),
            Target::Ribbon(RibbonField::Enabled) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::Speed) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::SpeedVariation) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::VerticalRange) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::HorizontalRange) => {
                matches!(owner, OwnerId::Feature(_))
            }
            Target::Particle(ParticleField::Lifespan) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::Rate) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::AreaWidth) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::AreaLength) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::ZSource) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::Enabled) => matches!(owner, OwnerId::Feature(_)),
            Target::Particle(ParticleField::Gravity) => matches!(owner, OwnerId::Feature(_)),
            Target::Color => matches!(owner, OwnerId::Material(_)),
            Target::Opacity => matches!(owner, OwnerId::Material(_)),
            Target::TextureUnit {
                field: TextureUnitField::Weight,
                ..
            } => matches!(owner, OwnerId::Material(_)),
            Target::TextureUnit {
                field: TextureUnitField::Translation,
                ..
            } => matches!(owner, OwnerId::Material(_)),
            Target::TextureUnit {
                field: TextureUnitField::Rotation,
                ..
            } => matches!(owner, OwnerId::Material(_)),
            Target::TextureUnit {
                field: TextureUnitField::Scale,
                ..
            } => matches!(owner, OwnerId::Material(_)),
        }
    }
}

impl Target {
    pub(crate) fn supports_feature(&self, value: &super::ActorKind<'_>) -> bool {
        matches!(
            (self, value),
            (Self::Camera(_), super::ActorKind::Camera(_))
                | (Self::Light(_), super::ActorKind::Light(_))
                | (Self::Attachment(_), super::ActorKind::Attachment(_))
                | (Self::Ribbon(_), super::ActorKind::Ribbon(_))
                | (Self::Particle(_), super::ActorKind::Particle(_))
        )
    }
}

impl Target {
    pub(crate) fn supports_universal_feature(&self, value: &NodeFeature<'_>) -> bool {
        use crate::model::{
            Attachment, Camera, CameraConvention, Light, LightActivation, ParticleEmitter,
            RibbonEmitter, TrailAtlasSlot, TrailControls,
        };
        match (self, value) {
            (Self::Light(field), NodeFeature::Light(Light::AmbientDiffuse(v))) => match field {
                LightField::Enabled => matches!(v.activation, LightActivation::Enabled(_)),
                _ => true,
            },
            (Self::Camera(field), NodeFeature::Camera(Camera::Aimed(v))) => {
                matches!(field, CameraField::Roll)
                    || matches!(v.convention, CameraConvention::Diagonal(_))
            }
            (Self::Attachment(_), NodeFeature::Attachment(Attachment::Point(value))) => {
                matches!(value.activation, AttachmentActivation::Enabled(_))
            }
            (Self::Ribbon(field), NodeFeature::Ribbon(RibbonEmitter::Trail(value))) => {
                match field {
                    RibbonField::TextureSlot => {
                        matches!(value.appearance.texture_slot, TrailAtlasSlot::Compact(_))
                    }
                    RibbonField::Enabled => {
                        matches!(value.controls, TrailControls::EmissionAngle(_))
                    }
                    _ => true,
                }
            }
            (Self::Particle(field), NodeFeature::Particle(ParticleEmitter::Simulated(value))) => {
                value.supports_age_driven_property(field)
            }
            _ => false,
        }
    }
}
