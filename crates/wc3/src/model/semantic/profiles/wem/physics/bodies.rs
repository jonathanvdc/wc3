//! Body behavior and independently optional game extension settings.
use super::Motion;
use super::{BodyId, ShapeId};
use crate::model::semantic::{Animated, NodeId};
use bitfield::bitfield;
bitfield! {
    /// Additional SC2 body controls with unresolved individual meanings.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct Sc2BodyFlags(u32);
    /// Complete authored option bits.
    pub bits,_:31,0;
}
impl Sc2BodyFlags {
    /// Retains all currently unnamed options.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
bitfield! {
    /// Independent local and world force-field channel selections.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct ForceChannels(u32);
    /// Complete authored channel selections.
    pub bits,_:31,0;
    /// Channels interpreted in the attached local frame.
    pub u16,local,set_local:15,0;
    /// Channels interpreted in world coordinates.
    pub u16,world,set_world:31,16;
}
impl ForceChannels {
    /// Constructs both selections without discarding any channels.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
/// SC2 collision-filter and skeleton drive settings.
#[derive(Clone, Debug)]
pub struct Sc2Body {
    /// External physics-material category, distinct from render materials.
    pub material: u32,
    /// Allows contacts.
    pub collidable: bool,
    /// Allows walking on this body.
    pub walkable: bool,
    /// Allows stacking on this body.
    pub stackable: bool,
    /// Enables collision simulation.
    pub simulate_collision: bool,
    /// Retains skeleton physics state through setup and deactivation.
    pub keeps_bone_driven: bool,
    /// Additional authored options with unresolved meanings.
    pub flags: Sc2BodyFlags,
}
/// WoW bone following and ragdoll grouping settings.
#[derive(Clone, Debug)]
pub struct WowBody {
    /// Fraction of driving-bone displacement applied each step.
    pub follow_factor: f32,
    /// Carries dependent bodies.
    pub has_children: bool,
    /// Carries ragdoll groups as their kinematic root.
    pub ragdoll_root: bool,
    /// Kinematic parent in the same simulation domain.
    pub parent: Option<BodyId>,
}
/// A simulation body may independently carry both game extension settings.
#[derive(Clone, Debug)]
pub struct Body {
    /// Hierarchy node attached to this body.
    pub node: NodeId,
    /// Dynamic, kinematic, or static behavior.
    pub motion: Motion,
    /// Authored simulation state and optional animation destination.
    pub simulates: Animated<bool>,
    /// Collision geometries attached to this body.
    pub shapes: Vec<ShapeId>,
    /// Linear and angular damping.
    pub damping: [f32; 2],
    /// Inertia multiplier.
    pub inertia_scale: f32,
    /// Gravity multiplier.
    pub gravity_scale: f32,
    /// Uses the nearest ancestor body's dynamic state.
    pub inherit_dynamic: bool,
    /// Remains kinematic during ragdoll activation.
    pub exempt_from_ragdoll: bool,
    /// Independent local and world force-field selections.
    pub force_channels: ForceChannels,
    /// Independently optional SC2 settings.
    pub sc2: Option<Sc2Body>,
    /// Independently optional WoW settings; bodies with this extension can constrain SC2 bodies.
    pub wow: Option<WowBody>,
    /// Protects manually authored geometry from bulk preparation.
    pub locked: bool,
}
