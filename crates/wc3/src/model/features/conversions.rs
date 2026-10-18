//! Lossless movement between constrained and universal feature vocabularies.
use super::{
    Attachment, Camera, EventSource, Light, NodeFeature, ParticleDetails, ParticleEmitter,
    RibbonEmitter,
};
use super::{ChildModel, ParticleDefinition, ProgrammedRibbon, Socket, TrailEmitter};
use crate::model::profiles::{m2, m3, mdlx, wmo};
use crate::model::scene;
use crate::model::PropertyValues;
impl<'a> FromFeature<mdlx::Node<'a>> for NodeFeature<'a> {
    fn from_feature(value: mdlx::Node<'a>, values: &mut PropertyValues) -> Self {
        match value {
            mdlx::Node::GeometryAnimation(value) => Self::GeometryAnimation(value),
            mdlx::Node::Particle(value) => Self::Particle(ParticleEmitter::Simulated(Box::new(
                FromFeature::from_feature(value, values),
            ))),
            mdlx::Node::AtlasParticles(value) => Self::Particle(ParticleEmitter::Simulated(
                Box::new(FromFeature::from_feature(value, values)),
            )),
            mdlx::Node::Ribbon(value) => Self::Ribbon(RibbonEmitter::Trail(Box::new(
                FromFeature::from_feature(value, values),
            ))),
            mdlx::Node::ParticleProgram(value) => Self::ParticleProgram(Box::new(value)),
            mdlx::Node::Light(value) => Self::Light(Light::AmbientDiffuse(Box::new(
                FromFeature::from_feature(value, values),
            ))),
            mdlx::Node::Camera(value) => Self::Camera(Camera::Aimed(Box::new(
                FromFeature::from_feature(value, values),
            ))),
            mdlx::Node::Attachment(value) => {
                Self::Attachment(Attachment::Child(FromFeature::from_feature(value, values)))
            }
            mdlx::Node::Event(value) => Self::Event(EventSource::Named(value)),
            mdlx::Node::CollisionShape(value) => Self::CollisionShape(value),
        }
    }
}
impl<'a> TryFromFeature<NodeFeature<'a>> for mdlx::Node<'a> {
    type Error = Box<NodeFeature<'a>>;
    fn try_from_feature(
        value: NodeFeature<'a>,
        values: &PropertyValues,
    ) -> Result<Self, Self::Error> {
        match value {
            NodeFeature::GeometryAnimation(value) => Ok(Self::GeometryAnimation(value)),
            NodeFeature::Particle(ParticleEmitter::Simulated(value)) => match value.details {
                ParticleDetails::Model(_) => match TryFromFeature::try_from_feature(*value, values)
                {
                    Ok(value) => Ok(Self::Particle(value)),
                    Err(value) => Err(Box::new(NodeFeature::Particle(ParticleEmitter::Simulated(
                        value,
                    )))),
                },
                ParticleDetails::Atlas(_) => match TryFromFeature::try_from_feature(*value, values)
                {
                    Ok(value) => Ok(Self::AtlasParticles(value)),
                    Err(value) => Err(Box::new(NodeFeature::Particle(ParticleEmitter::Simulated(
                        value,
                    )))),
                },
                _ => Err(Box::new(NodeFeature::Particle(ParticleEmitter::Simulated(
                    value,
                )))),
            },
            NodeFeature::Ribbon(RibbonEmitter::Trail(value)) => {
                match TryFromFeature::try_from_feature(*value, values) {
                    Ok(value) => Ok(Self::Ribbon(value)),
                    Err(value) => Err(Box::new(NodeFeature::Ribbon(RibbonEmitter::Trail(value)))),
                }
            }
            NodeFeature::ParticleProgram(value) => Ok(Self::ParticleProgram(*value)),
            NodeFeature::Light(Light::AmbientDiffuse(value)) => {
                match TryFromFeature::try_from_feature(*value, values) {
                    Ok(value) => Ok(Self::Light(value)),
                    Err(value) => Err(Box::new(NodeFeature::Light(Light::AmbientDiffuse(value)))),
                }
            }
            NodeFeature::Camera(Camera::Aimed(value)) => {
                match TryFromFeature::try_from_feature(*value, values) {
                    Ok(value) => Ok(Self::Camera(value)),
                    Err(value) => Err(Box::new(NodeFeature::Camera(Camera::Aimed(value)))),
                }
            }
            NodeFeature::Attachment(Attachment::Child(value)) => {
                match TryFromFeature::try_from_feature(value, values) {
                    Ok(value) => Ok(Self::Attachment(value)),
                    Err(value) => Err(Box::new(NodeFeature::Attachment(Attachment::Child(*value)))),
                }
            }
            NodeFeature::Event(EventSource::Named(value)) => Ok(Self::Event(value)),
            NodeFeature::CollisionShape(value) => Ok(Self::CollisionShape(value)),
            value => Err(Box::new(value)),
        }
    }
}
impl<'a> FromFeature<m2::ActorKind<'a>> for NodeFeature<'a> {
    fn from_feature(value: m2::ActorKind<'a>, values: &mut PropertyValues) -> Self {
        match value {
            m2::ActorKind::Attachment(value) => {
                Self::Attachment(Attachment::Socket(FromFeature::from_feature(value, values)))
            }
            m2::ActorKind::Light(value) => Self::Light(Light::AmbientDiffuse(Box::new(
                FromFeature::from_feature(value, values),
            ))),
            m2::ActorKind::Camera(value) => Self::Camera(Camera::Aimed(Box::new(
                FromFeature::from_feature(value, values),
            ))),
            m2::ActorKind::Ribbon(value) => Self::Ribbon(RibbonEmitter::Trail(Box::new(
                FromFeature::from_feature(*value, values),
            ))),
            m2::ActorKind::Particle(value) => Self::Particle(ParticleEmitter::Simulated(Box::new(
                FromFeature::from_feature(*value, values),
            ))),
            m2::ActorKind::Event(value) => Self::Event(EventSource::Identifier(value)),
        }
    }
}
impl<'a> TryFromFeature<NodeFeature<'a>> for m2::ActorKind<'a> {
    type Error = Box<NodeFeature<'a>>;
    fn try_from_feature(
        value: NodeFeature<'a>,
        values: &PropertyValues,
    ) -> Result<Self, Self::Error> {
        match value {
            NodeFeature::Attachment(Attachment::Socket(value)) => {
                match TryFromFeature::try_from_feature(value, values) {
                    Ok(value) => Ok(Self::Attachment(value)),
                    Err(value) => Err(Box::new(NodeFeature::Attachment(Attachment::Socket(
                        *value,
                    )))),
                }
            }
            NodeFeature::Light(Light::AmbientDiffuse(value)) => {
                match TryFromFeature::try_from_feature(*value, values) {
                    Ok(value) => Ok(Self::Light(value)),
                    Err(value) => Err(Box::new(NodeFeature::Light(Light::AmbientDiffuse(value)))),
                }
            }
            NodeFeature::Camera(Camera::Aimed(value)) => {
                match TryFromFeature::try_from_feature(*value, values) {
                    Ok(value) => Ok(Self::Camera(value)),
                    Err(value) => Err(Box::new(NodeFeature::Camera(Camera::Aimed(value)))),
                }
            }
            NodeFeature::Ribbon(RibbonEmitter::Trail(value)) => {
                match TryFromFeature::try_from_feature(*value, values) {
                    Ok(value) => Ok(Self::Ribbon(Box::new(value))),
                    Err(value) => Err(Box::new(NodeFeature::Ribbon(RibbonEmitter::Trail(value)))),
                }
            }
            NodeFeature::Particle(ParticleEmitter::Simulated(value)) => {
                match TryFromFeature::try_from_feature(*value, values) {
                    Ok(value) => Ok(Self::Particle(Box::new(value))),
                    Err(value) => Err(Box::new(NodeFeature::Particle(ParticleEmitter::Simulated(
                        value,
                    )))),
                }
            }
            NodeFeature::Event(EventSource::Identifier(value)) => Ok(Self::Event(value)),
            value => Err(Box::new(value)),
        }
    }
}
impl<'a> FromFeature<m3::Feature<'a>> for NodeFeature<'a> {
    fn from_feature(value: m3::Feature<'a>, values: &mut PropertyValues) -> Self {
        match value {
            m3::Feature::Extent(value) => Self::Extent(value),
            m3::Feature::ParticleCopy(value) => Self::ParticleCopy(value),
            m3::Feature::Projector(value) => Self::Projector(value),
            m3::Feature::Force(value) => Self::Force(value),
            m3::Feature::Warp(value) => Self::Warp(value),
            m3::Feature::HitTest(value) => Self::HitTest(value),
            m3::Feature::AttachmentVolume(value) => Self::AttachmentVolume(value),
            m3::Feature::Billboard(value) => Self::Billboard(value),
            m3::Feature::Turret(value) => Self::Turret(value),
            m3::Feature::IkJoint(value) => Self::IkJoint(value),
            m3::Feature::IkTwoJoint(value) => Self::IkTwoJoint(value),
            m3::Feature::IkCcd(value) => Self::IkCcd(value),
            m3::Feature::OneBoneSolver(value) => Self::OneBoneSolver(value),
            m3::Feature::ShadowBox(value) => Self::ShadowBox(value),
            m3::Feature::ViewVolume(value) => Self::ViewVolume(value),
            m3::Feature::Event(value) => Self::Event(EventSource::Command(value)),
            m3::Feature::Particle(value) => Self::Particle(ParticleEmitter::Simulated(Box::new(
                FromFeature::from_feature(*value, values),
            ))),
            m3::Feature::Ribbon(value) => Self::Ribbon(RibbonEmitter::Programmed(Box::new(
                FromFeature::from_feature(*value, values),
            ))),
            m3::Feature::Light(value) => Self::Light(Light::Specular(value)),
            m3::Feature::Camera(value) => Self::Camera(Camera::Optical(value)),
            m3::Feature::Attachment(value) => Self::Attachment(Attachment::Socket(
                FromFeature::from_feature(*value, values),
            )),
        }
    }
}
impl<'a> TryFromFeature<NodeFeature<'a>> for m3::Feature<'a> {
    type Error = Box<NodeFeature<'a>>;
    fn try_from_feature(
        value: NodeFeature<'a>,
        values: &PropertyValues,
    ) -> Result<Self, Self::Error> {
        match value {
            NodeFeature::Extent(value) => Ok(Self::Extent(value)),
            NodeFeature::ParticleCopy(value) => Ok(Self::ParticleCopy(value)),
            NodeFeature::Projector(value) => Ok(Self::Projector(value)),
            NodeFeature::Force(value) => Ok(Self::Force(value)),
            NodeFeature::Warp(value) => Ok(Self::Warp(value)),
            NodeFeature::HitTest(value) => Ok(Self::HitTest(value)),
            NodeFeature::AttachmentVolume(value) => Ok(Self::AttachmentVolume(value)),
            NodeFeature::Billboard(value) => Ok(Self::Billboard(value)),
            NodeFeature::Turret(value) => Ok(Self::Turret(value)),
            NodeFeature::IkJoint(value) => Ok(Self::IkJoint(value)),
            NodeFeature::IkTwoJoint(value) => Ok(Self::IkTwoJoint(value)),
            NodeFeature::IkCcd(value) => Ok(Self::IkCcd(value)),
            NodeFeature::OneBoneSolver(value) => Ok(Self::OneBoneSolver(value)),
            NodeFeature::ShadowBox(value) => Ok(Self::ShadowBox(value)),
            NodeFeature::ViewVolume(value) => Ok(Self::ViewVolume(value)),
            NodeFeature::Event(EventSource::Command(value)) => Ok(Self::Event(value)),
            NodeFeature::Particle(ParticleEmitter::Simulated(value)) => {
                match TryFromFeature::try_from_feature(*value, values) {
                    Ok(value) => Ok(Self::Particle(Box::new(value))),
                    Err(value) => Err(Box::new(NodeFeature::Particle(ParticleEmitter::Simulated(
                        value,
                    )))),
                }
            }
            NodeFeature::Ribbon(RibbonEmitter::Programmed(value)) => {
                match TryFromFeature::try_from_feature(*value, values) {
                    Ok(value) => Ok(Self::Ribbon(Box::new(value))),
                    Err(value) => Err(Box::new(NodeFeature::Ribbon(RibbonEmitter::Programmed(
                        value,
                    )))),
                }
            }
            NodeFeature::Light(Light::Specular(value)) => Ok(Self::Light(value)),
            NodeFeature::Camera(Camera::Optical(value)) => Ok(Self::Camera(value)),
            NodeFeature::Attachment(Attachment::Socket(value)) => {
                match TryFromFeature::try_from_feature(value, values) {
                    Ok(value) => Ok(Self::Attachment(Box::new(value))),
                    Err(value) => Err(Box::new(NodeFeature::Attachment(Attachment::Socket(
                        *value,
                    )))),
                }
            }
            value => Err(Box::new(value)),
        }
    }
}
impl<'a> From<wmo::Node<'a>> for NodeFeature<'a> {
    fn from(value: wmo::Node<'a>) -> Self {
        match value {
            wmo::Node::Attachment(value) => Self::Attachment(Attachment::Child(value.into())),
            wmo::Node::Light(value) => Self::Light(Light::World(value)),
            wmo::Node::PointLight(value) => Self::Light(Light::Point(value)),
            wmo::Node::LightProgram(value) => Self::LightProgram(value),
        }
    }
}
impl<'a> TryFrom<NodeFeature<'a>> for wmo::Node<'a> {
    type Error = Box<NodeFeature<'a>>;
    fn try_from(value: NodeFeature<'a>) -> Result<Self, Self::Error> {
        match value {
            NodeFeature::Attachment(Attachment::Child(value)) => match value.try_into() {
                Ok(value) => Ok(Self::Attachment(value)),
                Err(value) => Err(Box::new(NodeFeature::Attachment(Attachment::Child(*value)))),
            },
            NodeFeature::Light(Light::World(value)) => Ok(Self::Light(value)),
            NodeFeature::Light(Light::Point(value)) => Ok(Self::PointLight(value)),
            NodeFeature::LightProgram(value) => Ok(Self::LightProgram(value)),
            value => Err(Box::new(value)),
        }
    }
}
impl<'a> From<scene::Feature> for NodeFeature<'a> {
    fn from(value: scene::Feature) -> Self {
        match value {
            scene::Feature::Camera(value) => Self::Camera(Camera::Portable(value)),
            scene::Feature::Light(value) => Self::Light(Light::Portable(value)),
            scene::Feature::Attachment(value) => Self::Attachment(Attachment::Child(value.into())),
            scene::Feature::Particles(value) => Self::Particle(ParticleEmitter::Resource(value)),
            scene::Feature::Ribbon(value) => Self::Ribbon(RibbonEmitter::Resource(value)),
            scene::Feature::FaceActor(value) => Self::FaceActor(value),
        }
    }
}
impl<'a> TryFrom<NodeFeature<'a>> for scene::Feature {
    type Error = Box<NodeFeature<'a>>;
    fn try_from(value: NodeFeature<'a>) -> Result<Self, Self::Error> {
        match value {
            NodeFeature::Camera(Camera::Portable(value)) => Ok(Self::Camera(value)),
            NodeFeature::Light(Light::Portable(value)) => Ok(Self::Light(value)),
            NodeFeature::Attachment(Attachment::Child(value)) => match value.try_into() {
                Ok(value) => Ok(Self::Attachment(value)),
                Err(value) => Err(Box::new(NodeFeature::Attachment(Attachment::Child(*value)))),
            },
            NodeFeature::Particle(ParticleEmitter::Resource(value)) => Ok(Self::Particles(value)),
            NodeFeature::Ribbon(RibbonEmitter::Resource(value)) => Ok(Self::Ribbon(value)),
            NodeFeature::FaceActor(value) => Ok(Self::FaceActor(value)),
            value => Err(Box::new(value)),
        }
    }
}

impl<'a> From<ParticleDefinition<'a>> for NodeFeature<'a> {
    fn from(value: ParticleDefinition<'a>) -> Self {
        Self::Particle(ParticleEmitter::Simulated(Box::new(value)))
    }
}
impl<'a> From<TrailEmitter> for NodeFeature<'a> {
    fn from(value: TrailEmitter) -> Self {
        Self::Ribbon(RibbonEmitter::Trail(Box::new(value)))
    }
}
impl<'a> From<ProgrammedRibbon<'a>> for NodeFeature<'a> {
    fn from(value: ProgrammedRibbon<'a>) -> Self {
        Self::Ribbon(RibbonEmitter::Programmed(Box::new(value)))
    }
}
impl<'a> From<Socket<'a>> for NodeFeature<'a> {
    fn from(value: Socket<'a>) -> Self {
        Self::Attachment(Attachment::Socket(value))
    }
}
impl<'a> From<ChildModel<'a>> for NodeFeature<'a> {
    fn from(value: ChildModel<'a>) -> Self {
        Self::Attachment(Attachment::Child(value))
    }
}

/// Converts a feature definition, allocating any newly required value slots.
pub trait FromFeature<S>: Sized {
    /// Converts using the destination model's value allocation context.
    fn from_feature(source: S, values: &mut PropertyValues) -> Self;
}
impl<S, T: From<S>> FromFeature<S> for T {
    fn from_feature(source: S, _: &mut PropertyValues) -> Self {
        Self::from(source)
    }
}
/// Projects a feature definition using its model-owned constants and typed sources.
pub trait TryFromFeature<S>: Sized {
    /// The definition that could not be represented.
    type Error;
    /// Projects without changing the identities of retained slots.
    fn try_from_feature(source: S, values: &PropertyValues) -> Result<Self, Self::Error>;
}
impl<S, T: TryFrom<S>> TryFromFeature<S> for T {
    type Error = T::Error;
    fn try_from_feature(source: S, _: &PropertyValues) -> Result<Self, Self::Error> {
        Self::try_from(source)
    }
}
