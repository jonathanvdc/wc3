//! Logical behaviors grouped by purpose rather than their source vocabulary.
use crate::model::profiles::{m2, mdlx, Profile, Universal};
use crate::model::EffectResourceId;
use crate::model::{Link, NodeId, ValidateLinks};
mod illumination;
pub use illumination::*;
mod cameras;
pub use cameras::*;
mod attachments;
pub use attachments::*;

mod particles;
pub use particles::*;
mod ribbons;
pub use ribbons::*;
mod set;
pub use set::*;
mod conversions;
pub use conversions::{FromFeature, TryFromFeature};
mod links;

/// Complete logical payload of an attached behavior.
pub type NodeFeature<'a> = FeatureSet<'a, Universal>;
/// An external effect or a composed simulation definition.
#[derive(Clone, Debug)]
pub enum ParticleEmitter<'a> {
    /// Instance of an externally defined effect resource.
    Resource(EffectResourceId),
    /// Warcraft III particle emitter 1, instantiating model or image resources.
    MdlxEmitter1(Box<mdlx::ParticleEmitter1>),
    /// Warcraft III particle emitter 2, rendering atlas sprites and tails.
    MdlxEmitter2(Box<mdlx::ParticleEmitter2>),
    /// World of Warcraft particle emitter.
    M2(Box<m2::ParticleEmitter<'a>>),
}
/// Ribbon generation from persistent cross sections or an external resource.
#[derive(Clone, Debug)]
pub enum RibbonEmitter {
    /// Instance of an externally defined effect resource.
    Resource(EffectResourceId),
    /// Warcraft III ribbon emitter.
    Mdlx(Box<mdlx::RibbonEmitter>),
    /// World of Warcraft ribbon emitter.
    M2(Box<m2::RibbonEmitter>),
}
/// A feature's hierarchy anchor and logical definition.
#[derive(Clone, Debug)]
pub struct AttachedFeature<'a, P: Profile = Universal> {
    /// Hierarchy anchor of this behavior.
    pub node: NodeId,
    /// Complete behavior admitted by the model profile.
    pub payload: P::Feature<'a>,
}
/// Payload kinds required by relationships between attached features.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureKind {
    /// Warcraft III geometry color and opacity animation.
    MdlxGeometryAnimation,

    /// Static-world light referenced by group and doodad memberships.
    WmoLight,
    /// Region-local static point illumination.
    WmoPointLight,
    /// Region-local flicker and cookie light program.
    WmoPointProgram,
    /// Static-world doodad placement referenced by groups and placement sets.
    WmoDoodad,
    /// Behavior with no separately constrained relationship category.
    Other,
}
impl NodeFeature<'_> {
    /// Classifies payloads whose concrete type is required by another feature.
    pub(crate) fn profile_kind(&self) -> FeatureKind {
        match self {
            Self::GeometryAnimation(_) => FeatureKind::MdlxGeometryAnimation,

            Self::Light(Light::World(_)) => FeatureKind::WmoLight,
            Self::Light(Light::Point(_)) => FeatureKind::WmoPointLight,
            Self::LightProgram(_) => FeatureKind::WmoPointProgram,
            Self::Attachment(Attachment::Child(ChildModel {
                appearance: PlacementAppearance::World { .. },
                ..
            })) => FeatureKind::WmoDoodad,
            _ => FeatureKind::Other,
        }
    }
}

impl<P: Profile> ValidateLinks for AttachedFeature<'_, P> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.node.visit_links(visitor);
        self.payload.visit_links(visitor);
    }
}
