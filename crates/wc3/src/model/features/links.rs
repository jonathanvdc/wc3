//! Relationship traversal includes values not representable by a constrained profile.
use super::{Camera, Light, NodeFeature, ParticleEmitter, RibbonEmitter};
use crate::model::{FeatureKind, Link, ValidateLinks};
impl NodeFeature<'_> {
    /// Visits every graph relationship, including newly authored shared properties.
    pub(crate) fn visit_feature_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Light(Light::AmbientDiffuse(value)) => value.visit_links(visitor),
            Self::Camera(Camera::LookAt(value)) => value.visit_links(visitor),
            Self::Light(Light::SingleEmission(value)) => value.visit_links(visitor),
            Self::Camera(Camera::Node(value)) => value.visit_links(visitor),
            Self::Attachment(value) => value.visit_links(visitor),
            Self::Light(Light::World(value)) => {
                if value
                    .attenuation
                    .to_array()
                    .iter()
                    .any(|value| !value.is_finite())
                {
                    visitor(Link::InvalidValue);
                }
                if value.directional_terms.as_ref().is_some_and(|terms| {
                    terms.iter().any(|(direction, scalar)| {
                        !scalar.is_finite()
                            || direction.to_array().iter().any(|value| !value.is_finite())
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
                    || value
                        .attenuation
                        .to_array()
                        .iter()
                        .any(|value| !value.is_finite())
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
                        .to_array()
                        .iter()
                        .chain(&value.orientation.to_array())
                        .chain(&value.flicker.to_array())
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
            Self::Particle(ParticleEmitter::MdlxEmitter1(value)) => value.visit_links(visitor),
            Self::Particle(ParticleEmitter::MdlxEmitter2(value)) => value.visit_links(visitor),
            Self::Particle(ParticleEmitter::M2(value)) => value.visit_links(visitor),
            Self::Ribbon(RibbonEmitter::Mdlx(value)) => value.visit_links(visitor),
            Self::Ribbon(RibbonEmitter::M2(value)) => value.visit_links(visitor),
            Self::ParticleProgram(value) => {
                value.life_span.visit_links(visitor);
                value.emission_rate.visit_links(visitor);
                value.speed.visit_links(visitor);
                value.color.visit_links(visitor);
                value.opacity.visit_links(visitor);
                value.visibility.visit_links(visitor);
            }

            Self::Particle(ParticleEmitter::Resource(value))
            | Self::Ribbon(RibbonEmitter::Resource(value))
            | Self::FaceActor(value) => value.visit_links(visitor),
            Self::CollisionShape(_) => {}

            Self::Particle2(unavailable) | Self::PointLight(unavailable) => match *unavailable {},
        }
    }
}
