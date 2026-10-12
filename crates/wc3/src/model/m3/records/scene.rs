//! Native scene record revisions.
use super::dialect::*;
use super::{targets, AnimationValue};
use crate::model::m3::{self, Record, Ref};
use core::fmt::Debug;
/// Native field selections for independent `Event` section revisions.
pub trait EventDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `rtt_channel_index`; `()` omits the field.
    type RttChannelIndex: NativeField;
    /// Field representation for `extra_parameter`; `()` omits the field.
    type ExtraParameter: NativeField;
}

impl EventDialect for V0 {
    const VERSION: u32 = 0;
    type RttChannelIndex = ();
    type ExtraParameter = ();
}

impl EventDialect for V1 {
    const VERSION: u32 = 1;
    type RttChannelIndex = u32;
    type ExtraParameter = ();
}

impl EventDialect for V2 {
    const VERSION: u32 = 2;
    type RttChannelIndex = u32;
    type ExtraParameter = u32;
}

/// Fields of the native `EVNT` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct Event<D: EventDialect> {
    /// Links to CHAR bytes naming this event.
    pub name: Ref<targets::Character>,
    /// The purpose of this field has not been established.
    pub unknown: u32,
    /// Selects the associated bone in the BONE section.
    pub bone_index: u16,
    /// Keeps alignment bytes in their serialized position.
    pub padding: u16,
    /// Stores the four-by-four transformation associated with this event.
    pub transform: [[f32; 4]; 4],
    /// Selects the event operation using the client’s event-code namespace.
    pub event_type: u32,
    /// Links to optional event parameters encoded as CHAR bytes.
    pub option_string: Ref<targets::Character>,
    /// Selects the render-to-texture channel used by this event.
    pub rtt_channel_index: D::RttChannelIndex,
    /// Sets the extra parameter. Available from revision 2.
    pub extra_parameter: D::ExtraParameter,
}

impl<D: EventDialect> Record for Event<D> {
    const TAG: [u8; 4] = *b"EVNT";
    const VERSION: u32 = D::VERSION;
}

/// Native field selections for independent `Projector` section revisions.
pub trait ProjectorDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `projection_type`; `()` omits the field.
    type ProjectionType: NativeField;
}

impl ProjectorDialect for V4 {
    const VERSION: u32 = 4;
    type ProjectionType = u32;
}

impl ProjectorDialect for V5 {
    const VERSION: u32 = 5;
    type ProjectionType = u32;
}

/// Fields of the native `PROJ` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct Projector<D: ProjectorDialect> {
    /// Selects orthographic or perspective projection.
    pub projection_type: D::ProjectionType,
    /// Selects the associated bone in the BONE section.
    pub bone: u32,
    /// Selects the material-map entry in the MATM section.
    pub material_reference_index: u32,
    /// Animates the position offset.
    pub offset: AnimationValue<[f32; 3]>,
    /// Animates the pitch angle.
    pub pitch: AnimationValue<f32>,
    /// Animates the yaw angle.
    pub yaw: AnimationValue<f32>,
    /// Animates the roll angle.
    pub roll: AnimationValue<f32>,
    /// Animates the field of view.
    pub field_of_view: AnimationValue<f32>,
    /// Animates the aspect ratio.
    pub aspect_ratio: AnimationValue<f32>,
    /// Animates the near clip plane.
    pub near: AnimationValue<f32>,
    /// Animates the far clip plane.
    pub far: AnimationValue<f32>,
    /// Animates the box Z bottom offset.
    pub box_offset_z_bottom: AnimationValue<f32>,
    /// Animates the box Z top offset.
    pub box_offset_z_top: AnimationValue<f32>,
    /// Animates the box X left offset.
    pub box_offset_x_left: AnimationValue<f32>,
    /// Animates the box X right offset.
    pub box_offset_x_right: AnimationValue<f32>,
    /// Animates the box Y front offset.
    pub box_offset_y_front: AnimationValue<f32>,
    /// Animates the box Y back offset.
    pub box_offset_y_back: AnimationValue<f32>,
    /// Sets the projection falloff distance.
    pub falloff: f32,
    /// Sets opacity at the envelope’s creation.
    pub alpha_init: f32,
    /// Sets opacity at the envelope’s midpoint.
    pub alpha_mid: f32,
    /// Sets opacity at the envelope’s end.
    pub alpha_end: f32,
    /// Sets the phase duration for the attack portion of the projector’s envelope.
    pub lifetime_attack: f32,
    /// Sets the target time for the attack portion of the projector’s envelope.
    pub lifetime_attack_to: f32,
    /// Sets the phase duration for the hold portion of the projector’s envelope.
    pub lifetime_hold: f32,
    /// Sets the target time for the hold portion of the projector’s envelope.
    pub lifetime_hold_to: f32,
    /// Sets the phase duration for the decay portion of the projector’s envelope.
    pub lifetime_decay: f32,
    /// Sets the target time for the decay portion of the projector’s envelope.
    pub lifetime_decay_to: f32,
    /// Sets the distance-based attenuation.
    pub attenuation_distance: f32,
    /// Animates the active state.
    pub active: AnimationValue<u32>,
    /// Sets the render layer.
    pub layer: u32,
    /// Selects the detail level at which projector detail is reduced.
    pub lod_reduce: u32,
    /// Selects the detail level beyond which this effect is omitted.
    pub lod_cut: u32,
    /// Stores the bit mask controlling projector behavior.
    pub flags: u32,
}

impl<D: ProjectorDialect> Record for Projector<D> {
    const TAG: [u8; 4] = *b"PROJ";
    const VERSION: u32 = D::VERSION;
}

/// Native field selections for independent `HitTestShape` section revisions.
pub trait HitTestShapeDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `shape_type`; `()` omits the field.
    type ShapeType: NativeField;
}

impl HitTestShapeDialect for V0 {
    const VERSION: u32 = 0;
    type ShapeType = u32;
}

impl HitTestShapeDialect for V1 {
    const VERSION: u32 = 1;
    type ShapeType = u32;
}

/// Fields of the native `SSGS` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct HitTestShape<D: HitTestShapeDialect> {
    /// The role of the shape type storage is unresolved; its encoded representation is preserved.
    pub shape_type: D::ShapeType,
    /// Selects the associated bone in the BONE section.
    pub bone_index: u16,
    /// Keeps alignment bytes in their serialized position.
    pub padding: u16,
    /// Transforms the shape using a four-by-four matrix.
    pub transform: [[f32; 4]; 4],
    /// Links to VEC3 vertex positions when the selected shape is a mesh.
    pub vertex_positions: Ref<targets::Vector3>,
    /// Links to U16_ triangle indices when the selected shape is a mesh.
    pub face_indices: Ref<targets::U16>,
    /// Sets the shape’s X extent, interpreted as radius for spheres and capsules.
    pub size_x: f32,
    /// Sets the shape’s Y extent, interpreted as height for capsules and cylinders.
    pub size_y: f32,
    /// Sets the z dimension.
    pub size_z: f32,
}

impl<D: HitTestShapeDialect> Record for HitTestShape<D> {
    const TAG: [u8; 4] = *b"SSGS";
    const VERSION: u32 = D::VERSION;
}

/// Fields of the native `ATVL` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct AttachmentVolume {
    /// Selects the first bone in the BONE section.
    pub bone1: u32,
    /// Selects the second bone in the BONE section.
    pub bone2: u32,
    /// Sets the shape type.
    pub shape_type: u32,
    /// Selects the primary bone in the BONE section.
    pub bone_index: u16,
    /// Keeps alignment bytes in their serialized position.
    pub padding: u16,
    /// Transforms the volume using a four-by-four matrix.
    pub transform: [[f32; 4]; 4],
    /// Links to VEC3 positions defining the shape’s mesh vertices.
    pub vertex_positions: Ref<targets::Vector3>,
    /// Links to U16_ vertex indices forming the shape’s mesh triangles.
    pub face_indices: Ref<targets::U16>,
    /// Sets the x dimension.
    pub size_x: f32,
    /// Sets the y dimension.
    pub size_y: f32,
    /// Sets the z dimension.
    pub size_z: f32,
}

impl Record for AttachmentVolume {
    const TAG: [u8; 4] = *b"ATVL";
    const VERSION: u32 = 0;
}

/// Fields of the native `TRGD` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct TriggerData {
    /// Links to U32_ indices identifying the associated data entries.
    pub data_indices: Ref<targets::U32>,
    /// Links to CHAR bytes naming this trigger.
    pub name: Ref<targets::Character>,
}

impl Record for TriggerData {
    const TAG: [u8; 4] = *b"TRGD";
    const VERSION: u32 = 0;
}

/// Native field selections for independent `TurretBehavior` section revisions.
pub trait TurretBehaviorDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `unknown1`; `()` omits the field.
    type Unknown1: NativeField;
    /// Field representation for `unknown2`; `()` omits the field.
    type Unknown2: NativeField;
    /// Field representation for `yaw_weight`; `()` omits the field.
    type YawWeight: NativeField;
    /// Field representation for `pitch_weight`; `()` omits the field.
    type PitchWeight: NativeField;
}

impl TurretBehaviorDialect for V0 {
    const VERSION: u32 = 0;
    type Unknown1 = ();
    type Unknown2 = ();
    type YawWeight = ();
    type PitchWeight = ();
}

impl TurretBehaviorDialect for V1 {
    const VERSION: u32 = 1;
    type Unknown1 = ();
    type Unknown2 = ();
    type YawWeight = ();
    type PitchWeight = ();
}

impl TurretBehaviorDialect for V2 {
    const VERSION: u32 = 2;
    type Unknown1 = ();
    type Unknown2 = ();
    type YawWeight = ();
    type PitchWeight = ();
}

impl TurretBehaviorDialect for V3 {
    const VERSION: u32 = 3;
    type Unknown1 = ();
    type Unknown2 = ();
    type YawWeight = ();
    type PitchWeight = ();
}

impl TurretBehaviorDialect for V4 {
    const VERSION: u32 = 4;
    type Unknown1 = [f32; 4];
    type Unknown2 = [f32; 4];
    type YawWeight = f32;
    type PitchWeight = f32;
}

/// Fields of the native `PATU` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct TurretBehavior<D: TurretBehaviorDialect> {
    /// Transforms the turret using a four-by-four matrix.
    pub transform: [[f32; 4]; 4],
    /// Stores an uninterpreted vector whose role is not established.
    pub unknown1: D::Unknown1,
    /// Stores a second uninterpreted vector whose role is not established.
    pub unknown2: D::Unknown2,
    /// Selects the associated bone in the BONE section.
    pub bone_index: u16,
    /// Marks the turret as the primary turret when nonzero.
    pub use_as_main_turret: u8,
    /// Sets the turret group identifier.
    pub turret_group_id: u8,
    /// Enables enforcement of yaw limits.
    pub yaw_limited: u32,
    /// Limits yaw rotation to its minimum angle, measured in radians.
    pub yaw_min: f32,
    /// Limits yaw rotation to its maximum angle, measured in radians.
    pub yaw_max: f32,
    /// Sets the yaw rotation weight.
    pub yaw_weight: D::YawWeight,
    /// Enables enforcement of pitch limits.
    pub pitch_limited: u32,
    /// Limits pitch rotation to its minimum angle, measured in radians.
    pub pitch_min: f32,
    /// Limits pitch rotation to its maximum angle, measured in radians.
    pub pitch_max: f32,
    /// Sets the pitch rotation weight.
    pub pitch_weight: D::PitchWeight,
    /// The purpose of this field has not been established.
    pub unknown3: f32,
    /// The purpose of this field has not been established.
    pub unknown4: f32,
    /// Sets the offset from main bone.
    pub main_bone_offset: [f32; 3],
}

impl<D: TurretBehaviorDialect> Record for TurretBehavior<D> {
    const TAG: [u8; 4] = *b"PATU";
    const VERSION: u32 = D::VERSION;
}

/// Fields of the native `BBSC` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct BillboardBehavior {
    /// Links to dependent U16_ bone indices; reference samples leave this list empty and the traced client does not consume it.
    pub dependents: Ref<targets::U16>,
    /// Selects the associated bone in the BONE section.
    pub bone_index: u16,
    /// Selects the billboard mode and its allowed rotational axes.
    pub billboard_type: u8,
    /// Nonzero aims the bone toward the camera position; zero aligns it with the camera’s viewing direction.
    pub camera_look_at: u8,
    /// Supplies a quaternion pre-rotation for axis-constrained billboard modes 0, 1, and 2; this is not an up-direction vector.
    pub up: [f32; 4],
    /// Supplies a quaternion pre-rotation for billboard mode 6 when the bone has a bone parent; modes 3, 4, and 5 ignore it.
    pub forward: [f32; 4],
}

impl Record for BillboardBehavior {
    const TAG: [u8; 4] = *b"BBSC";
    const VERSION: u32 = 0;
}

/// Fields of the native `IKJT` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct IKJoint {
    /// Links to the U16_ indices of bones depending on this behavior.
    pub dependents: Ref<targets::U16>,
    /// Selects the first bone in the BONE section.
    pub bone_index1: u16,
    /// Selects the second bone in the BONE section.
    pub bone_index2: u16,
    /// Sets the positive upper raycast offset; reference samples use 1.5 or 3.0.
    pub raycast_up: f32,
    /// Sets the signed lower raycast offset; the search interval extends from z plus this offset to z plus `raycast_up`.
    pub raycast_down: f32,
    /// Sets the maximum IK solving speed.
    pub max_speed: f32,
    /// Sets the goal distance threshold.
    pub goal_threshold: f32,
}

impl Record for IKJoint {
    const TAG: [u8; 4] = *b"IKJT";
    const VERSION: u32 = 0;
}

/// Fields of the native `IK2J` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct IKTwoJoint {
    /// Links to the U16_ indices of bones depending on this behavior.
    pub dependents: Ref<targets::U16>,
    /// Selects the base of the inverse-kinematics chain, such as the upper arm or thigh.
    pub bone_base: u16,
    /// Selects the intermediate inverse-kinematics bone, such as the forearm or shin.
    pub bone_target: u16,
    /// Selects the inverse-kinematics endpoint, such as the hand or foot.
    pub bone_end: u16,
    /// Keeps alignment bytes in their serialized position.
    pub padding: u16,
    /// Sets the hinge rotation axis.
    pub hinge_axis: [f32; 3],
    /// Sets the maximum inner angle.
    pub max_angle_inner: f32,
    /// Sets the maximum outer angle.
    pub max_angle_outer: f32,
    /// Sets the search range upward.
    pub search_up: f32,
    /// Sets the search range downward.
    pub search_down: f32,
}

impl Record for IKTwoJoint {
    const TAG: [u8; 4] = *b"IK2J";
    const VERSION: u32 = 0;
}

/// Fields of the native `IKCC` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct IKCCD {
    /// Links to the U16_ indices of bones depending on this behavior.
    pub dependents: Ref<targets::U16>,
    /// Selects the base bone in the BONE section.
    pub bone_base: u16,
    /// Selects the target bone in the BONE section.
    pub bone_target: u16,
    /// Sets the search range upward.
    pub search_up: f32,
    /// Sets the search range downward.
    pub search_down: f32,
}

impl Record for IKCCD {
    const TAG: [u8; 4] = *b"IKCC";
    const VERSION: u32 = 0;
}

/// Fields of the native `PAOB` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct OneBoneSolver {
    /// Links to the U16_ indices of bones depending on this behavior.
    pub dependents: Ref<targets::U16>,
    /// Selects the primary bone in the BONE section.
    pub bone: u16,
    /// Selects the fallback bone in the BONE section.
    pub bone_fallback: u16,
    /// Stores the bit mask controlling solver behavior.
    pub flags: u32,
    /// Sets the maximum rotation angle.
    pub max_angle: f32,
}

impl Record for OneBoneSolver {
    const TAG: [u8; 4] = *b"PAOB";
    const VERSION: u32 = 0;
}

/// Fields of the native `VVOL` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct ViewVolume {
    /// Selects the associated bone in the BONE section.
    pub node_index: u32,
    /// Animates the half-size of the influence volume along each axis.
    pub size: AnimationValue<[f32; 3]>,
}

impl Record for ViewVolume {
    const TAG: [u8; 4] = *b"VVOL";
    const VERSION: u32 = 0;
}

/// Native field selections for independent `TrailingModel` section revisions.
pub trait TrailingModelDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `vectors`; `()` omits the field.
    type Vectors: NativeField;
}

impl TrailingModelDialect for V0 {
    const VERSION: u32 = 0;
    type Vectors = Ref<targets::Vector3>;
}

impl TrailingModelDialect for V1 {
    const VERSION: u32 = 1;
    type Vectors = Ref<targets::Vector3>;
}

/// Fields of the native `TMD_` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct TrailingModel<D: TrailingModelDialect> {
    /// Links to VEC3 vectors controlling the shape.
    pub vectors: D::Vectors,
    /// Unknown-purpose first parameter reported as 5.0 in the reference samples.
    pub param0: f32,
    /// Unknown-purpose second parameter reported as 1.0 in the reference samples.
    pub param1: f32,
    /// Animates an unknown-purpose scalar with a reported initial value of 0.5.
    pub anim_float0: AnimationValue<f32>,
    /// Animates another unknown-purpose scalar with a reported initial value of 1.0.
    pub anim_float1: AnimationValue<f32>,
    /// Unknown-purpose flag reported as one in the reference samples.
    pub flag: u32,
    /// Keeps reserved storage without assigning a meaning to its bits.
    pub reserved0: u32,
    /// Keeps reserved storage without assigning a meaning to its bits.
    pub reserved1: u32,
}

impl<D: TrailingModelDialect> Record for TrailingModel<D> {
    const TAG: [u8; 4] = *b"TMD_";
    const VERSION: u32 = D::VERSION;
}

/// Native field selections for independent `Force` section revisions.
pub trait ForceDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `force_type`; `()` omits the field.
    type ForceType: NativeField;
}

impl ForceDialect for V0 {
    const VERSION: u32 = 0;
    type ForceType = u32;
}

impl ForceDialect for V1 {
    const VERSION: u32 = 1;
    type ForceType = u32;
}

impl ForceDialect for V2 {
    const VERSION: u32 = 2;
    type ForceType = u32;
}

/// Fields of the native `FOR_` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct Force<D: ForceDialect> {
    /// Sets the force kind.
    pub force_type: D::ForceType,
    /// Sets the influence volume shape.
    pub force_shape: u32,
    /// Stores a possible local-versus-world scope selector; this interpretation has not been confirmed by a traced reader.
    pub unknown: u32,
    /// Selects the associated bone in the BONE section.
    pub bone_index: u32,
    /// Encodes force options for attenuation, vertical gradient, unbounded influence, and affected targets.
    pub flags: u32,
    /// Selects force channels; a matching bit connects this force with a body or emitter.
    pub local_channels: u32,
    /// Animates the force strength.
    pub strength: AnimationValue<f32>,
    /// Animates the influence width.
    pub width: AnimationValue<f32>,
    /// Animates the influence height.
    pub height: AnimationValue<f32>,
    /// Animates the influence length.
    pub length: AnimationValue<f32>,
}

impl<D: ForceDialect> Record for Force<D> {
    const TAG: [u8; 4] = *b"FOR_";
    const VERSION: u32 = D::VERSION;
}

/// Fields of the native `WRP_` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct Warp {
    /// Sets the warp type.
    pub warp_type: u32,
    /// Selects the associated bone in the BONE section.
    pub bone_index: u32,
    /// The purpose of this field has not been established.
    pub unknown: u32,
    /// Animates the warp radius.
    pub radius: AnimationValue<f32>,
    /// Animates the warp height.
    pub height: AnimationValue<f32>,
    /// Animates the warp strength.
    pub strength: AnimationValue<f32>,
    /// Animates the angular component.
    pub angular: AnimationValue<f32>,
    /// Animates the axial component.
    pub axial: AnimationValue<f32>,
    /// Animates the radial component.
    pub radial: AnimationValue<f32>,
}

impl Record for Warp {
    const TAG: [u8; 4] = *b"WRP_";
    const VERSION: u32 = 1;
}

/// Native field selections for independent `Light` section revisions.
pub trait LightDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `light_type`; `()` omits the field.
    type LightType: NativeField;
}

impl LightDialect for V0 {
    const VERSION: u32 = 0;
    type LightType = u16;
}

impl LightDialect for V1 {
    const VERSION: u32 = 1;
    type LightType = u16;
}

impl LightDialect for V2 {
    const VERSION: u32 = 2;
    type LightType = u16;
}

impl LightDialect for V3 {
    const VERSION: u32 = 3;
    type LightType = u16;
}

impl LightDialect for V4 {
    const VERSION: u32 = 4;
    type LightType = u16;
}

impl LightDialect for V5 {
    const VERSION: u32 = 5;
    type LightType = u16;
}

impl LightDialect for V6 {
    const VERSION: u32 = 6;
    type LightType = u16;
}

impl LightDialect for V7 {
    const VERSION: u32 = 7;
    type LightType = u16;
}

/// Fields of the native `LITE` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct Light<D: LightDialect> {
    /// Selects omnidirectional, spot, or directional illumination.
    pub light_type: D::LightType,
    /// Selects the associated bone in the BONE section.
    pub bone_index: u16,
    /// Controls lighting features, including shadowing, specular response, and ambient occlusion, through individual bits.
    pub flags: u32,
    /// Selects the detail level beyond which this effect is omitted.
    pub lod_cut: u32,
    /// Sets the shadow LOD cut-off level.
    pub shadow_lod_cut: u32,
    /// Animates the diffuse color using red, green, and blue components.
    pub diffuse_color: AnimationValue<[f32; 3]>,
    /// Animates the intensity multiplier.
    pub intensity_multiplier: AnimationValue<f32>,
    /// Animates the specular color using red, green, and blue components.
    pub specular_color: AnimationValue<[f32; 3]>,
    /// Animates the specular multiplier.
    pub specular_multiplier: AnimationValue<f32>,
    /// Animates the distance decay exponent.
    pub decay: AnimationValue<f32>,
    /// Sets the attenuation end distance.
    pub attenuation_end: f32,
    /// Animates the attenuation start distance.
    pub attenuation_start: AnimationValue<f32>,
    /// Animates the spot inner cone angle.
    pub hot_spot: AnimationValue<f32>,
    /// Animates the spot outer cone falloff.
    pub falloff: AnimationValue<f32>,
}

impl<D: LightDialect> Record for Light<D> {
    const TAG: [u8; 4] = *b"LITE";
    const VERSION: u32 = D::VERSION;
}

/// Native field selections for independent `Camera` section revisions.
pub trait CameraDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `reserved_0`; `()` omits the field.
    type Reserved0: NativeField;
    /// Field representation for `dof_type`; `()` omits the field.
    type DofType: NativeField;
    /// Field representation for `reserved_1`; `()` omits the field.
    type Reserved1: NativeField;
    /// Field representation for `far_clip`; `()` omits the field.
    type FarClip: NativeField;
    /// Field representation for `near_clip`; `()` omits the field.
    type NearClip: NativeField;
    /// Field representation for `near_falloff_start`; `()` omits the field.
    type NearFalloffStart: NativeField;
    /// Field representation for `near_falloff_end`; `()` omits the field.
    type NearFalloffEnd: NativeField;
    /// Field representation for `bokeh_f_stop`; `()` omits the field.
    type BokehFStop: NativeField;
    /// Field representation for `bokeh_max_co_c_diameter`; `()` omits the field.
    type BokehMaxCoCDiameter: NativeField;
}

impl CameraDialect for V2 {
    const VERSION: u32 = 2;
    type Reserved0 = f32;
    type DofType = ();
    type Reserved1 = f32;
    type FarClip = ();
    type NearClip = ();
    type NearFalloffStart = ();
    type NearFalloffEnd = ();
    type BokehFStop = ();
    type BokehMaxCoCDiameter = ();
}

impl CameraDialect for V3 {
    const VERSION: u32 = 3;
    type Reserved0 = ();
    type DofType = ();
    type Reserved1 = ();
    type FarClip = AnimationValue<f32>;
    type NearClip = AnimationValue<f32>;
    type NearFalloffStart = ();
    type NearFalloffEnd = ();
    type BokehFStop = ();
    type BokehMaxCoCDiameter = ();
}

impl CameraDialect for V4 {
    const VERSION: u32 = 4;
    type Reserved0 = ();
    type DofType = ();
    type Reserved1 = ();
    type FarClip = AnimationValue<f32>;
    type NearClip = AnimationValue<f32>;
    type NearFalloffStart = AnimationValue<f32>;
    type NearFalloffEnd = AnimationValue<f32>;
    type BokehFStop = ();
    type BokehMaxCoCDiameter = ();
}

impl CameraDialect for V5 {
    const VERSION: u32 = 5;
    type Reserved0 = ();
    type DofType = u32;
    type Reserved1 = ();
    type FarClip = AnimationValue<f32>;
    type NearClip = AnimationValue<f32>;
    type NearFalloffStart = AnimationValue<f32>;
    type NearFalloffEnd = AnimationValue<f32>;
    type BokehFStop = AnimationValue<f32>;
    type BokehMaxCoCDiameter = AnimationValue<f32>;
}

/// Fields of the native `CAM_` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct Camera<D: CameraDialect> {
    /// Selects the associated bone in the BONE section.
    pub bone_index: u32,
    /// Links to CHAR bytes naming this camera.
    pub name: Ref<targets::Character>,
    /// Animates the camera’s field-of-view angle in radians. Available from revision 2.
    pub field_of_view: AnimationValue<f32>,
    /// Selects vertical rather than horizontal field-of-view measurement; revisions 2 onward store a zero-or-one value.
    pub use_vertical_f_o_v: u32,
    /// The role of the reserved 0 storage is unresolved; its encoded representation is preserved.
    pub reserved_0: D::Reserved0,
    /// Selects the depth-of-field mode in revision 5; the reported default is 3.
    pub dof_type: D::DofType,
    /// The role of the reserved 1 storage is unresolved; its encoded representation is preserved.
    pub reserved_1: D::Reserved1,
    /// Animates the far clip plane. Available from revision 3.
    pub far_clip: D::FarClip,
    /// Animates the near clip plane. Available from revision 3.
    pub near_clip: D::NearClip,
    /// Animates the shadow clip distance. Available from revision 2.
    pub shadow_clip_distance: AnimationValue<f32>,
    /// Animates the depth-of-field focal point distance. Available from revision 2.
    pub focus_distance: AnimationValue<f32>,
    /// Animates the depth-of-field far focus range. Available from revision 2.
    pub far_focus_range: AnimationValue<f32>,
    /// Animates the depth-of-field near focus range. Available from revision 2.
    pub near_focus_range: AnimationValue<f32>,
    /// Animates the near falloff start. Available from revision 4.
    pub near_falloff_start: D::NearFalloffStart,
    /// Animates the near falloff end. Available from revision 4.
    pub near_falloff_end: D::NearFalloffEnd,
    /// Animates the depth-of-field strength. Available from revision 2.
    pub dof_amount: AnimationValue<f32>,
    /// Animates the bokeh f-stop. Available from revision 5.
    pub bokeh_f_stop: D::BokehFStop,
    /// Animates the maximum bokeh circle-of-confusion diameter in revision 5 onward.
    pub bokeh_max_co_c_diameter: D::BokehMaxCoCDiameter,
}

impl<D: CameraDialect> Record for Camera<D> {
    const TAG: [u8; 4] = *b"CAM_";
    const VERSION: u32 = D::VERSION;
}
