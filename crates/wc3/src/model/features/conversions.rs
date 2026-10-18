//! Lossless movement between constrained and universal feature vocabularies.
use super::{Attachment, Camera, Light, NodeFeature, ParticleEmitter, RibbonEmitter};
use super::{ChildModel, Socket};
use crate::model::profiles::{m2, mdlx, wmo};
use crate::model::scene;
use crate::model::PropertyValues;
impl<'a> FromFeature<mdlx::Node<'a>> for NodeFeature<'a> {
    fn from_feature(value: mdlx::Node<'a>, values: &mut PropertyValues) -> Self {
        match value {
            mdlx::Node::GeometryAnimation(value) => Self::GeometryAnimation(value),
            mdlx::Node::Particle(value) => {
                Self::Particle(ParticleEmitter::MdlxEmitter1(Box::new(value)))
            }
            mdlx::Node::Particle2(value) => {
                Self::Particle(ParticleEmitter::MdlxEmitter2(Box::new(value)))
            }
            mdlx::Node::Ribbon(value) => Self::Ribbon(RibbonEmitter::Mdlx(Box::new(value))),
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
            NodeFeature::Particle(ParticleEmitter::MdlxEmitter1(value)) => {
                Ok(Self::Particle(*value))
            }
            NodeFeature::Particle(ParticleEmitter::MdlxEmitter2(value)) => {
                Ok(Self::Particle2(*value))
            }
            NodeFeature::Ribbon(RibbonEmitter::Mdlx(value)) => Ok(Self::Ribbon(*value)),
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
            m2::ActorKind::Ribbon(value) => Self::Ribbon(RibbonEmitter::M2(value)),
            m2::ActorKind::Particle(value) => Self::Particle(ParticleEmitter::M2(value)),
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
            NodeFeature::Ribbon(RibbonEmitter::M2(value)) => Ok(Self::Ribbon(value)),
            NodeFeature::Particle(ParticleEmitter::M2(value)) => Ok(Self::Particle(value)),
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

impl<'a> From<mdlx::ParticleEmitter1> for NodeFeature<'a> {
    fn from(value: mdlx::ParticleEmitter1) -> Self {
        Self::Particle(ParticleEmitter::MdlxEmitter1(Box::new(value)))
    }
}
impl<'a> From<mdlx::ParticleEmitter2> for NodeFeature<'a> {
    fn from(value: mdlx::ParticleEmitter2) -> Self {
        Self::Particle(ParticleEmitter::MdlxEmitter2(Box::new(value)))
    }
}
impl<'a> From<m2::ParticleEmitter<'a>> for NodeFeature<'a> {
    fn from(value: m2::ParticleEmitter<'a>) -> Self {
        Self::Particle(ParticleEmitter::M2(Box::new(value)))
    }
}
impl<'a> From<mdlx::RibbonEmitter> for NodeFeature<'a> {
    fn from(value: mdlx::RibbonEmitter) -> Self {
        Self::Ribbon(RibbonEmitter::Mdlx(Box::new(value)))
    }
}
impl<'a> From<m2::RibbonEmitter> for NodeFeature<'a> {
    fn from(value: m2::RibbonEmitter) -> Self {
        Self::Ribbon(RibbonEmitter::M2(Box::new(value)))
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

macro_rules! particle_family {
    ($native:ty, $variant:ident) => {
        impl<'a> From<$native> for ParticleEmitter<'a> {
            fn from(value: $native) -> Self {
                Self::$variant(Box::new(value))
            }
        }
        impl<'a> TryFrom<ParticleEmitter<'a>> for $native {
            type Error = Box<ParticleEmitter<'a>>;
            fn try_from(value: ParticleEmitter<'a>) -> Result<Self, Self::Error> {
                match value {
                    ParticleEmitter::$variant(value) => Ok(*value),
                    value => Err(Box::new(value)),
                }
            }
        }
    };
}
particle_family!(mdlx::ParticleEmitter1, MdlxEmitter1);
particle_family!(mdlx::ParticleEmitter2, MdlxEmitter2);
particle_family!(m2::ParticleEmitter<'a>, M2);

macro_rules! ribbon_family {
    ($native:ty, $variant:ident) => {
        impl From<$native> for RibbonEmitter {
            fn from(value: $native) -> Self {
                Self::$variant(Box::new(value))
            }
        }
        impl TryFrom<RibbonEmitter> for $native {
            type Error = Box<RibbonEmitter>;
            fn try_from(value: RibbonEmitter) -> Result<Self, Self::Error> {
                match value {
                    RibbonEmitter::$variant(value) => Ok(*value),
                    value => Err(Box::new(value)),
                }
            }
        }
    };
}
ribbon_family!(mdlx::RibbonEmitter, Mdlx);
ribbon_family!(m2::RibbonEmitter, M2);
