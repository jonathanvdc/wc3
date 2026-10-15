//! Logical M3 collision shapes, bodies, joints, and cloth.
use super::{values::*, Property};
use crate::model::common::AsULE;
use crate::model::semantic::ids::Scope;
use crate::model::semantic::{Id, MeshId, NodeId, NodeInfluence, ZeroVec};

/// Identifies one cloth definition in the M3 profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ClothId(pub(crate) u32);
impl ClothId {
    /// Returns the logical arena index.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// Geometric collider definition independent of cooking or acceleration storage.
#[derive(Clone, Debug, PartialEq)]
pub enum ShapeGeometry<'a> {
    /// Axis-aligned box in the shape's local coordinate frame.
    Box {
        /// Nonnegative half-size along each local axis.
        half_extents: [f32; 3],
    },
    /// Spherical collision surface.
    Sphere {
        /// Nonnegative local radius.
        radius: f32,
        /// Additional authored axis controls whose runtime meanings are unresolved.
        axis_controls: [f32; 2],
    },
    /// Capsule with a straight cylindrical middle segment.
    Capsule {
        /// Nonnegative radial size.
        radius: f32,
        /// Nonnegative length of the cylindrical segment.
        length: f32,
        /// Additional authored axis control whose meaning is unresolved.
        axis_control: f32,
    },
    /// Cylindrical collision surface.
    Cylinder {
        /// Nonnegative radial size.
        radius: f32,
        /// Nonnegative longitudinal size.
        length: f32,
        /// Additional authored axis control whose meaning is unresolved.
        axis_control: f32,
    },
    /// Convex boundary defined by collision points or polygon connectivity.
    ConvexHull {
        /// Additional authored collider controls whose runtime roles are unresolved.
        controls: [f32; 3],
        /// Collision geometry supplying the hull's points and any authored connectivity.
        mesh: MeshId,
        /// Authored plane equations, when declared.
        planes: ZeroVec<'a, [f32; 4]>,
    },
    /// Triangle collision surface.
    TriangleMesh {
        /// Additional authored collider controls whose runtime roles are unresolved.
        controls: [f32; 3],
        /// Collision mesh supplying positions and triangle connectivity.
        mesh: MeshId,
        /// Authored plane equations, when declared.
        planes: ZeroVec<'a, [f32; 4]>,
    },
    /// Unrecognized game collider category with its understood numeric and geometric controls.
    Unknown {
        /// Original game category, retained without assigning an invented meaning.
        kind: PhysicsShapeKind,
        /// Three authored size/control values.
        dimensions: [f32; 3],
        /// Understood collision geometry, when supplied.
        mesh: Option<MeshId>,
        /// Authored plane equations, when supplied.
        planes: ZeroVec<'a, [f32; 4]>,
    },
}
/// Additional authored geometry carried alongside a primitive collider.
#[derive(Clone, Debug, PartialEq)]
pub struct AuxiliaryGeometry<'a> {
    /// Independently addressable collision geometry.
    pub mesh: MeshId,
    /// Authored plane equations associated with this geometry.
    pub planes: ZeroVec<'a, [f32; 4]>,
}
/// Collider placement and definition in body-local space.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape<'a> {
    /// Full affine placement; nonuniform scale remains part of the authored transform.
    pub transform: [[f32; 4]; 4],
    /// Actual geometric collider category and its parameters.
    pub geometry: ShapeGeometry<'a>,
    /// Additional declared geometric input for a primitive collider, when present.
    pub auxiliary_geometry: Option<AuxiliaryGeometry<'a>>,
    /// Authored contact margin, when declared separately from geometry.
    pub margin: Option<f32>,
}

/// Authored physical parameters for a rigid body.
#[derive(Clone, Debug, PartialEq)]
pub struct BodyParameters {
    /// Physics-material category interpreted by the game simulation.
    pub material: PhysicalMaterial,
    /// Authored body mass, independent of collision shape volume.
    pub mass: f32,
    /// Contact-friction coefficient.
    pub friction: f32,
    /// Coefficient controlling velocity recovery after contact.
    pub restitution: f32,
    /// Attenuation of translational velocity.
    pub linear_damping: f32,
    /// Attenuation of angular velocity.
    pub angular_damping: f32,
    /// Multiplier applied to the world's gravity contribution.
    pub gravity_factor: f32,
}
/// Authored transition between animated and simulated movement.
#[derive(Clone, Debug, PartialEq)]
pub struct DynamicState {
    /// Animated simulation-state control sampled according to the body flags.
    pub state: Property<u32>,
    /// Simulation-transition parameter whose precise runtime interpretation is unresolved.
    pub blend_out: f32,
}
/// A rigid body attached to an actual scene node.
#[derive(Clone, Debug, PartialEq)]
pub struct Body<'a> {
    id: BodyId,
    /// Node whose movement is controlled by this body.
    pub node: NodeId,
    /// Native simulation-type control; individual code meanings remain unresolved.
    pub simulation_type: u16,
    /// Explicit physical parameters, when authored rather than left unspecified.
    pub parameters: Option<BodyParameters>,
    /// Authored dynamic-state transition, when declared.
    pub dynamic: Option<DynamicState>,
    /// Collision shapes placed in body-local space.
    pub shapes: Vec<Shape<'a>>,
    /// Game simulation-control flags; unknown bits remain available.
    pub flags: RigidBodyFlags,
    /// Local force-channel membership.
    pub local_forces: LocalForces,
    /// World force-channel membership.
    pub world_forces: WorldForces,
    /// Simulation priority control whose scheduling details are unresolved.
    pub priority: u32,
}

/// Constraint between two authored skeleton anchors.
#[derive(Clone, Debug, PartialEq)]
pub struct Joint {
    /// Joint discriminator: 0 spherical, 1 revolute, 2 cone-twist, 3 weld.
    pub joint_type: JointKind,
    /// Scene anchor on the first side of the joint.
    pub node_a: NodeId,
    /// Scene anchor on the second side of the joint.
    pub node_b: NodeId,
    /// Matrix expressing the joint frame relative to the first bone.
    pub frame_a: [[f32; 4]; 4],
    /// Matrix expressing the joint frame relative to the second bone.
    pub frame_b: [[f32; 4]; 4],
    /// Word whose low byte controls angular-limit enforcement.
    pub limit_control: u32,
    /// Lower angular bound when joint limits are enabled.
    pub limit_min: f32,
    /// Upper angular bound when joint limits are enabled.
    pub limit_max: f32,
    /// Angular aperture of a cone-twist constraint.
    pub cone_angle: f32,
    /// Word whose low byte controls joint-friction enforcement.
    pub friction_control: u32,
    /// Factor scaling the joint friction torque estimate.
    pub friction: f32,
    /// Dimensionless damping ratio for a weld-joint spring.
    pub damping_ratio: f32,
    /// Frequency parameter for a weld-joint spring.
    pub angular_frequency: f32,
    /// Stored joint breaking threshold; runtime application is unresolved.
    pub break_threshold: f32,
    /// Signed joint collision-control word; its individual values have no established meaning.
    pub collision_control: i32,
}

/// Editable M3 ClothCollider values.
#[derive(Clone, Debug, PartialEq)]
pub struct ClothCollider {
    /// Matrix placing the capsule geometry relative to its bone attachment.
    pub transform: [[f32; 4]; 4],
    /// Radial size of the cloth capsule collider.
    pub radius: f32,
    /// Total longitudinal size of the cloth capsule collider.
    pub height: f32,
    /// Attachment frame; absence attaches the collider to model space.
    pub bone: Option<NodeId>,
}

/// Contribution of one simulated cage vertex to a proxy vertex.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CageInfluence {
    /// Vertex ordinal within the referenced cloth cage.
    pub vertex: u32,
    /// Fractional contribution; zero-weight entries carry no relationship.
    pub weight: f32,
}

/// Editable M3 ClothProxy values.
#[derive(Clone, Debug, PartialEq)]
pub struct ClothProxy<'a> {
    /// Mesh receiving cloth deformation.
    pub mesh: MeshId,
    /// Cloth cage used to deform the proxy mesh.
    pub cloth: ClothId,
    /// Four cage-vertex contributions for each proxy vertex.
    pub influences: ZeroVec<'a, [CageInfluence; 4]>,
}

/// Editable M3 Cloth values.
#[derive(Clone, Debug, PartialEq)]
pub struct Cloth<'a> {
    /// Mesh defining the simulated cloth cage.
    pub cage: MeshId,
    /// Four scene-node contributions anchoring each simulated particle.
    pub anchors: ZeroVec<'a, [Option<NodeInfluence>; 4]>,
    /// Unresolved authored cage-and-anchor control.
    pub anchor_control: u32,
    /// Particle movement controls; bit zero permits motion and remaining bits are unresolved.
    pub sim_enabled: ZeroVec<'a, u8>,
    /// Capsule colliders participating in cloth collision.
    pub colliders: Vec<ClothCollider>,
    /// Deformation bindings from simulated cage vertices to output meshes.
    pub proxies: Vec<ClothProxy<'a>>,
    /// Cloth density parameter interpreted by the game solver.
    pub density: f32,
    /// Parameter controlling how cloth tracks its anchors.
    pub tracking: f32,
    /// Resistance parameter for changes in cloth edge length.
    pub stretch_stiffness: f32,
    /// Stiffness parameter for the cloth horizontal constraint direction.
    pub horizontal_stiffness: f32,
    /// Resistance parameter for cloth bending constraints.
    pub bending_stiffness: f32,
    /// Cloth solver parameter attenuating particle motion.
    pub damping: f32,
    /// Contact-friction parameter for this body.
    pub friction: f32,
    /// Scale of the gravity contribution to cloth motion.
    pub gravity: f32,
    /// Multiplier for explosion forces applied to the cloth.
    pub explosion_scale: f32,
    /// Multiplier for wind forces applied to the cloth.
    pub wind_scale: f32,
    /// Resistance parameter for cloth shear constraints.
    pub shear_stiffness: f32,
    /// Coefficient scaling aerodynamic drag on the cloth.
    pub drag_factor: f32,
    /// Coefficient scaling aerodynamic lift on the cloth.
    pub lift_factor: f32,
    /// Response parameter for spherical collision constraints.
    pub sphere_stiffness: f32,
    /// Numeric cloth flattening mode.
    pub flatten: u32,
    /// Animated cloth-enable value; sampling is controlled by the cloth flag bits.
    pub active: Option<Property<u32>>,
    /// Word enabling collision against the skinned mesh.
    pub use_skin_collision: u32,
    /// Offset parameter for collision against the skinned mesh.
    pub skin_offset: f32,
    /// Exponent parameter for the skinned-mesh collision response.
    pub skin_exponent: f32,
    /// Stiffness parameter for the skinned-mesh collision response.
    pub skin_stiffness: f32,
    /// Unresolved channel-control word.
    pub local_channels: u32,
    /// Local wind vector containing both direction and magnitude.
    pub local_wind: [f32; 3],
}

/// Identity domain for authored rigid-body definitions.
pub enum BodyObject {}
/// Stable body handle, checked against its owning physics world's actual members.
pub type BodyId = Id<BodyObject>;
impl<'a> Body<'a> {
    /// Creates a body with a fresh stable identity and unspecified simulation parameters.
    pub fn new(node: NodeId) -> Self {
        Self {
            id: Scope::default().allocate(),
            node,
            simulation_type: 0,
            parameters: None,
            dynamic: None,
            shapes: Vec::new(),
            flags: RigidBodyFlags::default(),
            local_forces: LocalForces::default(),
            world_forces: WorldForces::default(),
            priority: 0,
        }
    }
    /// Stable identity of this body definition.
    pub fn id(&self) -> BodyId {
        self.id
    }
}
/// An authored constraint connecting bodies in one physics world.
#[derive(Clone, Debug, PartialEq)]
pub struct Constraint {
    /// First participating body.
    pub body_a: BodyId,
    /// Second participating body.
    pub body_b: BodyId,
    /// Additional bodies declared dependent on this constraint.
    pub dependents: Vec<BodyId>,
    /// Authored control bits; individual bit meanings remain unresolved.
    pub flags: ConstraintFlags,
    /// Force threshold associated with constraint breakage.
    pub break_force: f32,
}

impl AsULE for CageInfluence {
    type ULE = <(u32, f32) as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        (self.vertex, self.weight).to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let (vertex, weight) = <(u32, f32)>::from_unaligned(value);
        Self { vertex, weight }
    }
}
