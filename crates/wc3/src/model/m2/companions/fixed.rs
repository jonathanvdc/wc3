//! Fixed companion-file records, with their authored units and reserved fields intact.
use super::super as m2;

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// A sparse bone replacement transform stored in BOMT.
pub struct BoneMatrix {
    /// Row-major 4×4 replacement bone transform; translation occupies the final row.
    pub values: [f32; 16],
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// Column-major rotation basis and translation shared by box geometry and joint frames.
pub struct PhysicsFrame {
    /// First column of the frame rotation matrix (local X axis).
    pub axis_x: [f32; 3],
    /// Second column of the frame rotation matrix (local Y axis).
    pub axis_y: [f32; 3],
    /// Third column of the frame rotation matrix (local Z axis).
    pub axis_z: [f32; 3],
    /// Translation of the affine physics frame.
    pub origin: [f32; 3],
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// Oriented box geometry referenced by a SHAP/SHP2 entry.
pub struct PhysicsBox {
    /// Box orientation and origin in the shape's local coordinate system.
    pub frame: PhysicsFrame,
    /// Distances from the box center to its faces along each local axis.
    pub half_extents: [f32; 3],
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// Capsule geometry defined by two local endpoints and a radius.
pub struct PhysicsCapsule {
    /// First endpoint of the capsule center line in local shape coordinates.
    pub position1: [f32; 3],
    /// Second endpoint of the capsule center line in local shape coordinates.
    pub position2: [f32; 3],
    /// Radius in the same length units as the local shape coordinates.
    pub radius: f32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// Sphere geometry defined by a local center and radius.
pub struct PhysicsSphere {
    /// Shape center in local coordinates.
    pub position: [f32; 3],
    /// Radius in the same length units as the local shape coordinates.
    pub radius: f32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// BODY rigid-body definition for PHYS versions 0–1.
pub struct PhysicsBody {
    /// Body behavior: 0 is kinematic, 1 is dynamic; other values select static bodies.
    pub body_type: u16,
    /// Reserved padding with no established semantic meaning; preserved verbatim.
    pub reserved: u16,
    /// Body origin offset relative to its animated bone position, in model coordinate length units.
    pub position: [f32; 3],
    /// Index into the owning model or skeleton bone array.
    pub bone_index: u32,
    /// Index of the first SHAP/SHP2 shape reference belonging to this body.
    pub shape_index: i32,
    /// Number of consecutive SHAP/SHP2 shape references belonging to this body.
    pub shape_count: i32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// BDY2 rigid body, extending BODY with a gravity multiplier.
pub struct PhysicsBody2 {
    /// Earlier BODY fields extended by this record.
    pub body: PhysicsBody,
    /// Multiplier applied to gravity for this body.
    pub gravity_scale: f32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// BDY3 rigid body with damping, inertia, and attachment parameters.
pub struct PhysicsBody3 {
    /// Body behavior: 0 is kinematic, 1 is dynamic; other values select static bodies.
    pub body_type: u16,
    /// Index into the owning model or skeleton bone array.
    pub bone_index: u16,
    /// Body origin offset relative to its animated bone position, in model coordinate length units.
    pub position: [f32; 3],
    /// Index of the first SHAP/SHP2 shape reference belonging to this body.
    pub shape_index: i32,
    /// Number of consecutive SHAP/SHP2 shape references belonging to this body.
    pub shape_count: i32,
    /// Multiplier applied to gravity for this body.
    pub gravity_scale: f32,
    /// Multiplier applied to the body's rotational inertia.
    pub inertia_scale: f32,
    /// Damping parameter applied to linear velocity.
    pub linear_damping: f32,
    /// Damping parameter applied to angular velocity.
    pub angular_damping: f32,
    /// Body attachment encoding: 0x8000 marks an attachment host, 0x4000 a ragdoll root; low bits identify the host body.
    pub attachment: u16,
    /// Reserved padding with no established semantic meaning; preserved verbatim.
    pub reserved: u16,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// BDY4 rigid body with an animated-pose follow factor.
pub struct PhysicsBody4 {
    /// Body behavior: 0 is kinematic, 1 is dynamic; other values select static bodies.
    pub body_type: u16,
    /// Index into the owning model or skeleton bone array.
    pub bone_index: u16,
    /// Body origin offset relative to its animated bone position, in model coordinate length units.
    pub position: [f32; 3],
    /// Index of the first SHAP/SHP2 shape reference belonging to this body.
    pub shape_index: i32,
    /// Number of consecutive SHAP/SHP2 shape references belonging to this body.
    pub shape_count: i32,
    /// Multiplier applied to gravity for this body.
    pub gravity_scale: f32,
    /// Multiplier applied to the body's rotational inertia.
    pub inertia_scale: f32,
    /// Damping parameter applied to linear velocity.
    pub linear_damping: f32,
    /// Damping parameter applied to angular velocity.
    pub angular_damping: f32,
    /// Fraction of a kinematic body's displacement toward its animated pose applied per synchronization step.
    pub follow_factor: f32,
    /// Body attachment encoding: 0x8000 marks an attachment host, 0x4000 a ragdoll root; low bits identify the host body.
    pub attachment: u16,
    /// Reserved padding with no established semantic meaning; preserved verbatim.
    pub reserved: u16,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// SHAP geometry reference and fixture material properties.
pub struct PhysicsShape {
    /// Geometry selector: 0 = BOXS box, 1 = CAPS capsule, 2 = SPHS sphere, 3 = PLYT polytope.
    pub shape_type: u16,
    /// Index into the BOXS/CAPS/SPHS/PLYT geometry array selected by `shape_type`.
    pub shape_index: i16,
    /// Client fixture game flags; individual bit meanings are not established.
    pub game_flags: u16,
    /// Reserved padding with no established semantic meaning; preserved verbatim.
    pub reserved: u16,
    /// Surface friction coefficient used by the physics fixture.
    pub friction: f32,
    /// Coefficient of restitution controlling collision bounce.
    pub restitution: f32,
    /// Density used to determine fixture mass; the source does not establish an absolute mass unit.
    pub density: f32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// SHP2 geometry reference with additional unused serialized fields.
pub struct PhysicsShape2 {
    /// Earlier SHAP material and geometry-reference fields.
    pub shape: PhysicsShape,
    /// Additional serialized fields with no active client use established; preserved verbatim.
    pub unused: [f32; 2],
    /// Unused shape-definition word at offset 0x1c; preserved verbatim.
    pub unused1c: u16,
    /// Reserved padding with no established semantic meaning; preserved verbatim.
    pub reserved: u16,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// JOIN entry connecting two bodies to a typed joint payload.
pub struct PhysicsJoint {
    /// Index of the first connected body in the BODY/BDY2/BDY3/BDY4 array.
    pub body_a: u32,
    /// Index of the second connected body in the BODY/BDY2/BDY3/BDY4 array.
    pub body_b: u32,
    /// Reserved padding with no established semantic meaning; preserved verbatim.
    pub reserved: u32,
    /// Joint payload selector: 0 spherical, 1 shoulder, 2 weld, 3 revolute, 4 prismatic, 5 distance.
    pub joint_type: u16,
    /// Index into the joint-type chunk selected by `joint_type`.
    pub joint_id: i16,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// SHOJ shoulder constraint with degree-valued twist and swing limits.
pub struct PhysicsShoulder {
    /// Joint reference frame in body A's local coordinates.
    pub frame_a: PhysicsFrame,
    /// Joint reference frame in body B's local coordinates.
    pub frame_b: PhysicsFrame,
    /// Lower shoulder twist limit in degrees; the limit is enabled when the upper bound exceeds this value.
    pub lower_twist_angle: f32,
    /// Upper shoulder twist limit in degrees.
    pub upper_twist_angle: f32,
    /// Shoulder swing cone angle in degrees.
    pub cone_angle: f32,
    /// Maximum torque supplied by the joint motor; the source does not establish an absolute torque unit.
    pub max_motor_torque: f32,
    /// Joint motor mode: 0 free, 1 motor; other values retain their numeric encoding.
    pub motor_mode: u32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// SHJ2 shoulder constraint with motor spring frequency and damping.
pub struct PhysicsShoulder2 {
    /// Earlier SHOJ frame, angular-limit, and motor fields.
    pub shoulder: PhysicsShoulder,
    /// Motor spring frequency in hertz.
    pub motor_frequency_hz: f32,
    /// Dimensionless motor damping ratio.
    pub motor_damping_ratio: f32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// WELJ weld constraint with angular softness parameters.
pub struct PhysicsWeld {
    /// Joint reference frame in body A's local coordinates.
    pub frame_a: PhysicsFrame,
    /// Joint reference frame in body B's local coordinates.
    pub frame_b: PhysicsFrame,
    /// Angular constraint spring frequency in hertz; zero selects a rigid constraint.
    pub angular_frequency_hz: f32,
    /// Dimensionless damping ratio for the angular constraint.
    pub angular_damping_ratio: f32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// WLJ2 weld constraint with separate angular and linear softness.
pub struct PhysicsWeld2 {
    /// Earlier weld-joint constraint fields.
    pub weld: PhysicsWeld,
    /// Linear constraint spring frequency in hertz; zero selects a rigid constraint.
    pub linear_frequency_hz: f32,
    /// Dimensionless damping ratio for the linear constraint.
    pub linear_damping_ratio: f32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// WLJ3 weld constraint with one additional unused serialized scalar.
pub struct PhysicsWeld3 {
    /// Earlier weld-joint constraint fields.
    pub weld: PhysicsWeld2,
    /// Additional serialized scalar with no active client use established; preserved verbatim.
    pub unused: f32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// REVJ hinge constraint with degree-valued rotation limits.
pub struct PhysicsRevolute {
    /// Joint reference frame in body A's local coordinates.
    pub frame_a: PhysicsFrame,
    /// Joint reference frame in body B's local coordinates.
    pub frame_b: PhysicsFrame,
    /// Lower revolute rotation limit in degrees; the limit is enabled when the upper bound exceeds this value.
    pub lower_angle: f32,
    /// Upper revolute rotation limit in degrees.
    pub upper_angle: f32,
    /// Maximum torque supplied by the joint motor; the source does not establish an absolute torque unit.
    pub max_motor_torque: f32,
    /// Encoded revolute motor mode; numeric mode behavior is not established by the reference.
    pub motor_mode: u32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// REV2 hinge constraint with motor spring frequency and damping.
pub struct PhysicsRevolute2 {
    /// Earlier REVJ frame, angle-limit, and motor fields.
    pub revolute: PhysicsRevolute,
    /// Motor spring frequency in hertz.
    pub motor_frequency_hz: f32,
    /// Dimensionless motor damping ratio.
    pub motor_damping_ratio: f32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// SPHJ spherical constraint connecting two body-local anchor points.
pub struct PhysicsSpherical {
    /// Joint anchor point in body A's local coordinates.
    pub anchor_a: [f32; 3],
    /// Joint anchor point in body B's local coordinates.
    pub anchor_b: [f32; 3],
    /// Torque opposing relative spherical-joint rotation; the source does not establish an absolute torque unit.
    pub friction_torque: f32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// DSTJ distance constraint maintaining a target anchor separation.
pub struct PhysicsDistance {
    /// Joint anchor point in body A's local coordinates.
    pub anchor_a: [f32; 3],
    /// Joint anchor point in body B's local coordinates.
    pub anchor_b: [f32; 3],
    /// Target separation between the two local anchors, in the anchor coordinate length units.
    pub distance: f32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// PRSJ sliding constraint with translation limits and a motor.
pub struct PhysicsPrismatic {
    /// Joint reference frame in body A's local coordinates.
    pub frame_a: PhysicsFrame,
    /// Joint reference frame in body B's local coordinates.
    pub frame_b: PhysicsFrame,
    /// Lower prismatic translation limit in frame length units; enabled when the upper bound exceeds this value.
    pub lower_limit: f32,
    /// Upper prismatic translation limit in frame length units.
    pub upper_limit: f32,
    /// Translation defining the zero point for the prismatic limits.
    pub reference_translation: f32,
    /// Maximum prismatic motor force; the source does not establish an absolute force unit.
    pub max_motor_force: f32,
    /// Target prismatic translation velocity; the source does not establish its time unit.
    pub motor_speed: f32,
    /// Encoded prismatic motor mode; numeric mode behavior is not established by the reference.
    pub motor_mode: u32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// PRS2 sliding constraint with motor spring frequency and damping.
pub struct PhysicsPrismatic2 {
    /// Earlier PRSJ frame, translation-limit, and motor fields.
    pub prismatic: PhysicsPrismatic,
    /// Motor spring frequency in hertz.
    pub motor_frequency_hz: f32,
    /// Dimensionless motor damping ratio.
    pub motor_damping_ratio: f32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// PHYV interaction and relaxation parameters for vegetation physics.
pub struct PhysicsVegetation {
    /// Maximum vegetation bone displacement before clamping, in yards.
    pub position_max_push: f32,
    /// Vegetation displacement push parameter applied while a unit moves through the model.
    pub position_push_amount: f32,
    /// Rate parameter controlling how vegetation returns to its resting pose.
    pub position_relax_speed: f32,
    /// Maximum additional vegetation push along a moving unit's velocity.
    pub velocity_max_push: f32,
    /// Rate parameter controlling vegetation sway along the interacting unit's motion.
    pub velocity_speed: f32,
    /// Squared proximity threshold at which units begin pushing vegetation.
    pub minimum_push_distance_squared: f32,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// Directed half-edge in a convex PLYT hull.
pub struct PhysicsEdge {
    /// Signed relative edge offset: the twin of edge i is edge i + this value.
    pub twin_offset: i8,
    /// Index into this polytope's vertex array for the half-edge origin.
    pub origin_vertex: u8,
    /// Index into this polytope's face planes and face-first-edge arrays.
    pub face_index: u8,
    /// Index of the next half-edge around the same face.
    pub next_edge: u8,
}

#[derive(Clone, Debug, Default, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
/// Serialized PLYT counts, runtime pointer slots, and geometric measurements.
pub struct PolytopeHeader {
    /// Number of vertex positions in the following polytope data block.
    pub vertex_count: u32,
    /// Uninitialized padding at offset 0x04; preserved verbatim.
    pub reserved04: u32,
    /// Serialized 64-bit runtime vertex pointer, represented as two words; normally zero on disk.
    pub runtime_vertices: [u32; 2],
    /// Number of face planes and face-first-edge entries in the following data block.
    pub face_count: u32,
    /// Uninitialized padding at offset 0x14; preserved verbatim.
    pub reserved14: u32,
    /// Serialized 64-bit runtime face-plane pointer, represented as two words; normally zero on disk.
    pub runtime_planes: [u32; 2],
    /// Serialized 64-bit runtime face-first-edge pointer, represented as two words; normally zero on disk.
    pub runtime_faces: [u32; 2],
    /// Number of half-edge records in the following polytope data block.
    pub edge_count: u32,
    /// Uninitialized padding at offset 0x2c; preserved verbatim.
    pub reserved2c: u32,
    /// Serialized 64-bit runtime half-edge pointer, represented as two words; normally zero on disk.
    pub runtime_edges: [u32; 2],
    /// Volume-weighted centroid of the convex polytope.
    pub centroid: [f32; 3],
    /// Polytope volume, in the cube of its vertex-coordinate length unit.
    pub volume: f32,
    /// Polytope surface area, in the square of its vertex-coordinate length unit.
    pub surface_area: f32,
    /// Uninitialized padding at offset 0x4c; preserved verbatim.
    pub reserved4c: u32,
}
