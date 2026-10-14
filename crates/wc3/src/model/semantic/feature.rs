//! Effects and behaviors are individually typed, and can freely share one graph.
use crate::model::semantic::profiles::{FeaturePayload, Profile, Universal};
use crate::model::semantic::{profiles, scene, NodeId};
/// Complete logical payload of an attached behavior.
#[derive(Clone, Debug)]
pub enum NodeFeature<'a> {
    /// Format-independent scene behavior.
    Generic(Box<scene::Feature<'a>>),
    /// Warcraft III effects and scene actors.
    Mdlx(Box<profiles::mdlx::Node<'a>>),
    /// World of Warcraft animated actors.
    M2(Box<profiles::m2::ActorKind<'a>>),
    /// StarCraft II attached behavior.
    M3(Box<profiles::m3::Feature<'a>>),
    /// Static-world doodad or light behavior.
    Wmo(Box<profiles::wmo::Node<'a>>),
    /// Editable-model mixed-family behavior.
    Wem(Box<profiles::wem::Feature<'a>>),
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
    /// An editable-document StarCraft particle definition.
    WemStarcraftParticle,
    /// An editable-document emitter copy referencing a primary particle.
    WemStarcraftParticleCopy,
    /// An editable-document StarCraft ribbon definition.
    WemStarcraftRibbon,
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
    pub fn kind(&self) -> FeatureKind {
        match self {
            Self::Generic(_) => FeatureKind::Other,
            Self::Mdlx(value) => FeaturePayload::kind(value.as_ref()),
            Self::M2(value) => FeaturePayload::kind(value.as_ref()),
            Self::M3(value) => FeaturePayload::kind(value.as_ref()),
            Self::Wmo(value) => FeaturePayload::kind(value.as_ref()),
            Self::Wem(value) => FeaturePayload::kind(value.as_ref()),
        }
    }
}
