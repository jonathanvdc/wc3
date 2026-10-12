//! Document roots, model roots, material bindings, and shared metadata records.
//!
//! A document declares native profiles and collects drawable models, textures,
//! and animation clips. Each model keeps shared geometry and hierarchy data while
//! profile material sets bind common slot names to their own materials and looks.
use super::*;
use crate::model::wem::{self, Bool, InlineList, Record, Ref};
/// Static field selections for `NativeEntry` section or inline context revisions.
pub trait NativeEntryDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `text`; `()` omits this slot.
    type Text: NativeField;
}

impl NativeEntryDialect for V1 {
    const VERSION: u32 = 1;
    type Text = ();
}

impl NativeEntryDialect for V2 {
    const VERSION: u32 = 2;
    type Text = Ref<targets::Character>;
}

/// One metadata key with a numeric value and optional dialect-selected text.
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
pub struct NativeEntry<D: NativeEntryDialect> {
    /// CHAR bytes naming a profile-specific metadata key.
    pub name: Ref<targets::Character>,
    /// Signed numeric metadata value; interpretation and any scaling are defined by the key.
    pub value: i64,
    /// Optional-by-dialect CHAR payload independent of the numeric value.
    pub text: D::Text,
}
impl<D: NativeEntryDialect> Record for NativeEntry<D> {
    const TAG: [u8; 4] = *b"NBAG";
    const VERSION: u32 = D::VERSION;
}

/// An external resource identity expressed by namespace, numeric ID, or filename.
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
pub struct AssetKey {
    /// Resource namespace or kind identifier, including Diablo III SNO groups.
    pub group: u32,
    /// Resource identity within the source namespace; `0xffffffff` means no numeric identity.
    pub id: u32,
    /// CHAR bytes naming a resource when it is addressed by filename.
    pub path: Ref<targets::Character>,
}
impl Record for AssetKey {
    const TAG: [u8; 4] = *b"AKEY";
    const VERSION: u32 = 1;
}

/// Static field selections for `DocumentRecord` section or inline context revisions.
pub trait DocumentRecordDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `profiles`; `()` omits this slot.
    type Profiles: NativeField;
}

impl DocumentRecordDialect for V1 {
    const VERSION: u32 = 1;
    type Profiles = InlineList<ProfileId>;
}

/// The profile declarations and shared resource tables of a WEM document.
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
pub struct DocumentRecord<D: DocumentRecordDialect> {
    /// Inline list declaring the native profiles carried by this document.
    pub profiles: D::Profiles,
    /// Declared profile selected when the caller makes no explicit choice.
    pub default_profile: ProfileId,
    /// Coordinate basis identifier shared by the document geometry.
    pub space: CoordSpace,
    /// Scale metadata for authored model units; stored coordinates are not rescaled during decoding.
    pub unit_scale: f32,
    /// CHAR bytes naming the document.
    pub name: Ref<targets::Character>,
    /// Axis-aligned box and sphere radius encompassing document geometry.
    pub bounds: Extent,
    /// Drawable models collected by the document.
    pub models: Ref<targets::Model>,
    /// Texture identities shared by materials and effects.
    pub textures: Ref<targets::TextureRef>,
    /// Document-wide animation clips, each identifying the model it drives.
    pub clips: Ref<targets::Clip>,
    /// Named tag-to-clip maps used as model animation sets.
    pub anim_sets: Ref<targets::AnimSet>,
}
impl<D: DocumentRecordDialect> Record for DocumentRecord<D> {
    const TAG: [u8; 4] = *b"WDOC";
    const VERSION: u32 = D::VERSION;
}

/// A model material slot mapped to profile-local materials across looks.
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
pub struct SlotBinding {
    /// Set-local material index for each look; `0xffffffff` leaves that look unbound.
    pub by_look: Ref<targets::U32>,
}
impl Record for SlotBinding {
    const TAG: [u8; 4] = *b"SBND";
    const VERSION: u32 = 1;
}

/// Materials and look bindings belonging to one declared native profile.
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
pub struct ProfileMaterialSet {
    /// Native profile owning this material and look vocabulary.
    pub profile: ProfileId,
    /// Look identities and names local to this profile material set.
    pub looks: LookTable,
    /// Index in this set look table selected when no look is explicitly chosen.
    pub default_look: u32,
    /// Materials indexed locally within this profile set.
    pub materials: Ref<targets::Material>,
    /// One binding per model material slot, with entries ordered by this set looks.
    pub slot_bindings: Ref<targets::SlotBinding>,
    /// Model metadata specific to this profile, independent of common model fields.
    pub native: NativeBag,
}
impl Record for ProfileMaterialSet {
    const TAG: [u8; 4] = *b"PROF";
    const VERSION: u32 = 1;
}

/// Authoring settings for later level-of-detail generation.
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
pub struct LodExport {
    /// Requests generation of levels of detail one through three during later export.
    pub generate: Bool,
    /// Target LOD-one triangle count divided by the LOD-zero triangle count.
    pub ratio1: f32,
    /// Target LOD-two triangle count divided by the LOD-zero triangle count.
    pub ratio2: f32,
    /// Target LOD-three triangle count divided by the LOD-zero triangle count.
    pub ratio3: f32,
    /// Requests preservation of open mesh borders during simplification.
    pub lock_borders: Bool,
    /// Records that the source contained additional levels of detail.
    pub source_had_levels: Bool,
}

/// Static field selections for `ModelRecord` section or inline context revisions.
pub trait ModelRecordDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `nodes`; `()` omits this slot.
    type Nodes: NativeField;
    /// Representation of `test_poses`; `()` omits this slot.
    type TestPoses: NativeField;
    /// Representation of `lod_export`; `()` omits this slot.
    type LodExport: NativeField;
    /// Representation of `t_pose`; `()` omits this slot.
    type TPose: NativeField;
    /// Representation of `track_sets`; `()` omits this slot.
    type TrackSets: NativeField;
    /// Representation of `pose_stages`; `()` omits this slot.
    type PoseStages: NativeField;
    /// Representation of `physics`; `()` omits this slot.
    type Physics: NativeField;
}

impl ModelRecordDialect for V1 {
    const VERSION: u32 = 1;
    type Nodes = NodeTree<V1>;
    type TestPoses = ();
    type LodExport = ();
    type TPose = ();
    type TrackSets = ();
    type PoseStages = ();
    type Physics = ();
}

impl ModelRecordDialect for V2 {
    const VERSION: u32 = 2;
    type Nodes = NodeTree<V2>;
    type TestPoses = ();
    type LodExport = ();
    type TPose = ();
    type TrackSets = ();
    type PoseStages = ();
    type Physics = ();
}

impl ModelRecordDialect for V3 {
    const VERSION: u32 = 3;
    type Nodes = NodeTree<V3>;
    type TestPoses = Ref<targets::TestPose>;
    type LodExport = ();
    type TPose = ();
    type TrackSets = ();
    type PoseStages = ();
    type Physics = ();
}

impl ModelRecordDialect for V4 {
    const VERSION: u32 = 4;
    type Nodes = NodeTree<V4>;
    type TestPoses = Ref<targets::TestPose>;
    type LodExport = LodExport;
    type TPose = ();
    type TrackSets = ();
    type PoseStages = ();
    type Physics = ();
}

impl ModelRecordDialect for V5 {
    const VERSION: u32 = 5;
    type Nodes = NodeTree<V5>;
    type TestPoses = Ref<targets::TestPose>;
    type LodExport = LodExport;
    type TPose = u32;
    type TrackSets = ();
    type PoseStages = ();
    type Physics = ();
}

impl ModelRecordDialect for V6 {
    const VERSION: u32 = 6;
    type Nodes = NodeTree<V6>;
    type TestPoses = Ref<targets::TestPose>;
    type LodExport = LodExport;
    type TPose = u32;
    type TrackSets = Ref<targets::TrackSet>;
    type PoseStages = ();
    type Physics = ();
}

impl ModelRecordDialect for V7 {
    const VERSION: u32 = 7;
    type Nodes = NodeTree<V7>;
    type TestPoses = Ref<targets::TestPose>;
    type LodExport = LodExport;
    type TPose = u32;
    type TrackSets = Ref<targets::TrackSet>;
    type PoseStages = Ref<targets::PoseStage>;
    type Physics = ();
}

impl ModelRecordDialect for V8 {
    const VERSION: u32 = 8;
    type Nodes = NodeTree<V8>;
    type TestPoses = Ref<targets::TestPose>;
    type LodExport = LodExport;
    type TPose = u32;
    type TrackSets = Ref<targets::TrackSet>;
    type PoseStages = Ref<targets::PoseStage>;
    type Physics = PhysicsSet<V8>;
}

impl ModelRecordDialect for V9 {
    const VERSION: u32 = 9;
    type Nodes = NodeTree<V9>;
    type TestPoses = Ref<targets::TestPose>;
    type LodExport = LodExport;
    type TPose = u32;
    type TrackSets = Ref<targets::TrackSet>;
    type PoseStages = Ref<targets::PoseStage>;
    type Physics = PhysicsSet<V9>;
}

/// Shared model geometry and hierarchy with profile-specific material sets.
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
pub struct ModelRecord<D: ModelRecordDialect> {
    /// CHAR bytes naming the drawable model.
    pub name: Ref<targets::Character>,
    /// Shared geometry used by all native profiles of this model.
    pub meshes: Ref<targets::Mesh>,
    /// Shared hierarchy containing bones, lights, attachments, and effect nodes.
    pub nodes: D::Nodes,
    /// Inline list of CHAR slot names used to join shared draw sections to profile-local materials.
    pub material_slots: InlineList<Ref<targets::Character>>,
    /// Table identifying animatable properties of this model.
    pub anim_channels: AnimChannelTable,
    /// Document animation-set index used by default; `0xffffffff` selects none.
    pub anim_set: u32,
    /// Profile-specific materials and look bindings for the shared slot vocabulary.
    pub profile_sets: Ref<targets::ProfileMaterialSet>,
    /// Axis-aligned box and sphere radius encompassing model geometry.
    pub bounds: Extent,
    /// Saved pose records used by authoring and skinning workflows.
    pub test_poses: D::TestPoses,
    /// Level-of-detail preparation settings selected by the model dialect.
    pub lod_export: D::LodExport,
    /// Saved test-pose index selected for repose operations; `0xffffffff` selects none.
    pub t_pose: D::TPose,
    /// Named channel groups selected by clip layer bindings.
    pub track_sets: D::TrackSets,
    /// Post-blend pose operations ordered by execution stage.
    pub pose_stages: D::PoseStages,
    /// Collision bodies, constraints, cloth, and soft-body records selected by the model dialect.
    pub physics: D::Physics,
}
impl<D: ModelRecordDialect> Record for ModelRecord<D> {
    const TAG: [u8; 4] = *b"MODL";
    const VERSION: u32 = D::VERSION;
}

/// An axis-aligned box accompanied by a sphere radius.
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
pub struct Extent {
    /// Lower corner of the model-space axis-aligned box.
    pub minimum: [f32; 3],
    /// Upper corner of the model-space axis-aligned box.
    pub maximum: [f32; 3],
    /// Radius of the sphere stored alongside the model-space box.
    pub sphere_radius: f32,
}
impl Record for Extent {
    const TAG: [u8; 4] = *b"BNDS";
    const VERSION: u32 = 0;
}

/// An ordered collection of profile-specific metadata entries.
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
pub struct NativeBag {
    /// Named numeric and text metadata entries in insertion order.
    pub entries: Ref<targets::NativeEntry>,
}
