//! Named authoring choices retain unrecognized wire values.
use crate::model::wem;

/// Named ElementKind choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum ElementKind {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Document.
    #[wem(value = 1)]
    Document,
    /// Mesh.
    #[wem(value = 2)]
    Mesh,
    /// Face.
    #[wem(value = 3)]
    Face,
    /// Vertex.
    #[wem(value = 4)]
    Vertex,
    /// Halfedge.
    #[wem(value = 5)]
    Halfedge,
    /// Edge.
    #[wem(value = 6)]
    Edge,
    /// Section.
    #[wem(value = 7)]
    Section,
    /// Node.
    #[wem(value = 8)]
    Node,
    /// Material.
    #[wem(value = 9)]
    Material,
    /// Layer.
    #[wem(value = 10)]
    Layer,
    /// Slot.
    #[wem(value = 11)]
    Slot,
    /// Texture.
    #[wem(value = 12)]
    Texture,
    /// Look.
    #[wem(value = 13)]
    Look,
    /// Feature.
    #[wem(value = 14)]
    Feature,
    /// Actor.
    #[wem(value = 15)]
    Actor,
    /// Event.
    #[wem(value = 16)]
    Event,
    /// Clip.
    #[wem(value = 17)]
    Clip,
    /// Channel.
    #[wem(value = 18)]
    Channel,
    /// Track.
    #[wem(value = 19)]
    Track,
    /// Chunk.
    #[wem(value = 20)]
    Chunk,
    /// PhysicsRecord.
    #[wem(value = 21)]
    PhysicsRecord,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named MergeNodeAction choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum MergeNodeAction {
    #[default]
    /// Add.
    #[wem(value = 0)]
    Add,
    /// Into.
    #[wem(value = 1)]
    Into,
    /// Skip.
    #[wem(value = 2)]
    Skip,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Search choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum Search {
    #[default]
    /// Balanced.
    #[wem(value = 0)]
    Balanced,
    /// Maximum.
    #[wem(value = 1)]
    Maximum,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Measure choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum Measure {
    #[default]
    /// Skeleton.
    #[wem(value = 0)]
    Skeleton,
    /// Mesh.
    #[wem(value = 1)]
    Mesh,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Curve choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum Curve {
    #[default]
    /// Snapshot.
    #[wem(value = 0)]
    Snapshot,
    /// Linear.
    #[wem(value = 1)]
    Linear,
    /// Smoothstep.
    #[wem(value = 2)]
    Smoothstep,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named ReadRule choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum ReadRule {
    #[default]
    /// Wc3.
    #[wem(value = 0)]
    Wc3,
    /// Sc2.
    #[wem(value = 1)]
    Sc2,
    /// Wow.
    #[wem(value = 2)]
    Wow,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named BakePlays choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum BakePlays {
    #[default]
    /// FromClip.
    #[wem(value = 0)]
    FromClip,
    /// Loops.
    #[wem(value = 1)]
    Loops,
    /// Once.
    #[wem(value = 2)]
    Once,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named BakeFrom choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum BakeFrom {
    #[default]
    /// Auto.
    #[wem(value = 0)]
    Auto,
    /// Itself.
    #[wem(value = 1)]
    Itself,
    /// FirstFrame.
    #[wem(value = 2)]
    FirstFrame,
    /// Clip.
    #[wem(value = 3)]
    Clip,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named BakeTo choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum BakeTo {
    #[default]
    /// Auto.
    #[wem(value = 0)]
    Auto,
    /// Nothing.
    #[wem(value = 1)]
    Nothing,
    /// Clip.
    #[wem(value = 2)]
    Clip,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named LoopMatch choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum LoopMatch {
    #[default]
    /// Crossfade.
    #[wem(value = 0)]
    Crossfade,
    /// Offset.
    #[wem(value = 1)]
    Offset,
    /// Off.
    #[wem(value = 2)]
    Off,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named MatchWhere choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum MatchWhere {
    #[default]
    /// End.
    #[wem(value = 0)]
    End,
    /// Start.
    #[wem(value = 1)]
    Start,
    /// Both.
    #[wem(value = 2)]
    Both,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named MatchEase choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum MatchEase {
    #[default]
    /// Smooth.
    #[wem(value = 0)]
    Smooth,
    /// Linear.
    #[wem(value = 1)]
    Linear,
    /// EaseIn.
    #[wem(value = 2)]
    EaseIn,
    /// EaseOut.
    #[wem(value = 3)]
    EaseOut,
    /// Custom.
    #[wem(value = 4)]
    Custom,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named BakeFloor choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum BakeFloor {
    #[default]
    /// Grid.
    #[wem(value = 0)]
    Grid,
    /// Off.
    #[wem(value = 1)]
    Off,
    /// FollowsRoot.
    #[wem(value = 2)]
    FollowsRoot,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named BakeWorldForceKind choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum BakeWorldForceKind {
    #[default]
    /// Wind.
    #[wem(value = 0)]
    Wind,
    /// Blast.
    #[wem(value = 1)]
    Blast,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named StageKind choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum StageKind {
    #[default]
    /// Position.
    #[wem(value = 0)]
    Position,
    /// Orientation.
    #[wem(value = 1)]
    Orientation,
    /// LookAt.
    #[wem(value = 2)]
    LookAt,
    /// Link.
    #[wem(value = 3)]
    Link,
    /// LimbIk.
    #[wem(value = 4)]
    LimbIk,
    /// ChainIk.
    #[wem(value = 5)]
    ChainIk,
    /// Spring.
    #[wem(value = 6)]
    Spring,
    /// Ragdoll.
    #[wem(value = 7)]
    Ragdoll,
    /// Cloth.
    #[wem(value = 8)]
    Cloth,
    /// Count.
    #[wem(value = 9)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named NameSide choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum NameSide {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Left.
    #[wem(value = 1)]
    Left,
    /// Right.
    #[wem(value = 2)]
    Right,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named RemovePolicy choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum RemovePolicy {
    #[default]
    /// ReparentChildren.
    #[wem(value = 0)]
    ReparentChildren,
    /// RemoveSubtree.
    #[wem(value = 1)]
    RemoveSubtree,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SkinPolicy choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SkinPolicy {
    #[default]
    /// Refuse.
    #[wem(value = 0)]
    Refuse,
    /// ReassignToParent.
    #[wem(value = 1)]
    ReassignToParent,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named PoseSpace choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum PoseSpace {
    #[default]
    /// Model.
    #[wem(value = 0)]
    Model,
    /// ParentRelative.
    #[wem(value = 1)]
    ParentRelative,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named PoseStorage choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum PoseStorage {
    #[default]
    /// Trs.
    #[wem(value = 0)]
    Trs,
    /// Matrix.
    #[wem(value = 1)]
    Matrix,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SubdivideScheme choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SubdivideScheme {
    #[default]
    /// CatmullClark.
    #[wem(value = 0)]
    CatmullClark,
    /// Linear.
    #[wem(value = 1)]
    Linear,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SmoothScheme choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SmoothScheme {
    #[default]
    /// Laplacian.
    #[wem(value = 0)]
    Laplacian,
    /// Taubin.
    #[wem(value = 1)]
    Taubin,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named ExtrudeType choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum ExtrudeType {
    #[default]
    /// Group.
    #[wem(value = 0)]
    Group,
    /// LocalNormal.
    #[wem(value = 1)]
    LocalNormal,
    /// ByPolygon.
    #[wem(value = 2)]
    ByPolygon,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named InsetType choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum InsetType {
    #[default]
    /// Group.
    #[wem(value = 0)]
    Group,
    /// ByPolygon.
    #[wem(value = 1)]
    ByPolygon,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SliceMode choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SliceMode {
    #[default]
    /// Cut.
    #[wem(value = 0)]
    Cut,
    /// Split.
    #[wem(value = 1)]
    Split,
    /// Remove.
    #[wem(value = 2)]
    Remove,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named RepairPolicy choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum RepairPolicy {
    #[default]
    /// RepairAndRecord.
    #[wem(value = 0)]
    RepairAndRecord,
    /// RepairSilently.
    #[wem(value = 1)]
    RepairSilently,
    /// Refuse.
    #[wem(value = 2)]
    Refuse,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named TriangulationPolicy choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum TriangulationPolicy {
    #[default]
    /// FanFromFirstHalfedge.
    #[wem(value = 0)]
    FanFromFirstHalfedge,
    /// None.
    #[wem(value = 1)]
    None,
    /// Authored.
    #[wem(value = 2)]
    Authored,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Weighting choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u16)]
pub enum Weighting {
    #[default]
    /// Even.
    #[wem(value = 0)]
    Even,
    /// Angle.
    #[wem(value = 1)]
    Angle,
    /// Area.
    #[wem(value = 2)]
    Area,
    /// AreaAngle.
    #[wem(value = 3)]
    AreaAngle,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u16),
}

/// Named ElementLevel choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum ElementLevel {
    #[default]
    /// Vertex.
    #[wem(value = 0)]
    Vertex,
    /// Edge.
    #[wem(value = 1)]
    Edge,
    /// Border.
    #[wem(value = 2)]
    Border,
    /// Face.
    #[wem(value = 3)]
    Face,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named LinkDelimit choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum LinkDelimit {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Seam.
    #[wem(value = 1)]
    Seam,
    /// Sharp.
    #[wem(value = 2)]
    Sharp,
    /// Section.
    #[wem(value = 4)]
    Section,
    /// Angle.
    #[wem(value = 8)]
    Angle,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named ConvertRule choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum ConvertRule {
    #[default]
    /// Touching.
    #[wem(value = 0)]
    Touching,
    /// Enclosed.
    #[wem(value = 1)]
    Enclosed,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named RetopoMethod choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum RetopoMethod {
    #[default]
    /// Field.
    #[wem(value = 0)]
    Field,
    /// Layout.
    #[wem(value = 1)]
    Layout,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named ProjectShape choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum ProjectShape {
    #[default]
    /// Planar.
    #[wem(value = 0)]
    Planar,
    /// Cylinder.
    #[wem(value = 1)]
    Cylinder,
    /// Sphere.
    #[wem(value = 2)]
    Sphere,
    /// Box.
    #[wem(value = 3)]
    Box,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Grain choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum Grain {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// X.
    #[wem(value = 1)]
    X,
    /// Y.
    #[wem(value = 2)]
    Y,
    /// Z.
    #[wem(value = 3)]
    Z,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Parts choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum Parts {
    #[default]
    /// Overlap.
    #[wem(value = 0)]
    Overlap,
    /// Touch.
    #[wem(value = 1)]
    Touch,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named OpenParts choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum OpenParts {
    #[default]
    /// Solid.
    #[wem(value = 0)]
    Solid,
    /// Thicken.
    #[wem(value = 1)]
    Thicken,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Method choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum Method {
    #[default]
    /// Fracture.
    #[wem(value = 0)]
    Fracture,
    /// Slices.
    #[wem(value = 1)]
    Slices,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named FillingUvs choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum FillingUvs {
    #[default]
    /// Tiled.
    #[wem(value = 0)]
    Tiled,
    /// Square.
    #[wem(value = 1)]
    Square,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SwitchWhy choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SwitchWhy {
    #[default]
    /// Still.
    #[wem(value = 0)]
    Still,
    /// Rest.
    #[wem(value = 1)]
    Rest,
    /// Keyed.
    #[wem(value = 2)]
    Keyed,
    /// Dropped.
    #[wem(value = 3)]
    Dropped,
    /// Death.
    #[wem(value = 4)]
    Death,
    /// Carried.
    #[wem(value = 5)]
    Carried,
    /// Inherited.
    #[wem(value = 6)]
    Inherited,
    /// Exempt.
    #[wem(value = 7)]
    Exempt,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SwitchScope choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SwitchScope {
    #[default]
    /// Clip.
    #[wem(value = 0)]
    Clip,
    /// Simulate.
    #[wem(value = 1)]
    Simulate,
    /// Test.
    #[wem(value = 2)]
    Test,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}
