//! Additional WEM hierarchy and skin-authoring state lives on its actual node.
use super::poses::NodeSetup;
use super::{Availability, Bone, Metadata};
use crate::model::semantic::{Link, ValidateLinks};
use bitfield::bitfield;

bitfield! {
    /// Supplemental hierarchy bits not represented by common node behavior.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct SupplementalNodeFlags(u32);
    /// Returns the retained bit pattern.
    pub bits, _: 31, 0;
    /// Transforms are interpreted directly in model space.
    pub model_space, set_model_space: 8;
}
impl SupplementalNodeFlags {
    /// Retains supplemental bits while excluding common visibility and inheritance flags.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits & !0xff)
    }
}

/// Owner-local WEM hierarchy and skin-editing controls.
#[derive(Clone, Debug, Default)]
pub struct NodeControls<'a> {
    /// Author-declared role, retained independently from the attached payload kind.
    pub declared_role: DeclaredRole,
    /// Producer-defined family availability, independent of the attached payload kinds.
    pub availability: Availability,
    /// Restricts authoring scale changes to a uniform factor.
    pub uniform_scale_only: bool,
    /// Model-space interpretation and unnamed supplemental behavior.
    pub flags: SupplementalNodeFlags,
    /// Saved transforms, skin setup, and functional rig classification.
    pub setup: NodeSetup,
    /// Bone bounds and mesh-visibility controls; absence leaves this node without a bone declaration.
    pub bone: Option<Bone>,
    /// Ordered additional authoring properties.
    pub metadata: Metadata<'a>,
}
impl ValidateLinks for NodeControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.flags.bits() & 0xff != 0 {
            visitor(Link::InvalidValue);
        }
        self.setup.visit_links(visitor);
        self.bone.visit_links(visitor);
    }
}

/// Author-declared hierarchy role, independent of an attached payload's actual family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DeclaredRole {
    /// General transform or grouping node.
    #[default]
    Helper,
    /// Skeleton joint.
    Bone,
    /// External actor mount.
    Attachment,
    /// Illumination source.
    Light,
    /// Viewpoint.
    Camera,
    /// General particle generator.
    Particle,
    /// General ribbon generator.
    Ribbon,
    /// Event command anchor.
    Event,
    /// Collision primitive.
    Collision,
    /// Warcraft III first-generation particle emitter.
    WarcraftParticle1,
    /// Warcraft III second-generation particle emitter.
    WarcraftParticle2,
    /// Warcraft III ribbon emitter.
    WarcraftRibbon,
    /// StarCraft II particle emitter.
    StarCraftParticle,
    /// StarCraft II ribbon emitter.
    StarCraftRibbon,
    /// Warcraft III Popcorn emitter.
    WarcraftPopcorn,
    /// World of Warcraft particle emitter.
    WarcraftOnlineParticle,
    /// Force volume.
    Force,
    /// Vertex deformation volume.
    VertexWarp,
    /// Warcraft III facial animation declaration.
    WarcraftFaceAnimation,
    /// Producer-defined hierarchy role.
    Unknown(u8),
}
