//! Shared animation destinations with identical value and spatial semantics.
//!
//! Positions, camera projections, and activation stay in their native vocabularies
//! when their units, reference frames, or value domains differ.
use super::profiles::{Universal, UniversalProperty};
use super::{FeatureId, OwnerId, PropertyKey, PropertyPayload};
use std::any::TypeId;

/// A shared destination on an attached behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FeatureProperty {
    /// Visibility of content mounted at an exposed socket.
    SocketVisibility,
    /// Diffuse illumination color.
    LightDiffuseColor,
    /// Diffuse illumination multiplier.
    LightDiffuseIntensity,
    /// Ambient illumination color.
    LightAmbientColor,
    /// Ambient illumination multiplier.
    LightAmbientIntensity,
    /// Distance at which attenuation begins.
    LightAttenuationStart,
    /// Distance at which attenuation ends.
    LightAttenuationEnd,
    /// Camera roll in radians.
    CameraRoll,
    /// Vertical camera field of view in radians.
    CameraVerticalFieldOfView,
    /// RGB tint of generated trail sections.
    TrailColor,
    /// Opacity of generated trail sections.
    TrailOpacity,
    /// Cross-section extent above the trail center.
    TrailHeightAbove,
    /// Cross-section extent below the trail center.
    TrailHeightBelow,
    /// Continuous particle emission rate, in particles per second.
    ParticleContinuousRate,
    /// Particle lifetime in seconds.
    ParticleLifetime,
    /// Initial particle speed in model units per second.
    ParticleSpeed,
}
impl PropertyPayload for FeatureProperty {
    fn value_type(&self) -> Option<TypeId> {
        Some(match self {
            Self::LightDiffuseColor | Self::LightAmbientColor | Self::TrailColor => {
                TypeId::of::<[f32; 3]>()
            }
            _ => TypeId::of::<f32>(),
        })
    }
    fn accepts_owner(&self, owner: OwnerId) -> bool {
        matches!(owner, OwnerId::Feature(_))
    }
}
/// Shared light animation destinations.
pub mod light {
    use super::*;
    /// Diffuse illumination color.
    pub const DIFFUSE_COLOR: PropertyKey<FeatureId, [f32; 3], Universal> = PropertyKey::new(
        UniversalProperty::Feature(FeatureProperty::LightDiffuseColor),
    );
    /// Diffuse illumination intensity.
    pub const DIFFUSE_INTENSITY: PropertyKey<FeatureId, f32, Universal> = PropertyKey::new(
        UniversalProperty::Feature(FeatureProperty::LightDiffuseIntensity),
    );
    /// Ambient illumination color.
    pub const AMBIENT_COLOR: PropertyKey<FeatureId, [f32; 3], Universal> = PropertyKey::new(
        UniversalProperty::Feature(FeatureProperty::LightAmbientColor),
    );
    /// Ambient illumination intensity.
    pub const AMBIENT_INTENSITY: PropertyKey<FeatureId, f32, Universal> = PropertyKey::new(
        UniversalProperty::Feature(FeatureProperty::LightAmbientIntensity),
    );
    /// Beginning of attenuation range.
    pub const ATTENUATION_START: PropertyKey<FeatureId, f32, Universal> = PropertyKey::new(
        UniversalProperty::Feature(FeatureProperty::LightAttenuationStart),
    );
    /// End of attenuation range.
    pub const ATTENUATION_END: PropertyKey<FeatureId, f32, Universal> = PropertyKey::new(
        UniversalProperty::Feature(FeatureProperty::LightAttenuationEnd),
    );
}
/// Shared camera animation destinations.
pub mod camera {
    use super::*;
    /// Vertical field of view in radians, independently animated in Universal.
    pub const VERTICAL_FIELD_OF_VIEW: PropertyKey<FeatureId, f32, Universal> = PropertyKey::new(
        UniversalProperty::Feature(FeatureProperty::CameraVerticalFieldOfView),
    );
    /// Roll in radians about the viewing axis.
    pub const ROLL: PropertyKey<FeatureId, f32, Universal> =
        PropertyKey::new(UniversalProperty::Feature(FeatureProperty::CameraRoll));
}

/// Shared section-trail animation destinations.
pub mod trail {
    use super::*;
    /// RGB tint.
    pub const COLOR: PropertyKey<FeatureId, [f32; 3], Universal> =
        PropertyKey::new(UniversalProperty::Feature(FeatureProperty::TrailColor));
    /// Opacity multiplier.
    pub const OPACITY: PropertyKey<FeatureId, f32, Universal> =
        PropertyKey::new(UniversalProperty::Feature(FeatureProperty::TrailOpacity));
    /// Extent above the center line.
    pub const HEIGHT_ABOVE: PropertyKey<FeatureId, f32, Universal> = PropertyKey::new(
        UniversalProperty::Feature(FeatureProperty::TrailHeightAbove),
    );
    /// Extent below the center line.
    pub const HEIGHT_BELOW: PropertyKey<FeatureId, f32, Universal> = PropertyKey::new(
        UniversalProperty::Feature(FeatureProperty::TrailHeightBelow),
    );
}
/// Shared particle animation destinations with established units.
pub mod particle {
    use super::*;
    /// Continuous particle emission rate.
    pub const CONTINUOUS_RATE: PropertyKey<FeatureId, f32, Universal> = PropertyKey::new(
        UniversalProperty::Feature(FeatureProperty::ParticleContinuousRate),
    );
    /// Lifetime in seconds.
    pub const LIFETIME: PropertyKey<FeatureId, f32, Universal> = PropertyKey::new(
        UniversalProperty::Feature(FeatureProperty::ParticleLifetime),
    );
    /// Initial speed in model units per second.
    pub const SPEED: PropertyKey<FeatureId, f32, Universal> =
        PropertyKey::new(UniversalProperty::Feature(FeatureProperty::ParticleSpeed));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::profiles::{m2, mdlx, Mdlx, Narrows, UniversalProperty, M2};

    #[test]
    fn illumination_selectors_share_identity_and_narrow_exactly() {
        let wc3 = mdlx::properties::Target::Light(mdlx::properties::LightField::Color);
        let wow = m2::properties::Target::Light(m2::properties::LightField::DiffuseColor);
        let shared = FeatureProperty::LightDiffuseColor;
        assert_eq!(wc3.identity(), wow.identity());
        assert_eq!(wow.identity(), shared.identity());
        assert_eq!(
            <Mdlx as Narrows<Universal>>::property(UniversalProperty::Feature(shared)).unwrap(),
            wc3
        );
        assert_eq!(
            <M2 as Narrows<Universal>>::property(UniversalProperty::Feature(shared)).unwrap(),
            wow
        );
    }

    #[test]
    fn different_camera_reference_frames_and_activation_domains_stay_distinct() {
        let wc3 = mdlx::properties::Target::Camera(mdlx::properties::CameraField::EyeOffset);
        let wow = m2::properties::Target::Camera(m2::properties::CameraField::Position);
        assert_ne!(wc3.identity(), wow.identity());
        let wc3 = mdlx::properties::Target::Light(mdlx::properties::LightField::Visibility);
        let wow = m2::properties::Target::Light(m2::properties::LightField::Enabled);
        assert_ne!(wc3.identity(), wow.identity());
    }

    #[test]
    fn portable_lights_accept_shared_emission_but_not_ambient_channels() {
        use crate::model::profiles::FeaturePayload;
        use crate::model::{Light, NodeFeature};
        let feature = NodeFeature::Light(Light::Portable(Box::default()));
        for property in [
            FeatureProperty::LightDiffuseColor,
            FeatureProperty::LightDiffuseIntensity,
            FeatureProperty::LightAttenuationStart,
            FeatureProperty::LightAttenuationEnd,
        ] {
            assert!(feature.supports_property(&property.identity().to_owned()));
        }
        assert!(
            !feature.supports_property(&FeatureProperty::LightAmbientColor.identity().to_owned())
        );
    }

    #[test]
    fn common_fields_recover_their_original_native_selector() {
        use mdlx::properties::{CameraField, LightField, Target};
        let fields = [
            Target::Light(LightField::Color),
            Target::Light(LightField::Intensity),
            Target::Light(LightField::AmbientColor),
            Target::Light(LightField::AmbientIntensity),
            Target::Light(LightField::AttenuationStart),
            Target::Light(LightField::AttenuationEnd),
            Target::Camera(CameraField::Roll),
        ];
        for field in fields {
            assert_eq!(
                Target::from_identity(&field.identity().to_owned()),
                Some(field)
            );
        }
    }
}

#[cfg(test)]
mod m3_tests {
    use super::*;
    use crate::model::profiles::{m3, Narrows, UniversalProperty, M3};

    #[test]
    fn nested_light_selectors_preserve_identity_across_profiles() {
        use m3::properties::{Field, FieldPath, Target};
        let native = Target::Field(FieldPath::root(Field::Light).child(Field::DiffuseColor));
        assert_eq!(
            native.identity(),
            FeatureProperty::LightDiffuseColor.identity()
        );
        assert_eq!(
            <M3 as Narrows<Universal>>::property(UniversalProperty::Feature(
                FeatureProperty::LightDiffuseColor
            ))
            .unwrap(),
            native
        );
        let specular = Target::Field(FieldPath::root(Field::Light).child(Field::SpecularColor));
        assert_ne!(native.identity(), specular.identity());
    }
}

/// Socket content animation destinations.
pub mod socket {
    use super::*;
    /// Fractional visibility of mounted content.
    pub const VISIBILITY: PropertyKey<FeatureId, f32, Universal> = PropertyKey::new(
        UniversalProperty::Feature(FeatureProperty::SocketVisibility),
    );
}
