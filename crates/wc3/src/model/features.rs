//! Logical behaviors grouped by purpose rather than their source vocabulary.
use crate::model::profiles::{m2, m3, mdlx, Profile, Universal};
use crate::model::{Link, NodeId, Resource, ValidateLinks};
mod illumination;
pub use illumination::*;
mod cameras;
pub use cameras::*;
mod attachments;
pub use attachments::*;
mod programmed_effects;
pub use programmed_effects::*;
mod solvers;
pub use solvers::*;
mod effect_components;
pub use effect_components::*;
mod particles;
pub use particles::*;
mod ribbons;
pub use ribbons::*;
mod set;
pub use set::*;
mod conversions;
mod links;
mod selector_support;
/// Complete logical payload of an attached behavior.
pub type NodeFeature<'a> = FeatureSet<'a, Universal>;
/// An external effect or a composed simulation definition.
#[derive(Clone, Debug)]
pub enum ParticleEmitter<'a> {
    /// Instance of an externally defined effect resource.
    Resource(Resource<'a>),
    /// Authored emission, motion, lifetime, and particle-age appearance.
    Simulated(Box<ParticleDefinition<'a>>),
}
/// Ribbon generation from cross sections or programmed geometry.
#[derive(Clone, Debug)]
pub enum RibbonEmitter<'a> {
    /// Instance of an externally defined effect resource.
    Resource(Resource<'a>),
    /// Persistent sections with shared geometry, appearance, and rendering.
    Trail(Box<TrailEmitter>),
    /// Programmed geometry with particle-age appearance and forces.
    Programmed(Box<ProgrammedRibbon<'a>>),
}
/// Gameplay event identities and executable engine commands.
#[derive(Clone, Debug)]
pub enum EventSource<'a> {
    /// Named gameplay action emitted at timeline occurrences.
    Named(mdlx::Events<'a>),
    /// Four-character action code and external application data identity.
    Identifier(m2::Event),
    /// Structured engine command with typed resource and parameter controls.
    Command(Box<m3::EventCommand<'a>>),
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
    /// An M3 turret part anchored to a skeleton bone.
    M3Turret,
    /// Warcraft III geometry color and opacity animation.
    MdlxGeometryAnimation,
    /// StarCraft II particle system.
    M3Particle,
    /// StarCraft II emitter instance inheriting a particle definition.
    M3ParticleCopy,
    /// StarCraft II ribbon emitter.
    M3Ribbon,
    /// StarCraft II surface projector.
    M3Projection,
    /// Structured StarCraft II animated spatial bounds.
    M3Extent,
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
            Self::Particle(ParticleEmitter::Simulated(value))
                if matches!(value.details, ParticleDetails::Programmed(_)) =>
            {
                FeatureKind::M3Particle
            }
            Self::ParticleCopy(_) => FeatureKind::M3ParticleCopy,
            Self::Ribbon(RibbonEmitter::Programmed(_)) => FeatureKind::M3Ribbon,
            Self::Projector(_) => FeatureKind::M3Projection,
            Self::Extent(_) => FeatureKind::M3Extent,
            Self::Turret(_) => FeatureKind::M3Turret,
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
