//! Typed animation records for indexed WEM payloads and inline values.
use super::*;
use crate::model::wem::{self, Bool, InlineOption, Record, Ref};
/// Typed material channel ref payload in its serialized field order.
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
pub struct MaterialChannelRef {
    /// Source profile whose material set contains the target.
    pub profile: ProfileId,
    /// Index into the model’s shared material-slot list.
    pub slot: u32,
    /// Look index within the selected profile material set.
    pub look: u32,
}

/// Typed track target payload in its serialized field order.
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
pub struct TrackTarget {
    /// Domain containing the animated property, such as a node, material, mesh, or attribute.
    pub kind: TrackTargetKind,
    /// Model node index when the target domain is a node.
    pub node: u32,
    /// Model mesh index when the property belongs to mesh geometry.
    pub mesh: u32,
    /// Profile, material slot, and look identifying a material target.
    pub material: MaterialChannelRef,
    /// Sub-element index, such as a material layer; interpretation depends on kind and channel.
    pub sub: u32,
    /// Property identifier within the selected target domain.
    pub channel: Channel,
}

/// Typed anim channel payload in its serialized field order.
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
pub struct AnimChannel {
    /// Stable channel identity referenced by SubTrack.channel.
    pub id: u32,
    /// Domain and indices identifying the property driven by this channel.
    pub target: TrackTarget,
    /// Element representation used by the initial value and animation keys.
    pub value_type: AttrType,
    /// Byte-section reference containing the initial value in value_type’s encoding.
    pub init_value: Ref<targets::Byte>,
}
impl Record for AnimChannel {
    const TAG: [u8; 4] = *b"ACHN";
    const VERSION: u32 = 1;
}

/// Typed anim channel table payload in its serialized field order.
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
pub struct AnimChannelTable {
    /// ACHN section containing the model’s animation-channel declarations.
    pub channels: Ref<targets::AnimChannel>,
}

/// Typed track set payload in its serialized field order.
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
pub struct TrackSet {
    /// Character-section reference naming this selection of channels.
    pub name: Ref<targets::Character>,
    /// U32_ section listing the channel identities included in this set.
    pub channels: Ref<targets::U32>,
}
impl Record for TrackSet {
    const TAG: [u8; 4] = *b"TSET";
    const VERSION: u32 = 1;
}

/// Static field selections for `SubTrack` section or inline context revisions.
pub trait SubTrackDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `tcb`; `()` omits this slot.
    type Tcb: NativeField;
}

impl SubTrackDialect for V1 {
    const VERSION: u32 = 1;
    type Tcb = ();
}

impl SubTrackDialect for V2 {
    const VERSION: u32 = 2;
    type Tcb = Ref<targets::Real>;
}

/// Typed sub track payload in its serialized field order.
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
pub struct SubTrack<D: SubTrackDialect> {
    /// Identity of the AnimChannel driven by these keys.
    pub channel: u32,
    /// Interpolation convention determining the number of values stored for each key.
    pub interp: Interpolation,
    /// REAL section containing key times in seconds from the clip start.
    pub times: Ref<targets::Real>,
    /// Byte-section reference containing packed key values and any interpolation control values.
    pub values: Ref<targets::Byte>,
    /// REAL section holding tension/continuity/bias parameters in revisions that serialize them.
    pub tcb: D::Tcb,
}
impl<D: SubTrackDialect> Record for SubTrack<D> {
    const TAG: [u8; 4] = *b"STRK";
    const VERSION: u32 = D::VERSION;
}

/// Typed sub track container payload in its serialized field order.
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
pub struct SubTrackContainer {
    /// Character-section reference naming this animation layer.
    pub name: Ref<targets::Character>,
    /// Ordering priority used to combine this layer with other containers.
    pub priority: i32,
    /// Allows this container to run concurrently with another layer.
    pub concurrent: Bool,
    /// STRK section containing the channel-specific key arrays.
    pub sub_tracks: Ref<targets::SubTrack>,
    /// Name/value metadata retaining source-format container properties.
    pub native: NativeBag,
}
impl Record for SubTrackContainer {
    const TAG: [u8; 4] = *b"STCC";
    const VERSION: u32 = 1;
}

/// Typed clip event payload in its serialized field order.
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
pub struct ClipEvent {
    /// Time in seconds from the beginning of the clip.
    pub time: f32,
    /// Model node index at which the event occurs; this need not be an Event-kind node.
    pub node: u32,
    /// Character-section reference naming the event.
    pub name: Ref<targets::Character>,
    /// Source-specific event payload word.
    pub value: u32,
}
impl Record for ClipEvent {
    const TAG: [u8; 4] = *b"CLEV";
    const VERSION: u32 = 1;
}

/// Typed clip track set payload in its serialized field order.
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
pub struct ClipTrackSet {
    /// Index of a model TrackSet selected by the clip.
    pub set: u32,
    /// Priority assigned to this track-set selection within the clip.
    pub priority: i32,
}
impl Record for ClipTrackSet {
    const TAG: [u8; 4] = *b"CTSU";
    const VERSION: u32 = 1;
}

/// Typed bake world force payload in its serialized field order.
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
pub struct BakeWorldForce {
    /// Selects a wind or blast field.
    pub kind: BakeWorldForceKind,
    /// Beginning of the force interval, relative to the clip.
    pub start: f32,
    /// End of the force interval, relative to the clip.
    pub end: f32,
    /// Width of the force’s fade-in interval.
    pub ramp_in: f32,
    /// Width of the force’s fade-out interval.
    pub ramp_out: f32,
    /// Overall strength of the recorded world force.
    pub strength: f32,
    /// Wind direction in degrees around +Z, measured from +X.
    pub heading: f32,
    /// Wind elevation in degrees above the horizontal plane.
    pub rise: f32,
    /// Fraction of the wind strength assigned to gust variation.
    pub gusts: f32,
    /// Number of wind gusts per second.
    pub gust_speed: f32,
    /// Blast radius measured in model heights.
    pub radius: f32,
    /// Blast elevation above the model’s feet, measured in model heights.
    pub height: f32,
    /// Model node used as the centre of the force, when selected.
    pub centre_node: u32,
    /// Bit mask selecting the bodies affected by this force.
    pub channels: u32,
}
impl Record for BakeWorldForce {
    const TAG: [u8; 4] = *b"CPWF";
    const VERSION: u32 = 1;
}

/// Static field selections for `PhysicsBake` section or inline context revisions.
pub trait PhysicsBakeDialect: Revision {
    /// Representation of `cloth_ids`; `()` omits this slot.
    type ClothIds: NativeField;
    /// Representation of `cloth_fits`; `()` omits this slot.
    type ClothFits: NativeField;
    /// Representation of `cloth_sizes`; `()` omits this slot.
    type ClothSizes: NativeField;
    /// Representation of `soft_ids`; `()` omits this slot.
    type SoftIds: NativeField;
    /// Representation of `soft_lumps`; `()` omits this slot.
    type SoftLumps: NativeField;
    /// Representation of `soft_keys`; `()` omits this slot.
    type SoftKeys: NativeField;
    /// Representation of `soft_within`; `()` omits this slot.
    type SoftWithin: NativeField;
    /// Representation of `soft_follows`; `()` omits this slot.
    type SoftFollows: NativeField;
    /// Representation of `soft_push`; `()` omits this slot.
    type SoftPush: NativeField;
    /// Representation of `soft_at_end`; `()` omits this slot.
    type SoftAtEnd: NativeField;
    /// Representation of `soft_channels`; `()` omits this slot.
    type SoftChannels: NativeField;
    /// Representation of `soft_source`; `()` omits this slot.
    type SoftSource: NativeField;
    /// Representation of `made_settings`; `()` omits this slot.
    type MadeSettings: NativeField;
}

impl PhysicsBakeDialect for V0 {
    type ClothIds = ();
    type ClothFits = ();
    type ClothSizes = ();
    type SoftIds = ();
    type SoftLumps = ();
    type SoftKeys = ();
    type SoftWithin = ();
    type SoftFollows = ();
    type SoftPush = ();
    type SoftAtEnd = ();
    type SoftChannels = ();
    type SoftSource = ();
    type MadeSettings = ();
}

impl PhysicsBakeDialect for V1 {
    type ClothIds = ();
    type ClothFits = ();
    type ClothSizes = ();
    type SoftIds = ();
    type SoftLumps = ();
    type SoftKeys = ();
    type SoftWithin = ();
    type SoftFollows = ();
    type SoftPush = ();
    type SoftAtEnd = ();
    type SoftChannels = ();
    type SoftSource = ();
    type MadeSettings = ();
}

impl PhysicsBakeDialect for V2 {
    type ClothIds = ();
    type ClothFits = ();
    type ClothSizes = ();
    type SoftIds = ();
    type SoftLumps = ();
    type SoftKeys = ();
    type SoftWithin = ();
    type SoftFollows = ();
    type SoftPush = ();
    type SoftAtEnd = ();
    type SoftChannels = ();
    type SoftSource = ();
    type MadeSettings = ();
}

impl PhysicsBakeDialect for V3 {
    type ClothIds = ();
    type ClothFits = ();
    type ClothSizes = ();
    type SoftIds = ();
    type SoftLumps = ();
    type SoftKeys = ();
    type SoftWithin = ();
    type SoftFollows = ();
    type SoftPush = ();
    type SoftAtEnd = ();
    type SoftChannels = ();
    type SoftSource = ();
    type MadeSettings = ();
}

impl PhysicsBakeDialect for V4 {
    type ClothIds = ();
    type ClothFits = ();
    type ClothSizes = ();
    type SoftIds = ();
    type SoftLumps = ();
    type SoftKeys = ();
    type SoftWithin = ();
    type SoftFollows = ();
    type SoftPush = ();
    type SoftAtEnd = ();
    type SoftChannels = ();
    type SoftSource = ();
    type MadeSettings = ();
}

impl PhysicsBakeDialect for V5 {
    type ClothIds = ();
    type ClothFits = ();
    type ClothSizes = ();
    type SoftIds = ();
    type SoftLumps = ();
    type SoftKeys = ();
    type SoftWithin = ();
    type SoftFollows = ();
    type SoftPush = ();
    type SoftAtEnd = ();
    type SoftChannels = ();
    type SoftSource = ();
    type MadeSettings = ();
}

impl PhysicsBakeDialect for V6 {
    type ClothIds = Ref<targets::U32>;
    type ClothFits = Ref<targets::Real>;
    type ClothSizes = Ref<targets::Real>;
    type SoftIds = ();
    type SoftLumps = ();
    type SoftKeys = ();
    type SoftWithin = ();
    type SoftFollows = ();
    type SoftPush = ();
    type SoftAtEnd = ();
    type SoftChannels = ();
    type SoftSource = ();
    type MadeSettings = ();
}

impl PhysicsBakeDialect for V7 {
    type ClothIds = Ref<targets::U32>;
    type ClothFits = Ref<targets::Real>;
    type ClothSizes = Ref<targets::Real>;
    type SoftIds = Ref<targets::U32>;
    type SoftLumps = Ref<targets::U32>;
    type SoftKeys = Ref<targets::U32>;
    type SoftWithin = Ref<targets::Real>;
    type SoftFollows = Ref<targets::Real>;
    type SoftPush = Ref<targets::Real>;
    type SoftAtEnd = Ref<targets::Real>;
    type SoftChannels = Ref<targets::U32>;
    type SoftSource = Ref<targets::SubTrackContainer>;
    type MadeSettings = Bool;
}

impl PhysicsBakeDialect for V8 {
    type ClothIds = Ref<targets::U32>;
    type ClothFits = Ref<targets::Real>;
    type ClothSizes = Ref<targets::Real>;
    type SoftIds = Ref<targets::U32>;
    type SoftLumps = Ref<targets::U32>;
    type SoftKeys = Ref<targets::U32>;
    type SoftWithin = Ref<targets::Real>;
    type SoftFollows = Ref<targets::Real>;
    type SoftPush = Ref<targets::Real>;
    type SoftAtEnd = Ref<targets::Real>;
    type SoftChannels = Ref<targets::U32>;
    type SoftSource = Ref<targets::SubTrackContainer>;
    type MadeSettings = Bool;
}

impl PhysicsBakeDialect for V9 {
    type ClothIds = Ref<targets::U32>;
    type ClothFits = Ref<targets::Real>;
    type ClothSizes = Ref<targets::Real>;
    type SoftIds = Ref<targets::U32>;
    type SoftLumps = Ref<targets::U32>;
    type SoftKeys = Ref<targets::U32>;
    type SoftWithin = Ref<targets::Real>;
    type SoftFollows = Ref<targets::Real>;
    type SoftPush = Ref<targets::Real>;
    type SoftAtEnd = Ref<targets::Real>;
    type SoftChannels = Ref<targets::U32>;
    type SoftSource = Ref<targets::SubTrackContainer>;
    type MadeSettings = Bool;
}

impl PhysicsBakeDialect for V10 {
    type ClothIds = Ref<targets::U32>;
    type ClothFits = Ref<targets::Real>;
    type ClothSizes = Ref<targets::Real>;
    type SoftIds = Ref<targets::U32>;
    type SoftLumps = Ref<targets::U32>;
    type SoftKeys = Ref<targets::U32>;
    type SoftWithin = Ref<targets::Real>;
    type SoftFollows = Ref<targets::Real>;
    type SoftPush = Ref<targets::Real>;
    type SoftAtEnd = Ref<targets::Real>;
    type SoftChannels = Ref<targets::U32>;
    type SoftSource = Ref<targets::SubTrackContainer>;
    type MadeSettings = Bool;
}

impl PhysicsBakeDialect for V11 {
    type ClothIds = Ref<targets::U32>;
    type ClothFits = Ref<targets::Real>;
    type ClothSizes = Ref<targets::Real>;
    type SoftIds = Ref<targets::U32>;
    type SoftLumps = Ref<targets::U32>;
    type SoftKeys = Ref<targets::U32>;
    type SoftWithin = Ref<targets::Real>;
    type SoftFollows = Ref<targets::Real>;
    type SoftPush = Ref<targets::Real>;
    type SoftAtEnd = Ref<targets::Real>;
    type SoftChannels = Ref<targets::U32>;
    type SoftSource = Ref<targets::SubTrackContainer>;
    type MadeSettings = Bool;
}

impl PhysicsBakeDialect for V12 {
    type ClothIds = Ref<targets::U32>;
    type ClothFits = Ref<targets::Real>;
    type ClothSizes = Ref<targets::Real>;
    type SoftIds = Ref<targets::U32>;
    type SoftLumps = Ref<targets::U32>;
    type SoftKeys = Ref<targets::U32>;
    type SoftWithin = Ref<targets::Real>;
    type SoftFollows = Ref<targets::Real>;
    type SoftPush = Ref<targets::Real>;
    type SoftAtEnd = Ref<targets::Real>;
    type SoftChannels = Ref<targets::U32>;
    type SoftSource = Ref<targets::SubTrackContainer>;
    type MadeSettings = Bool;
}

impl PhysicsBakeDialect for V13 {
    type ClothIds = Ref<targets::U32>;
    type ClothFits = Ref<targets::Real>;
    type ClothSizes = Ref<targets::Real>;
    type SoftIds = Ref<targets::U32>;
    type SoftLumps = Ref<targets::U32>;
    type SoftKeys = Ref<targets::U32>;
    type SoftWithin = Ref<targets::Real>;
    type SoftFollows = Ref<targets::Real>;
    type SoftPush = Ref<targets::Real>;
    type SoftAtEnd = Ref<targets::Real>;
    type SoftChannels = Ref<targets::U32>;
    type SoftSource = Ref<targets::SubTrackContainer>;
    type MadeSettings = Bool;
}

impl PhysicsBakeDialect for V14 {
    type ClothIds = Ref<targets::U32>;
    type ClothFits = Ref<targets::Real>;
    type ClothSizes = Ref<targets::Real>;
    type SoftIds = Ref<targets::U32>;
    type SoftLumps = Ref<targets::U32>;
    type SoftKeys = Ref<targets::U32>;
    type SoftWithin = Ref<targets::Real>;
    type SoftFollows = Ref<targets::Real>;
    type SoftPush = Ref<targets::Real>;
    type SoftAtEnd = Ref<targets::Real>;
    type SoftChannels = Ref<targets::U32>;
    type SoftSource = Ref<targets::SubTrackContainer>;
    type MadeSettings = Bool;
}

impl PhysicsBakeDialect for V15 {
    type ClothIds = Ref<targets::U32>;
    type ClothFits = Ref<targets::Real>;
    type ClothSizes = Ref<targets::Real>;
    type SoftIds = Ref<targets::U32>;
    type SoftLumps = Ref<targets::U32>;
    type SoftKeys = Ref<targets::U32>;
    type SoftWithin = Ref<targets::Real>;
    type SoftFollows = Ref<targets::Real>;
    type SoftPush = Ref<targets::Real>;
    type SoftAtEnd = Ref<targets::Real>;
    type SoftChannels = Ref<targets::U32>;
    type SoftSource = Ref<targets::SubTrackContainer>;
    type MadeSettings = Bool;
}

impl PhysicsBakeDialect for V16 {
    type ClothIds = Ref<targets::U32>;
    type ClothFits = Ref<targets::Real>;
    type ClothSizes = Ref<targets::Real>;
    type SoftIds = Ref<targets::U32>;
    type SoftLumps = Ref<targets::U32>;
    type SoftKeys = Ref<targets::U32>;
    type SoftWithin = Ref<targets::Real>;
    type SoftFollows = Ref<targets::Real>;
    type SoftPush = Ref<targets::Real>;
    type SoftAtEnd = Ref<targets::Real>;
    type SoftChannels = Ref<targets::U32>;
    type SoftSource = Ref<targets::SubTrackContainer>;
    type MadeSettings = Bool;
}

impl PhysicsBakeDialect for V17 {
    type ClothIds = Ref<targets::U32>;
    type ClothFits = Ref<targets::Real>;
    type ClothSizes = Ref<targets::Real>;
    type SoftIds = Ref<targets::U32>;
    type SoftLumps = Ref<targets::U32>;
    type SoftKeys = Ref<targets::U32>;
    type SoftWithin = Ref<targets::Real>;
    type SoftFollows = Ref<targets::Real>;
    type SoftPush = Ref<targets::Real>;
    type SoftAtEnd = Ref<targets::Real>;
    type SoftChannels = Ref<targets::U32>;
    type SoftSource = Ref<targets::SubTrackContainer>;
    type MadeSettings = Bool;
}

impl PhysicsBakeDialect for V18 {
    type ClothIds = Ref<targets::U32>;
    type ClothFits = Ref<targets::Real>;
    type ClothSizes = Ref<targets::Real>;
    type SoftIds = Ref<targets::U32>;
    type SoftLumps = Ref<targets::U32>;
    type SoftKeys = Ref<targets::U32>;
    type SoftWithin = Ref<targets::Real>;
    type SoftFollows = Ref<targets::Real>;
    type SoftPush = Ref<targets::Real>;
    type SoftAtEnd = Ref<targets::Real>;
    type SoftChannels = Ref<targets::U32>;
    type SoftSource = Ref<targets::SubTrackContainer>;
    type MadeSettings = Bool;
}

impl PhysicsBakeDialect for V19 {
    type ClothIds = Ref<targets::U32>;
    type ClothFits = Ref<targets::Real>;
    type ClothSizes = Ref<targets::Real>;
    type SoftIds = Ref<targets::U32>;
    type SoftLumps = Ref<targets::U32>;
    type SoftKeys = Ref<targets::U32>;
    type SoftWithin = Ref<targets::Real>;
    type SoftFollows = Ref<targets::Real>;
    type SoftPush = Ref<targets::Real>;
    type SoftAtEnd = Ref<targets::Real>;
    type SoftChannels = Ref<targets::U32>;
    type SoftSource = Ref<targets::SubTrackContainer>;
    type MadeSettings = Bool;
}

/// Typed physics bake payload in its serialized field order.
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
pub struct PhysicsBake<D: PhysicsBakeDialect> {
    /// Saved pre-bake STCC containers used to restore the original clip keys.
    pub source: Ref<targets::SubTrackContainer>,
    /// Original container positions associated with the saved source arrays.
    pub positions: Ref<targets::U32>,
    /// Identities of channels written by the bake.
    pub channels: Ref<targets::U32>,
    /// Identities of channels introduced while baking.
    pub appended: Ref<targets::U32>,
    /// Fingerprint of the inputs used to create this bake.
    pub inputs: u64,
    /// Fingerprint of the generated keys, used to detect later edits.
    pub written: u64,
    /// Number of nodes written by the bake.
    pub nodes: u32,
    /// Number of generated animation keys.
    pub keys: u32,
    /// Number of TCB tracks converted to Hermite representation.
    pub restated: u32,
    /// Largest positional approximation error, in model units.
    pub max_error: f32,
    /// Largest angular mismatch at the loop seam before correction, in degrees.
    pub seam_before: f32,
    /// Largest angular mismatch at the loop seam after correction, in degrees.
    pub seam_after: f32,
    /// REAL section containing per-loop angular settling differences in degrees.
    pub settling: Ref<targets::Real>,
    /// U32_ section identifying the cloth records included in the bake.
    pub cloth_ids: D::ClothIds,
    /// REAL section storing cloth-fit errors for the corresponding cloth IDs.
    pub cloth_fits: D::ClothFits,
    /// REAL section storing the sizes of the corresponding cloth records.
    pub cloth_sizes: D::ClothSizes,
    /// U32_ section identifying the baked soft bodies.
    pub soft_ids: D::SoftIds,
    /// U32_ section counting keyed lumps for each baked soft body.
    pub soft_lumps: D::SoftLumps,
    /// U32_ section counting keys for each baked soft body.
    pub soft_keys: D::SoftKeys,
    /// REAL section storing soft-body approximation errors in model units.
    pub soft_within: D::SoftWithin,
    /// REAL section storing normalized soft-body following errors.
    pub soft_follows: D::SoftFollows,
    /// REAL section storing jiggle push relative to its configured limit.
    pub soft_push: D::SoftPush,
    /// REAL section storing residual jiggle motion at the end of a once-played clip.
    pub soft_at_end: D::SoftAtEnd,
    /// U32_ section identifying the channels overwritten by soft-body baking.
    pub soft_channels: D::SoftChannels,
    /// Saved STCC containers representing keys present before soft-body baking.
    pub soft_source: D::SoftSource,
    /// Records that this bake created the clip’s physics settings.
    pub made_settings: D::MadeSettings,
}

/// Static field selections for `ClipPhysics` section or inline context revisions.
pub trait ClipPhysicsDialect: Revision {
    /// Representation of `baked`; `()` omits this slot.
    type Baked: NativeField;
}

impl ClipPhysicsDialect for V0 {
    type Baked = InlineOption<PhysicsBake<V0>>;
}

impl ClipPhysicsDialect for V1 {
    type Baked = InlineOption<PhysicsBake<V1>>;
}

impl ClipPhysicsDialect for V2 {
    type Baked = InlineOption<PhysicsBake<V2>>;
}

impl ClipPhysicsDialect for V3 {
    type Baked = InlineOption<PhysicsBake<V3>>;
}

impl ClipPhysicsDialect for V4 {
    type Baked = InlineOption<PhysicsBake<V4>>;
}

impl ClipPhysicsDialect for V5 {
    type Baked = InlineOption<PhysicsBake<V5>>;
}

impl ClipPhysicsDialect for V6 {
    type Baked = InlineOption<PhysicsBake<V6>>;
}

impl ClipPhysicsDialect for V7 {
    type Baked = InlineOption<PhysicsBake<V7>>;
}

impl ClipPhysicsDialect for V8 {
    type Baked = InlineOption<PhysicsBake<V8>>;
}

impl ClipPhysicsDialect for V9 {
    type Baked = InlineOption<PhysicsBake<V9>>;
}

impl ClipPhysicsDialect for V10 {
    type Baked = InlineOption<PhysicsBake<V10>>;
}

impl ClipPhysicsDialect for V11 {
    type Baked = InlineOption<PhysicsBake<V11>>;
}

impl ClipPhysicsDialect for V12 {
    type Baked = InlineOption<PhysicsBake<V12>>;
}

impl ClipPhysicsDialect for V13 {
    type Baked = InlineOption<PhysicsBake<V13>>;
}

impl ClipPhysicsDialect for V14 {
    type Baked = InlineOption<PhysicsBake<V14>>;
}

impl ClipPhysicsDialect for V15 {
    type Baked = InlineOption<PhysicsBake<V15>>;
}

impl ClipPhysicsDialect for V16 {
    type Baked = InlineOption<PhysicsBake<V16>>;
}

impl ClipPhysicsDialect for V17 {
    type Baked = InlineOption<PhysicsBake<V17>>;
}

impl ClipPhysicsDialect for V18 {
    type Baked = InlineOption<PhysicsBake<V18>>;
}

impl ClipPhysicsDialect for V19 {
    type Baked = InlineOption<PhysicsBake<V19>>;
}

/// Typed clip physics payload in its serialized field order.
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
pub struct ClipPhysics<D: ClipPhysicsDialect> {
    /// Playback convention used for the recorded physics settings.
    pub plays: BakePlays,
    /// Source pose or clip selected before the physics interval.
    pub comes_from: BakeFrom,
    /// Character-section reference naming an explicit source clip.
    pub comes_from_clip: Ref<targets::Character>,
    /// Number of recorded preheat cycles before the clip interval.
    pub preheat: u8,
    /// Destination pose or clip selected after the physics interval.
    pub goes_to: BakeTo,
    /// Character-section reference naming an explicit destination clip.
    pub goes_to_clip: Ref<targets::Character>,
    /// Convention used to align the beginning and end of the physics interval.
    pub match_: LoopMatch,
    /// Selects which interval boundaries participate in matching.
    pub matching_where: MatchWhere,
    /// Transition width as a fraction of the clip duration.
    pub window: f32,
    /// Transition curve convention.
    pub ease: MatchEase,
    /// VEC2 section containing the normalized custom transition curve.
    pub curve: Ref<targets::Vector2>,
    /// Recorded approximation tolerance for physics-generated keys.
    pub tolerance: f32,
    /// Requests measurements on the deformed mesh rather than only on nodes.
    pub mesh_measure: Bool,
    /// Rate at which generated animation keys are recorded.
    pub key_rate: u8,
    /// Stored simulation stepping rate.
    pub step_rate: u8,
    /// Floor reference used by the recorded simulation settings.
    pub floor: BakeFloor,
    /// Carries the model forward according to the clip’s movement speed.
    pub travel: Bool,
    /// World-force section referenced by the clip’s physics settings.
    pub world: Ref<targets::BakeWorldForce>,
    /// Optional stored bake report and restoration metadata.
    pub baked: D::Baked,
}

/// Static field selections for `Clip` section or inline context revisions.
pub trait ClipDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `bounds`; `()` omits this slot.
    type Bounds: NativeField;
    /// Representation of `track_sets`; `()` omits this slot.
    type TrackSets: NativeField;
    /// Representation of `read_rule`; `()` omits this slot.
    type ReadRule: NativeField;
    /// Representation of `physics`; `()` omits this slot.
    type Physics: NativeField;
}

impl ClipDialect for V1 {
    const VERSION: u32 = 1;
    type Bounds = ();
    type TrackSets = ();
    type ReadRule = ();
    type Physics = ();
}

impl ClipDialect for V2 {
    const VERSION: u32 = 2;
    type Bounds = Extent;
    type TrackSets = ();
    type ReadRule = ();
    type Physics = ();
}

impl ClipDialect for V3 {
    const VERSION: u32 = 3;
    type Bounds = Extent;
    type TrackSets = Ref<targets::ClipTrackSet>;
    type ReadRule = ();
    type Physics = ();
}

impl ClipDialect for V4 {
    const VERSION: u32 = 4;
    type Bounds = Extent;
    type TrackSets = Ref<targets::ClipTrackSet>;
    type ReadRule = ReadRule;
    type Physics = ();
}

impl ClipDialect for V5 {
    const VERSION: u32 = 5;
    type Bounds = Extent;
    type TrackSets = Ref<targets::ClipTrackSet>;
    type ReadRule = ReadRule;
    type Physics = InlineOption<ClipPhysics<V5>>;
}

impl ClipDialect for V6 {
    const VERSION: u32 = 6;
    type Bounds = Extent;
    type TrackSets = Ref<targets::ClipTrackSet>;
    type ReadRule = ReadRule;
    type Physics = InlineOption<ClipPhysics<V6>>;
}

impl ClipDialect for V7 {
    const VERSION: u32 = 7;
    type Bounds = Extent;
    type TrackSets = Ref<targets::ClipTrackSet>;
    type ReadRule = ReadRule;
    type Physics = InlineOption<ClipPhysics<V7>>;
}

/// Typed clip payload in its serialized field order.
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
pub struct Clip<D: ClipDialect> {
    /// Character-section reference naming the animation clip.
    pub name: Ref<targets::Character>,
    /// Document model index whose channel table is used by this clip.
    pub model: u32,
    /// Clip length in seconds.
    pub duration: f32,
    /// Stored looping flag; its original byte encoding is retained.
    pub looping: Bool,
    /// Playback state bits, including automatic and world-clocked behavior.
    pub flags: ClipFlags,
    /// STCC section containing the clip’s animation layers.
    pub containers: Ref<targets::SubTrackContainer>,
    /// CLEV section containing timed events associated with model nodes.
    pub events: Ref<targets::ClipEvent>,
    /// Name/value metadata preserving source-specific clip properties.
    pub native: NativeBag,
    /// Recorded animated bounding volume in revisions that include it.
    pub bounds: D::Bounds,
    /// Section listing model track sets selected by this clip.
    pub track_sets: D::TrackSets,
    /// Rule used to combine this clip with the model’s other animation data.
    pub read_rule: D::ReadRule,
    /// Optional saved physics settings in revisions that include this slot.
    pub physics: D::Physics,
}
impl<D: ClipDialect> Record for Clip<D> {
    const TAG: [u8; 4] = *b"CLIP";
    const VERSION: u32 = D::VERSION;
}

/// Typed anim tag payload in its serialized field order.
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
pub struct AnimTag {
    /// Source-specific tag identity used to select an animation.
    pub tag_id: u32,
    /// Index into the document’s clip list.
    pub clip: u32,
}
impl Record for AnimTag {
    const TAG: [u8; 4] = *b"ATAG";
    const VERSION: u32 = 1;
}

/// Typed anim set payload in its serialized field order.
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
pub struct AnimSet {
    /// Character-section reference naming the animation selection table.
    pub name: Ref<targets::Character>,
    /// ATAG section mapping source tag identities to document clips.
    pub by_tag: Ref<targets::AnimTag>,
    /// Document animation-set index used as a fallback when this table has no matching tag.
    pub base_anim_set: u32,
}
impl Record for AnimSet {
    const TAG: [u8; 4] = *b"ASET";
    const VERSION: u32 = 1;
}

/// Typed stage source payload in its serialized field order.
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
pub struct StageSource {
    /// Stable source identity within its containing pose stage.
    pub id: u32,
    /// Model node supplying this stage input; a Link can use the invalid-node sentinel for world space.
    pub node: u32,
    /// Unanimated contribution of this source to the stage.
    pub weight: f32,
}
impl Record for StageSource {
    const TAG: [u8; 4] = *b"PSRC";
    const VERSION: u32 = 1;
}

/// Static field selections for `PoseStage` section or inline context revisions.
pub trait PoseStageDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `rig`; `()` omits this slot.
    type Rig: NativeField;
    /// Representation of `cloth`; `()` omits this slot.
    type Cloth: NativeField;
}

impl PoseStageDialect for V1 {
    const VERSION: u32 = 1;
    type Rig = ();
    type Cloth = ();
}

impl PoseStageDialect for V2 {
    const VERSION: u32 = 2;
    type Rig = u32;
    type Cloth = u32;
}

/// Typed pose stage payload in its serialized field order.
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
pub struct PoseStage<D: PoseStageDialect> {
    /// Stable stage identity referenced by animation targets.
    pub id: u32,
    /// Character-section reference naming this pose stage.
    pub name: Ref<targets::Character>,
    /// Constraint or physics operation represented by the stage.
    pub kind: StageKind,
    /// Enables this stage in the stored pose pipeline.
    pub enabled: Bool,
    /// U32_ section listing model nodes whose transforms the stage changes.
    pub driven: Ref<targets::U32>,
    /// U32_ section listing goal nodes used by the stage.
    pub targets: Ref<targets::U32>,
    /// PSRC section defining weighted source nodes for the stage.
    pub sources: Ref<targets::StageSource>,
    /// Preserves the stored transform offset when applying the stage.
    pub keep_offset: Bool,
    /// Local axis used by orientation or aiming constraints.
    pub axis: [f32; 3],
    /// Up-direction vector used by aiming constraints.
    pub up: [f32; 3],
    /// Model node providing an optional up reference.
    pub up_node: u32,
    /// Stored translation, rotation, and scale offset for the stage.
    pub offset: Transform,
    /// Spring stiffness parameter in the saved stage settings.
    pub stiffness: f32,
    /// Spring damping parameter in the saved stage settings.
    pub damping: f32,
    /// Gravity parameter recorded for the stage’s physics settings.
    pub gravity: f32,
    /// Identity of the physics rig used by a ragdoll stage.
    pub rig: D::Rig,
    /// Identity of the cloth record used by a cloth stage.
    pub cloth: D::Cloth,
}
impl<D: PoseStageDialect> Record for PoseStage<D> {
    const TAG: [u8; 4] = *b"PSTG";
    const VERSION: u32 = D::VERSION;
}
