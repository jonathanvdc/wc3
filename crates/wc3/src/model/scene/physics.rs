//! Collision geometry and body constraints with logical identities.
use crate::model::{Animated, Id, NodeId, ZeroVec};
/// Identity marker for collision geometry.
pub enum ShapeObject {}
/// Collision geometry identity scoped to one simulation system.
pub type ShapeId = Id<ShapeObject>;
/// Identity marker for a simulation body.
pub enum BodyObject {}
/// Body identity scoped to one simulation system.
pub type BodyId = Id<BodyObject>;
/// Identity marker for a body constraint.
pub enum JointObject {}
/// Constraint identity scoped to one simulation system.
pub type JointId = Id<JointObject>;
/// Collision geometry in local shape coordinates.
#[derive(Clone, Debug)]
pub enum Geometry<'a> {
    /// Box half dimensions.
    Box([f32; 3]),
    /// Sphere radius.
    Sphere(f32),
    /// Capsule radius and cap-center separation.
    Capsule {
        /// Radius.
        radius: f32,
        /// Cap-center separation.
        length: f32,
        /// Explicit authored cap centers, when present.
        centers: ZeroVec<'a, [f32; 3]>,
    },
    /// Cylinder radius and length.
    Cylinder {
        /// Radius.
        radius: f32,
        /// Length along local Z.
        length: f32,
    },
    /// Convex hull points.
    ConvexHull(ZeroVec<'a, [f32; 3]>),
    /// Triangle-mesh collision positions and primitives.
    Triangles {
        /// Local positions.
        positions: ZeroVec<'a, [f32; 3]>,
        /// Triangle vertex indices.
        triangles: ZeroVec<'a, [u32; 3]>,
    },
    /// Authored collision category with unresolved geometry semantics.
    Unknown {
        /// Source category selector.
        kind: u32,
        /// Authored half extents.
        half_extents: [f32; 3],
        /// Authored radius.
        radius: f32,
        /// Authored length.
        length: f32,
        /// Authored shape points.
        points: ZeroVec<'a, [f32; 3]>,
        /// Authored collision vertices.
        vertices: ZeroVec<'a, [f32; 3]>,
        /// Authored triangle indices.
        triangles: ZeroVec<'a, [u32; 3]>,
    },
}
/// Physical density, contact friction, and rebound response.
#[derive(Clone, Copy, Debug, Default)]
pub struct ContactMaterial {
    /// Density used to derive mass from volume.
    pub density: f32,
    /// Contact friction coefficient.
    pub friction: f32,
    /// Contact rebound coefficient.
    pub restitution: f32,
}
/// Authored collision shape and placement in a body frame.
#[derive(Clone, Debug)]
pub struct Shape<'a> {
    /// Collision geometry.
    pub geometry: Geometry<'a>,
    /// Local-to-body transform.
    pub transform: [[f32; 4]; 4],
    /// Contact response.
    pub material: ContactMaterial,
    /// Profile-specific contact callback categories with unresolved meanings.
    pub game_controls: u16,
}
/// Simulation motion behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Motion {
    /// Driven by simulation.
    #[default]
    Dynamic,
    /// Driven by authored animation.
    Kinematic,
    /// Immobile collider.
    Static,
    /// Authored behavior with unresolved meaning.
    Unknown(u32),
}
/// StarCraft body collision filtering and bone-drive settings.
#[derive(Clone, Debug, Default)]
pub struct CollisionFilter {
    /// External physics-material identity.
    pub material: u32,
    /// Enable collisions.
    pub collidable: bool,
    /// Allow walking on this body.
    pub walkable: bool,
    /// Allow stacking this body.
    pub stackable: bool,
    /// Enable collision simulation.
    pub simulate_collision: bool,
    /// Keep the bone drive active during setup and deactivation.
    pub bone_driven: bool,
    /// Authored controls with unresolved meanings.
    pub unclassified: CollisionFlags,
}
/// Kinematic following and ragdoll grouping behavior.
#[derive(Clone, Debug, Default)]
pub struct Follow {
    /// Fraction of the driving-bone displacement applied each step.
    pub factor: f32,
    /// Carries dependent bodies.
    pub has_children: bool,
    /// Carries all ragdoll groups as the root.
    pub ragdoll_root: bool,
    /// Kinematic parent body.
    pub parent: Option<BodyId>,
}
/// Body parameters and shared collision shape identities.
#[derive(Clone, Debug)]
pub struct Body {
    /// Hierarchy node that drives this body.
    pub node: NodeId,
    /// Motion behavior.
    pub motion: Motion,
    /// Simulation activation.
    pub simulates: Animated<bool>,
    /// Collision shapes attached to the body.
    pub shapes: Vec<ShapeId>,
    /// Translational and rotational damping coefficients.
    pub damping: [f32; 2],
    /// Inertia multiplier.
    pub inertia_scale: f32,
    /// Gravity multiplier.
    pub gravity_scale: f32,
    /// Inherit the nearest ancestor body's dynamic state.
    pub inherit_dynamic: bool,
    /// Keep the body kinematic during ragdoll activation.
    pub exempt_from_ragdoll: bool,
    /// Local force-field channel mask.
    pub local_force_channels: ForceChannels,
    /// World force-field channel mask.
    pub world_force_channels: ForceChannels,
    /// Optional collision filtering.
    pub collision_filter: Option<CollisionFilter>,
    /// Optional kinematic-drive grouping.
    pub follow: Option<Follow>,
    /// Protect manually tuned body authoring values.
    pub locked: bool,
}
/// Constraint behavior between bodies.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JointKind {
    /// Ball-and-socket constraint.
    #[default]
    Spherical,
    /// Single rotational axis.
    Revolute,
    /// Angular cone and twist limits.
    ConeTwist,
    /// Rigidly welded bodies.
    Weld,
    /// Single translational axis.
    Prismatic,
    /// Distance constraint.
    Distance,
    /// Constraint with unresolved behavior.
    Unknown(u32),
}
/// Friction model used by a joint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Friction {
    /// No friction.
    #[default]
    None,
    /// Constant drag.
    Constant,
    /// Viscous drag.
    Viscous,
    /// Friction model with unresolved behavior.
    Unknown(u32),
}
/// Joint motor behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Motor {
    /// Disabled motor.
    #[default]
    Off,
    /// Position-controlled motor.
    Position,
    /// Velocity-controlled motor.
    Velocity,
    /// Motor with unresolved behavior.
    Unknown(u32),
}
/// A complete authored two-body constraint.
#[derive(Clone, Debug)]
pub struct Joint {
    /// Constrained bodies.
    pub bodies: [BodyId; 2],
    /// Constraint kind.
    pub kind: JointKind,
    /// Constraint frames relative to each body's node.
    pub frames: [[[f32; 4]; 4]; 2],
    /// Permit collision between connected bodies.
    pub collide_connected: bool,
    /// Enable authored motion limits.
    pub limit_enabled: bool,
    /// Lower and upper angular or translational limits.
    pub limits: [f32; 2],
    /// Cone angular limit.
    pub cone: f32,
    /// Joint friction model.
    pub friction: Friction,
    /// Friction strength.
    pub friction_amount: f32,
    /// Angular spring frequency and damping.
    pub angular_spring: [f32; 2],
    /// Linear spring frequency and damping.
    pub linear_spring: [f32; 2],
    /// Resting separation of a distance joint.
    pub rest_length: f32,
    /// Breaking force and torque thresholds; zero disables the corresponding break condition.
    pub breaking: [f32; 2],
    /// Optional authored motor behavior.
    pub motor: Option<Motor>,
    /// Motor force or torque limit.
    pub max_motor_force: Option<f32>,
    /// Target motor speed.
    pub motor_speed: Option<f32>,
    /// Reference position for prismatic limits.
    pub reference_translation: Option<f32>,
    /// Protect manually tuned joint authoring values.
    pub locked: bool,
}

use bitfield::bitfield;
bitfield! {
    /// Retained unresolved collision-filter options.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct CollisionFlags(u32);
    /// Complete retained flag word.
    pub bits, _:31,0;
}
impl CollisionFlags {
    /// Retains all authored option bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
bitfield! {
    /// Independently selectable environmental force channels.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct ForceChannels(u16);
    /// Complete channel mask.
    pub bits, _:15,0;
}
impl ForceChannels {
    /// Retains all authored channel bits.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
}
