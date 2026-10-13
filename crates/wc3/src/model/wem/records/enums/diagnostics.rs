//! Named diagnostics choices retain unrecognized wire values.
use crate::model::wem;

/// Named ValidateLevel choices; unrecognized codes remain representable.
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
pub enum ValidateLevel {
    #[default]
    /// Structural.
    #[wem(value = 0)]
    Structural,
    /// Manifold.
    #[wem(value = 1)]
    Manifold,
    /// Profile.
    #[wem(value = 2)]
    Profile,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Severity choices; unrecognized codes remain representable.
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
pub enum Severity {
    #[default]
    /// Info.
    #[wem(value = 0)]
    Info,
    /// Warning.
    #[wem(value = 1)]
    Warning,
    /// Error.
    #[wem(value = 2)]
    Error,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named DiagCode choices; unrecognized codes remain representable.
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
pub enum DiagCode {
    #[default]
    /// Unspecified.
    #[wem(value = 0)]
    Unspecified,
    /// NonManifoldEdgeSplit.
    #[wem(value = 1)]
    NonManifoldEdgeSplit,
    /// NonManifoldVertexSplit.
    #[wem(value = 2)]
    NonManifoldVertexSplit,
    /// DegenerateFaceDropped.
    #[wem(value = 3)]
    DegenerateFaceDropped,
    /// DuplicateFaceDropped.
    #[wem(value = 4)]
    DuplicateFaceDropped,
    /// InconsistentWindingFlipped.
    #[wem(value = 5)]
    InconsistentWindingFlipped,
    /// NgonTriangulated.
    #[wem(value = 6)]
    NgonTriangulated,
    /// IsolatedVertexDropped.
    #[wem(value = 7)]
    IsolatedVertexDropped,
    /// ConnectivityCorrupt.
    #[wem(value = 8)]
    ConnectivityCorrupt,
    /// IndexOutOfRange.
    #[wem(value = 9)]
    IndexOutOfRange,
    /// AttributeCountMismatch.
    #[wem(value = 10)]
    AttributeCountMismatch,
    /// SkinBindingMalformed.
    #[wem(value = 11)]
    SkinBindingMalformed,
    /// NonCanonicalNumbering.
    #[wem(value = 12)]
    NonCanonicalNumbering,
    /// TriangulationMalformed.
    #[wem(value = 13)]
    TriangulationMalformed,
    /// StaleTriangulation.
    #[wem(value = 14)]
    StaleTriangulation,
    /// DuplicateEdge.
    #[wem(value = 15)]
    DuplicateEdge,
    /// ReservedLayerMistyped.
    #[wem(value = 16)]
    ReservedLayerMistyped,
    /// LossyBlendMode.
    #[wem(value = 17)]
    LossyBlendMode,
    /// DroppedNativeBlock.
    #[wem(value = 18)]
    DroppedNativeBlock,
    /// SlotNotBound.
    #[wem(value = 19)]
    SlotNotBound,
    /// MixedShadedUnshadedStack.
    #[wem(value = 20)]
    MixedShadedUnshadedStack,
    /// SceneReadingMultiLayerStack.
    #[wem(value = 21)]
    SceneReadingMultiLayerStack,
    /// CollapseRefused.
    #[wem(value = 22)]
    CollapseRefused,
    /// UnknownShaderCombo.
    #[wem(value = 23)]
    UnknownShaderCombo,
    /// UnsupportedMaterialKind.
    #[wem(value = 24)]
    UnsupportedMaterialKind,
    /// LossyKindConversion.
    #[wem(value = 25)]
    LossyKindConversion,
    /// FeatureDropped.
    #[wem(value = 26)]
    FeatureDropped,
    /// LayerDropped.
    #[wem(value = 27)]
    LayerDropped,
    /// TextureUnresolved.
    #[wem(value = 28)]
    TextureUnresolved,
    /// MaterialSlotUnused.
    #[wem(value = 29)]
    MaterialSlotUnused,
    /// MaterialBodyInvalid.
    #[wem(value = 30)]
    MaterialBodyInvalid,
    /// LookBindingMalformed.
    #[wem(value = 31)]
    LookBindingMalformed,
    /// LookDropped.
    #[wem(value = 32)]
    LookDropped,
    /// TeamPlateCovered.
    #[wem(value = 33)]
    TeamPlateCovered,
    /// TeamPlateUnlit.
    #[wem(value = 34)]
    TeamPlateUnlit,
    /// ShadedAdditiveFolded.
    #[wem(value = 35)]
    ShadedAdditiveFolded,
    /// PassFlagsFolded.
    #[wem(value = 36)]
    PassFlagsFolded,
    /// CoverageClipped.
    #[wem(value = 37)]
    CoverageClipped,
    /// BaseFadeShared.
    #[wem(value = 38)]
    BaseFadeShared,
    /// LitFold.
    #[wem(value = 39)]
    LitFold,
    /// UnlitFold.
    #[wem(value = 40)]
    UnlitFold,
    /// DoubleLit.
    #[wem(value = 41)]
    DoubleLit,
    /// LitEnvFolded.
    #[wem(value = 42)]
    LitEnvFolded,
    /// SoftKey.
    #[wem(value = 43)]
    SoftKey,
    /// ModAlphaFolded.
    #[wem(value = 44)]
    ModAlphaFolded,
    /// DepthFlagsDropped.
    #[wem(value = 45)]
    DepthFlagsDropped,
    /// FlipbookDropped.
    #[wem(value = 46)]
    FlipbookDropped,
    /// CompositeEmitted.
    #[wem(value = 47)]
    CompositeEmitted,
    /// PassOrderFolded.
    #[wem(value = 48)]
    PassOrderFolded,
    /// FresnelFolded.
    #[wem(value = 49)]
    FresnelFolded,
    /// ProfileNotCarried.
    #[wem(value = 50)]
    ProfileNotCarried,
    /// ProfileCoverageIncomplete.
    #[wem(value = 51)]
    ProfileCoverageIncomplete,
    /// BoneInfluenceLimit.
    #[wem(value = 52)]
    BoneInfluenceLimit,
    /// UvSetLimit.
    #[wem(value = 53)]
    UvSetLimit,
    /// IndexWidthExceeded.
    #[wem(value = 54)]
    IndexWidthExceeded,
    /// NgonUnsupported.
    #[wem(value = 55)]
    NgonUnsupported,
    /// VertexColorUnsupported.
    #[wem(value = 56)]
    VertexColorUnsupported,
    /// NativeKindProfileMismatch.
    #[wem(value = 57)]
    NativeKindProfileMismatch,
    /// BonePaletteLimit.
    #[wem(value = 58)]
    BonePaletteLimit,
    /// GeometryMismatch.
    #[wem(value = 59)]
    GeometryMismatch,
    /// SectionUndrawn.
    #[wem(value = 60)]
    SectionUndrawn,
    /// NodeRemovalRefused.
    #[wem(value = 61)]
    NodeRemovalRefused,
    /// SkinInfluenceReassigned.
    #[wem(value = 62)]
    SkinInfluenceReassigned,
    /// DanglingNodeReference.
    #[wem(value = 63)]
    DanglingNodeReference,
    /// BindPoseRecomposed.
    #[wem(value = 64)]
    BindPoseRecomposed,
    /// RigConventionChanged.
    #[wem(value = 65)]
    RigConventionChanged,
    /// BoneShearSplit.
    #[wem(value = 66)]
    BoneShearSplit,
    /// BoneShearProjected.
    #[wem(value = 67)]
    BoneShearProjected,
    /// NonUniformScaleFlattened.
    #[wem(value = 68)]
    NonUniformScaleFlattened,
    /// NodeKindNotCarried.
    #[wem(value = 69)]
    NodeKindNotCarried,
    /// MixedInterpolationInTrack.
    #[wem(value = 70)]
    MixedInterpolationInTrack,
    /// AnimChannelInvalidated.
    #[wem(value = 71)]
    AnimChannelInvalidated,
    /// ClipTargetMissing.
    #[wem(value = 72)]
    ClipTargetMissing,
    /// AnimTrackDropped.
    #[wem(value = 73)]
    AnimTrackDropped,
    /// AnimTrackApproximated.
    #[wem(value = 74)]
    AnimTrackApproximated,
    /// AnimClipRetimed.
    #[wem(value = 75)]
    AnimClipRetimed,
    /// AnimTcbBaked.
    #[wem(value = 76)]
    AnimTcbBaked,
    /// TrackSetBelowDefault.
    #[wem(value = 77)]
    TrackSetBelowDefault,
    /// GlobalAndClipChannel.
    #[wem(value = 78)]
    GlobalAndClipChannel,
    /// AnimTrackResampled.
    #[wem(value = 79)]
    AnimTrackResampled,
    /// AnimLayersFlattened.
    #[wem(value = 80)]
    AnimLayersFlattened,
    /// AnimStageBaked.
    #[wem(value = 81)]
    AnimStageBaked,
    /// AnimStageNotBaked.
    #[wem(value = 82)]
    AnimStageNotBaked,
    /// ClipRuleConverted.
    #[wem(value = 83)]
    ClipRuleConverted,
    /// AnimStageInvalid.
    #[wem(value = 84)]
    AnimStageInvalid,
    /// EventPayloadMismatch.
    #[wem(value = 85)]
    EventPayloadMismatch,
    /// AssetUnresolved.
    #[wem(value = 86)]
    AssetUnresolved,
    /// HardpointUnresolved.
    #[wem(value = 87)]
    HardpointUnresolved,
    /// UnknownChunkPreserved.
    #[wem(value = 88)]
    UnknownChunkPreserved,
    /// OrphanChunk.
    #[wem(value = 89)]
    OrphanChunk,
    /// UnsupportedVersion.
    #[wem(value = 90)]
    UnsupportedVersion,
    /// LegacyDocumentUpgraded.
    #[wem(value = 91)]
    LegacyDocumentUpgraded,
    /// OperationUnsupported.
    #[wem(value = 92)]
    OperationUnsupported,
    /// GeometryRescaled.
    #[wem(value = 93)]
    GeometryRescaled,
    /// LevelOfDetailDropped.
    #[wem(value = 94)]
    LevelOfDetailDropped,
    /// SkinProfileNotCarried.
    #[wem(value = 95)]
    SkinProfileNotCarried,
    /// SkinInfluenceDuplicated.
    #[wem(value = 96)]
    SkinInfluenceDuplicated,
    /// SkinWeightInvalid.
    #[wem(value = 97)]
    SkinWeightInvalid,
    /// SkinInfluencesUnsorted.
    #[wem(value = 98)]
    SkinInfluencesUnsorted,
    /// SkinSetupInvalid.
    #[wem(value = 99)]
    SkinSetupInvalid,
    /// PhysicsConstraintDropped.
    #[wem(value = 100)]
    PhysicsConstraintDropped,
    /// PhysicsVersionRefused.
    #[wem(value = 101)]
    PhysicsVersionRefused,
    /// PhysicsMaterialMerged.
    #[wem(value = 102)]
    PhysicsMaterialMerged,
    /// PhysicsGravityScaleDropped.
    #[wem(value = 103)]
    PhysicsGravityScaleDropped,
    /// PhysicsJointKindUnsupported.
    #[wem(value = 104)]
    PhysicsJointKindUnsupported,
    /// PhysicsJointFieldDropped.
    #[wem(value = 105)]
    PhysicsJointFieldDropped,
    /// PhysicsJointBodyAmbiguous.
    #[wem(value = 106)]
    PhysicsJointBodyAmbiguous,
    /// PhysicsHullSimplified.
    #[wem(value = 107)]
    PhysicsHullSimplified,
    /// PhysicsReferenceInvalid.
    #[wem(value = 108)]
    PhysicsReferenceInvalid,
    /// PhysicsJointSnaps.
    #[wem(value = 109)]
    PhysicsJointSnaps,
    /// PhysicsShapeDegenerate.
    #[wem(value = 110)]
    PhysicsShapeDegenerate,
    /// PhysicsUnsupported.
    #[wem(value = 111)]
    PhysicsUnsupported,
    /// ClothParticleLimit.
    #[wem(value = 112)]
    ClothParticleLimit,
    /// ClothAnchorBoneOutOfRange.
    #[wem(value = 113)]
    ClothAnchorBoneOutOfRange,
    /// ClothSectionSplit.
    #[wem(value = 114)]
    ClothSectionSplit,
    /// ClothTopologyInvalid.
    #[wem(value = 115)]
    ClothTopologyInvalid,
    /// PhysicsChunkDropped.
    #[wem(value = 116)]
    PhysicsChunkDropped,
    /// PhysicsRigDropped.
    #[wem(value = 117)]
    PhysicsRigDropped,
    /// Count.
    #[wem(value = 118)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u16),
}

/// Named ModelRefusal choices; unrecognized codes remain representable.
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
pub enum ModelRefusal {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// EmptySelection.
    #[wem(value = 1)]
    EmptySelection,
    /// DifferentMeshes.
    #[wem(value = 2)]
    DifferentMeshes,
    /// WouldPinch.
    #[wem(value = 3)]
    WouldPinch,
    /// CrackNeedsWeld.
    #[wem(value = 4)]
    CrackNeedsWeld,
    /// NotAdjacent.
    #[wem(value = 5)]
    NotAdjacent,
    /// BorderEdgeRemove.
    #[wem(value = 6)]
    BorderEdgeRemove,
    /// BorderEdgeChamfer.
    #[wem(value = 7)]
    BorderEdgeChamfer,
    /// WouldRepeatVertex.
    #[wem(value = 8)]
    WouldRepeatVertex,
    /// WouldDuplicateFace.
    #[wem(value = 9)]
    WouldDuplicateFace,
    /// WouldFold.
    #[wem(value = 10)]
    WouldFold,
    /// CollapseIllegal.
    #[wem(value = 11)]
    CollapseIllegal,
    /// TargetCollapseIllegal.
    #[wem(value = 12)]
    TargetCollapseIllegal,
    /// AllFaces.
    #[wem(value = 13)]
    AllFaces,
    /// ZeroAmount.
    #[wem(value = 14)]
    ZeroAmount,
    /// ConcaveTurn.
    #[wem(value = 15)]
    ConcaveTurn,
    /// DiagonalOutside.
    #[wem(value = 16)]
    DiagonalOutside,
    /// AdjacentCorners.
    #[wem(value = 17)]
    AdjacentCorners,
    /// NotOneFace.
    #[wem(value = 18)]
    NotOneFace,
    /// DiagonalExists.
    #[wem(value = 19)]
    DiagonalExists,
    /// CapTooShort.
    #[wem(value = 20)]
    CapTooShort,
    /// CrossesSection.
    #[wem(value = 21)]
    CrossesSection,
    /// DissolveVertexIllegal.
    #[wem(value = 22)]
    DissolveVertexIllegal,
    /// NothingToSlice.
    #[wem(value = 23)]
    NothingToSlice,
    /// NoSourceSide.
    #[wem(value = 24)]
    NoSourceSide,
    /// WindingDisagrees.
    #[wem(value = 25)]
    WindingDisagrees,
    /// LoopCountsDiffer.
    #[wem(value = 26)]
    LoopCountsDiffer,
    /// SameLoop.
    #[wem(value = 27)]
    SameLoop,
    /// HingeNotOnBoundary.
    #[wem(value = 28)]
    HingeNotOnBoundary,
    /// NotBuiltYet.
    #[wem(value = 29)]
    NotBuiltYet,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Level choices; unrecognized codes remain representable.
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
pub enum Level {
    #[default]
    /// Vertex.
    #[wem(value = 0)]
    Vertex,
    /// Edge.
    #[wem(value = 1)]
    Edge,
    /// Polygon.
    #[wem(value = 2)]
    Polygon,
    /// Mesh.
    #[wem(value = 3)]
    Mesh,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Status choices; unrecognized codes remain representable.
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
pub enum Status {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Deleted.
    #[wem(value = 1)]
    Deleted,
    /// Locked.
    #[wem(value = 2)]
    Locked,
    /// Selected.
    #[wem(value = 4)]
    Selected,
    /// Hidden.
    #[wem(value = 8)]
    Hidden,
    /// Feature.
    #[wem(value = 16)]
    Feature,
    /// Tagged.
    #[wem(value = 32)]
    Tagged,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Domain choices; unrecognized codes remain representable.
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
pub enum Domain {
    #[default]
    /// Vertex.
    #[wem(value = 0)]
    Vertex,
    /// Halfedge.
    #[wem(value = 1)]
    Halfedge,
    /// Edge.
    #[wem(value = 2)]
    Edge,
    /// Face.
    #[wem(value = 3)]
    Face,
    /// Mesh.
    #[wem(value = 4)]
    Mesh,
    /// Count.
    #[wem(value = 5)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named BuildError choices; unrecognized codes remain representable.
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
pub enum BuildError {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// NonManifoldEdge.
    #[wem(value = 1)]
    NonManifoldEdge,
    /// NonManifoldVertex.
    #[wem(value = 2)]
    NonManifoldVertex,
    /// InconsistentWinding.
    #[wem(value = 3)]
    InconsistentWinding,
    /// DegenerateFace.
    #[wem(value = 4)]
    DegenerateFace,
    /// VertexOutOfRange.
    #[wem(value = 5)]
    VertexOutOfRange,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named CutNote choices; unrecognized codes remain representable.
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
pub enum CutNote {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Stored.
    #[wem(value = 1)]
    Stored,
    /// Folded.
    #[wem(value = 2)]
    Folded,
    /// NoEar.
    #[wem(value = 3)]
    NoEar,
    /// Degenerate.
    #[wem(value = 4)]
    Degenerate,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Failure choices; unrecognized codes remain representable.
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
pub enum Failure {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Cancelled.
    #[wem(value = 1)]
    Cancelled,
    /// EmptyMesh.
    #[wem(value = 2)]
    EmptyMesh,
    /// NotManifold.
    #[wem(value = 3)]
    NotManifold,
    /// Unsolved.
    #[wem(value = 4)]
    Unsolved,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Refusal choices; unrecognized codes remain representable.
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
pub enum Refusal {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Closed.
    #[wem(value = 1)]
    Closed,
    /// NoFaces.
    #[wem(value = 2)]
    NoFaces,
    /// TooFewPins.
    #[wem(value = 3)]
    TooFewPins,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SeamsRefusal choices; unrecognized codes remain representable.
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
pub enum SeamsRefusal {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// NoLimbs.
    #[wem(value = 1)]
    NoLimbs,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named FractureRefusal choices; unrecognized codes remain representable.
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
pub enum FractureRefusal {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Missing.
    #[wem(value = 1)]
    Missing,
    /// Empty.
    #[wem(value = 2)]
    Empty,
    /// Cloth.
    #[wem(value = 3)]
    Cloth,
    /// Billboard.
    #[wem(value = 4)]
    Billboard,
    /// Repeated.
    #[wem(value = 5)]
    Repeated,
    /// NoPieces.
    #[wem(value = 6)]
    NoPieces,
    /// TooMany.
    #[wem(value = 7)]
    TooMany,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}
