//! Relationship traversal includes values not representable by a constrained profile.
use super::{
    Camera, CameraConvention, EventSource, Light, LightActivation, NodeFeature, ParticleEmitter,
    RibbonEmitter,
};
use crate::model::{FeatureKind, Link, ValidateLinks};
impl NodeFeature<'_> {
    /// Visits every graph relationship, including newly authored shared properties.
    pub(crate) fn visit_feature_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Light(Light::AmbientDiffuse(value)) => {
                value.diffuse.color.visit_links(visitor);
                value.diffuse.intensity.visit_links(visitor);
                value.ambient.color.visit_links(visitor);
                value.ambient.intensity.visit_links(visitor);
                for parameter in &value.attenuation {
                    parameter.visit_links(visitor);
                }
                match &value.activation {
                    LightActivation::Enabled(value) => value.visit_links(visitor),
                    LightActivation::Visibility(value) => {
                        value.visibility.visit_links(visitor);
                        if let Some(parameters) = &value.shadow_range {
                            for parameter in parameters {
                                parameter.visit_links(visitor);
                            }
                        }
                        if let Some(parameters) = &value.falloff {
                            for parameter in parameters {
                                parameter.visit_links(visitor);
                            }
                        }
                    }
                }
            }
            Self::Camera(Camera::Aimed(value)) => {
                value.eye_offset.visit_links(visitor);
                value.target_offset.visit_links(visitor);
                value.field_of_view.visit_links(visitor);
                value.roll.visit_links(visitor);
                if let CameraConvention::Vertical(value) = &value.convention {
                    value.visibility.visit_links(visitor);
                    value.focus_distance.visit_links(visitor);
                    value.focal_length.visit_links(visitor);
                    value.f_stop.visit_links(visitor);
                }
            }
            Self::Light(Light::Portable(value)) => {
                value.color.visit_links(visitor);
                value.intensity.visit_links(visitor);
                for parameter in value.attenuation.iter().chain(&value.cone) {
                    parameter.visit_links(visitor);
                }
                if let Some(parameters) = &value.shadow_range {
                    for parameter in parameters {
                        parameter.visit_links(visitor);
                    }
                }
            }
            Self::Camera(Camera::Portable(value)) => {
                value.field_of_view.visit_links(visitor);
                value.target.visit_links(visitor);
                value.focus_distance.visit_links(visitor);
                value.focal_length.visit_links(visitor);
                value.f_stop.visit_links(visitor);
            }
            Self::Attachment(value) => value.visit_links(visitor),
            Self::Light(Light::World(value)) => {
                if value.attenuation.iter().any(|value| !value.is_finite()) {
                    visitor(Link::InvalidValue);
                }
                if value.directional_terms.as_ref().is_some_and(|terms| {
                    terms.iter().any(|(direction, scalar)| {
                        !scalar.is_finite() || direction.iter().any(|value| !value.is_finite())
                    })
                }) {
                    visitor(Link::InvalidValue);
                }
                value.color.visit_links(visitor);
                value.intensity.visit_links(visitor);
                if let Some(modern) = &value.modern {
                    modern.cookie.visit_links(visitor);
                    modern.doodad_set.visit_links(visitor);
                }
            }
            Self::Light(Light::Point(value)) => {
                if !value.intensity.is_finite()
                    || !value.scalar_control.is_finite()
                    || value.attenuation.iter().any(|value| !value.is_finite())
                {
                    visitor(Link::InvalidValue);
                }
            }
            Self::LightProgram(value) => {
                visitor(Link::TypedFeature {
                    id: value.light,
                    kind: FeatureKind::WmoPointLight,
                });
                value.cookie.visit_links(visitor);
                if !value.intensity.is_finite()
                    || value
                        .attenuation
                        .iter()
                        .chain(&value.orientation)
                        .chain(&value.flicker)
                        .any(|value| !value.is_finite())
                {
                    visitor(Link::InvalidValue);
                }
            }
            Self::GeometryAnimation(value) => {
                value.mesh.visit_links(visitor);
                value.opacity.visit_links(visitor);
                value.color.visit_links(visitor);
            }
            Self::Particle(ParticleEmitter::Simulated(value)) => value.visit_links(visitor),
            Self::Ribbon(RibbonEmitter::Trail(value)) => value.visit_links(visitor),
            Self::ParticleProgram(value) => {
                value.life_span.visit_links(visitor);
                value.emission_rate.visit_links(visitor);
                value.speed.visit_links(visitor);
                value.color.visit_links(visitor);
                value.opacity.visit_links(visitor);
                value.visibility.visit_links(visitor);
            }
            Self::Light(Light::Specular(value)) => value.visit_links(visitor),
            Self::Camera(Camera::Optical(value)) => value.visit_links(visitor),
            Self::Ribbon(RibbonEmitter::Programmed(value)) => value.visit_links(visitor),
            Self::Event(EventSource::Command(value)) => value.visit_links(visitor),
            Self::Extent(value) => value.visit_links(visitor),
            Self::ParticleCopy(value) => value.visit_links(visitor),
            Self::Projector(value) => value.visit_links(visitor),
            Self::Force(value) => value.visit_links(visitor),
            Self::Warp(value) => value.visit_links(visitor),
            Self::HitTest(value) => value.visit_links(visitor),
            Self::AttachmentVolume(value) => value.visit_links(visitor),
            Self::Billboard(value) => value.visit_links(visitor),
            Self::Turret(value) => value.visit_links(visitor),
            Self::IkJoint(value) => value.visit_links(visitor),
            Self::IkTwoJoint(value) => value.visit_links(visitor),
            Self::IkCcd(value) => value.visit_links(visitor),
            Self::OneBoneSolver(value) => value.visit_links(visitor),
            Self::ShadowBox(value) => value.visit_links(visitor),
            Self::ViewVolume(value) => value.visit_links(visitor),
            Self::Particle(ParticleEmitter::Resource(value))
            | Self::Ribbon(RibbonEmitter::Resource(value))
            | Self::FaceActor(value) => value.visit_links(visitor),
            Self::Event(EventSource::Named(_) | EventSource::Identifier(_))
            | Self::CollisionShape(_) => {}

            Self::AtlasParticles(unavailable) | Self::PointLight(unavailable) => {
                match *unavailable {}
            }
        }
    }
}
