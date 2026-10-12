//! Collision bodies, constraints, cloth, and soft-body authoring records.
//!
//! Physics object identities are separate from section-table indices. Node, mesh,
//! and draw-section references address the document geometry and hierarchy tables.
//! Profile extensions and preparation recipes retain their own dialect-selected
//! fields. Reading these settings does not run their simulations or preparation.
use super::*;
use crate::model::wem::{self, Bool, InlineOption, Record, Ref};
/// A mesh or material-restricted portion of a soft body.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct SoftPart {
    /// Document mesh index selected as part of the soft body.
    pub mesh: u32,
    /// Material-slot restriction within the mesh; `0xffffffff` selects the entire mesh.
    pub material: u32,
}
impl Record for SoftPart {
    const TAG: [u8; 4] = *b"SPRT";
    const VERSION: u32 = 1;
}

/// A surface attachment represented by a face and barycentric coordinates.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct SoftPlace {
    /// Document mesh index carrying the effect attachment.
    pub mesh: u32,
    /// Face index on the attachment mesh.
    pub face: u32,
    /// Barycentric coordinates in the first triangle of the selected face.
    pub weights: [f32; 3],
    /// Attachment influence radius in model units; zero selects a point.
    pub radius: f32,
}

/// A deformation pivot and its bind-time geometry and node associations.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct SoftLump {
    /// Lump category selecting surface, bone, or copied-bone behavior.
    pub kind: SoftLumpKind,
    /// Bind-time surface position or bone pivot used to establish deformation weights.
    pub anchor: [f32; 3],
    /// Bind-time pivot position expressed in model coordinates.
    pub rest: [f32; 3],
    /// Approximate half-thickness of the body beneath this lump.
    pub depth: f32,
    /// Bind-time separation to interior geometry that deformation should preserve; zero requests none.
    pub clear: f32,
    /// Document node index carrying the lump; bone lumps can use their parent here.
    pub carrier: u32,
    /// Node represented by a bone lump or copied by a copy lump.
    pub node: u32,
    /// Identity of the effect that selected this lump; zero means none.
    pub effect: u32,
    /// One-based entry in the soft body rider list; zero selects no rigid rider.
    pub rider: u16,
    /// Permits jiggle to move this lump.
    pub swings: Bool,
    /// Marks a rigid rider lump, preventing scaling deformation.
    pub rigid: Bool,
}
impl Record for SoftLump {
    const TAG: [u8; 4] = *b"SLMP";
    const VERSION: u32 = 1;
}

/// Inflation and pulsation controls for a soft-body effect.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct SoftInflate {
    /// Direction mode used for inflation.
    pub along: SoftAlong,
    /// Concentrates inflation near the effect attachment and fades it with distance.
    pub toward_place: Bool,
    /// Blend from preserved surface creases toward a rounded body shape.
    pub round: f32,
    /// Requests volume compensation during inflation.
    pub keep_volume: Bool,
    /// Pattern controlling repeated inflation.
    pub pulse: SoftPulse,
    /// Pulse frequency in beats per second.
    pub rate: f32,
    /// Fraction of the inflation lost between pulses.
    pub depth: f32,
    /// Number of smoothing passes applied to surface inflation normals.
    pub smooth: u8,
}

/// Transient or travelling bulge controls for a soft-body effect.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct SoftBulge {
    /// Direction mode used for the bulge.
    pub along: SoftAlong,
    /// Bulge shape, such as a round bump or ring.
    pub shape: SoftBulgeShape,
    /// Requests volume compensation around the bulge.
    pub keep_volume: Bool,
    /// Trigger mode selecting spans, times, events, or contact.
    pub from: SoftComesFrom,
    /// CHAR name of an event used to trigger the effect.
    pub event: Ref<targets::Character>,
    /// Rise duration of an impact bulge, in seconds.
    pub rise: f32,
    /// Decay duration of an impact bulge, in seconds.
    pub fall: f32,
    /// Moves the bulge along its authored path.
    pub travels: Bool,
    /// Node indices defining the travel chain.
    pub chain: Ref<targets::U32>,
    /// Bind-time model-space positions forming the bulge travel path.
    pub line: Ref<targets::Vector3>,
}

/// Oscillation response and authored impulse attachments for a soft body.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct SoftJiggle {
    /// Preset selecting a group of jiggle response parameters.
    pub preset: SoftJigglePreset,
    /// Frequency of the slowest jiggle oscillation, in swings per second.
    pub looseness: f32,
    /// Overshoot control; zero removes rebound overshoot.
    pub bounce: f32,
    /// Gravity-induced sag control.
    pub weight: f32,
    /// Coupling strength between the movement of neighboring points.
    pub hold_shape: f32,
    /// Stiffness of deformation along the volume-changing direction.
    pub keep_volume: f32,
    /// Maximum lump displacement as a fraction of the region size.
    pub limit: f32,
    /// Restrains overall movement while permitting local deformation.
    pub keep_in_place: Bool,
    /// Enables interaction with the floor.
    pub meets_floor: Bool,
    /// Includes world force contributions in the response.
    pub world_forces: Bool,
    /// Identities assigned to authored jiggle impulses.
    pub kick_ids: Ref<targets::U32>,
    /// Mesh indices carrying the authored impulse attachments.
    pub kick_mesh: Ref<targets::U32>,
    /// Face indices carrying the authored impulse attachments.
    pub kick_face: Ref<targets::U32>,
    /// Barycentric coordinates for the impulse attachment faces.
    pub kick_weights: Ref<targets::Vector3>,
    /// Unit model-space impulse directions; an omitted direction points inward at the attachment.
    pub kick_direction: Ref<targets::Vector3>,
    /// Impulse acceleration strengths measured as multiples of gravitational acceleration.
    pub kick_strength: Ref<targets::Real>,
}

/// Pattern and axis controls for a soft-body wobble.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct SoftWobble {
    /// Pattern selecting jelly, sway, rolling, or randomized motion.
    pub pattern: SoftWobblePattern,
    /// Oscillation rate expressed in swings per second.
    pub speed: f32,
    /// Rolling-pattern wavelength measured in region-size units.
    pub size: f32,
    /// Phase delay of the far end relative to the near end.
    pub overlap: f32,
    /// Axis-selection mode used by the wobble.
    pub axis: SoftAxis,
    /// Document node supplying the axis when bone-axis mode is selected.
    pub axis_bone: u32,
    /// Requests volume compensation during wobble.
    pub keep_volume: Bool,
}

/// Rounding and swelling controls for a soft-body effect.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct SoftBlobify {
    /// Degree of rounding applied to the soft body shape.
    pub roundness: f32,
    /// Expansion control for the rounded body.
    pub swell: f32,
    /// Requests volume compensation while reshaping the body.
    pub keep_volume: Bool,
}

/// Drip settings, emission attachments, and generated strand identities.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct SoftDrip {
    /// Preset selecting the drip response settings.
    pub preset: SoftDripPreset,
    /// Law governing how a drip neck narrows.
    pub neck: SoftNeckLaw,
    /// Uses explicitly authored emission sites.
    pub sites_by_hand: Bool,
    /// Requested number of automatically selected emission sites.
    pub sites: u8,
    /// Face-shedding threshold measured in radians from the downward direction.
    pub overhang: f32,
    /// Site separation expressed as a fraction of the region size.
    pub spacing: f32,
    /// Extends the drip toward the ground.
    pub to_ground: Bool,
    /// Strand length as a fraction of region size when ground-reaching mode is disabled.
    pub length: f32,
    /// Drip progression measured in fractions of body height per second.
    pub speed: f32,
    /// Strand thickness as a fraction of the region size.
    pub thickness: f32,
    /// Neck width as a fraction of the strand thickness.
    pub neck_width: f32,
    /// Tip width multiplier relative to the strand thickness.
    pub tip: f32,
    /// Relative timing offset among emission sites.
    pub stagger: f32,
    /// Swing control where zero is stationary and one corresponds to twenty degrees.
    pub sway: f32,
    /// Stretching control for the strands.
    pub pull: f32,
    /// Seconds between detached droplets; zero disables detachment.
    pub pinch_every: f32,
    /// Document material-slot identity for drip geometry; `0xffffffff` inherits the surface material.
    pub material: u32,
    /// Mesh indices on which authored drip sites are attached.
    pub site_mesh: Ref<targets::U32>,
    /// Face indices for the authored drip sites.
    pub site_face: Ref<targets::U32>,
    /// Barycentric coordinates locating the authored sites on their faces.
    pub site_weights: Ref<targets::Vector3>,
    /// Per-site thickness fractions; zero uses the global thickness setting.
    pub site_thickness: Ref<targets::Real>,
    /// Generated strand node indices, three per strand for base, neck, and tip.
    pub strand_nodes: Ref<targets::U32>,
    /// Bind-time model-space positions of the generated strands.
    pub strand_at: Ref<targets::Vector3>,
    /// Generated strand radii in model units.
    pub strand_width: Ref<targets::Real>,
    /// One detached-drop node per emitting strand, ordered with the strands.
    pub drop_nodes: Ref<targets::U32>,
}

/// Axis-based squash and stretch controls for a soft body.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct SoftSquash {
    /// Selects automatic or explicitly keyed squash behavior.
    pub mode: SoftSquashMode,
    /// Axis-selection mode for squash and stretch.
    pub axis: SoftAxis,
    /// Document node supplying the deformation axis when bone-axis mode is used.
    pub axis_bone: u32,
    /// Strength of longitudinal stretching.
    pub stretch: f32,
    /// Strength of compression along the selected axis.
    pub squash: f32,
    /// Response control for changes in motion direction.
    pub turning: f32,
    /// Response control for motion trailing behind the body.
    pub drag: f32,
    /// Control for deformation preceding the main motion.
    pub wind_up: f32,
    /// Anchor mode selecting the bottom, center, or pivot.
    pub anchor: SoftAnchor,
    /// Interval in seconds over which motion speed is smoothed.
    pub smoothing: f32,
}

/// Progressive melting controls and the resulting floor and puddle settings.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct SoftMelt {
    /// Progression mode selecting the top, bottom, or whole body at once.
    pub order: SoftMeltOrder,
    /// Control for resistance to the melting deformation.
    pub solidity: f32,
    /// Uses the grid height as the melting floor instead of `floor`.
    pub on_grid: Bool,
    /// Melting floor height in model units.
    pub floor: f32,
    /// Maximum spread as a multiplier of the body footprint.
    pub puddle: f32,
    /// Amount of irregularity in the melting progression.
    pub uneven: f32,
    /// Reverses the authored progression.
    pub backward: Bool,
    /// Keeps the final state in clips identified as decay animations.
    pub stays_in_decay: Bool,
}

/// Ripple triggering, propagation, and attenuation controls.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct SoftRipple {
    /// Direction mode for the ripple displacement.
    pub along: SoftAlong,
    /// Playback mode selecting one occurrence or repeated ripples.
    pub plays: SoftRipplePlays,
    /// Trigger mode selecting spans, times, events, or contact.
    pub from: SoftComesFrom,
    /// CHAR name of an event used to trigger the ripple.
    pub event: Ref<targets::Character>,
    /// Ripple wavelength measured in region-size units.
    pub wavelength: f32,
    /// Propagation rate measured in region sizes per second.
    pub speed: f32,
    /// Selects propagation through the skin or the body.
    pub through: SoftThrough,
    /// Allows the ripple envelope to travel at a different rate from its crests.
    pub water: Bool,
    /// Envelope propagation speed as a multiplier of crest speed.
    pub envelope: f32,
    /// Distance control for attenuation away from the ripple source.
    pub fade_distance: f32,
    /// Temporal fade duration, in seconds.
    pub fade_time: f32,
}

/// A named deformation effect with its attachment and optional settings payloads.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct SoftEffect {
    /// Stable soft-effect identity used by lump references.
    pub id: u32,
    /// Effect category identifying the applicable settings payload.
    pub kind: SoftEffectKind,
    /// CHAR bytes naming the soft effect.
    pub name: Ref<targets::Character>,
    /// Allows this soft effect to contribute to deformation.
    pub enabled: Bool,
    /// Overall contribution strength of this effect.
    pub amount: f32,
    /// Applies the effect throughout the soft body rather than only near its attachment.
    pub everywhere: Bool,
    /// Allows this effect to excite the jiggle response.
    pub drives_jiggle: Bool,
    /// Randomization seed for reproducible effect choices.
    pub seed: u32,
    /// Mesh-surface attachment and influence radius for the effect.
    pub place: SoftPlace,
    /// Optional inflation settings.
    pub inflate: InlineOption<SoftInflate>,
    /// Optional bulge settings.
    pub bulge: InlineOption<SoftBulge>,
    /// Optional jiggle settings.
    pub jiggle: InlineOption<SoftJiggle>,
    /// Optional wobble settings.
    pub wobble: InlineOption<SoftWobble>,
    /// Optional rounding and swelling settings.
    pub blobify: InlineOption<SoftBlobify>,
    /// Optional drip-generation settings.
    pub drip: InlineOption<SoftDrip>,
    /// Optional squash-and-stretch settings.
    pub squash: InlineOption<SoftSquash>,
    /// Optional melting settings.
    pub melt: InlineOption<SoftMelt>,
    /// Optional ripple propagation settings.
    pub ripple: InlineOption<SoftRipple>,
}
impl Record for SoftEffect {
    const TAG: [u8; 4] = *b"SEFX";
    const VERSION: u32 = 1;
}

/// A group of mesh regions with deformation pivots, effects, and rigid riders.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct SoftBody {
    /// Stable identity of this authored soft body.
    pub id: u32,
    /// CHAR bytes naming the soft body.
    pub name: Ref<targets::Character>,
    /// Document mesh indices participating in the soft body.
    pub meshes: Ref<targets::U32>,
    /// Requested number of surface lumps, excluding lumps selected by effect attachments.
    pub detail: u32,
    /// Copies all participating bones, including unshared bones.
    pub copy_all: Bool,
    /// Keeps bones with descendants as rotational chain elements.
    pub chains_as_they_are: Bool,
    /// Mesh/material subsets treated as interior geometry.
    pub inside: Ref<targets::SoftPart>,
    /// Mesh/material subsets carried as rigid parts.
    pub riders: Ref<targets::SoftPart>,
    /// Spatial influence blending control expressed as a fraction of region size.
    pub blend: f32,
    /// Maximum influence count used for soft deformation weights.
    pub influences: u8,
    /// Uses equal-share influence groups for classic-profile skinning.
    pub equal_shares: Bool,
    /// Deformation pivots and their node/rider associations.
    pub lumps: Ref<targets::SoftLump>,
    /// Authored deformation and generation effects for this soft body.
    pub effects: Ref<targets::SoftEffect>,
    /// Physics collider identities participating in the soft-body response.
    pub colliders: Ref<targets::U32>,
    /// Document mesh index created by drip generation; `0xffffffff` means no generated mesh.
    pub drips: u32,
    /// Next identity to allocate for a new soft effect.
    pub next_effect: u32,
}
impl Record for SoftBody {
    const TAG: [u8; 4] = *b"SBDY";
    const VERSION: u32 = 1;
}

/// Density and contact coefficients assigned to collision geometry.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct PhysicsMaterial {
    /// Density coefficient used to derive body mass from shape volume.
    pub density: f32,
    /// Contact friction coefficient.
    pub friction: f32,
    /// Contact rebound coefficient.
    pub restitution: f32,
}

/// Static field selections for `PhysicsShape` section or inline context revisions.
pub trait PhysicsShapeDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `game_flags`; `()` omits this slot.
    type GameFlags: NativeField;
}

impl PhysicsShapeDialect for V1 {
    const VERSION: u32 = 1;
    type GameFlags = ();
}

impl PhysicsShapeDialect for V2 {
    const VERSION: u32 = 2;
    type GameFlags = u16;
}

/// Collision geometry expressed in a body-relative coordinate frame.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct PhysicsShape<D: PhysicsShapeDialect> {
    /// Collision shape category: box, sphere, capsule, cylinder, convex hull, or triangle mesh.
    pub kind: PhysicsShapeKind,
    /// Full matrix mapping shape coordinates into the attached body node frame.
    pub transform: [[f32; 4]; 4],
    /// Box half-sizes along the three shape axes.
    pub half_extents: [f32; 3],
    /// Radius parameter for spheres, capsules, and cylinders.
    pub radius: f32,
    /// Extent along local +Z: cylinder length or separation of capsule cap centers.
    pub length: f32,
    /// Shape-space hull points or explicitly authored capsule cap centers.
    pub points: Ref<targets::Vector3>,
    /// Triangle-mesh positions in the shape coordinate frame.
    pub vertices: Ref<targets::Vector3>,
    /// Triangle vertex indices grouped in triples.
    pub triangles: Ref<targets::U32>,
    /// Density, friction, and restitution assigned to the shape.
    pub material: PhysicsMaterial,
    /// Profile-specific contact-callback bits selected by this shape dialect.
    pub game_flags: D::GameFlags,
}
impl<D: PhysicsShapeDialect> Record for PhysicsShape<D> {
    const TAG: [u8; 4] = *b"PSHP";
    const VERSION: u32 = D::VERSION;
}

/// StarCraft II collision-filter and bone-drive settings.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct Sc2BodyExtension {
    /// StarCraft II physics-material identity, distinct from a render material slot.
    pub physics_material: u32,
    /// Enables the collision-filter collidable bit.
    pub collidable: Bool,
    /// Enables the collision-filter walkable bit.
    pub walkable: Bool,
    /// Enables the collision-filter stackable bit.
    pub stackable: Bool,
    /// Enables the collision-filter simulation bit.
    pub simulate_collision: Bool,
    /// Retains the bone physics-state bit during setup and deactivation.
    pub keeps_bone_driven: Bool,
    /// Additional authored body bits whose runtime interpretation is unresolved.
    pub untraced_flags: u32,
}

/// World of Warcraft body-following and ragdoll grouping settings.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct WowBodyExtension {
    /// Fraction of the kinematic-body displacement toward its driving bone applied per step.
    pub follow_factor: f32,
    /// Marks a kinematic body that carries dependent bodies.
    pub has_children: Bool,
    /// Marks the root whose kinematic motion carries the ragdoll groups.
    pub ragdoll_root: Bool,
    /// Identity of the kinematic parent body for a dynamic body.
    pub parent: u32,
}

/// Static field selections for `PhysicsBody` section or inline context revisions.
pub trait PhysicsBodyDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `wow`; `()` omits this slot.
    type Wow: NativeField;
    /// Representation of `locked`; `()` omits this slot.
    type Locked: NativeField;
}

impl PhysicsBodyDialect for V1 {
    const VERSION: u32 = 1;
    type Wow = ();
    type Locked = ();
}

impl PhysicsBodyDialect for V2 {
    const VERSION: u32 = 2;
    type Wow = InlineOption<WowBodyExtension>;
    type Locked = ();
}

impl PhysicsBodyDialect for V3 {
    const VERSION: u32 = 3;
    type Wow = InlineOption<WowBodyExtension>;
    type Locked = Bool;
}

/// A node attachment with collision geometry and motion settings.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct PhysicsBody<D: PhysicsBodyDialect> {
    /// Stable identity of this physics body.
    pub id: u32,
    /// Global document node index attached to the body.
    pub node: u32,
    /// Motion category selecting dynamic, kinematic, or static behavior.
    pub motion: BodyMotion,
    /// Default dynamic state used when no animation key overrides it.
    pub simulates: Bool,
    /// Collision shapes attached to this body.
    pub shapes: Ref<targets::PhysicsShape>,
    /// Attenuation coefficient for translational velocity.
    pub linear_damping: f32,
    /// Attenuation coefficient for rotational velocity.
    pub angular_damping: f32,
    /// Multiplier applied to body inertia.
    pub inertia_scale: f32,
    /// Multiplier applied to gravity for this body.
    pub gravity_scale: f32,
    /// Uses the nearest ancestor body dynamic state rather than its own.
    pub inherit_dynamic: Bool,
    /// Keeps the body kinematic during ragdoll activation.
    pub exempt_from_ragdoll: Bool,
    /// Force-field channel mask; low and high 16-bit halves retain local and world channels.
    pub force_channels: u32,
    /// Optional StarCraft II body settings.
    pub sc2: InlineOption<Sc2BodyExtension>,
    /// Optional World of Warcraft kinematic-drive and grouping settings.
    pub wow: D::Wow,
    /// Protects manually tuned body shapes from bulk authoring operations.
    pub locked: D::Locked,
}
impl<D: PhysicsBodyDialect> Record for PhysicsBody<D> {
    const TAG: [u8; 4] = *b"PBDY";
    const VERSION: u32 = D::VERSION;
}

/// Frequency and damping controls for a joint spring.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct JointSpring {
    /// Spring frequency expressed in hertz.
    pub hz: f32,
    /// Spring damping coefficient.
    pub damping: f32,
}

/// Static field selections for `PhysicsJoint` section or inline context revisions.
pub trait PhysicsJointDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `motor`; `()` omits this slot.
    type Motor: NativeField;
    /// Representation of `max_motor_force`; `()` omits this slot.
    type MaxMotorForce: NativeField;
    /// Representation of `motor_speed`; `()` omits this slot.
    type MotorSpeed: NativeField;
    /// Representation of `reference_translation`; `()` omits this slot.
    type ReferenceTranslation: NativeField;
    /// Representation of `locked`; `()` omits this slot.
    type Locked: NativeField;
}

impl PhysicsJointDialect for V1 {
    const VERSION: u32 = 1;
    type Motor = ();
    type MaxMotorForce = ();
    type MotorSpeed = ();
    type ReferenceTranslation = ();
    type Locked = ();
}

impl PhysicsJointDialect for V2 {
    const VERSION: u32 = 2;
    type Motor = JointMotorMode;
    type MaxMotorForce = f32;
    type MotorSpeed = f32;
    type ReferenceTranslation = f32;
    type Locked = ();
}

impl PhysicsJointDialect for V3 {
    const VERSION: u32 = 3;
    type Motor = JointMotorMode;
    type MaxMotorForce = f32;
    type MotorSpeed = f32;
    type ReferenceTranslation = f32;
    type Locked = Bool;
}

/// A constraint joining two physics body identities.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct PhysicsJoint<D: PhysicsJointDialect> {
    /// Stable identity of this joint.
    pub id: u32,
    /// Physics body identity on the parent side of the joint.
    pub body_a: u32,
    /// Physics body identity on the other side of the joint.
    pub body_b: u32,
    /// Constraint category selecting spherical, revolute, cone-twist, weld, prismatic, or distance behavior.
    pub kind: JointKind,
    /// Joint frame expressed in the node frame of body A.
    pub frame_a: [[f32; 4]; 4],
    /// Joint frame expressed in the node frame of body B.
    pub frame_b: [[f32; 4]; 4],
    /// Permits collision between the two connected bodies.
    pub collide_connected: Bool,
    /// Enables the stored motion limits.
    pub limit_enabled: Bool,
    /// Lower limit in radians, or distance for a prismatic joint.
    pub lower: f32,
    /// Upper limit in radians, or distance for a prismatic joint.
    pub upper: f32,
    /// Cone-twist angular limit in radians.
    pub cone: f32,
    /// Joint-friction mode.
    pub friction: JointFriction,
    /// Strength parameter for the selected friction mode.
    pub friction_amount: f32,
    /// Spring parameters for angular constraint motion.
    pub angular_spring: JointSpring,
    /// Spring parameters for translational constraint motion.
    pub linear_spring: JointSpring,
    /// Resting separation of a distance joint.
    pub rest_length: f32,
    /// Breaking-force threshold; zero makes the force limit unbreakable.
    pub break_force: f32,
    /// Breaking-torque threshold; zero makes the torque limit unbreakable.
    pub break_torque: f32,
    /// Motor mode selecting off, position, or velocity control.
    pub motor: D::Motor,
    /// Motor torque limit, or force limit for a prismatic joint.
    pub max_motor_force: D::MaxMotorForce,
    /// Target motor speed; prismatic motors use translational velocity.
    pub motor_speed: D::MotorSpeed,
    /// Reference position from which prismatic limits are measured.
    pub reference_translation: D::ReferenceTranslation,
    /// Protects manually tuned joint settings from bulk authoring operations.
    pub locked: D::Locked,
}
impl<D: PhysicsJointDialect> Record for PhysicsJoint<D> {
    const TAG: [u8; 4] = *b"PJNT";
    const VERSION: u32 = D::VERSION;
}

/// A draw-section index paired with the document mesh that owns it.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct SectionRef {
    /// Document mesh index containing the selected draw section.
    pub mesh: u32,
    /// Draw-section index within that mesh.
    pub section: u32,
}

/// Static field selections for `ClothCollider` section or inline context revisions.
pub trait ClothColliderDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `body`; `()` omits this slot.
    type Body: NativeField;
    /// Representation of `shape`; `()` omits this slot.
    type Shape: NativeField;
}

impl ClothColliderDialect for V1 {
    const VERSION: u32 = 1;
    type Body = ();
    type Shape = ();
}

impl ClothColliderDialect for V2 {
    const VERSION: u32 = 2;
    type Body = u32;
    type Shape = u32;
}

/// A node-attached collision primitive used by cloth and soft bodies.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct ClothCollider<D: ClothColliderDialect> {
    /// Stable identity of this cloth collider.
    pub id: u32,
    /// Document node index attaching the collider; `0xffffffff` selects the model root.
    pub node: u32,
    /// Cloth collision primitive selecting a capsule or plane.
    pub kind: ClothColliderKind,
    /// Matrix placing the collider relative to its attached node.
    pub transform: [[f32; 4]; 4],
    /// Radial size of a capsule collider.
    pub radius: f32,
    /// Longitudinal size of a capsule collider.
    pub length: f32,
    /// Identity of the body whose collision shape drives this collider; zero disables following.
    pub body: D::Body,
    /// Shape index within the driving body selected by `body`.
    pub shape: D::Shape,
}
impl<D: ClothColliderDialect> Record for ClothCollider<D> {
    const TAG: [u8; 4] = *b"PCOL";
    const VERSION: u32 = D::VERSION;
}

/// A draw-section attachment receiving cloth deformation.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct ClothBinding {
    /// Mesh/draw-section pair receiving deformation from the cloth cage.
    pub section: SectionRef,
}
impl Record for ClothBinding {
    const TAG: [u8; 4] = *b"CBND";
    const VERSION: u32 = 1;
}

/// StarCraft II cloth response and skinned-surface collision settings.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct Sc2ClothParams {
    /// Strength of cloth tracking toward its anchors.
    pub tracking: f32,
    /// Resistance of horizontal cloth constraints.
    pub horizontal_stiffness: f32,
    /// Resistance of shear constraints.
    pub shear_stiffness: f32,
    /// Multiplier for explosion forces applied to cloth.
    pub explosion_scale: f32,
    /// Multiplier for wind forces applied to cloth.
    pub wind_scale: f32,
    /// Aerodynamic drag coefficient.
    pub drag_factor: f32,
    /// Aerodynamic lift coefficient.
    pub lift_factor: f32,
    /// Response stiffness for sphere collisions.
    pub sphere_stiffness: f32,
    /// Enables flattening of the simulated cloth.
    pub flatten: Bool,
    /// Enables contact with the skinned mesh.
    pub use_skin_collision: Bool,
    /// Surface offset used by skinned-mesh contact.
    pub skin_offset: f32,
    /// Exponent controlling the skinned-mesh contact response.
    pub skin_exponent: f32,
    /// Stiffness of the skinned-mesh contact response.
    pub skin_stiffness: f32,
}

/// Authoring settings for cage generation, anchor selection, and motion baking.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct ClothRecipe {
    /// CHAR bytes naming this cloth preparation recipe.
    pub name: Ref<targets::Character>,
    /// Document node indices selected as candidate cloth anchors.
    pub bones: Ref<targets::U32>,
    /// Minimum fraction of skin weight on the selected cloth bones needed for a particle to move; lower fractions are pinned.
    pub threshold: f32,
    /// Cage-generation mode.
    pub cage: ClothCageKind,
    /// Requested particle count for a generated cage.
    pub particles: u32,
    /// Maximum binding distance as a fraction of cloth size.
    pub reach: f32,
    /// Uses manually authored pinned particles.
    pub pins_by_hand: Bool,
    /// Output mode selecting bone animation or full-detail deformation.
    pub bake_into: ClothBakeInto,
    /// Original draw-section index for each binding, within the corresponding binding mesh.
    pub from: Ref<targets::U32>,
    /// Original draw-section index within the cage mesh; `0xffffffff` denotes a procedurally generated cage.
    pub cage_from: u32,
}

/// Static field selections for `Cloth` section or inline context revisions.
pub trait ClothDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `recipe`; `()` omits this slot.
    type Recipe: NativeField;
}

impl ClothDialect for V1 {
    const VERSION: u32 = 1;
    type Recipe = ();
}

impl ClothDialect for V2 {
    const VERSION: u32 = 2;
    type Recipe = InlineOption<ClothRecipe>;
}

/// A simulated cage with output mesh bindings and collision settings.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct Cloth<D: ClothDialect> {
    /// Stable identity of this cloth object.
    pub id: u32,
    /// Mesh/draw-section pair providing the simulated cage.
    pub cage: SectionRef,
    /// Mesh sections deformed by the cage.
    pub bindings: Ref<targets::ClothBinding>,
    /// Collider identities participating in cloth collision.
    pub colliders: Ref<targets::U32>,
    /// Default enabled state of the cloth simulation.
    pub active: Bool,
    /// Density parameter assigned to the cloth.
    pub density: f32,
    /// Attenuation coefficient for cloth motion.
    pub damping: f32,
    /// Contact friction coefficient.
    pub friction: f32,
    /// Resistance to cloth stretching.
    pub stretch_stiffness: f32,
    /// Resistance to cloth bending.
    pub bend_stiffness: f32,
    /// Multiplier applied to the cloth gravity contribution.
    pub gravity_scale: f32,
    /// Wind vector assigned to the cloth.
    pub wind: [f32; 3],
    /// Optional StarCraft II solver parameters.
    pub sc2: InlineOption<Sc2ClothParams>,
    /// Optional preparation recipe selected by the cloth dialect.
    pub recipe: D::Recipe,
}
impl<D: ClothDialect> Record for Cloth<D> {
    const TAG: [u8; 4] = *b"CLTH";
    const VERSION: u32 = D::VERSION;
}

/// World of Warcraft vegetation push and relaxation parameters.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct WowVegetation {
    /// Maximum displacement contribution from position pushing.
    pub pos_max_push: f32,
    /// Strength of the position-push response.
    pub pos_push_amt: f32,
    /// Rate at which displaced vegetation returns toward its rest position.
    pub pos_relax_speed: f32,
    /// Maximum velocity-push contribution.
    pub vel_max_push: f32,
    /// Rate control for the velocity-driven response.
    pub vel_speed: f32,
    /// Squared-distance threshold for the push interaction.
    pub min_push_dist: f32,
}

/// World of Warcraft rig categorization and host-skeleton behavior settings.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct WowRigExtension {
    /// World of Warcraft rig behavior category.
    pub kind: WowPhysicsKind,
    /// Optional vegetation-response settings.
    pub vegetation: InlineOption<WowVegetation>,
    /// Host-skeleton name CRCs permitted to retain authored body follow factors.
    pub allow_list: Ref<targets::U32>,
}

/// Preparation settings for skeleton-derived collision bodies and joints.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct RagdollRecipe {
    /// Shape-generation mode for the torso.
    pub torso: PhysicsShapeKind,
    /// Shape-generation mode for the head.
    pub head: PhysicsShapeKind,
    /// Shape-generation mode for limb bodies.
    pub limbs: PhysicsShapeKind,
    /// Shape-generation mode for attached props.
    pub props: PhysicsShapeKind,
    /// Point-count control for generated convex hulls.
    pub hull_points: u8,
    /// Fit control determining how closely generated bodies follow the geometry.
    pub tightness: f32,
    /// Thickness control for generated collision shapes.
    pub thickness: f32,
    /// Joint-range selection mode.
    pub range: u8,
    /// Prefers hinge constraints where the preparation supports them.
    pub hinges: Bool,
    /// Permits collision between connected generated bodies.
    pub collide_connected: Bool,
    /// Combines short skeleton links during ragdoll preparation.
    pub fold_short_links: Bool,
    /// Welds attached props to their carrying body.
    pub weld_props: Bool,
    /// Physics-material preset selected for generated bodies.
    pub material: u32,
    /// Translational damping applied to generated bodies.
    pub linear_damping: f32,
    /// Rotational damping applied to generated bodies.
    pub angular_damping: f32,
}

/// Static field selections for `FractureRecipe` section or inline context revisions.
pub trait FractureRecipeDialect: Revision {
    /// Representation of `method`; `()` omits this slot.
    type Method: NativeField;
    /// Representation of `total`; `()` omits this slot.
    type Total: NativeField;
    /// Representation of `pinned`; `()` omits this slot.
    type Pinned: NativeField;
    /// Representation of `slice_along`; `()` omits this slot.
    type SliceAlong: NativeField;
    /// Representation of `slice_directions`; `()` omits this slot.
    type SliceDirections: NativeField;
    /// Representation of `slice_counts`; `()` omits this slot.
    type SliceCounts: NativeField;
    /// Representation of `slice_jitters`; `()` omits this slot.
    type SliceJitters: NativeField;
    /// Representation of `slice_tilts`; `()` omits this slot.
    type SliceTilts: NativeField;
    /// Representation of `slice_shifts`; `()` omits this slot.
    type SliceShifts: NativeField;
    /// Representation of `plane_normals`; `()` omits this slot.
    type PlaneNormals: NativeField;
    /// Representation of `plane_offsets`; `()` omits this slot.
    type PlaneOffsets: NativeField;
    /// Representation of `push`; `()` omits this slot.
    type Push: NativeField;
    /// Representation of `release`; `()` omits this slot.
    type Release: NativeField;
    /// Representation of `over`; `()` omits this slot.
    type Over: NativeField;
    /// Representation of `loosen`; `()` omits this slot.
    type Loosen: NativeField;
    /// Representation of `centre`; `()` omits this slot.
    type Centre: NativeField;
    /// Representation of `radius`; `()` omits this slot.
    type Radius: NativeField;
    /// Representation of `strength`; `()` omits this slot.
    type Strength: NativeField;
    /// Representation of `length`; `()` omits this slot.
    type Length: NativeField;
    /// Representation of `at`; `()` omits this slot.
    type At: NativeField;
    /// Representation of `filling_uvs`; `()` omits this slot.
    type FillingUvs: NativeField;
    /// Representation of `map_size`; `()` omits this slot.
    type MapSize: NativeField;
    /// Representation of `filling_made`; `()` omits this slot.
    type FillingMade: NativeField;
    /// Representation of `before`; `()` omits this slot.
    type Before: NativeField;
    /// Representation of `whole_gate`; `()` omits this slot.
    type WholeGate: NativeField;
    /// Representation of `pieces_gate`; `()` omits this slot.
    type PiecesGate: NativeField;
    /// Representation of `plane_centres`; `()` omits this slot.
    type PlaneCentres: NativeField;
    /// Representation of `plane_alongs`; `()` omits this slot.
    type PlaneAlongs: NativeField;
    /// Representation of `plane_widths`; `()` omits this slot.
    type PlaneWidths: NativeField;
    /// Representation of `plane_heights`; `()` omits this slot.
    type PlaneHeights: NativeField;
    /// Representation of `cut_skeleton`; `()` omits this slot.
    type CutSkeleton: NativeField;
    /// Representation of `bones`; `()` omits this slot.
    type Bones: NativeField;
    /// Representation of `cut_ragdoll`; `()` omits this slot.
    type CutRagdoll: NativeField;
}

impl FractureRecipeDialect for V0 {
    type Method = ();
    type Total = ();
    type Pinned = ();
    type SliceAlong = ();
    type SliceDirections = ();
    type SliceCounts = ();
    type SliceJitters = ();
    type SliceTilts = ();
    type SliceShifts = ();
    type PlaneNormals = ();
    type PlaneOffsets = ();
    type Push = ();
    type Release = ();
    type Over = ();
    type Loosen = ();
    type Centre = ();
    type Radius = ();
    type Strength = ();
    type Length = ();
    type At = ();
    type FillingUvs = ();
    type MapSize = ();
    type FillingMade = ();
    type Before = ();
    type WholeGate = ();
    type PiecesGate = ();
    type PlaneCentres = ();
    type PlaneAlongs = ();
    type PlaneWidths = ();
    type PlaneHeights = ();
    type CutSkeleton = ();
    type Bones = ();
    type CutRagdoll = ();
}

impl FractureRecipeDialect for V1 {
    type Method = ();
    type Total = ();
    type Pinned = ();
    type SliceAlong = ();
    type SliceDirections = ();
    type SliceCounts = ();
    type SliceJitters = ();
    type SliceTilts = ();
    type SliceShifts = ();
    type PlaneNormals = ();
    type PlaneOffsets = ();
    type Push = ();
    type Release = ();
    type Over = ();
    type Loosen = ();
    type Centre = ();
    type Radius = ();
    type Strength = ();
    type Length = ();
    type At = ();
    type FillingUvs = ();
    type MapSize = ();
    type FillingMade = ();
    type Before = ();
    type WholeGate = ();
    type PiecesGate = ();
    type PlaneCentres = ();
    type PlaneAlongs = ();
    type PlaneWidths = ();
    type PlaneHeights = ();
    type CutSkeleton = ();
    type Bones = ();
    type CutRagdoll = ();
}

impl FractureRecipeDialect for V2 {
    type Method = ();
    type Total = ();
    type Pinned = ();
    type SliceAlong = ();
    type SliceDirections = ();
    type SliceCounts = ();
    type SliceJitters = ();
    type SliceTilts = ();
    type SliceShifts = ();
    type PlaneNormals = ();
    type PlaneOffsets = ();
    type Push = ();
    type Release = ();
    type Over = ();
    type Loosen = ();
    type Centre = ();
    type Radius = ();
    type Strength = ();
    type Length = ();
    type At = ();
    type FillingUvs = ();
    type MapSize = ();
    type FillingMade = ();
    type Before = ();
    type WholeGate = ();
    type PiecesGate = ();
    type PlaneCentres = ();
    type PlaneAlongs = ();
    type PlaneWidths = ();
    type PlaneHeights = ();
    type CutSkeleton = ();
    type Bones = ();
    type CutRagdoll = ();
}

impl FractureRecipeDialect for V3 {
    type Method = ();
    type Total = ();
    type Pinned = ();
    type SliceAlong = ();
    type SliceDirections = ();
    type SliceCounts = ();
    type SliceJitters = ();
    type SliceTilts = ();
    type SliceShifts = ();
    type PlaneNormals = ();
    type PlaneOffsets = ();
    type Push = ();
    type Release = ();
    type Over = ();
    type Loosen = ();
    type Centre = ();
    type Radius = ();
    type Strength = ();
    type Length = ();
    type At = ();
    type FillingUvs = ();
    type MapSize = ();
    type FillingMade = ();
    type Before = ();
    type WholeGate = ();
    type PiecesGate = ();
    type PlaneCentres = ();
    type PlaneAlongs = ();
    type PlaneWidths = ();
    type PlaneHeights = ();
    type CutSkeleton = ();
    type Bones = ();
    type CutRagdoll = ();
}

impl FractureRecipeDialect for V4 {
    type Method = ();
    type Total = ();
    type Pinned = ();
    type SliceAlong = ();
    type SliceDirections = ();
    type SliceCounts = ();
    type SliceJitters = ();
    type SliceTilts = ();
    type SliceShifts = ();
    type PlaneNormals = ();
    type PlaneOffsets = ();
    type Push = ();
    type Release = ();
    type Over = ();
    type Loosen = ();
    type Centre = ();
    type Radius = ();
    type Strength = ();
    type Length = ();
    type At = ();
    type FillingUvs = ();
    type MapSize = ();
    type FillingMade = ();
    type Before = ();
    type WholeGate = ();
    type PiecesGate = ();
    type PlaneCentres = ();
    type PlaneAlongs = ();
    type PlaneWidths = ();
    type PlaneHeights = ();
    type CutSkeleton = ();
    type Bones = ();
    type CutRagdoll = ();
}

impl FractureRecipeDialect for V5 {
    type Method = u8;
    type Total = u16;
    type Pinned = Ref<targets::Byte>;
    type SliceAlong = Ref<targets::Byte>;
    type SliceDirections = Ref<targets::Vector3>;
    type SliceCounts = Ref<targets::U16>;
    type SliceJitters = Ref<targets::Real>;
    type SliceTilts = Ref<targets::Real>;
    type SliceShifts = Ref<targets::Real>;
    type PlaneNormals = Ref<targets::Vector3>;
    type PlaneOffsets = Ref<targets::Real>;
    type Push = u8;
    type Release = u8;
    type Over = f32;
    type Loosen = f32;
    type Centre = [f32; 3];
    type Radius = f32;
    type Strength = f32;
    type Length = f32;
    type At = f32;
    type FillingUvs = u8;
    type MapSize = u32;
    type FillingMade = Bool;
    type Before = ();
    type WholeGate = ();
    type PiecesGate = ();
    type PlaneCentres = ();
    type PlaneAlongs = ();
    type PlaneWidths = ();
    type PlaneHeights = ();
    type CutSkeleton = ();
    type Bones = ();
    type CutRagdoll = ();
}

impl FractureRecipeDialect for V6 {
    type Method = u8;
    type Total = u16;
    type Pinned = Ref<targets::Byte>;
    type SliceAlong = Ref<targets::Byte>;
    type SliceDirections = Ref<targets::Vector3>;
    type SliceCounts = Ref<targets::U16>;
    type SliceJitters = Ref<targets::Real>;
    type SliceTilts = Ref<targets::Real>;
    type SliceShifts = Ref<targets::Real>;
    type PlaneNormals = Ref<targets::Vector3>;
    type PlaneOffsets = Ref<targets::Real>;
    type Push = u8;
    type Release = u8;
    type Over = f32;
    type Loosen = f32;
    type Centre = [f32; 3];
    type Radius = f32;
    type Strength = f32;
    type Length = f32;
    type At = f32;
    type FillingUvs = u8;
    type MapSize = u32;
    type FillingMade = Bool;
    type Before = u8;
    type WholeGate = u32;
    type PiecesGate = u32;
    type PlaneCentres = ();
    type PlaneAlongs = ();
    type PlaneWidths = ();
    type PlaneHeights = ();
    type CutSkeleton = ();
    type Bones = ();
    type CutRagdoll = ();
}

impl FractureRecipeDialect for V7 {
    type Method = u8;
    type Total = u16;
    type Pinned = Ref<targets::Byte>;
    type SliceAlong = Ref<targets::Byte>;
    type SliceDirections = Ref<targets::Vector3>;
    type SliceCounts = Ref<targets::U16>;
    type SliceJitters = Ref<targets::Real>;
    type SliceTilts = Ref<targets::Real>;
    type SliceShifts = Ref<targets::Real>;
    type PlaneNormals = Ref<targets::Vector3>;
    type PlaneOffsets = Ref<targets::Real>;
    type Push = u8;
    type Release = u8;
    type Over = f32;
    type Loosen = f32;
    type Centre = [f32; 3];
    type Radius = f32;
    type Strength = f32;
    type Length = f32;
    type At = f32;
    type FillingUvs = u8;
    type MapSize = u32;
    type FillingMade = Bool;
    type Before = u8;
    type WholeGate = u32;
    type PiecesGate = u32;
    type PlaneCentres = Ref<targets::Vector3>;
    type PlaneAlongs = Ref<targets::Vector3>;
    type PlaneWidths = Ref<targets::Real>;
    type PlaneHeights = Ref<targets::Real>;
    type CutSkeleton = Bool;
    type Bones = Ref<targets::U32>;
    type CutRagdoll = ();
}

impl FractureRecipeDialect for V8 {
    type Method = u8;
    type Total = u16;
    type Pinned = Ref<targets::Byte>;
    type SliceAlong = Ref<targets::Byte>;
    type SliceDirections = Ref<targets::Vector3>;
    type SliceCounts = Ref<targets::U16>;
    type SliceJitters = Ref<targets::Real>;
    type SliceTilts = Ref<targets::Real>;
    type SliceShifts = Ref<targets::Real>;
    type PlaneNormals = Ref<targets::Vector3>;
    type PlaneOffsets = Ref<targets::Real>;
    type Push = u8;
    type Release = u8;
    type Over = f32;
    type Loosen = f32;
    type Centre = [f32; 3];
    type Radius = f32;
    type Strength = f32;
    type Length = f32;
    type At = f32;
    type FillingUvs = u8;
    type MapSize = u32;
    type FillingMade = Bool;
    type Before = u8;
    type WholeGate = u32;
    type PiecesGate = u32;
    type PlaneCentres = Ref<targets::Vector3>;
    type PlaneAlongs = Ref<targets::Vector3>;
    type PlaneWidths = Ref<targets::Real>;
    type PlaneHeights = Ref<targets::Real>;
    type CutSkeleton = Bool;
    type Bones = Ref<targets::U32>;
    type CutRagdoll = Bool;
}

impl FractureRecipeDialect for V9 {
    type Method = u8;
    type Total = u16;
    type Pinned = Ref<targets::Byte>;
    type SliceAlong = Ref<targets::Byte>;
    type SliceDirections = Ref<targets::Vector3>;
    type SliceCounts = Ref<targets::U16>;
    type SliceJitters = Ref<targets::Real>;
    type SliceTilts = Ref<targets::Real>;
    type SliceShifts = Ref<targets::Real>;
    type PlaneNormals = Ref<targets::Vector3>;
    type PlaneOffsets = Ref<targets::Real>;
    type Push = u8;
    type Release = u8;
    type Over = f32;
    type Loosen = f32;
    type Centre = [f32; 3];
    type Radius = f32;
    type Strength = f32;
    type Length = f32;
    type At = f32;
    type FillingUvs = u8;
    type MapSize = u32;
    type FillingMade = Bool;
    type Before = u8;
    type WholeGate = u32;
    type PiecesGate = u32;
    type PlaneCentres = Ref<targets::Vector3>;
    type PlaneAlongs = Ref<targets::Vector3>;
    type PlaneWidths = Ref<targets::Real>;
    type PlaneHeights = Ref<targets::Real>;
    type CutSkeleton = Bool;
    type Bones = Ref<targets::U32>;
    type CutRagdoll = Bool;
}

impl FractureRecipeDialect for V10 {
    type Method = u8;
    type Total = u16;
    type Pinned = Ref<targets::Byte>;
    type SliceAlong = Ref<targets::Byte>;
    type SliceDirections = Ref<targets::Vector3>;
    type SliceCounts = Ref<targets::U16>;
    type SliceJitters = Ref<targets::Real>;
    type SliceTilts = Ref<targets::Real>;
    type SliceShifts = Ref<targets::Real>;
    type PlaneNormals = Ref<targets::Vector3>;
    type PlaneOffsets = Ref<targets::Real>;
    type Push = u8;
    type Release = u8;
    type Over = f32;
    type Loosen = f32;
    type Centre = [f32; 3];
    type Radius = f32;
    type Strength = f32;
    type Length = f32;
    type At = f32;
    type FillingUvs = u8;
    type MapSize = u32;
    type FillingMade = Bool;
    type Before = u8;
    type WholeGate = u32;
    type PiecesGate = u32;
    type PlaneCentres = Ref<targets::Vector3>;
    type PlaneAlongs = Ref<targets::Vector3>;
    type PlaneWidths = Ref<targets::Real>;
    type PlaneHeights = Ref<targets::Real>;
    type CutSkeleton = Bool;
    type Bones = Ref<targets::U32>;
    type CutRagdoll = Bool;
}

impl FractureRecipeDialect for V11 {
    type Method = u8;
    type Total = u16;
    type Pinned = Ref<targets::Byte>;
    type SliceAlong = Ref<targets::Byte>;
    type SliceDirections = Ref<targets::Vector3>;
    type SliceCounts = Ref<targets::U16>;
    type SliceJitters = Ref<targets::Real>;
    type SliceTilts = Ref<targets::Real>;
    type SliceShifts = Ref<targets::Real>;
    type PlaneNormals = Ref<targets::Vector3>;
    type PlaneOffsets = Ref<targets::Real>;
    type Push = u8;
    type Release = u8;
    type Over = f32;
    type Loosen = f32;
    type Centre = [f32; 3];
    type Radius = f32;
    type Strength = f32;
    type Length = f32;
    type At = f32;
    type FillingUvs = u8;
    type MapSize = u32;
    type FillingMade = Bool;
    type Before = u8;
    type WholeGate = u32;
    type PiecesGate = u32;
    type PlaneCentres = Ref<targets::Vector3>;
    type PlaneAlongs = Ref<targets::Vector3>;
    type PlaneWidths = Ref<targets::Real>;
    type PlaneHeights = Ref<targets::Real>;
    type CutSkeleton = Bool;
    type Bones = Ref<targets::U32>;
    type CutRagdoll = Bool;
}

impl FractureRecipeDialect for V12 {
    type Method = u8;
    type Total = u16;
    type Pinned = Ref<targets::Byte>;
    type SliceAlong = Ref<targets::Byte>;
    type SliceDirections = Ref<targets::Vector3>;
    type SliceCounts = Ref<targets::U16>;
    type SliceJitters = Ref<targets::Real>;
    type SliceTilts = Ref<targets::Real>;
    type SliceShifts = Ref<targets::Real>;
    type PlaneNormals = Ref<targets::Vector3>;
    type PlaneOffsets = Ref<targets::Real>;
    type Push = u8;
    type Release = u8;
    type Over = f32;
    type Loosen = f32;
    type Centre = [f32; 3];
    type Radius = f32;
    type Strength = f32;
    type Length = f32;
    type At = f32;
    type FillingUvs = u8;
    type MapSize = u32;
    type FillingMade = Bool;
    type Before = u8;
    type WholeGate = u32;
    type PiecesGate = u32;
    type PlaneCentres = Ref<targets::Vector3>;
    type PlaneAlongs = Ref<targets::Vector3>;
    type PlaneWidths = Ref<targets::Real>;
    type PlaneHeights = Ref<targets::Real>;
    type CutSkeleton = Bool;
    type Bones = Ref<targets::U32>;
    type CutRagdoll = Bool;
}

impl FractureRecipeDialect for V13 {
    type Method = u8;
    type Total = u16;
    type Pinned = Ref<targets::Byte>;
    type SliceAlong = Ref<targets::Byte>;
    type SliceDirections = Ref<targets::Vector3>;
    type SliceCounts = Ref<targets::U16>;
    type SliceJitters = Ref<targets::Real>;
    type SliceTilts = Ref<targets::Real>;
    type SliceShifts = Ref<targets::Real>;
    type PlaneNormals = Ref<targets::Vector3>;
    type PlaneOffsets = Ref<targets::Real>;
    type Push = u8;
    type Release = u8;
    type Over = f32;
    type Loosen = f32;
    type Centre = [f32; 3];
    type Radius = f32;
    type Strength = f32;
    type Length = f32;
    type At = f32;
    type FillingUvs = u8;
    type MapSize = u32;
    type FillingMade = Bool;
    type Before = u8;
    type WholeGate = u32;
    type PiecesGate = u32;
    type PlaneCentres = Ref<targets::Vector3>;
    type PlaneAlongs = Ref<targets::Vector3>;
    type PlaneWidths = Ref<targets::Real>;
    type PlaneHeights = Ref<targets::Real>;
    type CutSkeleton = Bool;
    type Bones = Ref<targets::U32>;
    type CutRagdoll = Bool;
}

impl FractureRecipeDialect for V14 {
    type Method = u8;
    type Total = u16;
    type Pinned = Ref<targets::Byte>;
    type SliceAlong = Ref<targets::Byte>;
    type SliceDirections = Ref<targets::Vector3>;
    type SliceCounts = Ref<targets::U16>;
    type SliceJitters = Ref<targets::Real>;
    type SliceTilts = Ref<targets::Real>;
    type SliceShifts = Ref<targets::Real>;
    type PlaneNormals = Ref<targets::Vector3>;
    type PlaneOffsets = Ref<targets::Real>;
    type Push = u8;
    type Release = u8;
    type Over = f32;
    type Loosen = f32;
    type Centre = [f32; 3];
    type Radius = f32;
    type Strength = f32;
    type Length = f32;
    type At = f32;
    type FillingUvs = u8;
    type MapSize = u32;
    type FillingMade = Bool;
    type Before = u8;
    type WholeGate = u32;
    type PiecesGate = u32;
    type PlaneCentres = Ref<targets::Vector3>;
    type PlaneAlongs = Ref<targets::Vector3>;
    type PlaneWidths = Ref<targets::Real>;
    type PlaneHeights = Ref<targets::Real>;
    type CutSkeleton = Bool;
    type Bones = Ref<targets::U32>;
    type CutRagdoll = Bool;
}

impl FractureRecipeDialect for V15 {
    type Method = u8;
    type Total = u16;
    type Pinned = Ref<targets::Byte>;
    type SliceAlong = Ref<targets::Byte>;
    type SliceDirections = Ref<targets::Vector3>;
    type SliceCounts = Ref<targets::U16>;
    type SliceJitters = Ref<targets::Real>;
    type SliceTilts = Ref<targets::Real>;
    type SliceShifts = Ref<targets::Real>;
    type PlaneNormals = Ref<targets::Vector3>;
    type PlaneOffsets = Ref<targets::Real>;
    type Push = u8;
    type Release = u8;
    type Over = f32;
    type Loosen = f32;
    type Centre = [f32; 3];
    type Radius = f32;
    type Strength = f32;
    type Length = f32;
    type At = f32;
    type FillingUvs = u8;
    type MapSize = u32;
    type FillingMade = Bool;
    type Before = u8;
    type WholeGate = u32;
    type PiecesGate = u32;
    type PlaneCentres = Ref<targets::Vector3>;
    type PlaneAlongs = Ref<targets::Vector3>;
    type PlaneWidths = Ref<targets::Real>;
    type PlaneHeights = Ref<targets::Real>;
    type CutSkeleton = Bool;
    type Bones = Ref<targets::U32>;
    type CutRagdoll = Bool;
}

impl FractureRecipeDialect for V16 {
    type Method = u8;
    type Total = u16;
    type Pinned = Ref<targets::Byte>;
    type SliceAlong = Ref<targets::Byte>;
    type SliceDirections = Ref<targets::Vector3>;
    type SliceCounts = Ref<targets::U16>;
    type SliceJitters = Ref<targets::Real>;
    type SliceTilts = Ref<targets::Real>;
    type SliceShifts = Ref<targets::Real>;
    type PlaneNormals = Ref<targets::Vector3>;
    type PlaneOffsets = Ref<targets::Real>;
    type Push = u8;
    type Release = u8;
    type Over = f32;
    type Loosen = f32;
    type Centre = [f32; 3];
    type Radius = f32;
    type Strength = f32;
    type Length = f32;
    type At = f32;
    type FillingUvs = u8;
    type MapSize = u32;
    type FillingMade = Bool;
    type Before = u8;
    type WholeGate = u32;
    type PiecesGate = u32;
    type PlaneCentres = Ref<targets::Vector3>;
    type PlaneAlongs = Ref<targets::Vector3>;
    type PlaneWidths = Ref<targets::Real>;
    type PlaneHeights = Ref<targets::Real>;
    type CutSkeleton = Bool;
    type Bones = Ref<targets::U32>;
    type CutRagdoll = Bool;
}

impl FractureRecipeDialect for V17 {
    type Method = u8;
    type Total = u16;
    type Pinned = Ref<targets::Byte>;
    type SliceAlong = Ref<targets::Byte>;
    type SliceDirections = Ref<targets::Vector3>;
    type SliceCounts = Ref<targets::U16>;
    type SliceJitters = Ref<targets::Real>;
    type SliceTilts = Ref<targets::Real>;
    type SliceShifts = Ref<targets::Real>;
    type PlaneNormals = Ref<targets::Vector3>;
    type PlaneOffsets = Ref<targets::Real>;
    type Push = u8;
    type Release = u8;
    type Over = f32;
    type Loosen = f32;
    type Centre = [f32; 3];
    type Radius = f32;
    type Strength = f32;
    type Length = f32;
    type At = f32;
    type FillingUvs = u8;
    type MapSize = u32;
    type FillingMade = Bool;
    type Before = u8;
    type WholeGate = u32;
    type PiecesGate = u32;
    type PlaneCentres = Ref<targets::Vector3>;
    type PlaneAlongs = Ref<targets::Vector3>;
    type PlaneWidths = Ref<targets::Real>;
    type PlaneHeights = Ref<targets::Real>;
    type CutSkeleton = Bool;
    type Bones = Ref<targets::U32>;
    type CutRagdoll = Bool;
}

impl FractureRecipeDialect for V18 {
    type Method = u8;
    type Total = u16;
    type Pinned = Ref<targets::Byte>;
    type SliceAlong = Ref<targets::Byte>;
    type SliceDirections = Ref<targets::Vector3>;
    type SliceCounts = Ref<targets::U16>;
    type SliceJitters = Ref<targets::Real>;
    type SliceTilts = Ref<targets::Real>;
    type SliceShifts = Ref<targets::Real>;
    type PlaneNormals = Ref<targets::Vector3>;
    type PlaneOffsets = Ref<targets::Real>;
    type Push = u8;
    type Release = u8;
    type Over = f32;
    type Loosen = f32;
    type Centre = [f32; 3];
    type Radius = f32;
    type Strength = f32;
    type Length = f32;
    type At = f32;
    type FillingUvs = u8;
    type MapSize = u32;
    type FillingMade = Bool;
    type Before = u8;
    type WholeGate = u32;
    type PiecesGate = u32;
    type PlaneCentres = Ref<targets::Vector3>;
    type PlaneAlongs = Ref<targets::Vector3>;
    type PlaneWidths = Ref<targets::Real>;
    type PlaneHeights = Ref<targets::Real>;
    type CutSkeleton = Bool;
    type Bones = Ref<targets::U32>;
    type CutRagdoll = Bool;
}

impl FractureRecipeDialect for V19 {
    type Method = u8;
    type Total = u16;
    type Pinned = Ref<targets::Byte>;
    type SliceAlong = Ref<targets::Byte>;
    type SliceDirections = Ref<targets::Vector3>;
    type SliceCounts = Ref<targets::U16>;
    type SliceJitters = Ref<targets::Real>;
    type SliceTilts = Ref<targets::Real>;
    type SliceShifts = Ref<targets::Real>;
    type PlaneNormals = Ref<targets::Vector3>;
    type PlaneOffsets = Ref<targets::Real>;
    type Push = u8;
    type Release = u8;
    type Over = f32;
    type Loosen = f32;
    type Centre = [f32; 3];
    type Radius = f32;
    type Strength = f32;
    type Length = f32;
    type At = f32;
    type FillingUvs = u8;
    type MapSize = u32;
    type FillingMade = Bool;
    type Before = u8;
    type WholeGate = u32;
    type PiecesGate = u32;
    type PlaneCentres = Ref<targets::Vector3>;
    type PlaneAlongs = Ref<targets::Vector3>;
    type PlaneWidths = Ref<targets::Real>;
    type PlaneHeights = Ref<targets::Real>;
    type CutSkeleton = Bool;
    type Bones = Ref<targets::U32>;
    type CutRagdoll = Bool;
}

/// Fracture preparation, generated identities, and activation settings.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct FractureRecipe<D: FractureRecipeDialect> {
    /// Keeps copies of the complete source meshes alongside generated pieces.
    pub keep_whole: Bool,
    /// Source mesh index for each fracture target; missing sources use `0xffffffff`.
    pub sources: Ref<targets::U32>,
    /// Generated mesh indices, with outside and optional inside entries for each target.
    pub made: Ref<targets::U32>,
    /// Requested piece count per target; one retains the target whole.
    pub pieces: Ref<targets::U16>,
    /// One-based document node identities used by fracture skin bindings.
    pub skin_nodes: Ref<targets::U32>,
    /// Seed for repeatable fracture choices.
    pub seed: u32,
    /// Control for making pieces smaller near the blast center.
    pub near_blast: f32,
    /// Control for how uniform generated piece sizes should be.
    pub even: f32,
    /// Preferred grain axis: none, X, Y, or Z.
    pub grain: u8,
    /// Elongation control along the grain direction.
    pub stretch: f32,
    /// Minimum piece-size fraction relative to the average piece.
    pub smallest: f32,
    /// Treatment of open geometry during fracture.
    pub open_parts: u8,
    /// Shell thickness in model units; zero requests a size-derived value.
    pub thickness: f32,
    /// Material-slot index for fracture interiors; `0xffffffff` reuses the exterior material.
    pub inside: u32,
    /// Scale applied to generated interior texture coordinates.
    pub uv_scale: f32,
    /// Point-count control for piece collision hulls.
    pub hull_points: u8,
    /// Allows hollow geometry to split into separate pieces.
    pub split_hollow: Bool,
    /// Physics-material preset assigned to generated pieces.
    pub material: u32,
    /// Document node inserted between pieces and their parent, or `0xffffffff` for none.
    pub helper: u32,
    /// Document node defining the blast force field.
    pub field: u32,
    /// Local force-channel bit shared by the field and generated pieces.
    pub channel: u8,
    /// Model-space seed positions retained for each generated piece.
    pub seeds: Ref<targets::Vector3>,
    /// Seed positions associated with pieces that remain static.
    pub statics: Ref<targets::Vector3>,
    /// Generation method selecting general fracture or planar slices.
    pub method: D::Method,
    /// Shared requested piece total; zero leaves each target with its own count.
    pub total: D::Total,
    /// Per-target byte flags selecting independent piece counts.
    pub pinned: D::Pinned,
    /// Axis mode for each slice set: X, Y, Z, or an explicitly supplied direction.
    pub slice_along: D::SliceAlong,
    /// Direction vectors for custom-axis slice sets.
    pub slice_directions: D::SliceDirections,
    /// Plane counts for the slice sets.
    pub slice_counts: D::SliceCounts,
    /// Plane jitter as a fraction of half the slice spacing.
    pub slice_jitters: D::SliceJitters,
    /// Plane tilts in radians.
    pub slice_tilts: D::SliceTilts,
    /// Plane shifts in model units along their slicing directions.
    pub slice_shifts: D::SliceShifts,
    /// Normals of the actual cutting planes.
    pub plane_normals: D::PlaneNormals,
    /// Plane equation distances satisfying normal dot position = offset.
    pub plane_offsets: D::PlaneOffsets,
    /// Piece motion mode selecting blast, implosion, or no push.
    pub push: D::Push,
    /// Release ordering selecting simultaneous, bottom-up, top-down, or outward activation.
    pub release: D::Release,
    /// Duration of staggered piece activation, in seconds.
    pub over: D::Over,
    /// Reduction in piece collision-hull size expressed as a fraction of piece size.
    pub loosen: D::Loosen,
    /// Model-space center of the fracture activation.
    pub centre: D::Centre,
    /// Activation radius in model units.
    pub radius: D::Radius,
    /// Push acceleration expressed as a multiple of gravitational acceleration.
    pub strength: D::Strength,
    /// Duration of the push, in seconds.
    pub length: D::Length,
    /// Activation time measured in seconds from the clip start.
    pub at: D::At,
    /// Texture-coordinate layout for generated interior geometry.
    pub filling_uvs: D::FillingUvs,
    /// Resolution selected for the generated interior texture map.
    pub map_size: D::MapSize,
    /// Records whether the recipe created its interior material.
    pub filling_made: D::FillingMade,
    /// Geometry shown before activation: automatic choice, whole mesh, or pieces.
    pub before: D::Before,
    /// Document node whose visibility enables drawing of whole meshes.
    pub whole_gate: D::WholeGate,
    /// Document node whose visibility enables drawing of generated pieces.
    pub pieces_gate: D::PiecesGate,
    /// Centers of bounded cutting-plane rectangles.
    pub plane_centres: D::PlaneCentres,
    /// In-plane direction defining the rectangle width axis.
    pub plane_alongs: D::PlaneAlongs,
    /// Half-widths of cutting rectangles; zero selects an unbounded cut.
    pub plane_widths: D::PlaneWidths,
    /// Half-heights of cutting rectangles; zero selects an unbounded cut.
    pub plane_heights: D::PlaneHeights,
    /// Copies the affected skeleton onto generated pieces.
    pub cut_skeleton: D::CutSkeleton,
    /// Document node indices of additional skeleton copies created by the fracture.
    pub bones: D::Bones,
    /// Copies the affected ragdoll bodies and surviving joints onto generated pieces.
    pub cut_ragdoll: D::CutRagdoll,
}

/// Static field selections for `PhysicsRig` section or inline context revisions.
pub trait PhysicsRigDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `wow`; `()` omits this slot.
    type Wow: NativeField;
    /// Representation of `recipe`; `()` omits this slot.
    type Recipe: NativeField;
    /// Representation of `fracture`; `()` omits this slot.
    type Fracture: NativeField;
}

impl PhysicsRigDialect for V1 {
    const VERSION: u32 = 1;
    type Wow = ();
    type Recipe = ();
    type Fracture = ();
}

impl PhysicsRigDialect for V2 {
    const VERSION: u32 = 2;
    type Wow = InlineOption<WowRigExtension>;
    type Recipe = ();
    type Fracture = ();
}

impl PhysicsRigDialect for V3 {
    const VERSION: u32 = 3;
    type Wow = InlineOption<WowRigExtension>;
    type Recipe = InlineOption<RagdollRecipe>;
    type Fracture = ();
}

impl PhysicsRigDialect for V4 {
    const VERSION: u32 = 4;
    type Wow = InlineOption<WowRigExtension>;
    type Recipe = InlineOption<RagdollRecipe>;
    type Fracture = InlineOption<FractureRecipe<V4>>;
}

impl PhysicsRigDialect for V5 {
    const VERSION: u32 = 5;
    type Wow = InlineOption<WowRigExtension>;
    type Recipe = InlineOption<RagdollRecipe>;
    type Fracture = InlineOption<FractureRecipe<V5>>;
}

impl PhysicsRigDialect for V6 {
    const VERSION: u32 = 6;
    type Wow = InlineOption<WowRigExtension>;
    type Recipe = InlineOption<RagdollRecipe>;
    type Fracture = InlineOption<FractureRecipe<V6>>;
}

impl PhysicsRigDialect for V7 {
    const VERSION: u32 = 7;
    type Wow = InlineOption<WowRigExtension>;
    type Recipe = InlineOption<RagdollRecipe>;
    type Fracture = InlineOption<FractureRecipe<V7>>;
}

impl PhysicsRigDialect for V8 {
    const VERSION: u32 = 8;
    type Wow = InlineOption<WowRigExtension>;
    type Recipe = InlineOption<RagdollRecipe>;
    type Fracture = InlineOption<FractureRecipe<V8>>;
}

/// A body group with activation behavior and optional preparation recipes.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct PhysicsRig<D: PhysicsRigDialect> {
    /// Stable identity of this physics rig.
    pub id: u32,
    /// CHAR bytes naming the rig.
    pub name: Ref<targets::Character>,
    /// Activation mode selecting animation, death, continuous simulation, or disabled behavior.
    pub start: RigStart,
    /// Physics body identities belonging to the rig.
    pub bodies: Ref<targets::U32>,
    /// Optional World of Warcraft rig behavior settings.
    pub wow: D::Wow,
    /// Optional ragdoll preparation settings.
    pub recipe: D::Recipe,
    /// Optional fracture preparation and activation settings.
    pub fracture: D::Fracture,
}
impl<D: PhysicsRigDialect> Record for PhysicsRig<D> {
    const TAG: [u8; 4] = *b"PRIG";
    const VERSION: u32 = D::VERSION;
}

/// Static field selections for `PhysicsSet` section or inline context revisions.
pub trait PhysicsSetDialect: Revision {
    /// Representation of `soft`; `()` omits this slot.
    type Soft: NativeField;
}

impl PhysicsSetDialect for V0 {
    type Soft = ();
}

impl PhysicsSetDialect for V1 {
    type Soft = ();
}

impl PhysicsSetDialect for V2 {
    type Soft = ();
}

impl PhysicsSetDialect for V3 {
    type Soft = ();
}

impl PhysicsSetDialect for V4 {
    type Soft = ();
}

impl PhysicsSetDialect for V5 {
    type Soft = ();
}

impl PhysicsSetDialect for V6 {
    type Soft = ();
}

impl PhysicsSetDialect for V7 {
    type Soft = ();
}

impl PhysicsSetDialect for V8 {
    type Soft = ();
}

impl PhysicsSetDialect for V9 {
    type Soft = Ref<targets::SoftBody>;
}

impl PhysicsSetDialect for V10 {
    type Soft = Ref<targets::SoftBody>;
}

impl PhysicsSetDialect for V11 {
    type Soft = Ref<targets::SoftBody>;
}

impl PhysicsSetDialect for V12 {
    type Soft = Ref<targets::SoftBody>;
}

impl PhysicsSetDialect for V13 {
    type Soft = Ref<targets::SoftBody>;
}

impl PhysicsSetDialect for V14 {
    type Soft = Ref<targets::SoftBody>;
}

impl PhysicsSetDialect for V15 {
    type Soft = Ref<targets::SoftBody>;
}

impl PhysicsSetDialect for V16 {
    type Soft = Ref<targets::SoftBody>;
}

impl PhysicsSetDialect for V17 {
    type Soft = Ref<targets::SoftBody>;
}

impl PhysicsSetDialect for V18 {
    type Soft = Ref<targets::SoftBody>;
}

impl PhysicsSetDialect for V19 {
    type Soft = Ref<targets::SoftBody>;
}

/// Physics objects and the identity counter shared by their authoring state.
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
)]
pub struct PhysicsSet<D: PhysicsSetDialect> {
    /// Physics body records with stable identities.
    pub bodies: Ref<targets::PhysicsBody>,
    /// Constraints referring to bodies by their stable identities.
    pub joints: Ref<targets::PhysicsJoint>,
    /// Cloth collider records with stable identities.
    pub colliders: Ref<targets::ClothCollider>,
    /// Cloth objects, cages, and mesh bindings.
    pub cloths: Ref<targets::Cloth>,
    /// Physics rigs grouping bodies and preparation recipes.
    pub rigs: Ref<targets::PhysicsRig>,
    /// Next stable identity to allocate for a physics object.
    pub next_id: u32,
    /// Authored soft bodies and their deformation effects selected by this set dialect.
    pub soft: D::Soft,
}
