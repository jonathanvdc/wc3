//! Nodes and their scene, emitter, skin-authoring, and rig payloads.
//!
//! Node placement is separate from emitter parameters. Game-specific emitter
//! records keep their own coordinate conventions and controls; node links refer
//! to model nodes, material slots to the model’s slots, and texture indices to
//! the document texture table. Optional revision fields are selected by dialect
//! types. These codecs preserve the records without evaluating their effects.
use super::*;
use crate::model::wem::{self, Bool, InlineOption, Record, Ref};
/// Influence-envelope boundaries and the curve connecting them.
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
pub struct Envelope {
    /// Inner start parameter stored as a floating-point value.
    pub inner_start: f32,
    /// Inner end parameter stored as a floating-point value.
    pub inner_end: f32,
    /// Outer start parameter stored as a floating-point value.
    pub outer_start: f32,
    /// Outer end parameter stored as a floating-point value.
    pub outer_end: f32,
    /// Curve used to reduce influence between the envelope boundaries.
    pub falloff: EnvelopeFalloff,
}

/// Authoring constraint describing how a skinning joint can move.
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
pub struct Joint {
    /// Joint motion category: bending, twisting, spinning, or sliding.
    pub motion: JointMotion,
    /// Axis selected for the joint constraint.
    pub axis: JointAxis,
    /// Selects parent-space X, Y, and Z axes instead of model-space axes.
    pub parent_space: Bool,
    /// Lower motion limit: degrees for bending, model units for sliding; spinning ignores limits.
    pub low: f32,
    /// Upper motion limit: degrees for bending, model units for sliding; spinning ignores limits.
    pub high: f32,
}

/// Named base pose used to test skinning behavior.
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
pub struct TestPose {
    /// Character-section reference identifying this record.
    pub name: Ref<targets::Character>,
    /// Index in the document clip table used as the test-pose base; the invalid-index sentinel selects rest pose.
    pub clip: u32,
    /// Time within the selected base clip, in milliseconds.
    pub time_ms: f32,
}
impl Record for TestPose {
    const TAG: [u8; 4] = *b"TPOS";
    const VERSION: u32 = 1;
}

/// Node rotation and translation relative to a saved base pose.
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
pub struct PoseDelta {
    /// Quaternion rotation of the pose delta.
    pub rotation: [f32; 4],
    /// Translation component of the pose delta.
    pub translation: [f32; 3],
}
impl Record for PoseDelta {
    const TAG: [u8; 4] = *b"PDLT";
    const VERSION: u32 = 1;
}

/// Static field selections for `NodeSkinSetup` section or inline context revisions.
pub trait NodeSkinSetupDialect: Revision {
    /// Representation of `pose_sources`; `()` omits this slot.
    type PoseSources: NativeField;
}

impl NodeSkinSetupDialect for V0 {
    type PoseSources = ();
}

impl NodeSkinSetupDialect for V1 {
    type PoseSources = ();
}

impl NodeSkinSetupDialect for V2 {
    type PoseSources = ();
}

impl NodeSkinSetupDialect for V3 {
    type PoseSources = ();
}

impl NodeSkinSetupDialect for V4 {
    type PoseSources = ();
}

impl NodeSkinSetupDialect for V5 {
    type PoseSources = ();
}

impl NodeSkinSetupDialect for V6 {
    type PoseSources = ();
}

impl NodeSkinSetupDialect for V7 {
    type PoseSources = ();
}

impl NodeSkinSetupDialect for V8 {
    type PoseSources = ();
}

impl NodeSkinSetupDialect for V9 {
    type PoseSources = ();
}

impl NodeSkinSetupDialect for V10 {
    type PoseSources = ();
}

impl NodeSkinSetupDialect for V11 {
    type PoseSources = ();
}

impl NodeSkinSetupDialect for V12 {
    type PoseSources = Ref<targets::TPoseSourceRecord>;
}

impl NodeSkinSetupDialect for V13 {
    type PoseSources = Ref<targets::TPoseSourceRecord>;
}

impl NodeSkinSetupDialect for V14 {
    type PoseSources = Ref<targets::TPoseSourceRecord>;
}

impl NodeSkinSetupDialect for V15 {
    type PoseSources = Ref<targets::TPoseSourceRecord>;
}

impl NodeSkinSetupDialect for V16 {
    type PoseSources = Ref<targets::TPoseSourceRecord>;
}

impl NodeSkinSetupDialect for V17 {
    type PoseSources = Ref<targets::TPoseSourceRecord>;
}

impl NodeSkinSetupDialect for V18 {
    type PoseSources = Ref<targets::TPoseSourceRecord>;
}

impl NodeSkinSetupDialect for V19 {
    type PoseSources = Ref<targets::TPoseSourceRecord>;
}

/// Typed node skin setup payload in its serialized field order.
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
pub struct NodeSkinSetup<D: NodeSkinSetupDialect> {
    /// Locks this node’s recorded skinning setup.
    pub locked: Bool,
    /// Optional geometric envelope controlling the node’s skin influence.
    pub envelope: InlineOption<Envelope>,
    /// Optional joint settings associated with the node’s skinning setup.
    pub joint: InlineOption<Joint>,
    /// Model node index paired with this node for mirrored skin editing.
    pub mirror: u32,
    /// PDLT section containing saved corrections to this node’s poses.
    pub pose_deltas: Ref<targets::PoseDelta>,
    /// TPSR section identifying the origin of each saved pose correction.
    pub pose_sources: D::PoseSources,
}

/// Static field selections for `ForceFieldPayload` section or inline context revisions.
pub trait ForceFieldPayloadDialect: Revision {
    /// Representation of `transform`; `()` omits this slot.
    type Transform: NativeField;
}

impl ForceFieldPayloadDialect for V0 {
    type Transform = ();
}

impl ForceFieldPayloadDialect for V1 {
    type Transform = ();
}

impl ForceFieldPayloadDialect for V2 {
    type Transform = ();
}

impl ForceFieldPayloadDialect for V3 {
    type Transform = ();
}

impl ForceFieldPayloadDialect for V4 {
    type Transform = ();
}

impl ForceFieldPayloadDialect for V5 {
    type Transform = ();
}

impl ForceFieldPayloadDialect for V6 {
    type Transform = ();
}

impl ForceFieldPayloadDialect for V7 {
    type Transform = ();
}

impl ForceFieldPayloadDialect for V8 {
    type Transform = ();
}

impl ForceFieldPayloadDialect for V9 {
    type Transform = ();
}

impl ForceFieldPayloadDialect for V10 {
    type Transform = ();
}

impl ForceFieldPayloadDialect for V11 {
    type Transform = ();
}

impl ForceFieldPayloadDialect for V12 {
    type Transform = ();
}

impl ForceFieldPayloadDialect for V13 {
    type Transform = ();
}

impl ForceFieldPayloadDialect for V14 {
    type Transform = ();
}

impl ForceFieldPayloadDialect for V15 {
    type Transform = [[f32; 4]; 4];
}

impl ForceFieldPayloadDialect for V16 {
    type Transform = [[f32; 4]; 4];
}

impl ForceFieldPayloadDialect for V17 {
    type Transform = [[f32; 4]; 4];
}

impl ForceFieldPayloadDialect for V18 {
    type Transform = [[f32; 4]; 4];
}

impl ForceFieldPayloadDialect for V19 {
    type Transform = [[f32; 4]; 4];
}

/// Typed force field payload payload in its serialized field order.
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
pub struct ForceFieldPayload<D: ForceFieldPayloadDialect> {
    /// Force operation applied inside the field volume.
    pub kind: ForceKind,
    /// Geometric shape of the force field.
    pub volume: ForceVolume,
    /// Stored boolean byte controlling falloff; nonzero encodings are retained.
    pub falloff: Bool,
    /// Stored boolean byte controlling height gradient; nonzero encodings are retained.
    pub height_gradient: Bool,
    /// Stored boolean byte controlling unbounded; nonzero encodings are retained.
    pub unbounded: Bool,
    /// Stored boolean byte controlling affects particles; nonzero encodings are retained.
    pub affects_particles: Bool,
    /// Stored boolean byte controlling affects bodies; nonzero encodings are retained.
    pub affects_bodies: Bool,
    /// Force-channel mask matched against rigid-body force channels.
    pub channels: u32,
    /// Source force-field scope value; zero denotes the model itself. Other values are unresolved.
    pub scope: u32,
    /// Strength parameter stored as a floating-point value.
    pub strength: f32,
    /// Width parameter stored as a floating-point value.
    pub width: f32,
    /// Height parameter stored as a floating-point value.
    pub height: f32,
    /// Length parameter stored as a floating-point value.
    pub length: f32,
    /// Matrix mapping the force field’s own frame into its node frame.
    pub transform: D::Transform,
}

/// Typed vertex warp payload payload in its serialized field order.
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
pub struct VertexWarpPayload {
    /// Source vertex-warp mode code; the meaning of each value is unresolved.
    pub type_: u32,
    /// Unresolved reserved data retained in its original position and representation.
    pub reserved: u32,
    /// Radius parameter stored as a floating-point value.
    pub radius: f32,
    /// Height parameter stored as a floating-point value.
    pub height: f32,
    /// Strength parameter stored as a floating-point value.
    pub strength: f32,
    /// Angular parameter stored as a floating-point value.
    pub angular: f32,
    /// Axial parameter stored as a floating-point value.
    pub axial: f32,
    /// Radial parameter stored as a floating-point value.
    pub radial: f32,
}

/// Static field selections for `PoseSchema` section or inline context revisions.
pub trait PoseSchemaDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `storage`; `()` omits this slot.
    type Storage: NativeField;
}

impl PoseSchemaDialect for V1 {
    const VERSION: u32 = 1;
    type Storage = ();
}

impl PoseSchemaDialect for V2 {
    const VERSION: u32 = 2;
    type Storage = PoseStorage;
}

/// Typed pose schema payload in its serialized field order.
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
pub struct PoseSchema<D: PoseSchemaDialect> {
    /// Character-section reference naming the saved pose convention.
    pub name: Ref<targets::Character>,
    /// Coordinate space in which the pose is stored.
    pub space: PoseSpace,
    /// Marks matrices that represent the inverse of the named pose.
    pub inverse: Bool,
    /// Selects TRS or matrix pose storage in revisions that include this field.
    pub storage: D::Storage,
}
impl<D: PoseSchemaDialect> Record for PoseSchema<D> {
    const TAG: [u8; 4] = *b"PSCH";
    const VERSION: u32 = D::VERSION;
}

/// Static field selections for `NodeTree` section or inline context revisions.
pub trait NodeTreeDialect: Revision {
    /// Representation of `rig`; `()` omits this slot.
    type Rig: NativeField;
}

impl NodeTreeDialect for V0 {
    type Rig = ();
}

impl NodeTreeDialect for V1 {
    type Rig = ();
}

impl NodeTreeDialect for V2 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V3 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V4 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V5 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V6 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V7 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V8 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V9 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V10 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V11 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V12 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V13 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V14 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V15 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V16 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V17 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V18 {
    type Rig = RigConvention;
}

impl NodeTreeDialect for V19 {
    type Rig = RigConvention;
}

/// Typed node tree payload in its serialized field order.
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
pub struct NodeTree<D: NodeTreeDialect> {
    /// NODE section containing the hierarchy’s node definitions.
    pub nodes: Ref<targets::Node>,
    /// PSCH section describing each saved pose slot.
    pub pose_schema: Ref<targets::PoseSchema>,
    /// Saved pose index used as the hierarchy’s reference pose.
    pub authoritative_pose: u32,
    /// Convention used to interpret hierarchical transforms in revisions that include it.
    pub rig: D::Rig,
}

/// Typed transform payload in its serialized field order.
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
pub struct Transform {
    /// Translation vector in the enclosing record’s coordinate space.
    pub translation: [f32; 3],
    /// Quaternion rotation, stored as four scalar components.
    pub rotation: [f32; 4],
    /// Per-axis scale factors.
    pub scale: [f32; 3],
}
impl Record for Transform {
    const TAG: [u8; 4] = *b"XFRM";
    const VERSION: u32 = 1;
}

/// Typed sphere payload in its serialized field order.
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
pub struct Sphere {
    /// Position of the sphere center.
    pub center: [f32; 3],
    /// Sphere radius in model units.
    pub radius: f32,
}

/// Static field selections for `BonePayload` section or inline context revisions.
pub trait BonePayloadDialect: Revision {
    /// Representation of `gate_mesh`; `()` omits this slot.
    type GateMesh: NativeField;
}

impl BonePayloadDialect for V0 {
    type GateMesh = ();
}

impl BonePayloadDialect for V1 {
    type GateMesh = ();
}

impl BonePayloadDialect for V2 {
    type GateMesh = ();
}

impl BonePayloadDialect for V3 {
    type GateMesh = ();
}

impl BonePayloadDialect for V4 {
    type GateMesh = ();
}

impl BonePayloadDialect for V5 {
    type GateMesh = u32;
}

impl BonePayloadDialect for V6 {
    type GateMesh = u32;
}

impl BonePayloadDialect for V7 {
    type GateMesh = u32;
}

impl BonePayloadDialect for V8 {
    type GateMesh = u32;
}

impl BonePayloadDialect for V9 {
    type GateMesh = u32;
}

impl BonePayloadDialect for V10 {
    type GateMesh = u32;
}

impl BonePayloadDialect for V11 {
    type GateMesh = u32;
}

impl BonePayloadDialect for V12 {
    type GateMesh = u32;
}

impl BonePayloadDialect for V13 {
    type GateMesh = u32;
}

impl BonePayloadDialect for V14 {
    type GateMesh = u32;
}

impl BonePayloadDialect for V15 {
    type GateMesh = u32;
}

impl BonePayloadDialect for V16 {
    type GateMesh = u32;
}

impl BonePayloadDialect for V17 {
    type GateMesh = u32;
}

impl BonePayloadDialect for V18 {
    type GateMesh = u32;
}

impl BonePayloadDialect for V19 {
    type GateMesh = u32;
}

/// Typed bone payload payload in its serialized field order.
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
pub struct BonePayload<D: BonePayloadDialect> {
    /// Axis-aligned bounds associated with the bone.
    pub bounds: Extent,
    /// Bounding sphere associated with the bone.
    pub sphere: Sphere,
    /// Model mesh index whose visibility is driven by this bone.
    pub gate_mesh: D::GateMesh,
}

/// Typed attachment payload payload in its serialized field order.
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
pub struct AttachmentPayload {
    /// External model identity when the attachment refers outside the document.
    pub asset: AssetKey,
    /// Document model index attached at this location.
    pub model: u32,
}

/// Static field selections for `LightPayload` section or inline context revisions.
pub trait LightPayloadDialect: Revision {
    /// Representation of `shadow_casting`; `()` omits this slot.
    type ShadowCasting: NativeField;
    /// Representation of `shadow_casting_start`; `()` omits this slot.
    type ShadowCastingStart: NativeField;
    /// Representation of `shadow_casting_end`; `()` omits this slot.
    type ShadowCastingEnd: NativeField;
    /// Representation of `quadratic_falloff`; `()` omits this slot.
    type QuadraticFalloff: NativeField;
    /// Representation of `linear_falloff`; `()` omits this slot.
    type LinearFalloff: NativeField;
    /// Representation of `damping`; `()` omits this slot.
    type Damping: NativeField;
}

impl LightPayloadDialect for V0 {
    type ShadowCasting = ();
    type ShadowCastingStart = ();
    type ShadowCastingEnd = ();
    type QuadraticFalloff = ();
    type LinearFalloff = ();
    type Damping = ();
}

impl LightPayloadDialect for V1 {
    type ShadowCasting = ();
    type ShadowCastingStart = ();
    type ShadowCastingEnd = ();
    type QuadraticFalloff = ();
    type LinearFalloff = ();
    type Damping = ();
}

impl LightPayloadDialect for V2 {
    type ShadowCasting = ();
    type ShadowCastingStart = ();
    type ShadowCastingEnd = ();
    type QuadraticFalloff = ();
    type LinearFalloff = ();
    type Damping = ();
}

impl LightPayloadDialect for V3 {
    type ShadowCasting = ();
    type ShadowCastingStart = ();
    type ShadowCastingEnd = ();
    type QuadraticFalloff = ();
    type LinearFalloff = ();
    type Damping = ();
}

impl LightPayloadDialect for V4 {
    type ShadowCasting = ();
    type ShadowCastingStart = ();
    type ShadowCastingEnd = ();
    type QuadraticFalloff = ();
    type LinearFalloff = ();
    type Damping = ();
}

impl LightPayloadDialect for V5 {
    type ShadowCasting = ();
    type ShadowCastingStart = ();
    type ShadowCastingEnd = ();
    type QuadraticFalloff = ();
    type LinearFalloff = ();
    type Damping = ();
}

impl LightPayloadDialect for V6 {
    type ShadowCasting = ();
    type ShadowCastingStart = ();
    type ShadowCastingEnd = ();
    type QuadraticFalloff = ();
    type LinearFalloff = ();
    type Damping = ();
}

impl LightPayloadDialect for V7 {
    type ShadowCasting = Bool;
    type ShadowCastingStart = f32;
    type ShadowCastingEnd = f32;
    type QuadraticFalloff = f32;
    type LinearFalloff = f32;
    type Damping = f32;
}

impl LightPayloadDialect for V8 {
    type ShadowCasting = Bool;
    type ShadowCastingStart = f32;
    type ShadowCastingEnd = f32;
    type QuadraticFalloff = f32;
    type LinearFalloff = f32;
    type Damping = f32;
}

impl LightPayloadDialect for V9 {
    type ShadowCasting = Bool;
    type ShadowCastingStart = f32;
    type ShadowCastingEnd = f32;
    type QuadraticFalloff = f32;
    type LinearFalloff = f32;
    type Damping = f32;
}

impl LightPayloadDialect for V10 {
    type ShadowCasting = Bool;
    type ShadowCastingStart = f32;
    type ShadowCastingEnd = f32;
    type QuadraticFalloff = f32;
    type LinearFalloff = f32;
    type Damping = f32;
}

impl LightPayloadDialect for V11 {
    type ShadowCasting = Bool;
    type ShadowCastingStart = f32;
    type ShadowCastingEnd = f32;
    type QuadraticFalloff = f32;
    type LinearFalloff = f32;
    type Damping = f32;
}

impl LightPayloadDialect for V12 {
    type ShadowCasting = Bool;
    type ShadowCastingStart = f32;
    type ShadowCastingEnd = f32;
    type QuadraticFalloff = f32;
    type LinearFalloff = f32;
    type Damping = f32;
}

impl LightPayloadDialect for V13 {
    type ShadowCasting = Bool;
    type ShadowCastingStart = f32;
    type ShadowCastingEnd = f32;
    type QuadraticFalloff = f32;
    type LinearFalloff = f32;
    type Damping = f32;
}

impl LightPayloadDialect for V14 {
    type ShadowCasting = Bool;
    type ShadowCastingStart = f32;
    type ShadowCastingEnd = f32;
    type QuadraticFalloff = f32;
    type LinearFalloff = f32;
    type Damping = f32;
}

impl LightPayloadDialect for V15 {
    type ShadowCasting = Bool;
    type ShadowCastingStart = f32;
    type ShadowCastingEnd = f32;
    type QuadraticFalloff = f32;
    type LinearFalloff = f32;
    type Damping = f32;
}

impl LightPayloadDialect for V16 {
    type ShadowCasting = Bool;
    type ShadowCastingStart = f32;
    type ShadowCastingEnd = f32;
    type QuadraticFalloff = f32;
    type LinearFalloff = f32;
    type Damping = f32;
}

impl LightPayloadDialect for V17 {
    type ShadowCasting = Bool;
    type ShadowCastingStart = f32;
    type ShadowCastingEnd = f32;
    type QuadraticFalloff = f32;
    type LinearFalloff = f32;
    type Damping = f32;
}

impl LightPayloadDialect for V18 {
    type ShadowCasting = Bool;
    type ShadowCastingStart = f32;
    type ShadowCastingEnd = f32;
    type QuadraticFalloff = f32;
    type LinearFalloff = f32;
    type Damping = f32;
}

impl LightPayloadDialect for V19 {
    type ShadowCasting = Bool;
    type ShadowCastingStart = f32;
    type ShadowCastingEnd = f32;
    type QuadraticFalloff = f32;
    type LinearFalloff = f32;
    type Damping = f32;
}

/// Typed light payload payload in its serialized field order.
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
pub struct LightPayload<D: LightPayloadDialect> {
    /// Selects an omnidirectional, directional, or spot light.
    pub kind: LightKind,
    /// RGB light color components.
    pub color: [f32; 3],
    /// Multiplier applied to the light color.
    pub intensity: f32,
    /// Distance at which the light’s range attenuation begins.
    pub attenuation_start: f32,
    /// Distance at which the light’s range attenuation ends.
    pub attenuation_end: f32,
    /// Inner angle of a spot light’s cone.
    pub hot_spot: f32,
    /// Outer angle of a spot light’s cone.
    pub falloff: f32,
    /// Enables shadow casting in revisions that store this control.
    pub shadow_casting: D::ShadowCasting,
    /// Near distance of the light’s shadow volume.
    pub shadow_casting_start: D::ShadowCastingStart,
    /// Far distance of the light’s shadow volume.
    pub shadow_casting_end: D::ShadowCastingEnd,
    /// Coefficient of the quadratic distance-attenuation term.
    pub quadratic_falloff: D::QuadraticFalloff,
    /// Coefficient of the linear distance-attenuation term.
    pub linear_falloff: D::LinearFalloff,
    /// Exponential distance-attenuation coefficient.
    pub damping: D::Damping,
}

/// Static field selections for `CameraPayload` section or inline context revisions.
pub trait CameraPayloadDialect: Revision {
    /// Representation of `target`; `()` omits this slot.
    type Target: NativeField;
    /// Representation of `focus_distance`; `()` omits this slot.
    type FocusDistance: NativeField;
    /// Representation of `focal_length`; `()` omits this slot.
    type FocalLength: NativeField;
    /// Representation of `f_stop`; `()` omits this slot.
    type FStop: NativeField;
}

impl CameraPayloadDialect for V0 {
    type Target = ();
    type FocusDistance = ();
    type FocalLength = ();
    type FStop = ();
}

impl CameraPayloadDialect for V1 {
    type Target = ();
    type FocusDistance = ();
    type FocalLength = ();
    type FStop = ();
}

impl CameraPayloadDialect for V2 {
    type Target = ();
    type FocusDistance = ();
    type FocalLength = ();
    type FStop = ();
}

impl CameraPayloadDialect for V3 {
    type Target = ();
    type FocusDistance = ();
    type FocalLength = ();
    type FStop = ();
}

impl CameraPayloadDialect for V4 {
    type Target = [f32; 3];
    type FocusDistance = ();
    type FocalLength = ();
    type FStop = ();
}

impl CameraPayloadDialect for V5 {
    type Target = [f32; 3];
    type FocusDistance = ();
    type FocalLength = ();
    type FStop = ();
}

impl CameraPayloadDialect for V6 {
    type Target = [f32; 3];
    type FocusDistance = ();
    type FocalLength = ();
    type FStop = ();
}

impl CameraPayloadDialect for V7 {
    type Target = [f32; 3];
    type FocusDistance = ();
    type FocalLength = ();
    type FStop = ();
}

impl CameraPayloadDialect for V8 {
    type Target = [f32; 3];
    type FocusDistance = ();
    type FocalLength = ();
    type FStop = ();
}

impl CameraPayloadDialect for V9 {
    type Target = [f32; 3];
    type FocusDistance = ();
    type FocalLength = ();
    type FStop = ();
}

impl CameraPayloadDialect for V10 {
    type Target = [f32; 3];
    type FocusDistance = ();
    type FocalLength = ();
    type FStop = ();
}

impl CameraPayloadDialect for V11 {
    type Target = [f32; 3];
    type FocusDistance = ();
    type FocalLength = ();
    type FStop = ();
}

impl CameraPayloadDialect for V12 {
    type Target = [f32; 3];
    type FocusDistance = ();
    type FocalLength = ();
    type FStop = ();
}

impl CameraPayloadDialect for V13 {
    type Target = [f32; 3];
    type FocusDistance = f32;
    type FocalLength = f32;
    type FStop = f32;
}

impl CameraPayloadDialect for V14 {
    type Target = [f32; 3];
    type FocusDistance = f32;
    type FocalLength = f32;
    type FStop = f32;
}

impl CameraPayloadDialect for V15 {
    type Target = [f32; 3];
    type FocusDistance = f32;
    type FocalLength = f32;
    type FStop = f32;
}

impl CameraPayloadDialect for V16 {
    type Target = [f32; 3];
    type FocusDistance = f32;
    type FocalLength = f32;
    type FStop = f32;
}

impl CameraPayloadDialect for V17 {
    type Target = [f32; 3];
    type FocusDistance = f32;
    type FocalLength = f32;
    type FStop = f32;
}

impl CameraPayloadDialect for V18 {
    type Target = [f32; 3];
    type FocusDistance = f32;
    type FocalLength = f32;
    type FStop = f32;
}

impl CameraPayloadDialect for V19 {
    type Target = [f32; 3];
    type FocusDistance = f32;
    type FocalLength = f32;
    type FStop = f32;
}

/// Typed camera payload payload in its serialized field order.
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
pub struct CameraPayload<D: CameraPayloadDialect> {
    /// Stored field-of-view parameter; source conventions are retained.
    pub fov: f32,
    /// Near camera clipping distance in model units.
    pub near_clip: f32,
    /// Far camera clipping distance in model units.
    pub far_clip: f32,
    /// Model-space point toward which the camera is aimed.
    pub target: D::Target,
    /// Depth-of-field focus distance; zero denotes an unset value.
    pub focus_distance: D::FocusDistance,
    /// Stored depth-of-field focal-length parameter; zero denotes an unset value.
    pub focal_length: D::FocalLength,
    /// Depth-of-field aperture parameter; zero denotes an unset value.
    pub f_stop: D::FStop,
}

/// Typed wc3 face fx payload payload in its serialized field order.
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
pub struct Wc3FaceFxPayload {
    /// External identity of the FaceFX actor associated with this node.
    pub actor: AssetKey,
}

/// Typed particle payload payload in its serialized field order.
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
pub struct ParticlePayload {
    /// External identity of the particle system placed by this node.
    pub system: AssetKey,
}

/// Typed ribbon payload payload in its serialized field order.
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
pub struct RibbonPayload {
    /// External identity of the ribbon system placed by this node.
    pub system: AssetKey,
}

/// Static field selections for `EventPayload` section or inline context revisions.
pub trait EventPayloadDialect: Revision {
    /// Representation of `category`; `()` omits this slot.
    type Category: NativeField;
    /// Representation of `code`; `()` omits this slot.
    type Code: NativeField;
}

impl EventPayloadDialect for V0 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V1 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V2 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V3 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V4 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V5 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V6 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V7 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V8 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V9 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V10 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V11 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V12 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V13 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V14 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V15 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V16 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V17 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V18 {
    type Category = ();
    type Code = ();
}

impl EventPayloadDialect for V19 {
    type Category = EventCategory;
    type Code = Ref<targets::Character>;
}

/// Typed event payload payload in its serialized field order.
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
pub struct EventPayload<D: EventPayloadDialect> {
    /// Resource or record identity; its namespace is determined by this record.
    pub id: u32,
    /// Source event category in revisions that include this field.
    pub category: D::Category,
    /// Section reference containing code; the target revision is selected by its index entry.
    pub code: D::Code,
}

/// Typed collision shape desc payload in its serialized field order.
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
pub struct CollisionShapeDesc {
    /// Primitive collision-shape category.
    pub kind: CollisionShapeKind,
    /// Axis-aligned box bounds used by box-shaped collision records.
    pub bounds: Extent,
    /// Sphere centre and radius used by spherical collision records.
    pub sphere: Sphere,
    /// Capsule or cylinder length along its local axis.
    pub height: f32,
}

/// Typed collision payload payload in its serialized field order.
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
pub struct CollisionPayload {
    /// Inline primitive dimensions of this collision node.
    pub shape: CollisionShapeDesc,
}

/// Static field selections for `Node` section or inline context revisions.
pub trait NodeDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `pose_matrices`; `()` omits this slot.
    type PoseMatrices: NativeField;
    /// Representation of `payload`; `()` omits this slot.
    type Payload: NativeField;
    /// Representation of `skin`; `()` omits this slot.
    type Skin: NativeField;
    /// Representation of `rig`; `()` omits this slot.
    type Rig: NativeField;
    /// Representation of `profiles`; `()` omits this slot.
    type Profiles: NativeField;
}

impl NodeDialect for V1 {
    const VERSION: u32 = 1;
    type PoseMatrices = ();
    type Payload = NodePayload<V1>;
    type Skin = ();
    type Rig = ();
    type Profiles = ();
}

impl NodeDialect for V2 {
    const VERSION: u32 = 2;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V2>;
    type Skin = ();
    type Rig = ();
    type Profiles = ();
}

impl NodeDialect for V3 {
    const VERSION: u32 = 3;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V3>;
    type Skin = ();
    type Rig = ();
    type Profiles = ();
}

impl NodeDialect for V4 {
    const VERSION: u32 = 4;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V4>;
    type Skin = ();
    type Rig = ();
    type Profiles = ();
}

impl NodeDialect for V5 {
    const VERSION: u32 = 5;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V5>;
    type Skin = ();
    type Rig = ();
    type Profiles = ();
}

impl NodeDialect for V6 {
    const VERSION: u32 = 6;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V6>;
    type Skin = NodeSkinSetup<V6>;
    type Rig = ();
    type Profiles = ();
}

impl NodeDialect for V7 {
    const VERSION: u32 = 7;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V7>;
    type Skin = NodeSkinSetup<V7>;
    type Rig = ();
    type Profiles = ();
}

impl NodeDialect for V8 {
    const VERSION: u32 = 8;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V8>;
    type Skin = NodeSkinSetup<V8>;
    type Rig = ();
    type Profiles = ();
}

impl NodeDialect for V9 {
    const VERSION: u32 = 9;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V9>;
    type Skin = NodeSkinSetup<V9>;
    type Rig = ();
    type Profiles = ();
}

impl NodeDialect for V10 {
    const VERSION: u32 = 10;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V10>;
    type Skin = NodeSkinSetup<V10>;
    type Rig = NodeRig;
    type Profiles = ();
}

impl NodeDialect for V11 {
    const VERSION: u32 = 11;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V11>;
    type Skin = NodeSkinSetup<V11>;
    type Rig = NodeRig;
    type Profiles = ();
}

impl NodeDialect for V12 {
    const VERSION: u32 = 12;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V12>;
    type Skin = NodeSkinSetup<V12>;
    type Rig = NodeRig;
    type Profiles = ();
}

impl NodeDialect for V13 {
    const VERSION: u32 = 13;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V13>;
    type Skin = NodeSkinSetup<V13>;
    type Rig = NodeRig;
    type Profiles = ();
}

impl NodeDialect for V14 {
    const VERSION: u32 = 14;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V14>;
    type Skin = NodeSkinSetup<V14>;
    type Rig = NodeRig;
    type Profiles = ();
}

impl NodeDialect for V15 {
    const VERSION: u32 = 15;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V15>;
    type Skin = NodeSkinSetup<V15>;
    type Rig = NodeRig;
    type Profiles = ();
}

impl NodeDialect for V16 {
    const VERSION: u32 = 16;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V16>;
    type Skin = NodeSkinSetup<V16>;
    type Rig = NodeRig;
    type Profiles = u32;
}

impl NodeDialect for V17 {
    const VERSION: u32 = 17;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V17>;
    type Skin = NodeSkinSetup<V17>;
    type Rig = NodeRig;
    type Profiles = u32;
}

impl NodeDialect for V18 {
    const VERSION: u32 = 18;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V18>;
    type Skin = NodeSkinSetup<V18>;
    type Rig = NodeRig;
    type Profiles = u32;
}

impl NodeDialect for V19 {
    const VERSION: u32 = 19;
    type PoseMatrices = Ref<targets::Matrix44>;
    type Payload = NodePayload<V19>;
    type Skin = NodeSkinSetup<V19>;
    type Rig = NodeRig;
    type Profiles = u32;
}

/// Typed node payload in its serialized field order.
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
pub struct Node<D: NodeDialect> {
    /// Character-section reference naming this hierarchy node.
    pub name: Ref<targets::Character>,
    /// Parent node index within the model; the invalid-index sentinel denotes a root.
    pub parent: u32,
    /// Declared node kind, retained independently from the payload discriminator.
    pub kind: NodeKind,
    /// Inheritance and billboard state bits for this node.
    pub flags: NodeFlags,
    /// Node pivot position in model coordinates.
    pub pivot: [f32; 3],
    /// Stored local translation, rotation, and scale.
    pub local: Transform,
    /// Marks nodes whose source animation uses one scale component.
    pub uniform_scale_only: Bool,
    /// XFRM section containing saved translation/rotation/scale poses.
    pub poses: Ref<targets::Transform>,
    /// MTX4 section containing saved matrix poses in revisions two and later.
    pub pose_matrices: D::PoseMatrices,
    /// Inline node-specific parameters selected by their independent discriminator.
    pub payload: D::Payload,
    /// NBAG metadata preserving source-specific node properties.
    pub native: NativeBag,
    /// Stored skinning setup in revisions six and later.
    pub skin: D::Skin,
    /// Stored rig classification in revisions ten and later.
    pub rig: D::Rig,
    /// Bit mask selecting the source profiles that include this node.
    pub profiles: D::Profiles,
}
impl<D: NodeDialect> Record for Node<D> {
    const TAG: [u8; 4] = *b"NODE";
    const VERSION: u32 = D::VERSION;
}

/// Typed wc3 particle emitter1 payload payload in its serialized field order.
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
pub struct Wc3ParticleEmitter1Payload {
    /// Particle emission-rate control; its stored source-format representation is retained.
    pub emission_rate: f32,
    /// Gravity control retained from the source emitter.
    pub gravity: f32,
    /// Longitude parameter stored as a floating-point value.
    pub longitude: f32,
    /// Latitude parameter stored as a floating-point value.
    pub latitude: f32,
    /// Lifetime of an emitted particle, in seconds.
    pub lifespan: f32,
    /// Initial particle-speed control retained from the source emitter.
    pub speed: f32,
    /// Asset identifying the model instantiated for each particle.
    pub spawn_model: AssetKey,
    /// Stored boolean byte controlling uses mdl; nonzero encodings are retained.
    pub uses_mdl: Bool,
    /// Stored boolean byte controlling uses tga; nonzero encodings are retained.
    pub uses_tga: Bool,
}

/// Typed wc3 particle segment payload in its serialized field order.
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
pub struct Wc3ParticleSegment {
    /// RGB color of emitted geometry.
    pub color: [f32; 3],
    /// Opacity control for the emitted geometry.
    pub alpha: u8,
    /// Scaling parameter stored as a floating-point value.
    pub scaling: f32,
}

/// Typed wc3 particle interval payload in its serialized field order.
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
pub struct Wc3ParticleInterval {
    /// First texture-atlas cell in the interval.
    pub start: u32,
    /// Last texture-atlas cell in the interval.
    pub end: u32,
    /// Number of repetitions of the atlas interval.
    pub repeat: u32,
}

/// Typed wc3 particle emitter2 payload payload in its serialized field order.
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
pub struct Wc3ParticleEmitter2Payload {
    /// Initial particle-speed control retained from the source emitter.
    pub speed: f32,
    /// Variation parameter stored as a floating-point value.
    pub variation: f32,
    /// Latitude parameter stored as a floating-point value.
    pub latitude: f32,
    /// Gravity control retained from the source emitter.
    pub gravity: f32,
    /// Lifetime of an emitted particle, in seconds.
    pub lifespan: f32,
    /// Particle emission-rate control; its stored source-format representation is retained.
    pub emission_rate: f32,
    /// Width parameter stored as a floating-point value.
    pub width: f32,
    /// Length parameter stored as a floating-point value.
    pub length: f32,
    /// Warcraft III particle blending mode, using the particle-specific numbering.
    pub filter: Wc3ParticleFilter,
    /// Number of rows in the particle texture atlas.
    pub rows: u32,
    /// Number of columns in the particle texture atlas.
    pub columns: u32,
    /// Selects particle heads, tails, or both for drawing.
    pub head_or_tail: Wc3ParticleHeadOrTail,
    /// Length control for the particle trail.
    pub tail_length: f32,
    /// Time parameter; units follow the enclosing animation or effect.
    pub time: f32,
    /// Color and scale at the start of particle life.
    pub start: Wc3ParticleSegment,
    /// Color and scale at the intermediate age point.
    pub middle: Wc3ParticleSegment,
    /// Color and scale at the end of particle life.
    pub end: Wc3ParticleSegment,
    /// Atlas interval used by particle heads during the life phase.
    pub head_life: Wc3ParticleInterval,
    /// Atlas interval used by particle heads during decay.
    pub head_decay: Wc3ParticleInterval,
    /// Atlas interval used by particle tails during the life phase.
    pub tail_life: Wc3ParticleInterval,
    /// Atlas interval used by particle tails during decay.
    pub tail_decay: Wc3ParticleInterval,
    /// Document texture index used by the emitter.
    pub texture: u32,
    /// Warcraft III replaceable-texture identifier.
    pub replaceable_id: u32,
    /// Stored boolean byte controlling squirt; nonzero encodings are retained.
    pub squirt: Bool,
    /// Signed drawing-priority plane of the emitter.
    pub priority_plane: i32,
    /// Stored boolean byte controlling unshaded; nonzero encodings are retained.
    pub unshaded: Bool,
    /// Stored boolean byte controlling sort prims far z; nonzero encodings are retained.
    pub sort_prims_far_z: Bool,
    /// Stored boolean byte controlling line emitter; nonzero encodings are retained.
    pub line_emitter: Bool,
    /// Stored boolean byte controlling unfogged; nonzero encodings are retained.
    pub unfogged: Bool,
    /// Stored boolean byte controlling xy quad; nonzero encodings are retained.
    pub xy_quad: Bool,
}

/// Typed wc3 ribbon emitter payload payload in its serialized field order.
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
pub struct Wc3RibbonEmitterPayload {
    /// Height above parameter stored as a floating-point value.
    pub height_above: f32,
    /// Height below parameter stored as a floating-point value.
    pub height_below: f32,
    /// Opacity control for the emitted geometry.
    pub alpha: f32,
    /// RGB color of emitted geometry.
    pub color: [f32; 3],
    /// Lifetime of an emitted particle, in seconds.
    pub lifespan: f32,
    /// Initial texture-slot selection for the ribbon material.
    pub texture_slot: u32,
    /// Particle emission-rate control; its stored source-format representation is retained.
    pub emission_rate: u32,
    /// Number of rows in the particle texture atlas.
    pub rows: u32,
    /// Number of columns in the particle texture atlas.
    pub columns: u32,
    /// Index of the model material slot drawn by this emitter.
    pub material_slot: u32,
    /// Gravity control retained from the source emitter.
    pub gravity: f32,
}

/// Typed wc3 corn emitter payload payload in its serialized field order.
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
pub struct Wc3CornEmitterPayload {
    /// Lifetime of an emitted particle, in seconds.
    pub lifespan: f32,
    /// Particle emission-rate control; its stored source-format representation is retained.
    pub emission_rate: f32,
    /// Initial particle-speed control retained from the source emitter.
    pub speed: f32,
    /// RGB color of emitted geometry.
    pub color: [f32; 3],
    /// Opacity control for the emitted geometry.
    pub alpha: f32,
    /// Warcraft III replaceable-texture identifier.
    pub replaceable_id: u32,
    /// Asset identifying the external PopcornFX effect.
    pub effect: AssetKey,
    /// Section reference containing anim visibility guide; the target revision is selected by its index entry.
    pub anim_visibility_guide: Ref<targets::Character>,
    /// Stored boolean byte controlling unshaded; nonzero encodings are retained.
    pub unshaded: Bool,
    /// Stored boolean byte controlling sort prims far z; nonzero encodings are retained.
    pub sort_prims_far_z: Bool,
    /// Stored boolean byte controlling unfogged; nonzero encodings are retained.
    pub unfogged: Bool,
    /// Stored boolean byte controlling popcorn scaling; nonzero encodings are retained.
    pub popcorn_scaling: Bool,
}

/// Static field selections for `M2ParticleEmitterPayload` section or inline context revisions.
pub trait M2ParticleEmitterPayloadDialect: Revision {
    /// Representation of `texture2_scale`; `()` omits this slot.
    type Texture2Scale: NativeField;
    /// Representation of `texture3_scale`; `()` omits this slot.
    type Texture3Scale: NativeField;
    /// Representation of `texture2_scroll_mid`; `()` omits this slot.
    type Texture2ScrollMid: NativeField;
    /// Representation of `texture3_scroll_mid`; `()` omits this slot.
    type Texture3ScrollMid: NativeField;
    /// Representation of `texture2_scroll_range`; `()` omits this slot.
    type Texture2ScrollRange: NativeField;
    /// Representation of `texture3_scroll_range`; `()` omits this slot.
    type Texture3ScrollRange: NativeField;
    /// Representation of `inherit_velocity_scale`; `()` omits this slot.
    type InheritVelocityScale: NativeField;
    /// Representation of `follow_speed1`; `()` omits this slot.
    type FollowSpeed1: NativeField;
    /// Representation of `follow_scale1`; `()` omits this slot.
    type FollowScale1: NativeField;
    /// Representation of `follow_speed2`; `()` omits this slot.
    type FollowSpeed2: NativeField;
    /// Representation of `follow_scale2`; `()` omits this slot.
    type FollowScale2: NativeField;
    /// Representation of `tumble_min`; `()` omits this slot.
    type TumbleMin: NativeField;
    /// Representation of `tumble_max`; `()` omits this slot.
    type TumbleMax: NativeField;
    /// Representation of `color_mult`; `()` omits this slot.
    type ColorMult: NativeField;
    /// Representation of `alpha_mult`; `()` omits this slot.
    type AlphaMult: NativeField;
    /// Representation of `alpha_cutoff_times`; `()` omits this slot.
    type AlphaCutoffTimes: NativeField;
    /// Representation of `alpha_cutoffs`; `()` omits this slot.
    type AlphaCutoffs: NativeField;
    /// Representation of `particle_model`; `()` omits this slot.
    type ParticleModel: NativeField;
    /// Representation of `trail_model`; `()` omits this slot.
    type TrailModel: NativeField;
}

impl M2ParticleEmitterPayloadDialect for V0 {
    type Texture2Scale = ();
    type Texture3Scale = ();
    type Texture2ScrollMid = ();
    type Texture3ScrollMid = ();
    type Texture2ScrollRange = ();
    type Texture3ScrollRange = ();
    type InheritVelocityScale = ();
    type FollowSpeed1 = ();
    type FollowScale1 = ();
    type FollowSpeed2 = ();
    type FollowScale2 = ();
    type TumbleMin = ();
    type TumbleMax = ();
    type ColorMult = ();
    type AlphaMult = ();
    type AlphaCutoffTimes = ();
    type AlphaCutoffs = ();
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V1 {
    type Texture2Scale = ();
    type Texture3Scale = ();
    type Texture2ScrollMid = ();
    type Texture3ScrollMid = ();
    type Texture2ScrollRange = ();
    type Texture3ScrollRange = ();
    type InheritVelocityScale = ();
    type FollowSpeed1 = ();
    type FollowScale1 = ();
    type FollowSpeed2 = ();
    type FollowScale2 = ();
    type TumbleMin = ();
    type TumbleMax = ();
    type ColorMult = ();
    type AlphaMult = ();
    type AlphaCutoffTimes = ();
    type AlphaCutoffs = ();
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V2 {
    type Texture2Scale = ();
    type Texture3Scale = ();
    type Texture2ScrollMid = ();
    type Texture3ScrollMid = ();
    type Texture2ScrollRange = ();
    type Texture3ScrollRange = ();
    type InheritVelocityScale = ();
    type FollowSpeed1 = ();
    type FollowScale1 = ();
    type FollowSpeed2 = ();
    type FollowScale2 = ();
    type TumbleMin = ();
    type TumbleMax = ();
    type ColorMult = ();
    type AlphaMult = ();
    type AlphaCutoffTimes = ();
    type AlphaCutoffs = ();
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V3 {
    type Texture2Scale = ();
    type Texture3Scale = ();
    type Texture2ScrollMid = ();
    type Texture3ScrollMid = ();
    type Texture2ScrollRange = ();
    type Texture3ScrollRange = ();
    type InheritVelocityScale = ();
    type FollowSpeed1 = ();
    type FollowScale1 = ();
    type FollowSpeed2 = ();
    type FollowScale2 = ();
    type TumbleMin = ();
    type TumbleMax = ();
    type ColorMult = ();
    type AlphaMult = ();
    type AlphaCutoffTimes = ();
    type AlphaCutoffs = ();
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V4 {
    type Texture2Scale = ();
    type Texture3Scale = ();
    type Texture2ScrollMid = ();
    type Texture3ScrollMid = ();
    type Texture2ScrollRange = ();
    type Texture3ScrollRange = ();
    type InheritVelocityScale = ();
    type FollowSpeed1 = ();
    type FollowScale1 = ();
    type FollowSpeed2 = ();
    type FollowScale2 = ();
    type TumbleMin = ();
    type TumbleMax = ();
    type ColorMult = ();
    type AlphaMult = ();
    type AlphaCutoffTimes = ();
    type AlphaCutoffs = ();
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V5 {
    type Texture2Scale = ();
    type Texture3Scale = ();
    type Texture2ScrollMid = ();
    type Texture3ScrollMid = ();
    type Texture2ScrollRange = ();
    type Texture3ScrollRange = ();
    type InheritVelocityScale = ();
    type FollowSpeed1 = ();
    type FollowScale1 = ();
    type FollowSpeed2 = ();
    type FollowScale2 = ();
    type TumbleMin = ();
    type TumbleMax = ();
    type ColorMult = ();
    type AlphaMult = ();
    type AlphaCutoffTimes = ();
    type AlphaCutoffs = ();
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V6 {
    type Texture2Scale = ();
    type Texture3Scale = ();
    type Texture2ScrollMid = ();
    type Texture3ScrollMid = ();
    type Texture2ScrollRange = ();
    type Texture3ScrollRange = ();
    type InheritVelocityScale = ();
    type FollowSpeed1 = ();
    type FollowScale1 = ();
    type FollowSpeed2 = ();
    type FollowScale2 = ();
    type TumbleMin = ();
    type TumbleMax = ();
    type ColorMult = ();
    type AlphaMult = ();
    type AlphaCutoffTimes = ();
    type AlphaCutoffs = ();
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V7 {
    type Texture2Scale = ();
    type Texture3Scale = ();
    type Texture2ScrollMid = ();
    type Texture3ScrollMid = ();
    type Texture2ScrollRange = ();
    type Texture3ScrollRange = ();
    type InheritVelocityScale = ();
    type FollowSpeed1 = ();
    type FollowScale1 = ();
    type FollowSpeed2 = ();
    type FollowScale2 = ();
    type TumbleMin = ();
    type TumbleMax = ();
    type ColorMult = ();
    type AlphaMult = ();
    type AlphaCutoffTimes = ();
    type AlphaCutoffs = ();
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V8 {
    type Texture2Scale = ();
    type Texture3Scale = ();
    type Texture2ScrollMid = ();
    type Texture3ScrollMid = ();
    type Texture2ScrollRange = ();
    type Texture3ScrollRange = ();
    type InheritVelocityScale = ();
    type FollowSpeed1 = ();
    type FollowScale1 = ();
    type FollowSpeed2 = ();
    type FollowScale2 = ();
    type TumbleMin = ();
    type TumbleMax = ();
    type ColorMult = ();
    type AlphaMult = ();
    type AlphaCutoffTimes = ();
    type AlphaCutoffs = ();
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V9 {
    type Texture2Scale = f32;
    type Texture3Scale = f32;
    type Texture2ScrollMid = [f32; 2];
    type Texture3ScrollMid = [f32; 2];
    type Texture2ScrollRange = [f32; 2];
    type Texture3ScrollRange = [f32; 2];
    type InheritVelocityScale = ();
    type FollowSpeed1 = ();
    type FollowScale1 = ();
    type FollowSpeed2 = ();
    type FollowScale2 = ();
    type TumbleMin = ();
    type TumbleMax = ();
    type ColorMult = ();
    type AlphaMult = ();
    type AlphaCutoffTimes = ();
    type AlphaCutoffs = ();
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V10 {
    type Texture2Scale = f32;
    type Texture3Scale = f32;
    type Texture2ScrollMid = [f32; 2];
    type Texture3ScrollMid = [f32; 2];
    type Texture2ScrollRange = [f32; 2];
    type Texture3ScrollRange = [f32; 2];
    type InheritVelocityScale = ();
    type FollowSpeed1 = ();
    type FollowScale1 = ();
    type FollowSpeed2 = ();
    type FollowScale2 = ();
    type TumbleMin = ();
    type TumbleMax = ();
    type ColorMult = ();
    type AlphaMult = ();
    type AlphaCutoffTimes = ();
    type AlphaCutoffs = ();
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V11 {
    type Texture2Scale = f32;
    type Texture3Scale = f32;
    type Texture2ScrollMid = [f32; 2];
    type Texture3ScrollMid = [f32; 2];
    type Texture2ScrollRange = [f32; 2];
    type Texture3ScrollRange = [f32; 2];
    type InheritVelocityScale = f32;
    type FollowSpeed1 = f32;
    type FollowScale1 = f32;
    type FollowSpeed2 = f32;
    type FollowScale2 = f32;
    type TumbleMin = [f32; 3];
    type TumbleMax = [f32; 3];
    type ColorMult = f32;
    type AlphaMult = f32;
    type AlphaCutoffTimes = Ref<targets::Real>;
    type AlphaCutoffs = Ref<targets::Real>;
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V12 {
    type Texture2Scale = f32;
    type Texture3Scale = f32;
    type Texture2ScrollMid = [f32; 2];
    type Texture3ScrollMid = [f32; 2];
    type Texture2ScrollRange = [f32; 2];
    type Texture3ScrollRange = [f32; 2];
    type InheritVelocityScale = f32;
    type FollowSpeed1 = f32;
    type FollowScale1 = f32;
    type FollowSpeed2 = f32;
    type FollowScale2 = f32;
    type TumbleMin = [f32; 3];
    type TumbleMax = [f32; 3];
    type ColorMult = f32;
    type AlphaMult = f32;
    type AlphaCutoffTimes = Ref<targets::Real>;
    type AlphaCutoffs = Ref<targets::Real>;
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V13 {
    type Texture2Scale = f32;
    type Texture3Scale = f32;
    type Texture2ScrollMid = [f32; 2];
    type Texture3ScrollMid = [f32; 2];
    type Texture2ScrollRange = [f32; 2];
    type Texture3ScrollRange = [f32; 2];
    type InheritVelocityScale = f32;
    type FollowSpeed1 = f32;
    type FollowScale1 = f32;
    type FollowSpeed2 = f32;
    type FollowScale2 = f32;
    type TumbleMin = [f32; 3];
    type TumbleMax = [f32; 3];
    type ColorMult = f32;
    type AlphaMult = f32;
    type AlphaCutoffTimes = Ref<targets::Real>;
    type AlphaCutoffs = Ref<targets::Real>;
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V14 {
    type Texture2Scale = f32;
    type Texture3Scale = f32;
    type Texture2ScrollMid = [f32; 2];
    type Texture3ScrollMid = [f32; 2];
    type Texture2ScrollRange = [f32; 2];
    type Texture3ScrollRange = [f32; 2];
    type InheritVelocityScale = f32;
    type FollowSpeed1 = f32;
    type FollowScale1 = f32;
    type FollowSpeed2 = f32;
    type FollowScale2 = f32;
    type TumbleMin = [f32; 3];
    type TumbleMax = [f32; 3];
    type ColorMult = f32;
    type AlphaMult = f32;
    type AlphaCutoffTimes = Ref<targets::Real>;
    type AlphaCutoffs = Ref<targets::Real>;
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V15 {
    type Texture2Scale = f32;
    type Texture3Scale = f32;
    type Texture2ScrollMid = [f32; 2];
    type Texture3ScrollMid = [f32; 2];
    type Texture2ScrollRange = [f32; 2];
    type Texture3ScrollRange = [f32; 2];
    type InheritVelocityScale = f32;
    type FollowSpeed1 = f32;
    type FollowScale1 = f32;
    type FollowSpeed2 = f32;
    type FollowScale2 = f32;
    type TumbleMin = [f32; 3];
    type TumbleMax = [f32; 3];
    type ColorMult = f32;
    type AlphaMult = f32;
    type AlphaCutoffTimes = Ref<targets::Real>;
    type AlphaCutoffs = Ref<targets::Real>;
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V16 {
    type Texture2Scale = f32;
    type Texture3Scale = f32;
    type Texture2ScrollMid = [f32; 2];
    type Texture3ScrollMid = [f32; 2];
    type Texture2ScrollRange = [f32; 2];
    type Texture3ScrollRange = [f32; 2];
    type InheritVelocityScale = f32;
    type FollowSpeed1 = f32;
    type FollowScale1 = f32;
    type FollowSpeed2 = f32;
    type FollowScale2 = f32;
    type TumbleMin = [f32; 3];
    type TumbleMax = [f32; 3];
    type ColorMult = f32;
    type AlphaMult = f32;
    type AlphaCutoffTimes = Ref<targets::Real>;
    type AlphaCutoffs = Ref<targets::Real>;
    type ParticleModel = ();
    type TrailModel = ();
}

impl M2ParticleEmitterPayloadDialect for V17 {
    type Texture2Scale = f32;
    type Texture3Scale = f32;
    type Texture2ScrollMid = [f32; 2];
    type Texture3ScrollMid = [f32; 2];
    type Texture2ScrollRange = [f32; 2];
    type Texture3ScrollRange = [f32; 2];
    type InheritVelocityScale = f32;
    type FollowSpeed1 = f32;
    type FollowScale1 = f32;
    type FollowSpeed2 = f32;
    type FollowScale2 = f32;
    type TumbleMin = [f32; 3];
    type TumbleMax = [f32; 3];
    type ColorMult = f32;
    type AlphaMult = f32;
    type AlphaCutoffTimes = Ref<targets::Real>;
    type AlphaCutoffs = Ref<targets::Real>;
    type ParticleModel = AssetKey;
    type TrailModel = AssetKey;
}

impl M2ParticleEmitterPayloadDialect for V18 {
    type Texture2Scale = f32;
    type Texture3Scale = f32;
    type Texture2ScrollMid = [f32; 2];
    type Texture3ScrollMid = [f32; 2];
    type Texture2ScrollRange = [f32; 2];
    type Texture3ScrollRange = [f32; 2];
    type InheritVelocityScale = f32;
    type FollowSpeed1 = f32;
    type FollowScale1 = f32;
    type FollowSpeed2 = f32;
    type FollowScale2 = f32;
    type TumbleMin = [f32; 3];
    type TumbleMax = [f32; 3];
    type ColorMult = f32;
    type AlphaMult = f32;
    type AlphaCutoffTimes = Ref<targets::Real>;
    type AlphaCutoffs = Ref<targets::Real>;
    type ParticleModel = AssetKey;
    type TrailModel = AssetKey;
}

impl M2ParticleEmitterPayloadDialect for V19 {
    type Texture2Scale = f32;
    type Texture3Scale = f32;
    type Texture2ScrollMid = [f32; 2];
    type Texture3ScrollMid = [f32; 2];
    type Texture2ScrollRange = [f32; 2];
    type Texture3ScrollRange = [f32; 2];
    type InheritVelocityScale = f32;
    type FollowSpeed1 = f32;
    type FollowScale1 = f32;
    type FollowSpeed2 = f32;
    type FollowScale2 = f32;
    type TumbleMin = [f32; 3];
    type TumbleMax = [f32; 3];
    type ColorMult = f32;
    type AlphaMult = f32;
    type AlphaCutoffTimes = Ref<targets::Real>;
    type AlphaCutoffs = Ref<targets::Real>;
    type ParticleModel = AssetKey;
    type TrailModel = AssetKey;
}

/// Typed m2 particle emitter payload payload in its serialized field order.
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
pub struct M2ParticleEmitterPayload<D: M2ParticleEmitterPayloadDialect> {
    /// M2 particle-emitter identifier; negative values are retained.
    pub particle_id: i32,
    /// Stored state bits; unknown bits are retained.
    pub flags: u32,
    /// Document texture index used by the emitter.
    pub texture: u32,
    /// Second texture index in the document texture table, used by multi-texture emitters.
    pub texture2: u32,
    /// Third texture index in the document texture table, used by multi-texture emitters.
    pub texture3: u32,
    /// M2 blend-mode number, independent of Warcraft III particle blending modes.
    pub blend: u32,
    /// M2 emitter shape code: 1 plane, 2 sphere, 3 spline, or 4 bone.
    pub emitter_type: u32,
    /// Color index selector; sentinel and unrecognized values are retained.
    pub color_index: u32,
    /// Number of rows in the particle texture atlas.
    pub rows: u32,
    /// Number of columns in the particle texture atlas.
    pub columns: u32,
    /// Signed drawing-priority plane of the emitter.
    pub priority_plane: i32,
    /// Initial particle-speed control retained from the source emitter.
    pub speed: f32,
    /// Speed variation parameter stored as a floating-point value.
    pub speed_variation: f32,
    /// Vertical range parameter stored as a floating-point value.
    pub vertical_range: f32,
    /// Horizontal range parameter stored as a floating-point value.
    pub horizontal_range: f32,
    /// Gravity control retained from the source emitter.
    pub gravity: [f32; 3],
    /// Lifetime of an emitted particle, in seconds.
    pub lifespan: f32,
    /// Lifespan variation parameter stored as a floating-point value.
    pub lifespan_variation: f32,
    /// Particle emission-rate control; its stored source-format representation is retained.
    pub emission_rate: f32,
    /// Emission rate variation parameter stored as a floating-point value.
    pub emission_rate_variation: f32,
    /// Width parameter stored as a floating-point value.
    pub width: f32,
    /// Length parameter stored as a floating-point value.
    pub length: f32,
    /// Z source parameter stored as a floating-point value.
    pub z_source: f32,
    /// Section reference containing color times; the target revision is selected by its index entry.
    pub color_times: Ref<targets::Real>,
    /// Section reference containing colors; the target revision is selected by its index entry.
    pub colors: Ref<targets::Vector3>,
    /// Section reference containing alpha times; the target revision is selected by its index entry.
    pub alpha_times: Ref<targets::Real>,
    /// Section reference containing alphas; the target revision is selected by its index entry.
    pub alphas: Ref<targets::Real>,
    /// Section reference containing scale times; the target revision is selected by its index entry.
    pub scale_times: Ref<targets::Real>,
    /// Section reference containing scales; the target revision is selected by its index entry.
    pub scales: Ref<targets::Vector2>,
    /// Variation in the two components of the particle scale.
    pub scale_variation: [f32; 2],
    /// Section reference containing head cell times; the target revision is selected by its index entry.
    pub head_cell_times: Ref<targets::Real>,
    /// Section reference containing head cells; the target revision is selected by its index entry.
    pub head_cells: Ref<targets::U32>,
    /// Section reference containing tail cell times; the target revision is selected by its index entry.
    pub tail_cell_times: Ref<targets::Real>,
    /// Section reference containing tail cells; the target revision is selected by its index entry.
    pub tail_cells: Ref<targets::U32>,
    /// Length control for the particle trail.
    pub tail_length: f32,
    /// Twinkle speed parameter stored as a floating-point value.
    pub twinkle_speed: f32,
    /// Twinkle percent parameter stored as a floating-point value.
    pub twinkle_percent: f32,
    /// Two-component particle-scale control for the twinkle effect; component interpretation is unresolved.
    pub twinkle_scale: [f32; 2],
    /// Drag applied to particle motion.
    pub drag: f32,
    /// Base spin parameter stored as a floating-point value.
    pub base_spin: f32,
    /// Base spin variation parameter stored as a floating-point value.
    pub base_spin_variation: f32,
    /// Spin parameter stored as a floating-point value.
    pub spin: f32,
    /// Spin variation parameter stored as a floating-point value.
    pub spin_variation: f32,
    /// Wind vector applied to particle motion.
    pub wind: [f32; 3],
    /// Wind time parameter; units follow the enclosing animation or effect.
    pub wind_time: f32,
    /// Section reference containing spline points; the target revision is selected by its index entry.
    pub spline_points: Ref<targets::Vector3>,
    /// Texture2 scale parameter stored as a floating-point value.
    pub texture2_scale: D::Texture2Scale,
    /// Texture3 scale parameter stored as a floating-point value.
    pub texture3_scale: D::Texture3Scale,
    /// Central UV scrolling rate of texture layer 2, in texture-coordinate units per second.
    pub texture2_scroll_mid: D::Texture2ScrollMid,
    /// Central UV scrolling rate of texture layer 3, in texture-coordinate units per second.
    pub texture3_scroll_mid: D::Texture3ScrollMid,
    /// Random spread around the UV scrolling rate of texture layer 2, in texture-coordinate units per second.
    pub texture2_scroll_range: D::Texture2ScrollRange,
    /// Random spread around the UV scrolling rate of texture layer 3, in texture-coordinate units per second.
    pub texture3_scroll_range: D::Texture3ScrollRange,
    /// Inherit velocity scale parameter stored as a floating-point value.
    pub inherit_velocity_scale: D::InheritVelocityScale,
    /// Follow speed1 parameter stored as a floating-point value.
    pub follow_speed1: D::FollowSpeed1,
    /// Follow scale1 parameter stored as a floating-point value.
    pub follow_scale1: D::FollowScale1,
    /// Follow speed2 parameter stored as a floating-point value.
    pub follow_speed2: D::FollowSpeed2,
    /// Follow scale2 parameter stored as a floating-point value.
    pub follow_scale2: D::FollowScale2,
    /// Lower per-axis bound on model-particle angular velocity, in radians per second.
    pub tumble_min: D::TumbleMin,
    /// Upper per-axis bound on model-particle angular velocity, in radians per second.
    pub tumble_max: D::TumbleMax,
    /// Color mult parameter stored as a floating-point value.
    pub color_mult: D::ColorMult,
    /// Alpha mult parameter stored as a floating-point value.
    pub alpha_mult: D::AlphaMult,
    /// Section reference containing alpha cutoff times; the target revision is selected by its index entry.
    pub alpha_cutoff_times: D::AlphaCutoffTimes,
    /// Section reference containing alpha cutoffs; the target revision is selected by its index entry.
    pub alpha_cutoffs: D::AlphaCutoffs,
    /// M2 asset drawn for each particle instead of a textured quad.
    pub particle_model: D::ParticleModel,
    /// M2 asset supplying child emitters that trail the particles.
    pub trail_model: D::TrailModel,
}

/// Typed sc2 property payload in its serialized field order.
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
pub struct Sc2Property<T> {
    /// Unanimated value of this emitter property.
    pub init_value: T,
    /// Value used when this emitter property is reset.
    pub null_value: T,
}

/// Typed sc2 variation payload in its serialized field order.
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
pub struct Sc2Variation {
    /// Variation function code retained from the source emitter.
    pub type_: u32,
    /// Magnitude of variation applied to the associated property.
    pub amplitude: Sc2Property<f32>,
    /// Frequency parameter of the selected variation function.
    pub frequency: Sc2Property<f32>,
}

/// StarCraft II particle controls, instance settings, and links to secondary effects.
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
pub struct Sc2ParticleEmitterPayload {
    /// Node index of the original StarCraft II particle emitter; the invalid-node sentinel denotes an independent emitter.
    pub copy_of: u32,
    /// Index of the model material slot drawn by this emitter.
    pub material_slot: u32,
    /// Supplementary StarCraft II emitter flags; unknown bits remain available.
    pub additional_flags: u32,
    /// Initial particle-speed property, including its unanimated and reset values.
    pub initial_speed: Sc2Property<f32>,
    /// Randomized initial-speed property in the source emitter.
    pub initial_speed_random: Sc2Property<f32>,
    /// Initial yaw property for emitted particles.
    pub initial_yaw: Sc2Property<f32>,
    /// Initial pitch property for emitted particles.
    pub initial_pitch: Sc2Property<f32>,
    /// Initial horizontal emission property in the StarCraft II emitter frame.
    pub initial_horizontal: Sc2Property<f32>,
    /// Initial vertical emission property in the StarCraft II emitter frame.
    pub initial_vertical: Sc2Property<f32>,
    /// Particle lifetime control retained from the source emitter.
    pub lifetime: Sc2Property<f32>,
    /// Random contribution to the particle-lifetime property.
    pub lifetime_random: Sc2Property<f32>,
    /// Kill radius parameter stored as a floating-point value.
    pub kill_radius: f32,
    /// Source X-gravity word. Its integer interpretation in particle emitters is unresolved.
    pub gravity_x: u32,
    /// Source Y-gravity word. Its integer interpretation in particle emitters is unresolved.
    pub gravity_y: u32,
    /// Gravity control retained from the source emitter.
    pub gravity: f32,
    /// Normalized particle age at the intermediate size control point.
    pub size_mid_time: f32,
    /// Normalized particle age at the intermediate color control point.
    pub color_mid_time: f32,
    /// Normalized particle age at the intermediate alpha control point.
    pub alpha_mid_time: f32,
    /// Normalized particle age at the intermediate rotation control point.
    pub rotation_mid_time: f32,
    /// Size mid hold time parameter; units follow the enclosing animation or effect.
    pub size_mid_hold_time: f32,
    /// Color mid hold time parameter; units follow the enclosing animation or effect.
    pub color_mid_hold_time: f32,
    /// Alpha mid hold time parameter; units follow the enclosing animation or effect.
    pub alpha_mid_hold_time: f32,
    /// Rotation mid hold time parameter; units follow the enclosing animation or effect.
    pub rotation_mid_hold_time: f32,
    /// Three-component particle-size property.
    pub size_animation: Sc2Property<[f32; 3]>,
    /// Three-component particle-rotation property.
    pub rotation_animation: Sc2Property<[f32; 3]>,
    /// Color property at the beginning of particle life.
    pub color_start: Sc2Property<[f32; 4]>,
    /// Color property at the intermediate particle-age point.
    pub color_mid: Sc2Property<[f32; 4]>,
    /// Color property at the end of particle life.
    pub color_end: Sc2Property<[f32; 4]>,
    /// Drag applied to particle motion.
    pub drag: f32,
    /// Mass parameter stored as a floating-point value.
    pub mass: f32,
    /// Mass random parameter stored as a floating-point value.
    pub mass_random: f32,
    /// Mass size multiplier parameter stored as a floating-point value.
    pub mass_size_multiplier: f32,
    /// Mask selecting local force channels.
    pub local_forces: u16,
    /// Mask selecting world force channels.
    pub world_forces: u16,
    /// Fallback mask for local force channels.
    pub local_forces_fallback: u16,
    /// Fallback mask for world force channels.
    pub world_forces_fallback: u16,
    /// World forces mass multiplier parameter stored as a floating-point value.
    pub world_forces_mass_multiplier: f32,
    /// Noise amplitude parameter stored as a floating-point value.
    pub noise_amplitude: f32,
    /// Noise frequency parameter stored as a floating-point value.
    pub noise_frequency: f32,
    /// Noise coherence parameter stored as a floating-point value.
    pub noise_coherence: f32,
    /// Noise edge parameter stored as a floating-point value.
    pub noise_edge: f32,
    /// Index plus length selector; sentinel and unrecognized values are retained.
    pub index_plus_length: u32,
    /// Stored upper bound on the particle population.
    pub max_particles: u32,
    /// Particle emission-rate control; its stored source-format representation is retained.
    pub emission_rate: Sc2Property<f32>,
    /// Shape from which this StarCraft II emitter produces particles.
    pub emitter_shape: Sc2EmitterShape,
    /// Dimensions of the outer emitter shape.
    pub shape_outer: Sc2Property<[f32; 3]>,
    /// Dimensions of the inner emitter shape.
    pub shape_inner: Sc2Property<[f32; 3]>,
    /// Outer radius control of the emitter’s source shape.
    pub outer_radius: Sc2Property<f32>,
    /// Inner radius control of the emitter’s source shape.
    pub inner_radius: Sc2Property<f32>,
    /// Section reference containing shape sections; the target revision is selected by its index entry.
    pub shape_sections: Ref<targets::U32>,
    /// Source velocity-mode code; the meanings of its values are unresolved.
    pub velocity_type: u32,
    /// Source control word enabling size randomization.
    pub size_random_enable: u32,
    /// Random contribution to the three-component size property.
    pub size_random_animation: Sc2Property<[f32; 3]>,
    /// Source control word enabling rotation randomization.
    pub rotation_random_enable: u32,
    /// Random contribution to the three-component rotation property.
    pub rotation_random_animation: Sc2Property<[f32; 3]>,
    /// Source control word enabling color randomization.
    pub color_random_enable: u32,
    /// Random color contribution at the beginning of particle life.
    pub color_start_random: Sc2Property<[f32; 4]>,
    /// Random color contribution at the intermediate age point.
    pub color_mid_random: Sc2Property<[f32; 4]>,
    /// Random color contribution at the end of particle life.
    pub color_end_random: Sc2Property<[f32; 4]>,
    /// Source control word enabling alpha randomization.
    pub alpha_random_enable: u32,
    /// Particle count emitted by a burst; animated changes use step selection.
    pub squirt_amount: Sc2Property<u32>,
    /// Flipbook start init index selector; sentinel and unrecognized values are retained.
    pub flipbook_start_init_index: u8,
    /// Flipbook start stop index selector; sentinel and unrecognized values are retained.
    pub flipbook_start_stop_index: u8,
    /// Flipbook end init index selector; sentinel and unrecognized values are retained.
    pub flipbook_end_init_index: u8,
    /// Flipbook end stop index selector; sentinel and unrecognized values are retained.
    pub flipbook_end_stop_index: u8,
    /// Flipbook mid time parameter; units follow the enclosing animation or effect.
    pub flipbook_mid_time: f32,
    /// Number of columns in the emitter’s flipbook texture atlas.
    pub flipbook_columns: u16,
    /// Number of rows in the emitter’s flipbook texture atlas.
    pub flipbook_rows: u16,
    /// Flipbook column fraction parameter stored as a floating-point value.
    pub flipbook_column_fraction: f32,
    /// Flipbook row fraction parameter stored as a floating-point value.
    pub flipbook_row_fraction: f32,
    /// Particle collision bounce parameter.
    pub bounce: f32,
    /// Particle collision friction parameter.
    pub friction: f32,
    /// Node index of the particle emitter triggered by collisions.
    pub collision_spawn: u32,
    /// Lower bound on the collision-triggered particle count.
    pub collision_spawn_min: u32,
    /// Upper bound on the collision-triggered particle count.
    pub collision_spawn_max: u32,
    /// Collision spawn chance parameter stored as a floating-point value.
    pub collision_spawn_chance: f32,
    /// Collision spawn energy parameter stored as a floating-point value.
    pub collision_spawn_energy: f32,
    /// Source control word for particle death after bouncing; detailed value semantics are unresolved.
    pub collision_die_bounce: u32,
    /// Geometry and orientation mode used to draw a particle instance.
    pub instance_type: Sc2ParticleInstance,
    /// Length control for the particle trail.
    pub tail_length: f32,
    /// Three-component orientation parameter of the particle instance.
    pub instance_angle: [f32; 3],
    /// Instance distance parameter stored as a floating-point value.
    pub instance_distance: f32,
    /// Pitch variation settings, including the function, amplitude, and frequency.
    pub pitch: Sc2Variation,
    /// Yaw variation settings, including the function, amplitude, and frequency.
    pub yaw: Sc2Variation,
    /// Initial particle-speed control retained from the source emitter.
    pub speed: Sc2Variation,
    /// Size variation settings, including the function, amplitude, and frequency.
    pub size: Sc2Variation,
    /// Opacity control for the emitted geometry.
    pub alpha: Sc2Variation,
    /// Color variation settings, including the function, amplitude, and frequency.
    pub color: Sc2Variation,
    /// Rotation variation settings, including the function, amplitude, and frequency.
    pub rotation: Sc2Variation,
    /// Horizontal variation settings, including the function, amplitude, and frequency.
    pub horizontal: Sc2Variation,
    /// Vertical variation settings, including the function, amplitude, and frequency.
    pub vertical: Sc2Variation,
    /// Animated particle-velocity control retained in the StarCraft II representation.
    pub particle_velocity: Sc2Property<f32>,
    /// Animated phase-shift control; its effect-specific interpretation is unresolved.
    pub phase_shift: Sc2Property<f32>,
    /// Stored state bits; unknown bits are retained.
    pub flags: u32,
    /// Rotation behavior flags of the StarCraft II emitter.
    pub rotation_flags: u32,
    /// Interpolation mode for the particle color controls.
    pub color_smoothing: Sc2Smoothing,
    /// Interpolation mode for the particle size controls.
    pub size_smoothing: Sc2Smoothing,
    /// Interpolation mode for the particle rotation controls.
    pub rotation_smoothing: Sc2Smoothing,
    /// Animated alpha cutoff used when drawing particles.
    pub alpha_threshold: Sc2Property<f32>,
    /// Animated offset of particle texture coordinates.
    pub uv_offset: Sc2Property<[f32; 2]>,
    /// Animated three-component texture-orientation property.
    pub uv_angle: Sc2Property<[f32; 3]>,
    /// Animated tiling factors for the two texture-coordinate axes.
    pub uv_tiling: Sc2Property<[f32; 2]>,
    /// Section reference containing spline points; the target revision is selected by its index entry.
    pub spline_points: Ref<targets::Sc2PropertyVector3>,
    /// Wind multiplier parameter stored as a floating-point value.
    pub wind_multiplier: f32,
    /// Stored level-of-detail reduction control.
    pub lod_reduce: u32,
    /// Stored level-of-detail cutoff control.
    pub lod_cut: u32,
    /// Animated lower-bound control; its source effect semantics are unresolved.
    pub lower_bound: Sc2Property<f32>,
    /// Animated upper-bound control; its source effect semantics are unresolved.
    pub upper_bound: Sc2Property<f32>,
    /// Node index of the particle emitter used to produce trails.
    pub trail_link: u32,
    /// Trail chance parameter stored as a floating-point value.
    pub trail_chance: f32,
    /// Animated emission-rate control for the linked trail emitter.
    pub trail_emission_rate: Sc2Property<f32>,
    /// Splat projection index selector; sentinel and unrecognized values are retained.
    pub splat_projection_index: i32,
    /// Splat chance parameter stored as a floating-point value.
    pub splat_chance: f32,
    /// Section reference containing models; the target revision is selected by its index entry.
    pub models: Ref<targets::AssetKey>,
    /// Spawn ribbon on bounce chance parameter stored as a floating-point value.
    pub spawn_ribbon_on_bounce_chance: f32,
    /// Node index of the ribbon emitter triggered by particle bounces.
    pub ribbon_link: u32,
    /// Noise smoothness parameter stored as a floating-point value.
    pub noise_smoothness: f32,
}

/// Node-anchored control point of a StarCraft II ribbon spline.
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
pub struct Sc2RibbonSplinePoint {
    /// Node index anchoring this ribbon spline point.
    pub node: u32,
    /// Emission-position offset at this spline point.
    pub emission_offset: [f32; 3],
    /// Emission direction vector at this spline point.
    pub emission_vector: [f32; 3],
    /// Animated velocity control at this spline point.
    pub velocity: Sc2Property<f32>,
    /// Unresolved reserved data retained in its original position and representation.
    pub reserved: u32,
    /// Animated velocity multiplier at the spline base.
    pub velocity_base_factor: Sc2Property<f32>,
    /// Animated velocity multiplier at the spline end.
    pub velocity_end_factor: Sc2Property<f32>,
    /// Yaw variation variation settings, including the function, amplitude, and frequency.
    pub yaw_variation: Sc2Variation,
    /// Pitch variation variation settings, including the function, amplitude, and frequency.
    pub pitch_variation: Sc2Variation,
    /// Velocity variation variation settings, including the function, amplitude, and frequency.
    pub velocity_variation: Sc2Variation,
    /// Animated yaw control at this spline point.
    pub yaw: Sc2Property<f32>,
    /// Animated pitch control at this spline point.
    pub pitch: Sc2Property<f32>,
    /// Emission vector norm factor parameter stored as a floating-point value.
    pub emission_vector_norm_factor: f32,
    /// Velocity norm factor parameter stored as a floating-point value.
    pub velocity_norm_factor: f32,
}
impl Record for Sc2RibbonSplinePoint {
    const TAG: [u8; 4] = *b"S2RS";
    const VERSION: u32 = 1;
}

/// StarCraft II ribbon controls and optional spline definition.
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
pub struct Sc2RibbonEmitterPayload {
    /// Index of the model material slot drawn by this emitter.
    pub material_slot: u32,
    /// Supplementary StarCraft II emitter flags; unknown bits remain available.
    pub additional_flags: u32,
    /// Initial particle-speed property, including its unanimated and reset values.
    pub initial_speed: Sc2Property<f32>,
    /// Randomized initial-speed property in the source emitter.
    pub initial_speed_random: Sc2Property<f32>,
    /// Initial yaw property for emitted particles.
    pub initial_yaw: Sc2Property<f32>,
    /// Initial pitch property for emitted particles.
    pub initial_pitch: Sc2Property<f32>,
    /// Initial horizontal emission property in the StarCraft II emitter frame.
    pub initial_horizontal: Sc2Property<f32>,
    /// Initial vertical emission property in the StarCraft II emitter frame.
    pub initial_vertical: Sc2Property<f32>,
    /// Particle lifetime control retained from the source emitter.
    pub lifetime: Sc2Property<f32>,
    /// Random contribution to the particle-lifetime property.
    pub lifetime_random: Sc2Property<f32>,
    /// Source ribbon kill-radius word; its integer encoding is retained without assigning distance units.
    pub kill_radius: u32,
    /// Gravity x parameter stored as a floating-point value.
    pub gravity_x: f32,
    /// Gravity y parameter stored as a floating-point value.
    pub gravity_y: f32,
    /// Gravity control retained from the source emitter.
    pub gravity: f32,
    /// Normalized particle age at the intermediate size control point.
    pub size_mid_time: f32,
    /// Normalized particle age at the intermediate color control point.
    pub color_mid_time: f32,
    /// Normalized particle age at the intermediate alpha control point.
    pub alpha_mid_time: f32,
    /// Normalized particle age at the intermediate rotation control point.
    pub rotation_mid_time: f32,
    /// Size mid hold time parameter; units follow the enclosing animation or effect.
    pub size_mid_hold_time: f32,
    /// Color mid hold time parameter; units follow the enclosing animation or effect.
    pub color_mid_hold_time: f32,
    /// Alpha mid hold time parameter; units follow the enclosing animation or effect.
    pub alpha_mid_hold_time: f32,
    /// Rotation mid hold time parameter; units follow the enclosing animation or effect.
    pub rotation_mid_hold_time: f32,
    /// Three-component particle-size property.
    pub size_animation: Sc2Property<[f32; 3]>,
    /// Three-component particle-rotation property.
    pub rotation_animation: Sc2Property<[f32; 3]>,
    /// Color property at the beginning of particle life.
    pub color_start: Sc2Property<[f32; 4]>,
    /// Color property at the intermediate particle-age point.
    pub color_mid: Sc2Property<[f32; 4]>,
    /// Color property at the end of particle life.
    pub color_end: Sc2Property<[f32; 4]>,
    /// Drag applied to particle motion.
    pub drag: f32,
    /// Mass parameter stored as a floating-point value.
    pub mass: f32,
    /// Mass random parameter stored as a floating-point value.
    pub mass_random: f32,
    /// Mass size multiplier parameter stored as a floating-point value.
    pub mass_size_multiplier: f32,
    /// Mask selecting local force channels.
    pub local_forces: u16,
    /// Mask selecting world force channels.
    pub world_forces: u16,
    /// Fallback mask for local force channels.
    pub local_forces_fallback: u16,
    /// Fallback mask for world force channels.
    pub world_forces_fallback: u16,
    /// World forces mass multiplier parameter stored as a floating-point value.
    pub world_forces_mass_multiplier: f32,
    /// Noise amplitude parameter stored as a floating-point value.
    pub noise_amplitude: f32,
    /// Noise frequency parameter stored as a floating-point value.
    pub noise_frequency: f32,
    /// Noise coherence parameter stored as a floating-point value.
    pub noise_coherence: f32,
    /// Noise edge parameter stored as a floating-point value.
    pub noise_edge: f32,
    /// Index plus length selector; sentinel and unrecognized values are retained.
    pub index_plus_length: u32,
    /// Shape from which this StarCraft II emitter produces particles.
    pub emitter_shape: u32,
    /// StarCraft II ribbon geometry mode.
    pub ribbon_type: Sc2RibbonType,
    /// Divisions parameter stored as a floating-point value.
    pub divisions: f32,
    /// Ribbon edge-count control.
    pub edges: u32,
    /// Inner radius control of the emitter’s source shape.
    pub inner_radius: f32,
    /// Animated maximum-length control for the ribbon.
    pub max_length: Sc2Property<f32>,
    /// Section reference containing spline points; the target revision is selected by its index entry.
    pub spline_points: Ref<targets::Sc2RibbonSplinePoint>,
    /// Animated ribbon activation control.
    pub active: Sc2Property<u32>,
    /// Stored state bits; unknown bits are retained.
    pub flags: u32,
    /// Interpolation mode for the particle size controls.
    pub size_smoothing: Sc2Smoothing,
    /// Interpolation mode for the particle color controls.
    pub color_smoothing: Sc2Smoothing,
    /// Particle collision friction parameter.
    pub friction: f32,
    /// Particle collision bounce parameter.
    pub bounce: f32,
    /// Stored level-of-detail reduction control.
    pub lod_reduce: u32,
    /// Stored level-of-detail cutoff control.
    pub lod_cut: u32,
    /// Yaw variation settings, including the function, amplitude, and frequency.
    pub yaw: Sc2Variation,
    /// Pitch variation settings, including the function, amplitude, and frequency.
    pub pitch: Sc2Variation,
    /// Initial particle-speed control retained from the source emitter.
    pub speed: Sc2Variation,
    /// Size variation settings, including the function, amplitude, and frequency.
    pub size: Sc2Variation,
    /// Opacity control for the emitted geometry.
    pub alpha: Sc2Variation,
    /// Animated particle-velocity control retained in the StarCraft II representation.
    pub particle_velocity: Sc2Property<f32>,
    /// Animated ribbon overlay control; its detailed source semantics are unresolved.
    pub overlay: Sc2Property<f32>,
    /// Unresolved deprecated unknown data retained in its original position and representation.
    pub deprecated_unknown: i32,
}

/// Authoring classification and motion-following relationships for one rig node.
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
pub struct NodeRig {
    /// Functional role assigned to the node in the authoring rig.
    pub role: RigRole,
    /// Side of the rig to which the node belongs.
    pub side: RigSide,
    /// Limb category used by the rig’s joint controls.
    pub limb: RigLimb,
    /// Node index whose motion this node follows outside its parent chain; the invalid-node sentinel means none.
    pub rides_with: u32,
    /// Policy governing whether a limb end holds its position during rig manipulation.
    pub plant: RigPlant,
    /// Fraction of the limb end’s twist assigned to this twist bone.
    pub twist_share: f32,
    /// Origin of the detected or edited role, side, and limb classifications.
    pub source: RigSource,
    /// Origin of the rides-with assignment, tracked separately from the node classification.
    pub rides_source: RigSource,
}

/// Typed helper payload payload in its serialized field order.
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
pub struct HelperPayload {}

impl Record for Sc2Property<[f32; 3]> {
    const TAG: [u8; 4] = *b"S2P3";
    const VERSION: u32 = 1;
}
