//! Open scalar domains retain unrecognized enumeration codes and flag bits.
use crate::model::wem;
/// Native ProfileId codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ProfileId(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for ProfileId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<ProfileId> for u32 {
    fn from(value: ProfileId) -> Self {
        value.0
    }
}
impl ProfileId {
    /// Stored code for `Generic`.
    pub const GENERIC: Self = Self(0);
    /// Stored code for `Wc3Classic`.
    pub const WC3_CLASSIC: Self = Self(1);
    /// Stored code for `Wc3Reforged`.
    pub const WC3_REFORGED: Self = Self(2);
    /// Stored code for `Wow`.
    pub const WOW: Self = Self(3);
    /// Stored code for `Sc2`.
    pub const SC2: Self = Self(4);
    /// Stored code for `Heroes`.
    pub const HEROES: Self = Self(5);
    /// Stored code for `Diablo3`.
    pub const DIABLO3: Self = Self(6);
    /// Stored code for `Wc3Definitive`.
    pub const WC3_DEFINITIVE: Self = Self(7);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(8);
}
/// Native Game codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Game(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Game {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Game> for u8 {
    fn from(value: Game) -> Self {
        value.0
    }
}
impl Game {
    /// Stored code for `Warcraft`.
    pub const WARCRAFT: Self = Self(0);
    /// Stored code for `StarCraft`.
    pub const STAR_CRAFT: Self = Self(1);
    /// Stored code for `Wow`.
    pub const WOW: Self = Self(2);
    /// Stored code for `Diablo`.
    pub const DIABLO: Self = Self(3);
}
/// Native CoordSpace codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct CoordSpace(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for CoordSpace {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<CoordSpace> for u8 {
    fn from(value: CoordSpace) -> Self {
        value.0
    }
}
impl CoordSpace {
    /// Stored code for `Blizzard`.
    pub const BLIZZARD: Self = Self(0);
    /// Stored code for `Sc2`.
    pub const SC2: Self = Self(1);
}
/// Native Handedness codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Handedness(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Handedness {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Handedness> for u8 {
    fn from(value: Handedness) -> Self {
        value.0
    }
}
impl Handedness {
    /// Stored code for `Right`.
    pub const RIGHT: Self = Self(0);
    /// Stored code for `Left`.
    pub const LEFT: Self = Self(1);
}
/// Native RigConvention codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct RigConvention(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for RigConvention {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<RigConvention> for u8 {
    fn from(value: RigConvention) -> Self {
        value.0
    }
}
impl RigConvention {
    /// Stored code for `PivotRelative`.
    pub const PIVOT_RELATIVE: Self = Self(0);
    /// Stored code for `ExplicitBind`.
    pub const EXPLICIT_BIND: Self = Self(1);
}
/// Native WindingOrder codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct WindingOrder(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for WindingOrder {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<WindingOrder> for u8 {
    fn from(value: WindingOrder) -> Self {
        value.0
    }
}
impl WindingOrder {
    /// Stored code for `CounterClockwise`.
    pub const COUNTER_CLOCKWISE: Self = Self(0);
    /// Stored code for `Clockwise`.
    pub const CLOCKWISE: Self = Self(1);
}
/// Native IndexWidth codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct IndexWidth(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for IndexWidth {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<IndexWidth> for u8 {
    fn from(value: IndexWidth) -> Self {
        value.0
    }
}
impl IndexWidth {
    /// Stored code for `U16`.
    pub const U16: Self = Self(0);
    /// Stored code for `U32`.
    pub const U32: Self = Self(1);
}
/// Native BlendMode codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct BlendMode(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for BlendMode {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<BlendMode> for u32 {
    fn from(value: BlendMode) -> Self {
        value.0
    }
}
impl BlendMode {
    /// Stored code for `Opaque`.
    pub const OPAQUE: Self = Self(0);
    /// Stored code for `AlphaKey`.
    pub const ALPHA_KEY: Self = Self(1);
    /// Stored code for `AlphaBlend`.
    pub const ALPHA_BLEND: Self = Self(2);
    /// Stored code for `Additive`.
    pub const ADDITIVE: Self = Self(3);
    /// Stored code for `AdditiveAlpha`.
    pub const ADDITIVE_ALPHA: Self = Self(4);
    /// Stored code for `Modulate`.
    pub const MODULATE: Self = Self(5);
    /// Stored code for `Modulate2x`.
    pub const MODULATE2X: Self = Self(6);
    /// Stored code for `BlendAdd`.
    pub const BLEND_ADD: Self = Self(7);
    /// Stored code for `Transparent`.
    pub const TRANSPARENT: Self = Self(8);
    /// Stored code for `PremultipliedAlpha`.
    pub const PREMULTIPLIED_ALPHA: Self = Self(9);
}
/// Native CullMode codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct CullMode(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for CullMode {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<CullMode> for u8 {
    fn from(value: CullMode) -> Self {
        value.0
    }
}
impl CullMode {
    /// Stored code for `Back`.
    pub const BACK: Self = Self(0);
    /// Stored code for `Front`.
    pub const FRONT: Self = Self(1);
    /// Stored code for `None`.
    pub const NONE: Self = Self(2);
}
/// Native MaterialKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct MaterialKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for MaterialKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<MaterialKind> for u8 {
    fn from(value: MaterialKind) -> Self {
        value.0
    }
}
impl MaterialKind {
    /// Stored code for `Composite`.
    pub const COMPOSITE: Self = Self(0);
    /// Stored code for `Combiners`.
    pub const COMBINERS: Self = Self(1);
    /// Stored code for `LegacyDeferred`.
    pub const LEGACY_DEFERRED: Self = Self(2);
    /// Stored code for `PBRDeferred`.
    pub const PBR_DEFERRED: Self = Self(3);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(4);
}
/// Native NativeKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct NativeKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for NativeKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<NativeKind> for u8 {
    fn from(value: NativeKind) -> Self {
        value.0
    }
}
impl NativeKind {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Mdx`.
    pub const MDX: Self = Self(1);
    /// Stored code for `M2`.
    pub const M2: Self = Self(2);
    /// Stored code for `M3`.
    pub const M3: Self = Self(3);
    /// Stored code for `D3`.
    pub const D3: Self = Self(4);
}
/// Native AssetHint codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct AssetHint(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for AssetHint {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<AssetHint> for u8 {
    fn from(value: AssetHint) -> Self {
        value.0
    }
}
impl AssetHint {
    /// Stored code for `Unknown`.
    pub const UNKNOWN: Self = Self(0);
    /// Stored code for `Appearance`.
    pub const APPEARANCE: Self = Self(1);
    /// Stored code for `Actor`.
    pub const ACTOR: Self = Self(2);
}
/// Native ValidateLevel codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ValidateLevel(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for ValidateLevel {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ValidateLevel> for u8 {
    fn from(value: ValidateLevel) -> Self {
        value.0
    }
}
impl ValidateLevel {
    /// Stored code for `Structural`.
    pub const STRUCTURAL: Self = Self(0);
    /// Stored code for `Manifold`.
    pub const MANIFOLD: Self = Self(1);
    /// Stored code for `Profile`.
    pub const PROFILE: Self = Self(2);
}
/// Native AxisPreset codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct AxisPreset(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for AxisPreset {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<AxisPreset> for u8 {
    fn from(value: AxisPreset) -> Self {
        value.0
    }
}
impl AxisPreset {
    /// Stored code for `YUp`.
    pub const Y_UP: Self = Self(0);
    /// Stored code for `ZUpMax`.
    pub const Z_UP_MAX: Self = Self(1);
    /// Stored code for `Native`.
    pub const NATIVE: Self = Self(2);
}
/// Native LengthUnit codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct LengthUnit(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for LengthUnit {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<LengthUnit> for u8 {
    fn from(value: LengthUnit) -> Self {
        value.0
    }
}
impl LengthUnit {
    /// Stored code for `Millimetre`.
    pub const MILLIMETRE: Self = Self(0);
    /// Stored code for `Centimetre`.
    pub const CENTIMETRE: Self = Self(1);
    /// Stored code for `Metre`.
    pub const METRE: Self = Self(2);
    /// Stored code for `Inch`.
    pub const INCH: Self = Self(3);
    /// Stored code for `Foot`.
    pub const FOOT: Self = Self(4);
}
/// Native ImageChannel codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ImageChannel(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for ImageChannel {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ImageChannel> for u8 {
    fn from(value: ImageChannel) -> Self {
        value.0
    }
}
impl ImageChannel {
    /// Stored code for `All`.
    pub const ALL: Self = Self(0);
    /// Stored code for `Red`.
    pub const RED: Self = Self(1);
    /// Stored code for `Green`.
    pub const GREEN: Self = Self(2);
    /// Stored code for `Blue`.
    pub const BLUE: Self = Self(3);
    /// Stored code for `Alpha`.
    pub const ALPHA: Self = Self(4);
}
/// Native Severity codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Severity(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Severity {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Severity> for u8 {
    fn from(value: Severity) -> Self {
        value.0
    }
}
impl Severity {
    /// Stored code for `Info`.
    pub const INFO: Self = Self(0);
    /// Stored code for `Warning`.
    pub const WARNING: Self = Self(1);
    /// Stored code for `Error`.
    pub const ERROR: Self = Self(2);
}
/// Native DiagCode codes, represented by their stored u16 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct DiagCode(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u16,
);
impl From<u16> for DiagCode {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<DiagCode> for u16 {
    fn from(value: DiagCode) -> Self {
        value.0
    }
}
impl DiagCode {
    /// Stored code for `Unspecified`.
    pub const UNSPECIFIED: Self = Self(0);
    /// Stored code for `NonManifoldEdgeSplit`.
    pub const NON_MANIFOLD_EDGE_SPLIT: Self = Self(1);
    /// Stored code for `NonManifoldVertexSplit`.
    pub const NON_MANIFOLD_VERTEX_SPLIT: Self = Self(2);
    /// Stored code for `DegenerateFaceDropped`.
    pub const DEGENERATE_FACE_DROPPED: Self = Self(3);
    /// Stored code for `DuplicateFaceDropped`.
    pub const DUPLICATE_FACE_DROPPED: Self = Self(4);
    /// Stored code for `InconsistentWindingFlipped`.
    pub const INCONSISTENT_WINDING_FLIPPED: Self = Self(5);
    /// Stored code for `NgonTriangulated`.
    pub const NGON_TRIANGULATED: Self = Self(6);
    /// Stored code for `IsolatedVertexDropped`.
    pub const ISOLATED_VERTEX_DROPPED: Self = Self(7);
    /// Stored code for `ConnectivityCorrupt`.
    pub const CONNECTIVITY_CORRUPT: Self = Self(8);
    /// Stored code for `IndexOutOfRange`.
    pub const INDEX_OUT_OF_RANGE: Self = Self(9);
    /// Stored code for `AttributeCountMismatch`.
    pub const ATTRIBUTE_COUNT_MISMATCH: Self = Self(10);
    /// Stored code for `SkinBindingMalformed`.
    pub const SKIN_BINDING_MALFORMED: Self = Self(11);
    /// Stored code for `NonCanonicalNumbering`.
    pub const NON_CANONICAL_NUMBERING: Self = Self(12);
    /// Stored code for `TriangulationMalformed`.
    pub const TRIANGULATION_MALFORMED: Self = Self(13);
    /// Stored code for `StaleTriangulation`.
    pub const STALE_TRIANGULATION: Self = Self(14);
    /// Stored code for `DuplicateEdge`.
    pub const DUPLICATE_EDGE: Self = Self(15);
    /// Stored code for `ReservedLayerMistyped`.
    pub const RESERVED_LAYER_MISTYPED: Self = Self(16);
    /// Stored code for `LossyBlendMode`.
    pub const LOSSY_BLEND_MODE: Self = Self(17);
    /// Stored code for `DroppedNativeBlock`.
    pub const DROPPED_NATIVE_BLOCK: Self = Self(18);
    /// Stored code for `SlotNotBound`.
    pub const SLOT_NOT_BOUND: Self = Self(19);
    /// Stored code for `MixedShadedUnshadedStack`.
    pub const MIXED_SHADED_UNSHADED_STACK: Self = Self(20);
    /// Stored code for `SceneReadingMultiLayerStack`.
    pub const SCENE_READING_MULTI_LAYER_STACK: Self = Self(21);
    /// Stored code for `CollapseRefused`.
    pub const COLLAPSE_REFUSED: Self = Self(22);
    /// Stored code for `UnknownShaderCombo`.
    pub const UNKNOWN_SHADER_COMBO: Self = Self(23);
    /// Stored code for `UnsupportedMaterialKind`.
    pub const UNSUPPORTED_MATERIAL_KIND: Self = Self(24);
    /// Stored code for `LossyKindConversion`.
    pub const LOSSY_KIND_CONVERSION: Self = Self(25);
    /// Stored code for `FeatureDropped`.
    pub const FEATURE_DROPPED: Self = Self(26);
    /// Stored code for `LayerDropped`.
    pub const LAYER_DROPPED: Self = Self(27);
    /// Stored code for `TextureUnresolved`.
    pub const TEXTURE_UNRESOLVED: Self = Self(28);
    /// Stored code for `MaterialSlotUnused`.
    pub const MATERIAL_SLOT_UNUSED: Self = Self(29);
    /// Stored code for `MaterialBodyInvalid`.
    pub const MATERIAL_BODY_INVALID: Self = Self(30);
    /// Stored code for `LookBindingMalformed`.
    pub const LOOK_BINDING_MALFORMED: Self = Self(31);
    /// Stored code for `LookDropped`.
    pub const LOOK_DROPPED: Self = Self(32);
    /// Stored code for `TeamPlateCovered`.
    pub const TEAM_PLATE_COVERED: Self = Self(33);
    /// Stored code for `TeamPlateUnlit`.
    pub const TEAM_PLATE_UNLIT: Self = Self(34);
    /// Stored code for `ShadedAdditiveFolded`.
    pub const SHADED_ADDITIVE_FOLDED: Self = Self(35);
    /// Stored code for `PassFlagsFolded`.
    pub const PASS_FLAGS_FOLDED: Self = Self(36);
    /// Stored code for `CoverageClipped`.
    pub const COVERAGE_CLIPPED: Self = Self(37);
    /// Stored code for `BaseFadeShared`.
    pub const BASE_FADE_SHARED: Self = Self(38);
    /// Stored code for `LitFold`.
    pub const LIT_FOLD: Self = Self(39);
    /// Stored code for `UnlitFold`.
    pub const UNLIT_FOLD: Self = Self(40);
    /// Stored code for `DoubleLit`.
    pub const DOUBLE_LIT: Self = Self(41);
    /// Stored code for `LitEnvFolded`.
    pub const LIT_ENV_FOLDED: Self = Self(42);
    /// Stored code for `SoftKey`.
    pub const SOFT_KEY: Self = Self(43);
    /// Stored code for `ModAlphaFolded`.
    pub const MOD_ALPHA_FOLDED: Self = Self(44);
    /// Stored code for `DepthFlagsDropped`.
    pub const DEPTH_FLAGS_DROPPED: Self = Self(45);
    /// Stored code for `FlipbookDropped`.
    pub const FLIPBOOK_DROPPED: Self = Self(46);
    /// Stored code for `CompositeEmitted`.
    pub const COMPOSITE_EMITTED: Self = Self(47);
    /// Stored code for `PassOrderFolded`.
    pub const PASS_ORDER_FOLDED: Self = Self(48);
    /// Stored code for `FresnelFolded`.
    pub const FRESNEL_FOLDED: Self = Self(49);
    /// Stored code for `ProfileNotCarried`.
    pub const PROFILE_NOT_CARRIED: Self = Self(50);
    /// Stored code for `ProfileCoverageIncomplete`.
    pub const PROFILE_COVERAGE_INCOMPLETE: Self = Self(51);
    /// Stored code for `BoneInfluenceLimit`.
    pub const BONE_INFLUENCE_LIMIT: Self = Self(52);
    /// Stored code for `UvSetLimit`.
    pub const UV_SET_LIMIT: Self = Self(53);
    /// Stored code for `IndexWidthExceeded`.
    pub const INDEX_WIDTH_EXCEEDED: Self = Self(54);
    /// Stored code for `NgonUnsupported`.
    pub const NGON_UNSUPPORTED: Self = Self(55);
    /// Stored code for `VertexColorUnsupported`.
    pub const VERTEX_COLOR_UNSUPPORTED: Self = Self(56);
    /// Stored code for `NativeKindProfileMismatch`.
    pub const NATIVE_KIND_PROFILE_MISMATCH: Self = Self(57);
    /// Stored code for `BonePaletteLimit`.
    pub const BONE_PALETTE_LIMIT: Self = Self(58);
    /// Stored code for `GeometryMismatch`.
    pub const GEOMETRY_MISMATCH: Self = Self(59);
    /// Stored code for `SectionUndrawn`.
    pub const SECTION_UNDRAWN: Self = Self(60);
    /// Stored code for `NodeRemovalRefused`.
    pub const NODE_REMOVAL_REFUSED: Self = Self(61);
    /// Stored code for `SkinInfluenceReassigned`.
    pub const SKIN_INFLUENCE_REASSIGNED: Self = Self(62);
    /// Stored code for `DanglingNodeReference`.
    pub const DANGLING_NODE_REFERENCE: Self = Self(63);
    /// Stored code for `BindPoseRecomposed`.
    pub const BIND_POSE_RECOMPOSED: Self = Self(64);
    /// Stored code for `RigConventionChanged`.
    pub const RIG_CONVENTION_CHANGED: Self = Self(65);
    /// Stored code for `BoneShearSplit`.
    pub const BONE_SHEAR_SPLIT: Self = Self(66);
    /// Stored code for `BoneShearProjected`.
    pub const BONE_SHEAR_PROJECTED: Self = Self(67);
    /// Stored code for `NonUniformScaleFlattened`.
    pub const NON_UNIFORM_SCALE_FLATTENED: Self = Self(68);
    /// Stored code for `NodeKindNotCarried`.
    pub const NODE_KIND_NOT_CARRIED: Self = Self(69);
    /// Stored code for `MixedInterpolationInTrack`.
    pub const MIXED_INTERPOLATION_IN_TRACK: Self = Self(70);
    /// Stored code for `AnimChannelInvalidated`.
    pub const ANIM_CHANNEL_INVALIDATED: Self = Self(71);
    /// Stored code for `ClipTargetMissing`.
    pub const CLIP_TARGET_MISSING: Self = Self(72);
    /// Stored code for `AnimTrackDropped`.
    pub const ANIM_TRACK_DROPPED: Self = Self(73);
    /// Stored code for `AnimTrackApproximated`.
    pub const ANIM_TRACK_APPROXIMATED: Self = Self(74);
    /// Stored code for `AnimClipRetimed`.
    pub const ANIM_CLIP_RETIMED: Self = Self(75);
    /// Stored code for `AnimTcbBaked`.
    pub const ANIM_TCB_BAKED: Self = Self(76);
    /// Stored code for `TrackSetBelowDefault`.
    pub const TRACK_SET_BELOW_DEFAULT: Self = Self(77);
    /// Stored code for `GlobalAndClipChannel`.
    pub const GLOBAL_AND_CLIP_CHANNEL: Self = Self(78);
    /// Stored code for `AnimTrackResampled`.
    pub const ANIM_TRACK_RESAMPLED: Self = Self(79);
    /// Stored code for `AnimLayersFlattened`.
    pub const ANIM_LAYERS_FLATTENED: Self = Self(80);
    /// Stored code for `AnimStageBaked`.
    pub const ANIM_STAGE_BAKED: Self = Self(81);
    /// Stored code for `AnimStageNotBaked`.
    pub const ANIM_STAGE_NOT_BAKED: Self = Self(82);
    /// Stored code for `ClipRuleConverted`.
    pub const CLIP_RULE_CONVERTED: Self = Self(83);
    /// Stored code for `AnimStageInvalid`.
    pub const ANIM_STAGE_INVALID: Self = Self(84);
    /// Stored code for `EventPayloadMismatch`.
    pub const EVENT_PAYLOAD_MISMATCH: Self = Self(85);
    /// Stored code for `AssetUnresolved`.
    pub const ASSET_UNRESOLVED: Self = Self(86);
    /// Stored code for `HardpointUnresolved`.
    pub const HARDPOINT_UNRESOLVED: Self = Self(87);
    /// Stored code for `UnknownChunkPreserved`.
    pub const UNKNOWN_CHUNK_PRESERVED: Self = Self(88);
    /// Stored code for `OrphanChunk`.
    pub const ORPHAN_CHUNK: Self = Self(89);
    /// Stored code for `UnsupportedVersion`.
    pub const UNSUPPORTED_VERSION: Self = Self(90);
    /// Stored code for `LegacyDocumentUpgraded`.
    pub const LEGACY_DOCUMENT_UPGRADED: Self = Self(91);
    /// Stored code for `OperationUnsupported`.
    pub const OPERATION_UNSUPPORTED: Self = Self(92);
    /// Stored code for `GeometryRescaled`.
    pub const GEOMETRY_RESCALED: Self = Self(93);
    /// Stored code for `LevelOfDetailDropped`.
    pub const LEVEL_OF_DETAIL_DROPPED: Self = Self(94);
    /// Stored code for `SkinProfileNotCarried`.
    pub const SKIN_PROFILE_NOT_CARRIED: Self = Self(95);
    /// Stored code for `SkinInfluenceDuplicated`.
    pub const SKIN_INFLUENCE_DUPLICATED: Self = Self(96);
    /// Stored code for `SkinWeightInvalid`.
    pub const SKIN_WEIGHT_INVALID: Self = Self(97);
    /// Stored code for `SkinInfluencesUnsorted`.
    pub const SKIN_INFLUENCES_UNSORTED: Self = Self(98);
    /// Stored code for `SkinSetupInvalid`.
    pub const SKIN_SETUP_INVALID: Self = Self(99);
    /// Stored code for `PhysicsConstraintDropped`.
    pub const PHYSICS_CONSTRAINT_DROPPED: Self = Self(100);
    /// Stored code for `PhysicsVersionRefused`.
    pub const PHYSICS_VERSION_REFUSED: Self = Self(101);
    /// Stored code for `PhysicsMaterialMerged`.
    pub const PHYSICS_MATERIAL_MERGED: Self = Self(102);
    /// Stored code for `PhysicsGravityScaleDropped`.
    pub const PHYSICS_GRAVITY_SCALE_DROPPED: Self = Self(103);
    /// Stored code for `PhysicsJointKindUnsupported`.
    pub const PHYSICS_JOINT_KIND_UNSUPPORTED: Self = Self(104);
    /// Stored code for `PhysicsJointFieldDropped`.
    pub const PHYSICS_JOINT_FIELD_DROPPED: Self = Self(105);
    /// Stored code for `PhysicsJointBodyAmbiguous`.
    pub const PHYSICS_JOINT_BODY_AMBIGUOUS: Self = Self(106);
    /// Stored code for `PhysicsHullSimplified`.
    pub const PHYSICS_HULL_SIMPLIFIED: Self = Self(107);
    /// Stored code for `PhysicsReferenceInvalid`.
    pub const PHYSICS_REFERENCE_INVALID: Self = Self(108);
    /// Stored code for `PhysicsJointSnaps`.
    pub const PHYSICS_JOINT_SNAPS: Self = Self(109);
    /// Stored code for `PhysicsShapeDegenerate`.
    pub const PHYSICS_SHAPE_DEGENERATE: Self = Self(110);
    /// Stored code for `PhysicsUnsupported`.
    pub const PHYSICS_UNSUPPORTED: Self = Self(111);
    /// Stored code for `ClothParticleLimit`.
    pub const CLOTH_PARTICLE_LIMIT: Self = Self(112);
    /// Stored code for `ClothAnchorBoneOutOfRange`.
    pub const CLOTH_ANCHOR_BONE_OUT_OF_RANGE: Self = Self(113);
    /// Stored code for `ClothSectionSplit`.
    pub const CLOTH_SECTION_SPLIT: Self = Self(114);
    /// Stored code for `ClothTopologyInvalid`.
    pub const CLOTH_TOPOLOGY_INVALID: Self = Self(115);
    /// Stored code for `PhysicsChunkDropped`.
    pub const PHYSICS_CHUNK_DROPPED: Self = Self(116);
    /// Stored code for `PhysicsRigDropped`.
    pub const PHYSICS_RIG_DROPPED: Self = Self(117);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(118);
}
/// Native ElementKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ElementKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for ElementKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ElementKind> for u8 {
    fn from(value: ElementKind) -> Self {
        value.0
    }
}
impl ElementKind {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Document`.
    pub const DOCUMENT: Self = Self(1);
    /// Stored code for `Mesh`.
    pub const MESH: Self = Self(2);
    /// Stored code for `Face`.
    pub const FACE: Self = Self(3);
    /// Stored code for `Vertex`.
    pub const VERTEX: Self = Self(4);
    /// Stored code for `Halfedge`.
    pub const HALFEDGE: Self = Self(5);
    /// Stored code for `Edge`.
    pub const EDGE: Self = Self(6);
    /// Stored code for `Section`.
    pub const SECTION: Self = Self(7);
    /// Stored code for `Node`.
    pub const NODE: Self = Self(8);
    /// Stored code for `Material`.
    pub const MATERIAL: Self = Self(9);
    /// Stored code for `Layer`.
    pub const LAYER: Self = Self(10);
    /// Stored code for `Slot`.
    pub const SLOT: Self = Self(11);
    /// Stored code for `Texture`.
    pub const TEXTURE: Self = Self(12);
    /// Stored code for `Look`.
    pub const LOOK: Self = Self(13);
    /// Stored code for `Feature`.
    pub const FEATURE: Self = Self(14);
    /// Stored code for `Actor`.
    pub const ACTOR: Self = Self(15);
    /// Stored code for `Event`.
    pub const EVENT: Self = Self(16);
    /// Stored code for `Clip`.
    pub const CLIP: Self = Self(17);
    /// Stored code for `Channel`.
    pub const CHANNEL: Self = Self(18);
    /// Stored code for `Track`.
    pub const TRACK: Self = Self(19);
    /// Stored code for `Chunk`.
    pub const CHUNK: Self = Self(20);
    /// Stored code for `PhysicsRecord`.
    pub const PHYSICS_RECORD: Self = Self(21);
}
/// Native MergeNodeAction codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct MergeNodeAction(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for MergeNodeAction {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<MergeNodeAction> for u8 {
    fn from(value: MergeNodeAction) -> Self {
        value.0
    }
}
impl MergeNodeAction {
    /// Stored code for `Add`.
    pub const ADD: Self = Self(0);
    /// Stored code for `Into`.
    pub const INTO: Self = Self(1);
    /// Stored code for `Skip`.
    pub const SKIP: Self = Self(2);
}
/// Native Channel codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Channel(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Channel {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Channel> for u8 {
    fn from(value: Channel) -> Self {
        value.0
    }
}
impl Channel {
    /// Stored code for `Translation`.
    pub const TRANSLATION: Self = Self(0);
    /// Stored code for `Rotation`.
    pub const ROTATION: Self = Self(1);
    /// Stored code for `Scale`.
    pub const SCALE: Self = Self(2);
    /// Stored code for `Visibility`.
    pub const VISIBILITY: Self = Self(3);
    /// Stored code for `Color`.
    pub const COLOR: Self = Self(4);
    /// Stored code for `Alpha`.
    pub const ALPHA: Self = Self(5);
    /// Stored code for `Intensity`.
    pub const INTENSITY: Self = Self(6);
    /// Stored code for `AttenuationStart`.
    pub const ATTENUATION_START: Self = Self(7);
    /// Stored code for `AttenuationEnd`.
    pub const ATTENUATION_END: Self = Self(8);
    /// Stored code for `UvTranslate`.
    pub const UV_TRANSLATE: Self = Self(9);
    /// Stored code for `UvRotate`.
    pub const UV_ROTATE: Self = Self(10);
    /// Stored code for `UvScale`.
    pub const UV_SCALE: Self = Self(11);
    /// Stored code for `Weight`.
    pub const WEIGHT: Self = Self(12);
    /// Stored code for `TextureIndex`.
    pub const TEXTURE_INDEX: Self = Self(13);
    /// Stored code for `Emissive`.
    pub const EMISSIVE: Self = Self(14);
    /// Stored code for `EmitterProperty`.
    pub const EMITTER_PROPERTY: Self = Self(15);
    /// Stored code for `ShadowCastingStart`.
    pub const SHADOW_CASTING_START: Self = Self(16);
    /// Stored code for `ShadowCastingEnd`.
    pub const SHADOW_CASTING_END: Self = Self(17);
    /// Stored code for `QuadraticFalloff`.
    pub const QUADRATIC_FALLOFF: Self = Self(18);
    /// Stored code for `LinearFalloff`.
    pub const LINEAR_FALLOFF: Self = Self(19);
    /// Stored code for `Damping`.
    pub const DAMPING: Self = Self(20);
    /// Stored code for `StageWeight`.
    pub const STAGE_WEIGHT: Self = Self(21);
    /// Stored code for `StageSourceWeight`.
    pub const STAGE_SOURCE_WEIGHT: Self = Self(22);
    /// Stored code for `StageSourceEnabled`.
    pub const STAGE_SOURCE_ENABLED: Self = Self(23);
    /// Stored code for `FocusDistance`.
    pub const FOCUS_DISTANCE: Self = Self(24);
    /// Stored code for `FocalLength`.
    pub const FOCAL_LENGTH: Self = Self(25);
    /// Stored code for `FStop`.
    pub const F_STOP: Self = Self(26);
    /// Stored code for `Target`.
    pub const TARGET: Self = Self(27);
    /// Stored code for `Roll`.
    pub const ROLL: Self = Self(28);
    /// Stored code for `PhysicsDynamic`.
    pub const PHYSICS_DYNAMIC: Self = Self(29);
    /// Stored code for `ClothActive`.
    pub const CLOTH_ACTIVE: Self = Self(30);
    /// Stored code for `PhysicsRagdoll`.
    pub const PHYSICS_RAGDOLL: Self = Self(31);
    /// Stored code for `PhysicsBlend`.
    pub const PHYSICS_BLEND: Self = Self(32);
    /// Stored code for `ClothDriverTranslation`.
    pub const CLOTH_DRIVER_TRANSLATION: Self = Self(33);
    /// Stored code for `ClothDriverRotation`.
    pub const CLOTH_DRIVER_ROTATION: Self = Self(34);
    /// Stored code for `NormalTextureIndex`.
    pub const NORMAL_TEXTURE_INDEX: Self = Self(35);
    /// Stored code for `OrmTextureIndex`.
    pub const ORM_TEXTURE_INDEX: Self = Self(36);
    /// Stored code for `EmissiveTextureIndex`.
    pub const EMISSIVE_TEXTURE_INDEX: Self = Self(37);
    /// Stored code for `TeamColorTextureIndex`.
    pub const TEAM_COLOR_TEXTURE_INDEX: Self = Self(38);
    /// Stored code for `EnvironmentTextureIndex`.
    pub const ENVIRONMENT_TEXTURE_INDEX: Self = Self(39);
    /// Stored code for `SoftAmount`.
    pub const SOFT_AMOUNT: Self = Self(40);
    /// Stored code for `SoftLumpTranslation`.
    pub const SOFT_LUMP_TRANSLATION: Self = Self(41);
    /// Stored code for `SoftLumpRotation`.
    pub const SOFT_LUMP_ROTATION: Self = Self(42);
    /// Stored code for `SoftLumpScale`.
    pub const SOFT_LUMP_SCALE: Self = Self(43);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(44);
}
/// Native TrackTargetKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct TrackTargetKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for TrackTargetKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<TrackTargetKind> for u8 {
    fn from(value: TrackTargetKind) -> Self {
        value.0
    }
}
impl TrackTargetKind {
    /// Stored code for `Node`.
    pub const NODE: Self = Self(0);
    /// Stored code for `MaterialLayer`.
    pub const MATERIAL_LAYER: Self = Self(1);
    /// Stored code for `MaterialFeature`.
    pub const MATERIAL_FEATURE: Self = Self(2);
    /// Stored code for `Section`.
    pub const SECTION: Self = Self(3);
    /// Stored code for `Physics`.
    pub const PHYSICS: Self = Self(4);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(5);
}
/// Native Search codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Search(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Search {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Search> for u8 {
    fn from(value: Search) -> Self {
        value.0
    }
}
impl Search {
    /// Stored code for `Balanced`.
    pub const BALANCED: Self = Self(0);
    /// Stored code for `Maximum`.
    pub const MAXIMUM: Self = Self(1);
}
/// Native Measure codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Measure(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Measure {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Measure> for u8 {
    fn from(value: Measure) -> Self {
        value.0
    }
}
impl Measure {
    /// Stored code for `Skeleton`.
    pub const SKELETON: Self = Self(0);
    /// Stored code for `Mesh`.
    pub const MESH: Self = Self(1);
}
/// Native Curve codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Curve(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Curve {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Curve> for u8 {
    fn from(value: Curve) -> Self {
        value.0
    }
}
impl Curve {
    /// Stored code for `Snapshot`.
    pub const SNAPSHOT: Self = Self(0);
    /// Stored code for `Linear`.
    pub const LINEAR: Self = Self(1);
    /// Stored code for `Smoothstep`.
    pub const SMOOTHSTEP: Self = Self(2);
}
/// Native Interpolation codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Interpolation(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Interpolation {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Interpolation> for u8 {
    fn from(value: Interpolation) -> Self {
        value.0
    }
}
impl Interpolation {
    /// Stored code for `Step`.
    pub const STEP: Self = Self(0);
    /// Stored code for `Linear`.
    pub const LINEAR: Self = Self(1);
    /// Stored code for `Hermite`.
    pub const HERMITE: Self = Self(2);
    /// Stored code for `Bezier`.
    pub const BEZIER: Self = Self(3);
    /// Stored code for `Slerp`.
    pub const SLERP: Self = Self(4);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(5);
}
/// Native ClipFlags codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ClipFlags(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for ClipFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<ClipFlags> for u32 {
    fn from(value: ClipFlags) -> Self {
        value.0
    }
}
impl ClipFlags {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `AutoPlay`.
    pub const AUTO_PLAY: Self = Self(1);
    /// Stored code for `Persistent`.
    pub const PERSISTENT: Self = Self(2);
    /// Stored code for `WorldClocked`.
    pub const WORLD_CLOCKED: Self = Self(4);
}
/// Native ReadRule codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ReadRule(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for ReadRule {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ReadRule> for u8 {
    fn from(value: ReadRule) -> Self {
        value.0
    }
}
impl ReadRule {
    /// Stored code for `Wc3`.
    pub const WC3: Self = Self(0);
    /// Stored code for `Sc2`.
    pub const SC2: Self = Self(1);
    /// Stored code for `Wow`.
    pub const WOW: Self = Self(2);
}
/// Native BakePlays codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct BakePlays(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for BakePlays {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<BakePlays> for u8 {
    fn from(value: BakePlays) -> Self {
        value.0
    }
}
impl BakePlays {
    /// Stored code for `FromClip`.
    pub const FROM_CLIP: Self = Self(0);
    /// Stored code for `Loops`.
    pub const LOOPS: Self = Self(1);
    /// Stored code for `Once`.
    pub const ONCE: Self = Self(2);
}
/// Native BakeFrom codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct BakeFrom(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for BakeFrom {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<BakeFrom> for u8 {
    fn from(value: BakeFrom) -> Self {
        value.0
    }
}
impl BakeFrom {
    /// Stored code for `Auto`.
    pub const AUTO: Self = Self(0);
    /// Stored code for `Itself`.
    pub const ITSELF: Self = Self(1);
    /// Stored code for `FirstFrame`.
    pub const FIRST_FRAME: Self = Self(2);
    /// Stored code for `Clip`.
    pub const CLIP: Self = Self(3);
}
/// Native BakeTo codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct BakeTo(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for BakeTo {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<BakeTo> for u8 {
    fn from(value: BakeTo) -> Self {
        value.0
    }
}
impl BakeTo {
    /// Stored code for `Auto`.
    pub const AUTO: Self = Self(0);
    /// Stored code for `Nothing`.
    pub const NOTHING: Self = Self(1);
    /// Stored code for `Clip`.
    pub const CLIP: Self = Self(2);
}
/// Native LoopMatch codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct LoopMatch(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for LoopMatch {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<LoopMatch> for u8 {
    fn from(value: LoopMatch) -> Self {
        value.0
    }
}
impl LoopMatch {
    /// Stored code for `Crossfade`.
    pub const CROSSFADE: Self = Self(0);
    /// Stored code for `Offset`.
    pub const OFFSET: Self = Self(1);
    /// Stored code for `Off`.
    pub const OFF: Self = Self(2);
}
/// Native MatchWhere codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct MatchWhere(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for MatchWhere {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<MatchWhere> for u8 {
    fn from(value: MatchWhere) -> Self {
        value.0
    }
}
impl MatchWhere {
    /// Stored code for `End`.
    pub const END: Self = Self(0);
    /// Stored code for `Start`.
    pub const START: Self = Self(1);
    /// Stored code for `Both`.
    pub const BOTH: Self = Self(2);
}
/// Native MatchEase codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct MatchEase(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for MatchEase {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<MatchEase> for u8 {
    fn from(value: MatchEase) -> Self {
        value.0
    }
}
impl MatchEase {
    /// Stored code for `Smooth`.
    pub const SMOOTH: Self = Self(0);
    /// Stored code for `Linear`.
    pub const LINEAR: Self = Self(1);
    /// Stored code for `EaseIn`.
    pub const EASE_IN: Self = Self(2);
    /// Stored code for `EaseOut`.
    pub const EASE_OUT: Self = Self(3);
    /// Stored code for `Custom`.
    pub const CUSTOM: Self = Self(4);
}
/// Native BakeFloor codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct BakeFloor(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for BakeFloor {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<BakeFloor> for u8 {
    fn from(value: BakeFloor) -> Self {
        value.0
    }
}
impl BakeFloor {
    /// Stored code for `Grid`.
    pub const GRID: Self = Self(0);
    /// Stored code for `Off`.
    pub const OFF: Self = Self(1);
    /// Stored code for `FollowsRoot`.
    pub const FOLLOWS_ROOT: Self = Self(2);
}
/// Native BakeWorldForceKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct BakeWorldForceKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for BakeWorldForceKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<BakeWorldForceKind> for u8 {
    fn from(value: BakeWorldForceKind) -> Self {
        value.0
    }
}
impl BakeWorldForceKind {
    /// Stored code for `Wind`.
    pub const WIND: Self = Self(0);
    /// Stored code for `Blast`.
    pub const BLAST: Self = Self(1);
}
/// Native StageKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct StageKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for StageKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<StageKind> for u8 {
    fn from(value: StageKind) -> Self {
        value.0
    }
}
impl StageKind {
    /// Stored code for `Position`.
    pub const POSITION: Self = Self(0);
    /// Stored code for `Orientation`.
    pub const ORIENTATION: Self = Self(1);
    /// Stored code for `LookAt`.
    pub const LOOK_AT: Self = Self(2);
    /// Stored code for `Link`.
    pub const LINK: Self = Self(3);
    /// Stored code for `LimbIk`.
    pub const LIMB_IK: Self = Self(4);
    /// Stored code for `ChainIk`.
    pub const CHAIN_IK: Self = Self(5);
    /// Stored code for `Spring`.
    pub const SPRING: Self = Self(6);
    /// Stored code for `Ragdoll`.
    pub const RAGDOLL: Self = Self(7);
    /// Stored code for `Cloth`.
    pub const CLOTH: Self = Self(8);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(9);
}
/// Native SurfaceRole codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SurfaceRole(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SurfaceRole {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SurfaceRole> for u8 {
    fn from(value: SurfaceRole) -> Self {
        value.0
    }
}
impl SurfaceRole {
    /// Stored code for `BaseColor`.
    pub const BASE_COLOR: Self = Self(0);
    /// Stored code for `Normal`.
    pub const NORMAL: Self = Self(1);
    /// Stored code for `Orm`.
    pub const ORM: Self = Self(2);
    /// Stored code for `Occlusion`.
    pub const OCCLUSION: Self = Self(3);
    /// Stored code for `Emissive`.
    pub const EMISSIVE: Self = Self(4);
    /// Stored code for `Specular`.
    pub const SPECULAR: Self = Self(5);
    /// Stored code for `Gloss`.
    pub const GLOSS: Self = Self(6);
    /// Stored code for `Height`.
    pub const HEIGHT: Self = Self(7);
    /// Stored code for `Metallic`.
    pub const METALLIC: Self = Self(8);
    /// Stored code for `Roughness`.
    pub const ROUGHNESS: Self = Self(9);
    /// Stored code for `Dropped`.
    pub const DROPPED: Self = Self(10);
}
/// Native FeatureKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct FeatureKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for FeatureKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<FeatureKind> for u8 {
    fn from(value: FeatureKind) -> Self {
        value.0
    }
}
impl FeatureKind {
    /// Stored code for `Fresnel`.
    pub const FRESNEL: Self = Self(0);
    /// Stored code for `UvAnimation`.
    pub const UV_ANIMATION: Self = Self(1);
    /// Stored code for `LayerShading`.
    pub const LAYER_SHADING: Self = Self(2);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(3);
}
/// Native NativeSync codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct NativeSync(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for NativeSync {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<NativeSync> for u8 {
    fn from(value: NativeSync) -> Self {
        value.0
    }
}
impl NativeSync {
    /// Stored code for `Absent`.
    pub const ABSENT: Self = Self(0);
    /// Stored code for `InSync`.
    pub const IN_SYNC: Self = Self(1);
    /// Stored code for `NativeAuthoritative`.
    pub const NATIVE_AUTHORITATIVE: Self = Self(2);
    /// Stored code for `CommonEdited`.
    pub const COMMON_EDITED: Self = Self(3);
}
/// Native MaterialFlags codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct MaterialFlags(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for MaterialFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<MaterialFlags> for u32 {
    fn from(value: MaterialFlags) -> Self {
        value.0
    }
}
impl MaterialFlags {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Unlit`.
    pub const UNLIT: Self = Self(1);
    /// Stored code for `Unfogged`.
    pub const UNFOGGED: Self = Self(2);
    /// Stored code for `NoShadowCast`.
    pub const NO_SHADOW_CAST: Self = Self(4);
    /// Stored code for `NoShadowReceive`.
    pub const NO_SHADOW_RECEIVE: Self = Self(8);
    /// Stored code for `SortNearZ`.
    pub const SORT_NEAR_Z: Self = Self(16);
    /// Stored code for `SortFarZ`.
    pub const SORT_FAR_Z: Self = Self(32);
    /// Stored code for `Invisible`.
    pub const INVISIBLE: Self = Self(64);
}
/// Native UVMappingMode codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct UVMappingMode(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for UVMappingMode {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<UVMappingMode> for u8 {
    fn from(value: UVMappingMode) -> Self {
        value.0
    }
}
impl UVMappingMode {
    /// Stored code for `ExplicitUV`.
    pub const EXPLICIT_UV: Self = Self(0);
    /// Stored code for `EnvSphere`.
    pub const ENV_SPHERE: Self = Self(1);
    /// Stored code for `EnvCube`.
    pub const ENV_CUBE: Self = Self(2);
}
/// Native SurfaceChannel codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SurfaceChannel(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SurfaceChannel {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SurfaceChannel> for u8 {
    fn from(value: SurfaceChannel) -> Self {
        value.0
    }
}
impl SurfaceChannel {
    /// Stored code for `Color`.
    pub const COLOR: Self = Self(0);
    /// Stored code for `Emissive`.
    pub const EMISSIVE: Self = Self(1);
    /// Stored code for `Specular`.
    pub const SPECULAR: Self = Self(2);
    /// Stored code for `Normal`.
    pub const NORMAL: Self = Self(3);
    /// Stored code for `AmbientOcclusion`.
    pub const AMBIENT_OCCLUSION: Self = Self(4);
    /// Stored code for `Environment`.
    pub const ENVIRONMENT: Self = Self(5);
    /// Stored code for `Coverage`.
    pub const COVERAGE: Self = Self(6);
    /// Stored code for `Gloss`.
    pub const GLOSS: Self = Self(7);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(8);
}
/// Native CompositeOp codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct CompositeOp(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for CompositeOp {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<CompositeOp> for u8 {
    fn from(value: CompositeOp) -> Self {
        value.0
    }
}
impl CompositeOp {
    /// Stored code for `Set`.
    pub const SET: Self = Self(0);
    /// Stored code for `AlphaKey`.
    pub const ALPHA_KEY: Self = Self(1);
    /// Stored code for `AlphaBlend`.
    pub const ALPHA_BLEND: Self = Self(2);
    /// Stored code for `Add`.
    pub const ADD: Self = Self(3);
    /// Stored code for `AddAlpha`.
    pub const ADD_ALPHA: Self = Self(4);
    /// Stored code for `Modulate`.
    pub const MODULATE: Self = Self(5);
    /// Stored code for `Modulate2x`.
    pub const MODULATE2X: Self = Self(6);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(7);
}
/// Native CombinerOp codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct CombinerOp(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for CombinerOp {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<CombinerOp> for u8 {
    fn from(value: CombinerOp) -> Self {
        value.0
    }
}
impl CombinerOp {
    /// Stored code for `Opaque`.
    pub const OPAQUE: Self = Self(0);
    /// Stored code for `Mod`.
    pub const MOD: Self = Self(1);
    /// Stored code for `Mod2x`.
    pub const MOD2X: Self = Self(2);
    /// Stored code for `Add`.
    pub const ADD: Self = Self(3);
    /// Stored code for `Decal`.
    pub const DECAL: Self = Self(4);
    /// Stored code for `Fade`.
    pub const FADE: Self = Self(5);
    /// Stored code for `Pass`.
    pub const PASS: Self = Self(6);
    /// Stored code for `AddAlpha`.
    pub const ADD_ALPHA: Self = Self(7);
    /// Stored code for `MaskedMod`.
    pub const MASKED_MOD: Self = Self(8);
    /// Stored code for `MaskedMod2x`.
    pub const MASKED_MOD2X: Self = Self(9);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(10);
}
/// Native LegacySlot codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct LegacySlot(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for LegacySlot {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<LegacySlot> for u8 {
    fn from(value: LegacySlot) -> Self {
        value.0
    }
}
impl LegacySlot {
    /// Stored code for `Diffuse`.
    pub const DIFFUSE: Self = Self(0);
    /// Stored code for `Normal`.
    pub const NORMAL: Self = Self(1);
    /// Stored code for `Specular`.
    pub const SPECULAR: Self = Self(2);
    /// Stored code for `Gloss`.
    pub const GLOSS: Self = Self(3);
    /// Stored code for `Emissive`.
    pub const EMISSIVE: Self = Self(4);
    /// Stored code for `Environment`.
    pub const ENVIRONMENT: Self = Self(5);
    /// Stored code for `AmbientOcclusion`.
    pub const AMBIENT_OCCLUSION: Self = Self(6);
    /// Stored code for `Height`.
    pub const HEIGHT: Self = Self(7);
    /// Stored code for `Lightmap`.
    pub const LIGHTMAP: Self = Self(8);
    /// Stored code for `Detail`.
    pub const DETAIL: Self = Self(9);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(10);
}
/// Native PbrSlot codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct PbrSlot(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for PbrSlot {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<PbrSlot> for u8 {
    fn from(value: PbrSlot) -> Self {
        value.0
    }
}
impl PbrSlot {
    /// Stored code for `BaseColor`.
    pub const BASE_COLOR: Self = Self(0);
    /// Stored code for `Normal`.
    pub const NORMAL: Self = Self(1);
    /// Stored code for `Orm`.
    pub const ORM: Self = Self(2);
    /// Stored code for `Metallic`.
    pub const METALLIC: Self = Self(3);
    /// Stored code for `Roughness`.
    pub const ROUGHNESS: Self = Self(4);
    /// Stored code for `AmbientOcclusion`.
    pub const AMBIENT_OCCLUSION: Self = Self(5);
    /// Stored code for `Emissive`.
    pub const EMISSIVE: Self = Self(6);
    /// Stored code for `Environment`.
    pub const ENVIRONMENT: Self = Self(7);
    /// Stored code for `TeamColorMask`.
    pub const TEAM_COLOR_MASK: Self = Self(8);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(9);
}
/// Native ColorSpace codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ColorSpace(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for ColorSpace {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ColorSpace> for u8 {
    fn from(value: ColorSpace) -> Self {
        value.0
    }
}
impl ColorSpace {
    /// Stored code for `Auto`.
    pub const AUTO: Self = Self(0);
    /// Stored code for `Linear`.
    pub const LINEAR: Self = Self(1);
    /// Stored code for `Srgb`.
    pub const SRGB: Self = Self(2);
}
/// Native WrapMode codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct WrapMode(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for WrapMode {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<WrapMode> for u8 {
    fn from(value: WrapMode) -> Self {
        value.0
    }
}
impl WrapMode {
    /// Stored code for `Repeat`.
    pub const REPEAT: Self = Self(0);
    /// Stored code for `Clamp`.
    pub const CLAMP: Self = Self(1);
    /// Stored code for `Mirror`.
    pub const MIRROR: Self = Self(2);
}
/// Native TextureKeyKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct TextureKeyKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for TextureKeyKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<TextureKeyKind> for u8 {
    fn from(value: TextureKeyKind) -> Self {
        value.0
    }
}
impl TextureKeyKind {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Path`.
    pub const PATH: Self = Self(1);
    /// Stored code for `FileDataId`.
    pub const FILE_DATA_ID: Self = Self(2);
    /// Stored code for `SnoId`.
    pub const SNO_ID: Self = Self(3);
}
/// Native EnvelopeFalloff codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct EnvelopeFalloff(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for EnvelopeFalloff {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<EnvelopeFalloff> for u8 {
    fn from(value: EnvelopeFalloff) -> Self {
        value.0
    }
}
impl EnvelopeFalloff {
    /// Stored code for `Hard`.
    pub const HARD: Self = Self(0);
    /// Stored code for `Linear`.
    pub const LINEAR: Self = Self(1);
    /// Stored code for `Smooth`.
    pub const SMOOTH: Self = Self(2);
    /// Stored code for `Gaussian`.
    pub const GAUSSIAN: Self = Self(3);
}
/// Native JointMotion codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct JointMotion(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for JointMotion {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<JointMotion> for u8 {
    fn from(value: JointMotion) -> Self {
        value.0
    }
}
impl JointMotion {
    /// Stored code for `Bend`.
    pub const BEND: Self = Self(0);
    /// Stored code for `Twist`.
    pub const TWIST: Self = Self(1);
    /// Stored code for `Spin`.
    pub const SPIN: Self = Self(2);
    /// Stored code for `Slide`.
    pub const SLIDE: Self = Self(3);
}
/// Native JointAxis codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct JointAxis(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for JointAxis {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<JointAxis> for u8 {
    fn from(value: JointAxis) -> Self {
        value.0
    }
}
impl JointAxis {
    /// Stored code for `X`.
    pub const X: Self = Self(0);
    /// Stored code for `Y`.
    pub const Y: Self = Self(1);
    /// Stored code for `Z`.
    pub const Z: Self = Self(2);
    /// Stored code for `Along`.
    pub const ALONG: Self = Self(3);
}
/// Native TPoseSource codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct TPoseSource(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for TPoseSource {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<TPoseSource> for u8 {
    fn from(value: TPoseSource) -> Self {
        value.0
    }
}
impl TPoseSource {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Donor`.
    pub const DONOR: Self = Self(1);
    /// Stored code for `Record`.
    pub const RECORD: Self = Self(2);
    /// Stored code for `Clips`.
    pub const CLIPS: Self = Self(3);
    /// Stored code for `BindFrame`.
    pub const BIND_FRAME: Self = Self(4);
    /// Stored code for `You`.
    pub const YOU: Self = Self(5);
}
/// Native MirrorAxis codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct MirrorAxis(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for MirrorAxis {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<MirrorAxis> for u8 {
    fn from(value: MirrorAxis) -> Self {
        value.0
    }
}
impl MirrorAxis {
    /// Stored code for `X`.
    pub const X: Self = Self(0);
    /// Stored code for `Y`.
    pub const Y: Self = Self(1);
    /// Stored code for `Z`.
    pub const Z: Self = Self(2);
}
/// Native MirrorSource codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct MirrorSource(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for MirrorSource {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<MirrorSource> for u8 {
    fn from(value: MirrorSource) -> Self {
        value.0
    }
}
impl MirrorSource {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Override`.
    pub const OVERRIDE: Self = Self(1);
    /// Stored code for `Name`.
    pub const NAME: Self = Self(2);
    /// Stored code for `Position`.
    pub const POSITION: Self = Self(3);
    /// Stored code for `Self`.
    pub const SELF: Self = Self(4);
}
/// Native NameSide codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct NameSide(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for NameSide {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<NameSide> for u8 {
    fn from(value: NameSide) -> Self {
        value.0
    }
}
impl NameSide {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Left`.
    pub const LEFT: Self = Self(1);
    /// Stored code for `Right`.
    pub const RIGHT: Self = Self(2);
}
/// Native EnvelopePreset codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct EnvelopePreset(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for EnvelopePreset {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<EnvelopePreset> for u8 {
    fn from(value: EnvelopePreset) -> Self {
        value.0
    }
}
impl EnvelopePreset {
    /// Stored code for `Mechanical`.
    pub const MECHANICAL: Self = Self(0);
    /// Stored code for `Organic`.
    pub const ORGANIC: Self = Self(1);
    /// Stored code for `Hybrid`.
    pub const HYBRID: Self = Self(2);
}
/// Native M3MaterialKind codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3MaterialKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M3MaterialKind {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M3MaterialKind> for u32 {
    fn from(value: M3MaterialKind) -> Self {
        value.0
    }
}
impl M3MaterialKind {
    /// Stored code for `Standard`.
    pub const STANDARD: Self = Self(1);
    /// Stored code for `Displacement`.
    pub const DISPLACEMENT: Self = Self(2);
    /// Stored code for `Composite`.
    pub const COMPOSITE: Self = Self(3);
    /// Stored code for `Terrain`.
    pub const TERRAIN: Self = Self(4);
    /// Stored code for `Volume`.
    pub const VOLUME: Self = Self(5);
    /// Stored code for `VolumeNoise`.
    pub const VOLUME_NOISE: Self = Self(6);
    /// Stored code for `Creep`.
    pub const CREEP: Self = Self(7);
    /// Stored code for `Hair`.
    pub const HAIR: Self = Self(8);
    /// Stored code for `SplatTerrainBake`.
    pub const SPLAT_TERRAIN_BAKE: Self = Self(9);
    /// Stored code for `Reflection`.
    pub const REFLECTION: Self = Self(10);
    /// Stored code for `LensFlare`.
    pub const LENS_FLARE: Self = Self(11);
    /// Stored code for `DataDriven`.
    pub const DATA_DRIVEN: Self = Self(12);
}
/// Native M3MaterialAdditionalFlag codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3MaterialAdditionalFlag(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M3MaterialAdditionalFlag {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M3MaterialAdditionalFlag> for u32 {
    fn from(value: M3MaterialAdditionalFlag) -> Self {
        value.0
    }
}
impl M3MaterialAdditionalFlag {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `DepthBlendFalloff`.
    pub const DEPTH_BLEND_FALLOFF: Self = Self(1);
    /// Stored code for `VertexColor`.
    pub const VERTEX_COLOR: Self = Self(4);
    /// Stored code for `VertexAlpha`.
    pub const VERTEX_ALPHA: Self = Self(8);
}
/// Native M3MaterialFlag codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3MaterialFlag(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M3MaterialFlag {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M3MaterialFlag> for u32 {
    fn from(value: M3MaterialFlag) -> Self {
        value.0
    }
}
impl M3MaterialFlag {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `VertexColor`.
    pub const VERTEX_COLOR: Self = Self(1);
    /// Stored code for `VertexAlpha`.
    pub const VERTEX_ALPHA: Self = Self(2);
    /// Stored code for `NormalBlend`.
    pub const NORMAL_BLEND: Self = Self(4);
    /// Stored code for `TwoSided`.
    pub const TWO_SIDED: Self = Self(8);
    /// Stored code for `Unshaded`.
    pub const UNSHADED: Self = Self(16);
    /// Stored code for `NoShadowsCast`.
    pub const NO_SHADOWS_CAST: Self = Self(32);
    /// Stored code for `NoHitTest`.
    pub const NO_HIT_TEST: Self = Self(64);
    /// Stored code for `NoShadowsReceive`.
    pub const NO_SHADOWS_RECEIVE: Self = Self(128);
    /// Stored code for `DepthPrepass`.
    pub const DEPTH_PREPASS: Self = Self(256);
    /// Stored code for `TerrainHDR`.
    pub const TERRAIN_HDR: Self = Self(512);
    /// Stored code for `SimulateRoughness`.
    pub const SIMULATE_ROUGHNESS: Self = Self(2048);
    /// Stored code for `PixelForwardLighting`.
    pub const PIXEL_FORWARD_LIGHTING: Self = Self(4096);
    /// Stored code for `Unfogged`.
    pub const UNFOGGED: Self = Self(8192);
    /// Stored code for `TransparentShadows`.
    pub const TRANSPARENT_SHADOWS: Self = Self(16384);
    /// Stored code for `DecalLighting`.
    pub const DECAL_LIGHTING: Self = Self(32768);
    /// Stored code for `TransparentDepthEffects`.
    pub const TRANSPARENT_DEPTH_EFFECTS: Self = Self(65536);
    /// Stored code for `TransparentLocalLights`.
    pub const TRANSPARENT_LOCAL_LIGHTS: Self = Self(131072);
    /// Stored code for `DisableSoft`.
    pub const DISABLE_SOFT: Self = Self(262144);
    /// Stored code for `DoubleLambert`.
    pub const DOUBLE_LAMBERT: Self = Self(524288);
    /// Stored code for `HairLayerSorting`.
    pub const HAIR_LAYER_SORTING: Self = Self(1048576);
    /// Stored code for `AcceptSplats`.
    pub const ACCEPT_SPLATS: Self = Self(2097152);
    /// Stored code for `DecalLowRequired`.
    pub const DECAL_LOW_REQUIRED: Self = Self(4194304);
    /// Stored code for `EmisLowRequired`.
    pub const EMIS_LOW_REQUIRED: Self = Self(8388608);
    /// Stored code for `SpecLowRequired`.
    pub const SPEC_LOW_REQUIRED: Self = Self(16777216);
    /// Stored code for `AcceptSplatsOnly`.
    pub const ACCEPT_SPLATS_ONLY: Self = Self(33554432);
    /// Stored code for `BackgroundObject`.
    pub const BACKGROUND_OBJECT: Self = Self(67108864);
    /// Stored code for `NormalBlend2`.
    pub const NORMAL_BLEND2: Self = Self(134217728);
    /// Stored code for `DepthPrepassLowRequired`.
    pub const DEPTH_PREPASS_LOW_REQUIRED: Self = Self(268435456);
    /// Stored code for `NoHighlighting`.
    pub const NO_HIGHLIGHTING: Self = Self(536870912);
    /// Stored code for `ClampOutput`.
    pub const CLAMP_OUTPUT: Self = Self(1073741824);
    /// Stored code for `GeometryVisible`.
    pub const GEOMETRY_VISIBLE: Self = Self(2147483648);
}
/// Native M3BlendMode codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3BlendMode(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M3BlendMode {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M3BlendMode> for u32 {
    fn from(value: M3BlendMode) -> Self {
        value.0
    }
}
impl M3BlendMode {
    /// Stored code for `Opaque`.
    pub const OPAQUE: Self = Self(0);
    /// Stored code for `AlphaBlend`.
    pub const ALPHA_BLEND: Self = Self(1);
    /// Stored code for `Add`.
    pub const ADD: Self = Self(2);
    /// Stored code for `AlphaAdd`.
    pub const ALPHA_ADD: Self = Self(3);
    /// Stored code for `Mod`.
    pub const MOD: Self = Self(4);
    /// Stored code for `Mod2x`.
    pub const MOD2X: Self = Self(5);
}
/// Native M3TextureLayerFlag codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3TextureLayerFlag(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M3TextureLayerFlag {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M3TextureLayerFlag> for u32 {
    fn from(value: M3TextureLayerFlag) -> Self {
        value.0
    }
}
impl M3TextureLayerFlag {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `UVWrapX`.
    pub const UV_WRAP_X: Self = Self(4);
    /// Stored code for `UVWrapY`.
    pub const UV_WRAP_Y: Self = Self(8);
    /// Stored code for `ColorInvert`.
    pub const COLOR_INVERT: Self = Self(16);
    /// Stored code for `ColorClamp`.
    pub const COLOR_CLAMP: Self = Self(32);
    /// Stored code for `ColorAdd`.
    pub const COLOR_ADD: Self = Self(64);
    /// Stored code for `ColorMultiply`.
    pub const COLOR_MULTIPLY: Self = Self(128);
    /// Stored code for `ParticleUVFlipbook`.
    pub const PARTICLE_UV_FLIPBOOK: Self = Self(256);
    /// Stored code for `Video`.
    pub const VIDEO: Self = Self(512);
    /// Stored code for `Color`.
    pub const COLOR: Self = Self(1024);
    /// Stored code for `ReplaceTextureSource`.
    pub const REPLACE_TEXTURE_SOURCE: Self = Self(2048);
    /// Stored code for `FresnelTransform`.
    pub const FRESNEL_TRANSFORM: Self = Self(16384);
    /// Stored code for `FresnelNormalize`.
    pub const FRESNEL_NORMALIZE: Self = Self(32768);
}
/// Native M3UVMappingMode codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3UVMappingMode(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M3UVMappingMode {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M3UVMappingMode> for u32 {
    fn from(value: M3UVMappingMode) -> Self {
        value.0
    }
}
impl M3UVMappingMode {
    /// Stored code for `ExplicitUV0`.
    pub const EXPLICIT_UV0: Self = Self(0);
    /// Stored code for `ExplicitUV1`.
    pub const EXPLICIT_UV1: Self = Self(1);
    /// Stored code for `ReflectCubicEnvio`.
    pub const REFLECT_CUBIC_ENVIO: Self = Self(2);
    /// Stored code for `ReflectSphericalEnvio`.
    pub const REFLECT_SPHERICAL_ENVIO: Self = Self(3);
    /// Stored code for `PlanarLocalZ`.
    pub const PLANAR_LOCAL_Z: Self = Self(4);
    /// Stored code for `PlanarWorldZ`.
    pub const PLANAR_WORLD_Z: Self = Self(5);
    /// Stored code for `ParticleFlipbook`.
    pub const PARTICLE_FLIPBOOK: Self = Self(6);
    /// Stored code for `CubicEnvio`.
    pub const CUBIC_ENVIO: Self = Self(7);
    /// Stored code for `SphericalEnvio`.
    pub const SPHERICAL_ENVIO: Self = Self(8);
    /// Stored code for `ExplicitUV2`.
    pub const EXPLICIT_UV2: Self = Self(9);
    /// Stored code for `ExplicitUV3`.
    pub const EXPLICIT_UV3: Self = Self(10);
    /// Stored code for `PlanarLocalX`.
    pub const PLANAR_LOCAL_X: Self = Self(11);
    /// Stored code for `PlanarLocalY`.
    pub const PLANAR_LOCAL_Y: Self = Self(12);
    /// Stored code for `PlanarWorldX`.
    pub const PLANAR_WORLD_X: Self = Self(13);
    /// Stored code for `PlanarWorldY`.
    pub const PLANAR_WORLD_Y: Self = Self(14);
    /// Stored code for `ScreenSpace`.
    pub const SCREEN_SPACE: Self = Self(15);
    /// Stored code for `TriPlanarLocal`.
    pub const TRI_PLANAR_LOCAL: Self = Self(16);
    /// Stored code for `TriPlanarWorld`.
    pub const TRI_PLANAR_WORLD: Self = Self(17);
    /// Stored code for `TriPlanarWorldLocalZ`.
    pub const TRI_PLANAR_WORLD_LOCAL_Z: Self = Self(18);
}
/// Native M3ColorChannelSelect codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3ColorChannelSelect(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M3ColorChannelSelect {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M3ColorChannelSelect> for u32 {
    fn from(value: M3ColorChannelSelect) -> Self {
        value.0
    }
}
impl M3ColorChannelSelect {
    /// Stored code for `RGB`.
    pub const RGB: Self = Self(0);
    /// Stored code for `RGBA`.
    pub const RGBA: Self = Self(1);
    /// Stored code for `Alpha`.
    pub const ALPHA: Self = Self(2);
    /// Stored code for `Red`.
    pub const RED: Self = Self(3);
    /// Stored code for `Green`.
    pub const GREEN: Self = Self(4);
    /// Stored code for `Blue`.
    pub const BLUE: Self = Self(5);
}
/// Native M3FresnelMode codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3FresnelMode(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M3FresnelMode {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M3FresnelMode> for u32 {
    fn from(value: M3FresnelMode) -> Self {
        value.0
    }
}
impl M3FresnelMode {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Standard`.
    pub const STANDARD: Self = Self(1);
    /// Stored code for `Inverted`.
    pub const INVERTED: Self = Self(2);
}
/// Native M3MaterialClass codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3MaterialClass(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M3MaterialClass {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M3MaterialClass> for u32 {
    fn from(value: M3MaterialClass) -> Self {
        value.0
    }
}
impl M3MaterialClass {
    /// Stored code for `Unit`.
    pub const UNIT: Self = Self(0);
    /// Stored code for `Building`.
    pub const BUILDING: Self = Self(1);
    /// Stored code for `Doodad`.
    pub const DOODAD: Self = Self(2);
    /// Stored code for `SpecialFX`.
    pub const SPECIAL_FX: Self = Self(3);
}
/// Native M3LayerBlendOp codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3LayerBlendOp(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M3LayerBlendOp {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M3LayerBlendOp> for u32 {
    fn from(value: M3LayerBlendOp) -> Self {
        value.0
    }
}
impl M3LayerBlendOp {
    /// Stored code for `Mod`.
    pub const MOD: Self = Self(0);
    /// Stored code for `Mod2x`.
    pub const MOD2X: Self = Self(1);
    /// Stored code for `Add`.
    pub const ADD: Self = Self(2);
    /// Stored code for `Lerp`.
    pub const LERP: Self = Self(3);
    /// Stored code for `TeamColorEmissiveAdd`.
    pub const TEAM_COLOR_EMISSIVE_ADD: Self = Self(4);
    /// Stored code for `TeamColorDiffuseAdd`.
    pub const TEAM_COLOR_DIFFUSE_ADD: Self = Self(5);
    /// Stored code for `AddNoAlpha`.
    pub const ADD_NO_ALPHA: Self = Self(6);
}
/// Native M3SpecularMode codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3SpecularMode(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M3SpecularMode {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M3SpecularMode> for u32 {
    fn from(value: M3SpecularMode) -> Self {
        value.0
    }
}
impl M3SpecularMode {
    /// Stored code for `RGB`.
    pub const RGB: Self = Self(0);
    /// Stored code for `AlphaOnly`.
    pub const ALPHA_ONLY: Self = Self(1);
}
/// Native M3VolumeFalloffType codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3VolumeFalloffType(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M3VolumeFalloffType {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M3VolumeFalloffType> for u32 {
    fn from(value: M3VolumeFalloffType) -> Self {
        value.0
    }
}
impl M3VolumeFalloffType {
    /// Stored code for `Linear`.
    pub const LINEAR: Self = Self(0);
    /// Stored code for `Exponential`.
    pub const EXPONENTIAL: Self = Self(1);
}
/// Native M3VolumeNoiseCameraMode codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3VolumeNoiseCameraMode(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M3VolumeNoiseCameraMode {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M3VolumeNoiseCameraMode> for u32 {
    fn from(value: M3VolumeNoiseCameraMode) -> Self {
        value.0
    }
}
impl M3VolumeNoiseCameraMode {
    /// Stored code for `Outside`.
    pub const OUTSIDE: Self = Self(0);
    /// Stored code for `Inside`.
    pub const INSIDE: Self = Self(1);
}
/// Native M3VolumeNoiseMaterialFlag codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3VolumeNoiseMaterialFlag(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M3VolumeNoiseMaterialFlag {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M3VolumeNoiseMaterialFlag> for u32 {
    fn from(value: M3VolumeNoiseMaterialFlag) -> Self {
        value.0
    }
}
impl M3VolumeNoiseMaterialFlag {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `DrawAfterTransparency`.
    pub const DRAW_AFTER_TRANSPARENCY: Self = Self(1);
}
/// Native M3ReflectionMaterialFlag codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3ReflectionMaterialFlag(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M3ReflectionMaterialFlag {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M3ReflectionMaterialFlag> for u32 {
    fn from(value: M3ReflectionMaterialFlag) -> Self {
        value.0
    }
}
impl M3ReflectionMaterialFlag {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `UseReflectionMap`.
    pub const USE_REFLECTION_MAP: Self = Self(1);
    /// Stored code for `UseDisplacementMap`.
    pub const USE_DISPLACEMENT_MAP: Self = Self(2);
    /// Stored code for `RenderInTransparentPass`.
    pub const RENDER_IN_TRANSPARENT_PASS: Self = Self(4);
    /// Stored code for `Blurring`.
    pub const BLURRING: Self = Self(8);
    /// Stored code for `UseBlurMap`.
    pub const USE_BLUR_MAP: Self = Self(16);
}
/// Native M3MaterialShaderType codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M3MaterialShaderType(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for M3MaterialShaderType {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<M3MaterialShaderType> for u8 {
    fn from(value: M3MaterialShaderType) -> Self {
        value.0
    }
}
impl M3MaterialShaderType {
    /// Stored code for `Material`.
    pub const MATERIAL: Self = Self(0);
    /// Stored code for `MaterialMedium`.
    pub const MATERIAL_MEDIUM: Self = Self(1);
    /// Stored code for `MaterialSimple`.
    pub const MATERIAL_SIMPLE: Self = Self(2);
    /// Stored code for `MaterialParticle`.
    pub const MATERIAL_PARTICLE: Self = Self(3);
    /// Stored code for `MaterialSplat`.
    pub const MATERIAL_SPLAT: Self = Self(4);
}
/// Native MdxMaterialFlag codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct MdxMaterialFlag(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for MdxMaterialFlag {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<MdxMaterialFlag> for u32 {
    fn from(value: MdxMaterialFlag) -> Self {
        value.0
    }
}
impl MdxMaterialFlag {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `ConstantColor`.
    pub const CONSTANT_COLOR: Self = Self(1);
    /// Stored code for `TwoSided`.
    pub const TWO_SIDED: Self = Self(2);
    /// Stored code for `Unfogged`.
    pub const UNFOGGED: Self = Self(4);
    /// Stored code for `SortPrimsNearZ`.
    pub const SORT_PRIMS_NEAR_Z: Self = Self(8);
    /// Stored code for `SortPrimsFarZ`.
    pub const SORT_PRIMS_FAR_Z: Self = Self(16);
    /// Stored code for `FullResolution`.
    pub const FULL_RESOLUTION: Self = Self(32);
}
/// Native MdxFilterMode codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct MdxFilterMode(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for MdxFilterMode {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<MdxFilterMode> for u32 {
    fn from(value: MdxFilterMode) -> Self {
        value.0
    }
}
impl MdxFilterMode {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Transparent`.
    pub const TRANSPARENT: Self = Self(1);
    /// Stored code for `Blend`.
    pub const BLEND: Self = Self(2);
    /// Stored code for `Additive`.
    pub const ADDITIVE: Self = Self(3);
    /// Stored code for `AddAlpha`.
    pub const ADD_ALPHA: Self = Self(4);
    /// Stored code for `Modulate`.
    pub const MODULATE: Self = Self(5);
    /// Stored code for `Modulate2x`.
    pub const MODULATE2X: Self = Self(6);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(7);
}
/// Native MdxShadingFlag codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct MdxShadingFlag(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for MdxShadingFlag {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<MdxShadingFlag> for u32 {
    fn from(value: MdxShadingFlag) -> Self {
        value.0
    }
}
impl MdxShadingFlag {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Unshaded`.
    pub const UNSHADED: Self = Self(1);
    /// Stored code for `SphereEnvMap`.
    pub const SPHERE_ENV_MAP: Self = Self(2);
    /// Stored code for `WrapWidth`.
    pub const WRAP_WIDTH: Self = Self(4);
    /// Stored code for `WrapHeight`.
    pub const WRAP_HEIGHT: Self = Self(8);
    /// Stored code for `TwoSided`.
    pub const TWO_SIDED: Self = Self(16);
    /// Stored code for `Unfogged`.
    pub const UNFOGGED: Self = Self(32);
    /// Stored code for `NoDepthTest`.
    pub const NO_DEPTH_TEST: Self = Self(64);
    /// Stored code for `NoDepthSet`.
    pub const NO_DEPTH_SET: Self = Self(128);
    /// Stored code for `Unlit`.
    pub const UNLIT: Self = Self(256);
    /// Stored code for `BackFacesForShadows`.
    pub const BACK_FACES_FOR_SHADOWS: Self = Self(512);
    /// Stored code for `AmbientOcclusion`.
    pub const AMBIENT_OCCLUSION: Self = Self(1024);
}
/// Native MdxShaderType codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct MdxShaderType(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for MdxShaderType {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<MdxShaderType> for u32 {
    fn from(value: MdxShaderType) -> Self {
        value.0
    }
}
impl MdxShaderType {
    /// Stored code for `SD`.
    pub const SD: Self = Self(0);
    /// Stored code for `HD`.
    pub const HD: Self = Self(1);
    /// Stored code for `SDOnHD`.
    pub const SD_ON_HD: Self = Self(2);
    /// Stored code for `Terrain`.
    pub const TERRAIN: Self = Self(3);
    /// Stored code for `Water`.
    pub const WATER: Self = Self(4);
    /// Stored code for `Fog`.
    pub const FOG: Self = Self(5);
    /// Stored code for `Foliage`.
    pub const FOLIAGE: Self = Self(6);
    /// Stored code for `FoliagePush`.
    pub const FOLIAGE_PUSH: Self = Self(7);
    /// Stored code for `Sprite`.
    pub const SPRITE: Self = Self(8);
    /// Stored code for `DebugTexture`.
    pub const DEBUG_TEXTURE: Self = Self(9);
    /// Stored code for `DepthOfField`.
    pub const DEPTH_OF_FIELD: Self = Self(10);
    /// Stored code for `BloomCombine`.
    pub const BLOOM_COMBINE: Self = Self(11);
    /// Stored code for `BloomExtract`.
    pub const BLOOM_EXTRACT: Self = Self(12);
    /// Stored code for `GaussianBlur`.
    pub const GAUSSIAN_BLUR: Self = Self(13);
    /// Stored code for `Tonemap`.
    pub const TONEMAP: Self = Self(14);
    /// Stored code for `Movie`.
    pub const MOVIE: Self = Self(15);
    /// Stored code for `FFXCMAAEdge0`.
    pub const FFXCMAA_EDGE0: Self = Self(16);
    /// Stored code for `FFXCMAAEdge1`.
    pub const FFXCMAA_EDGE1: Self = Self(17);
    /// Stored code for `FFXCMAAEdgeCombine`.
    pub const FFXCMAA_EDGE_COMBINE: Self = Self(18);
    /// Stored code for `FFXCMAAProcessAndApply`.
    pub const FFXCMAA_PROCESS_AND_APPLY: Self = Self(19);
    /// Stored code for `PopcornFX`.
    pub const POPCORN_FX: Self = Self(20);
    /// Stored code for `ConeIndicator`.
    pub const CONE_INDICATOR: Self = Self(21);
    /// Stored code for `CliffBlightMiscTerrain`.
    pub const CLIFF_BLIGHT_MISC_TERRAIN: Self = Self(22);
    /// Stored code for `Distortion`.
    pub const DISTORTION: Self = Self(23);
    /// Stored code for `Crystal`.
    pub const CRYSTAL: Self = Self(24);
    /// Stored code for `Imgui`.
    pub const IMGUI: Self = Self(25);
}
/// Native MdxSlotType codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct MdxSlotType(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for MdxSlotType {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<MdxSlotType> for u32 {
    fn from(value: MdxSlotType) -> Self {
        value.0
    }
}
impl MdxSlotType {
    /// Stored code for `DiffuseMap`.
    pub const DIFFUSE_MAP: Self = Self(0);
    /// Stored code for `NormalMap`.
    pub const NORMAL_MAP: Self = Self(1);
    /// Stored code for `ORMMap`.
    pub const ORM_MAP: Self = Self(2);
    /// Stored code for `EmissiveMap`.
    pub const EMISSIVE_MAP: Self = Self(3);
    /// Stored code for `TeamColor`.
    pub const TEAM_COLOR: Self = Self(4);
    /// Stored code for `EnvironmentMap`.
    pub const ENVIRONMENT_MAP: Self = Self(5);
    /// Stored code for `Unknown`.
    pub const UNKNOWN: Self = Self(6);
}
/// Native M2BlendingMode codes, represented by their stored u16 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M2BlendingMode(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u16,
);
impl From<u16> for M2BlendingMode {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<M2BlendingMode> for u16 {
    fn from(value: M2BlendingMode) -> Self {
        value.0
    }
}
impl M2BlendingMode {
    /// Stored code for `Opaque`.
    pub const OPAQUE: Self = Self(0);
    /// Stored code for `AlphaKey`.
    pub const ALPHA_KEY: Self = Self(1);
    /// Stored code for `Alpha`.
    pub const ALPHA: Self = Self(2);
    /// Stored code for `NoAlphaAdd`.
    pub const NO_ALPHA_ADD: Self = Self(3);
    /// Stored code for `Add`.
    pub const ADD: Self = Self(4);
    /// Stored code for `Mod`.
    pub const MOD: Self = Self(5);
    /// Stored code for `Mod2x`.
    pub const MOD2X: Self = Self(6);
    /// Stored code for `BlendAdd`.
    pub const BLEND_ADD: Self = Self(7);
}
/// Native D3Group codes, represented by their stored i32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct D3Group(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub i32,
);
impl From<i32> for D3Group {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
impl From<D3Group> for i32 {
    fn from(value: D3Group) -> Self {
        value.0
    }
}
impl D3Group {
    /// Stored code for `Unknown`.
    pub const UNKNOWN: Self = Self(-1);
    /// Stored code for `Actor`.
    pub const ACTOR: Self = Self(1);
    /// Stored code for `Adventure`.
    pub const ADVENTURE: Self = Self(2);
    /// Stored code for `AIBehavior`.
    pub const AI_BEHAVIOR: Self = Self(3);
    /// Stored code for `AIState`.
    pub const AI_STATE: Self = Self(4);
    /// Stored code for `AmbientSound`.
    pub const AMBIENT_SOUND: Self = Self(5);
    /// Stored code for `Anim`.
    pub const ANIM: Self = Self(6);
    /// Stored code for `Anim2D`.
    pub const ANIM2_D: Self = Self(7);
    /// Stored code for `AnimSet`.
    pub const ANIM_SET: Self = Self(8);
    /// Stored code for `Appearance`.
    pub const APPEARANCE: Self = Self(9);
    /// Stored code for `Hero`.
    pub const HERO: Self = Self(10);
    /// Stored code for `Cloth`.
    pub const CLOTH: Self = Self(11);
    /// Stored code for `Conversation`.
    pub const CONVERSATION: Self = Self(12);
    /// Stored code for `ConversationList`.
    pub const CONVERSATION_LIST: Self = Self(13);
    /// Stored code for `EffectGroup`.
    pub const EFFECT_GROUP: Self = Self(14);
    /// Stored code for `Encounter`.
    pub const ENCOUNTER: Self = Self(15);
    /// Stored code for `Explosion`.
    pub const EXPLOSION: Self = Self(17);
    /// Stored code for `FlagSet`.
    pub const FLAG_SET: Self = Self(18);
    /// Stored code for `Font`.
    pub const FONT: Self = Self(19);
    /// Stored code for `GameBalance`.
    pub const GAME_BALANCE: Self = Self(20);
    /// Stored code for `Globals`.
    pub const GLOBALS: Self = Self(21);
    /// Stored code for `LevelArea`.
    pub const LEVEL_AREA: Self = Self(22);
    /// Stored code for `Light`.
    pub const LIGHT: Self = Self(23);
    /// Stored code for `MarkerSet`.
    pub const MARKER_SET: Self = Self(24);
    /// Stored code for `Monster`.
    pub const MONSTER: Self = Self(25);
    /// Stored code for `Observer`.
    pub const OBSERVER: Self = Self(26);
    /// Stored code for `Particle`.
    pub const PARTICLE: Self = Self(27);
    /// Stored code for `Physics`.
    pub const PHYSICS: Self = Self(28);
    /// Stored code for `Power`.
    pub const POWER: Self = Self(29);
    /// Stored code for `Quest`.
    pub const QUEST: Self = Self(31);
    /// Stored code for `Rope`.
    pub const ROPE: Self = Self(32);
    /// Stored code for `Scene`.
    pub const SCENE: Self = Self(33);
    /// Stored code for `SceneGroup`.
    pub const SCENE_GROUP: Self = Self(34);
    /// Stored code for `Script`.
    pub const SCRIPT: Self = Self(35);
    /// Stored code for `ShaderMap`.
    pub const SHADER_MAP: Self = Self(36);
    /// Stored code for `Shaders`.
    pub const SHADERS: Self = Self(37);
    /// Stored code for `Shakes`.
    pub const SHAKES: Self = Self(38);
    /// Stored code for `SkillKit`.
    pub const SKILL_KIT: Self = Self(39);
    /// Stored code for `Sound`.
    pub const SOUND: Self = Self(40);
    /// Stored code for `SoundBank`.
    pub const SOUND_BANK: Self = Self(41);
    /// Stored code for `StringList`.
    pub const STRING_LIST: Self = Self(42);
    /// Stored code for `Surface`.
    pub const SURFACE: Self = Self(43);
    /// Stored code for `Textures`.
    pub const TEXTURES: Self = Self(44);
    /// Stored code for `Trail`.
    pub const TRAIL: Self = Self(45);
    /// Stored code for `UI`.
    pub const UI: Self = Self(46);
    /// Stored code for `Weather`.
    pub const WEATHER: Self = Self(47);
    /// Stored code for `Worlds`.
    pub const WORLDS: Self = Self(48);
    /// Stored code for `Recipe`.
    pub const RECIPE: Self = Self(49);
    /// Stored code for `Condition`.
    pub const CONDITION: Self = Self(51);
    /// Stored code for `TreasureClass`.
    pub const TREASURE_CLASS: Self = Self(52);
    /// Stored code for `Account`.
    pub const ACCOUNT: Self = Self(53);
    /// Stored code for `Conductor`.
    pub const CONDUCTOR: Self = Self(54);
    /// Stored code for `TimedEvent`.
    pub const TIMED_EVENT: Self = Self(55);
    /// Stored code for `Act`.
    pub const ACT: Self = Self(56);
    /// Stored code for `Material`.
    pub const MATERIAL: Self = Self(57);
    /// Stored code for `QuestRange`.
    pub const QUEST_RANGE: Self = Self(58);
    /// Stored code for `Lore`.
    pub const LORE: Self = Self(59);
    /// Stored code for `Reverb`.
    pub const REVERB: Self = Self(60);
    /// Stored code for `PhysMesh`.
    pub const PHYS_MESH: Self = Self(61);
    /// Stored code for `Music`.
    pub const MUSIC: Self = Self(62);
    /// Stored code for `Tutorial`.
    pub const TUTORIAL: Self = Self(63);
    /// Stored code for `BossEncounter`.
    pub const BOSS_ENCOUNTER: Self = Self(64);
    /// Stored code for `ControlScheme`.
    pub const CONTROL_SCHEME: Self = Self(65);
    /// Stored code for `Accolade`.
    pub const ACCOLADE: Self = Self(66);
    /// Stored code for `AnimTree`.
    pub const ANIM_TREE: Self = Self(67);
    /// Stored code for `Vibrations`.
    pub const VIBRATIONS: Self = Self(68);
    /// Stored code for `DungeonFinder`.
    pub const DUNGEON_FINDER: Self = Self(69);
}
/// Native M2MaterialFlag codes, represented by their stored u16 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M2MaterialFlag(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u16,
);
impl From<u16> for M2MaterialFlag {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<M2MaterialFlag> for u16 {
    fn from(value: M2MaterialFlag) -> Self {
        value.0
    }
}
impl M2MaterialFlag {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Unlit`.
    pub const UNLIT: Self = Self(1);
    /// Stored code for `Unfogged`.
    pub const UNFOGGED: Self = Self(2);
    /// Stored code for `TwoSided`.
    pub const TWO_SIDED: Self = Self(4);
    /// Stored code for `DepthTest`.
    pub const DEPTH_TEST: Self = Self(8);
    /// Stored code for `DepthWrite`.
    pub const DEPTH_WRITE: Self = Self(16);
    /// Stored code for `NoAlphaComposite`.
    pub const NO_ALPHA_COMPOSITE: Self = Self(2048);
}
/// Native ForceKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ForceKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for ForceKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ForceKind> for u8 {
    fn from(value: ForceKind) -> Self {
        value.0
    }
}
impl ForceKind {
    /// Stored code for `Directional`.
    pub const DIRECTIONAL: Self = Self(0);
    /// Stored code for `Radial`.
    pub const RADIAL: Self = Self(1);
    /// Stored code for `Drag`.
    pub const DRAG: Self = Self(2);
    /// Stored code for `Vortex`.
    pub const VORTEX: Self = Self(3);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(4);
}
/// Native ForceVolume codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ForceVolume(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for ForceVolume {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ForceVolume> for u8 {
    fn from(value: ForceVolume) -> Self {
        value.0
    }
}
impl ForceVolume {
    /// Stored code for `Sphere`.
    pub const SPHERE: Self = Self(0);
    /// Stored code for `Cylinder`.
    pub const CYLINDER: Self = Self(1);
    /// Stored code for `Box`.
    pub const BOX: Self = Self(2);
    /// Stored code for `Hemisphere`.
    pub const HEMISPHERE: Self = Self(3);
    /// Stored code for `Cone`.
    pub const CONE: Self = Self(4);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(5);
}
/// Native ForceFieldProperty codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ForceFieldProperty(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for ForceFieldProperty {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<ForceFieldProperty> for u32 {
    fn from(value: ForceFieldProperty) -> Self {
        value.0
    }
}
impl ForceFieldProperty {
    /// Stored code for `Strength`.
    pub const STRENGTH: Self = Self(0);
    /// Stored code for `Width`.
    pub const WIDTH: Self = Self(1);
    /// Stored code for `Height`.
    pub const HEIGHT: Self = Self(2);
    /// Stored code for `Length`.
    pub const LENGTH: Self = Self(3);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(4);
}
/// Native VertexWarpProperty codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct VertexWarpProperty(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for VertexWarpProperty {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<VertexWarpProperty> for u32 {
    fn from(value: VertexWarpProperty) -> Self {
        value.0
    }
}
impl VertexWarpProperty {
    /// Stored code for `Radius`.
    pub const RADIUS: Self = Self(0);
    /// Stored code for `Height`.
    pub const HEIGHT: Self = Self(1);
    /// Stored code for `Strength`.
    pub const STRENGTH: Self = Self(2);
    /// Stored code for `Angular`.
    pub const ANGULAR: Self = Self(3);
    /// Stored code for `Axial`.
    pub const AXIAL: Self = Self(4);
    /// Stored code for `Radial`.
    pub const RADIAL: Self = Self(5);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(6);
}
/// Native RemovePolicy codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct RemovePolicy(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for RemovePolicy {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<RemovePolicy> for u8 {
    fn from(value: RemovePolicy) -> Self {
        value.0
    }
}
impl RemovePolicy {
    /// Stored code for `ReparentChildren`.
    pub const REPARENT_CHILDREN: Self = Self(0);
    /// Stored code for `RemoveSubtree`.
    pub const REMOVE_SUBTREE: Self = Self(1);
}
/// Native SkinPolicy codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SkinPolicy(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SkinPolicy {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SkinPolicy> for u8 {
    fn from(value: SkinPolicy) -> Self {
        value.0
    }
}
impl SkinPolicy {
    /// Stored code for `Refuse`.
    pub const REFUSE: Self = Self(0);
    /// Stored code for `ReassignToParent`.
    pub const REASSIGN_TO_PARENT: Self = Self(1);
}
/// Native NodePresence codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct NodePresence(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for NodePresence {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<NodePresence> for u8 {
    fn from(value: NodePresence) -> Self {
        value.0
    }
}
impl NodePresence {
    /// Stored code for `Present`.
    pub const PRESENT: Self = Self(0);
    /// Stored code for `Placement`.
    pub const PLACEMENT: Self = Self(1);
    /// Stored code for `Absent`.
    pub const ABSENT: Self = Self(2);
}
/// Native PoseSpace codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct PoseSpace(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for PoseSpace {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<PoseSpace> for u8 {
    fn from(value: PoseSpace) -> Self {
        value.0
    }
}
impl PoseSpace {
    /// Stored code for `Model`.
    pub const MODEL: Self = Self(0);
    /// Stored code for `ParentRelative`.
    pub const PARENT_RELATIVE: Self = Self(1);
}
/// Native PoseStorage codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct PoseStorage(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for PoseStorage {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<PoseStorage> for u8 {
    fn from(value: PoseStorage) -> Self {
        value.0
    }
}
impl PoseStorage {
    /// Stored code for `Trs`.
    pub const TRS: Self = Self(0);
    /// Stored code for `Matrix`.
    pub const MATRIX: Self = Self(1);
}
/// Native NodeKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct NodeKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for NodeKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<NodeKind> for u8 {
    fn from(value: NodeKind) -> Self {
        value.0
    }
}
impl NodeKind {
    /// Stored code for `Helper`.
    pub const HELPER: Self = Self(0);
    /// Stored code for `Bone`.
    pub const BONE: Self = Self(1);
    /// Stored code for `Attachment`.
    pub const ATTACHMENT: Self = Self(2);
    /// Stored code for `Light`.
    pub const LIGHT: Self = Self(3);
    /// Stored code for `Camera`.
    pub const CAMERA: Self = Self(4);
    /// Stored code for `ParticleEmitter`.
    pub const PARTICLE_EMITTER: Self = Self(5);
    /// Stored code for `RibbonEmitter`.
    pub const RIBBON_EMITTER: Self = Self(6);
    /// Stored code for `Event`.
    pub const EVENT: Self = Self(7);
    /// Stored code for `CollisionShape`.
    pub const COLLISION_SHAPE: Self = Self(8);
    /// Stored code for `Wc3ParticleEmitter1`.
    pub const WC3_PARTICLE_EMITTER1: Self = Self(9);
    /// Stored code for `Wc3ParticleEmitter2`.
    pub const WC3_PARTICLE_EMITTER2: Self = Self(10);
    /// Stored code for `Wc3RibbonEmitter`.
    pub const WC3_RIBBON_EMITTER: Self = Self(11);
    /// Stored code for `Sc2ParticleEmitter`.
    pub const SC2_PARTICLE_EMITTER: Self = Self(12);
    /// Stored code for `Sc2RibbonEmitter`.
    pub const SC2_RIBBON_EMITTER: Self = Self(13);
    /// Stored code for `Wc3CornEmitter`.
    pub const WC3_CORN_EMITTER: Self = Self(14);
    /// Stored code for `M2ParticleEmitter`.
    pub const M2_PARTICLE_EMITTER: Self = Self(15);
    /// Stored code for `ForceField`.
    pub const FORCE_FIELD: Self = Self(16);
    /// Stored code for `VertexWarp`.
    pub const VERTEX_WARP: Self = Self(17);
    /// Stored code for `Wc3FaceFx`.
    pub const WC3_FACE_FX: Self = Self(18);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(19);
}
/// Native NodeFlags codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct NodeFlags(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for NodeFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<NodeFlags> for u32 {
    fn from(value: NodeFlags) -> Self {
        value.0
    }
}
impl NodeFlags {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Hidden`.
    pub const HIDDEN: Self = Self(1);
    /// Stored code for `DontInheritTranslation`.
    pub const DONT_INHERIT_TRANSLATION: Self = Self(2);
    /// Stored code for `DontInheritRotation`.
    pub const DONT_INHERIT_ROTATION: Self = Self(4);
    /// Stored code for `DontInheritScale`.
    pub const DONT_INHERIT_SCALE: Self = Self(8);
    /// Stored code for `Billboarded`.
    pub const BILLBOARDED: Self = Self(16);
    /// Stored code for `BillboardLockX`.
    pub const BILLBOARD_LOCK_X: Self = Self(32);
    /// Stored code for `BillboardLockY`.
    pub const BILLBOARD_LOCK_Y: Self = Self(64);
    /// Stored code for `BillboardLockZ`.
    pub const BILLBOARD_LOCK_Z: Self = Self(128);
    /// Stored code for `ModelSpace`.
    pub const MODEL_SPACE: Self = Self(256);
}
/// Native LightKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct LightKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for LightKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<LightKind> for u8 {
    fn from(value: LightKind) -> Self {
        value.0
    }
}
impl LightKind {
    /// Stored code for `Omni`.
    pub const OMNI: Self = Self(0);
    /// Stored code for `Directional`.
    pub const DIRECTIONAL: Self = Self(1);
    /// Stored code for `Spot`.
    pub const SPOT: Self = Self(2);
    /// Stored code for `Ambient`.
    pub const AMBIENT: Self = Self(3);
}
/// Native EventCategory codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct EventCategory(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for EventCategory {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<EventCategory> for u8 {
    fn from(value: EventCategory) -> Self {
        value.0
    }
}
impl EventCategory {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Sound`.
    pub const SOUND: Self = Self(1);
    /// Stored code for `Splat`.
    pub const SPLAT: Self = Self(2);
    /// Stored code for `Footprint`.
    pub const FOOTPRINT: Self = Self(3);
    /// Stored code for `UberSplat`.
    pub const UBER_SPLAT: Self = Self(4);
    /// Stored code for `SpawnModel`.
    pub const SPAWN_MODEL: Self = Self(5);
    /// Stored code for `MorphShow`.
    pub const MORPH_SHOW: Self = Self(6);
    /// Stored code for `MorphHide`.
    pub const MORPH_HIDE: Self = Self(7);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(8);
}
/// Native CollisionShapeKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct CollisionShapeKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for CollisionShapeKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<CollisionShapeKind> for u8 {
    fn from(value: CollisionShapeKind) -> Self {
        value.0
    }
}
impl CollisionShapeKind {
    /// Stored code for `Box`.
    pub const BOX: Self = Self(0);
    /// Stored code for `Sphere`.
    pub const SPHERE: Self = Self(1);
    /// Stored code for `Capsule`.
    pub const CAPSULE: Self = Self(2);
    /// Stored code for `Plane`.
    pub const PLANE: Self = Self(3);
    /// Stored code for `Cylinder`.
    pub const CYLINDER: Self = Self(4);
    /// Stored code for `Hull`.
    pub const HULL: Self = Self(5);
}
/// Native EmitterLink codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct EmitterLink(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for EmitterLink {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<EmitterLink> for u8 {
    fn from(value: EmitterLink) -> Self {
        value.0
    }
}
impl EmitterLink {
    /// Stored code for `CopySource`.
    pub const COPY_SOURCE: Self = Self(0);
    /// Stored code for `CollisionSpawn`.
    pub const COLLISION_SPAWN: Self = Self(1);
    /// Stored code for `Trail`.
    pub const TRAIL: Self = Self(2);
    /// Stored code for `BounceRibbon`.
    pub const BOUNCE_RIBBON: Self = Self(3);
    /// Stored code for `SplineBone`.
    pub const SPLINE_BONE: Self = Self(4);
    /// Stored code for `SkinMirror`.
    pub const SKIN_MIRROR: Self = Self(5);
    /// Stored code for `RigRidesWith`.
    pub const RIG_RIDES_WITH: Self = Self(6);
}
/// Native Wc3Particle1Property codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Wc3Particle1Property(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for Wc3Particle1Property {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<Wc3Particle1Property> for u32 {
    fn from(value: Wc3Particle1Property) -> Self {
        value.0
    }
}
impl Wc3Particle1Property {
    /// Stored code for `EmissionRate`.
    pub const EMISSION_RATE: Self = Self(0);
    /// Stored code for `Gravity`.
    pub const GRAVITY: Self = Self(1);
    /// Stored code for `Longitude`.
    pub const LONGITUDE: Self = Self(2);
    /// Stored code for `Latitude`.
    pub const LATITUDE: Self = Self(3);
    /// Stored code for `Lifespan`.
    pub const LIFESPAN: Self = Self(4);
    /// Stored code for `Speed`.
    pub const SPEED: Self = Self(5);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(6);
}
/// Native Wc3Particle2Property codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Wc3Particle2Property(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for Wc3Particle2Property {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<Wc3Particle2Property> for u32 {
    fn from(value: Wc3Particle2Property) -> Self {
        value.0
    }
}
impl Wc3Particle2Property {
    /// Stored code for `Speed`.
    pub const SPEED: Self = Self(0);
    /// Stored code for `Variation`.
    pub const VARIATION: Self = Self(1);
    /// Stored code for `Latitude`.
    pub const LATITUDE: Self = Self(2);
    /// Stored code for `Gravity`.
    pub const GRAVITY: Self = Self(3);
    /// Stored code for `EmissionRate`.
    pub const EMISSION_RATE: Self = Self(4);
    /// Stored code for `Width`.
    pub const WIDTH: Self = Self(5);
    /// Stored code for `Length`.
    pub const LENGTH: Self = Self(6);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(7);
}
/// Native Wc3RibbonProperty codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Wc3RibbonProperty(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for Wc3RibbonProperty {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<Wc3RibbonProperty> for u32 {
    fn from(value: Wc3RibbonProperty) -> Self {
        value.0
    }
}
impl Wc3RibbonProperty {
    /// Stored code for `HeightAbove`.
    pub const HEIGHT_ABOVE: Self = Self(0);
    /// Stored code for `HeightBelow`.
    pub const HEIGHT_BELOW: Self = Self(1);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(2);
}
/// Native Wc3CornProperty codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Wc3CornProperty(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for Wc3CornProperty {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<Wc3CornProperty> for u32 {
    fn from(value: Wc3CornProperty) -> Self {
        value.0
    }
}
impl Wc3CornProperty {
    /// Stored code for `Lifespan`.
    pub const LIFESPAN: Self = Self(0);
    /// Stored code for `EmissionRate`.
    pub const EMISSION_RATE: Self = Self(1);
    /// Stored code for `Speed`.
    pub const SPEED: Self = Self(2);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(3);
}
/// Native Sc2ParticleProperty codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Sc2ParticleProperty(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for Sc2ParticleProperty {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<Sc2ParticleProperty> for u32 {
    fn from(value: Sc2ParticleProperty) -> Self {
        value.0
    }
}
impl Sc2ParticleProperty {
    /// Stored code for `InitialSpeed`.
    pub const INITIAL_SPEED: Self = Self(0);
    /// Stored code for `InitialSpeedRandom`.
    pub const INITIAL_SPEED_RANDOM: Self = Self(1);
    /// Stored code for `InitialYaw`.
    pub const INITIAL_YAW: Self = Self(2);
    /// Stored code for `InitialPitch`.
    pub const INITIAL_PITCH: Self = Self(3);
    /// Stored code for `InitialHorizontal`.
    pub const INITIAL_HORIZONTAL: Self = Self(4);
    /// Stored code for `InitialVertical`.
    pub const INITIAL_VERTICAL: Self = Self(5);
    /// Stored code for `Lifetime`.
    pub const LIFETIME: Self = Self(6);
    /// Stored code for `LifetimeRandom`.
    pub const LIFETIME_RANDOM: Self = Self(7);
    /// Stored code for `SizeAnimation`.
    pub const SIZE_ANIMATION: Self = Self(8);
    /// Stored code for `RotationAnimation`.
    pub const ROTATION_ANIMATION: Self = Self(9);
    /// Stored code for `ColorStart`.
    pub const COLOR_START: Self = Self(10);
    /// Stored code for `ColorMid`.
    pub const COLOR_MID: Self = Self(11);
    /// Stored code for `ColorEnd`.
    pub const COLOR_END: Self = Self(12);
    /// Stored code for `EmissionRate`.
    pub const EMISSION_RATE: Self = Self(13);
    /// Stored code for `ShapeOuter`.
    pub const SHAPE_OUTER: Self = Self(14);
    /// Stored code for `ShapeInner`.
    pub const SHAPE_INNER: Self = Self(15);
    /// Stored code for `OuterRadius`.
    pub const OUTER_RADIUS: Self = Self(16);
    /// Stored code for `InnerRadius`.
    pub const INNER_RADIUS: Self = Self(17);
    /// Stored code for `SizeRandomAnimation`.
    pub const SIZE_RANDOM_ANIMATION: Self = Self(18);
    /// Stored code for `RotationRandomAnimation`.
    pub const ROTATION_RANDOM_ANIMATION: Self = Self(19);
    /// Stored code for `ColorStartRandom`.
    pub const COLOR_START_RANDOM: Self = Self(20);
    /// Stored code for `ColorMidRandom`.
    pub const COLOR_MID_RANDOM: Self = Self(21);
    /// Stored code for `ColorEndRandom`.
    pub const COLOR_END_RANDOM: Self = Self(22);
    /// Stored code for `SquirtAmount`.
    pub const SQUIRT_AMOUNT: Self = Self(23);
    /// Stored code for `PitchAmplitude`.
    pub const PITCH_AMPLITUDE: Self = Self(24);
    /// Stored code for `PitchFrequency`.
    pub const PITCH_FREQUENCY: Self = Self(25);
    /// Stored code for `YawAmplitude`.
    pub const YAW_AMPLITUDE: Self = Self(26);
    /// Stored code for `YawFrequency`.
    pub const YAW_FREQUENCY: Self = Self(27);
    /// Stored code for `SpeedAmplitude`.
    pub const SPEED_AMPLITUDE: Self = Self(28);
    /// Stored code for `SpeedFrequency`.
    pub const SPEED_FREQUENCY: Self = Self(29);
    /// Stored code for `SizeAmplitude`.
    pub const SIZE_AMPLITUDE: Self = Self(30);
    /// Stored code for `SizeFrequency`.
    pub const SIZE_FREQUENCY: Self = Self(31);
    /// Stored code for `AlphaAmplitude`.
    pub const ALPHA_AMPLITUDE: Self = Self(32);
    /// Stored code for `AlphaFrequency`.
    pub const ALPHA_FREQUENCY: Self = Self(33);
    /// Stored code for `ColorAmplitude`.
    pub const COLOR_AMPLITUDE: Self = Self(34);
    /// Stored code for `ColorFrequency`.
    pub const COLOR_FREQUENCY: Self = Self(35);
    /// Stored code for `RotationAmplitude`.
    pub const ROTATION_AMPLITUDE: Self = Self(36);
    /// Stored code for `RotationFrequency`.
    pub const ROTATION_FREQUENCY: Self = Self(37);
    /// Stored code for `HorizontalAmplitude`.
    pub const HORIZONTAL_AMPLITUDE: Self = Self(38);
    /// Stored code for `HorizontalFrequency`.
    pub const HORIZONTAL_FREQUENCY: Self = Self(39);
    /// Stored code for `VerticalAmplitude`.
    pub const VERTICAL_AMPLITUDE: Self = Self(40);
    /// Stored code for `VerticalFrequency`.
    pub const VERTICAL_FREQUENCY: Self = Self(41);
    /// Stored code for `ParticleVelocity`.
    pub const PARTICLE_VELOCITY: Self = Self(42);
    /// Stored code for `PhaseShift`.
    pub const PHASE_SHIFT: Self = Self(43);
    /// Stored code for `AlphaThreshold`.
    pub const ALPHA_THRESHOLD: Self = Self(44);
    /// Stored code for `UvOffset`.
    pub const UV_OFFSET: Self = Self(45);
    /// Stored code for `UvAngle`.
    pub const UV_ANGLE: Self = Self(46);
    /// Stored code for `UvTiling`.
    pub const UV_TILING: Self = Self(47);
    /// Stored code for `SplinePoint`.
    pub const SPLINE_POINT: Self = Self(48);
    /// Stored code for `LowerBound`.
    pub const LOWER_BOUND: Self = Self(49);
    /// Stored code for `UpperBound`.
    pub const UPPER_BOUND: Self = Self(50);
    /// Stored code for `TrailEmissionRate`.
    pub const TRAIL_EMISSION_RATE: Self = Self(51);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(52);
}
/// Native Sc2RibbonProperty codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Sc2RibbonProperty(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for Sc2RibbonProperty {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<Sc2RibbonProperty> for u32 {
    fn from(value: Sc2RibbonProperty) -> Self {
        value.0
    }
}
impl Sc2RibbonProperty {
    /// Stored code for `InitialSpeed`.
    pub const INITIAL_SPEED: Self = Self(0);
    /// Stored code for `InitialSpeedRandom`.
    pub const INITIAL_SPEED_RANDOM: Self = Self(1);
    /// Stored code for `InitialYaw`.
    pub const INITIAL_YAW: Self = Self(2);
    /// Stored code for `InitialPitch`.
    pub const INITIAL_PITCH: Self = Self(3);
    /// Stored code for `InitialHorizontal`.
    pub const INITIAL_HORIZONTAL: Self = Self(4);
    /// Stored code for `InitialVertical`.
    pub const INITIAL_VERTICAL: Self = Self(5);
    /// Stored code for `Lifetime`.
    pub const LIFETIME: Self = Self(6);
    /// Stored code for `LifetimeRandom`.
    pub const LIFETIME_RANDOM: Self = Self(7);
    /// Stored code for `SizeAnimation`.
    pub const SIZE_ANIMATION: Self = Self(8);
    /// Stored code for `RotationAnimation`.
    pub const ROTATION_ANIMATION: Self = Self(9);
    /// Stored code for `ColorStart`.
    pub const COLOR_START: Self = Self(10);
    /// Stored code for `ColorMid`.
    pub const COLOR_MID: Self = Self(11);
    /// Stored code for `ColorEnd`.
    pub const COLOR_END: Self = Self(12);
    /// Stored code for `MaxLength`.
    pub const MAX_LENGTH: Self = Self(13);
    /// Stored code for `Active`.
    pub const ACTIVE: Self = Self(14);
    /// Stored code for `YawAmplitude`.
    pub const YAW_AMPLITUDE: Self = Self(15);
    /// Stored code for `YawFrequency`.
    pub const YAW_FREQUENCY: Self = Self(16);
    /// Stored code for `PitchAmplitude`.
    pub const PITCH_AMPLITUDE: Self = Self(17);
    /// Stored code for `PitchFrequency`.
    pub const PITCH_FREQUENCY: Self = Self(18);
    /// Stored code for `SpeedAmplitude`.
    pub const SPEED_AMPLITUDE: Self = Self(19);
    /// Stored code for `SpeedFrequency`.
    pub const SPEED_FREQUENCY: Self = Self(20);
    /// Stored code for `SizeAmplitude`.
    pub const SIZE_AMPLITUDE: Self = Self(21);
    /// Stored code for `SizeFrequency`.
    pub const SIZE_FREQUENCY: Self = Self(22);
    /// Stored code for `AlphaAmplitude`.
    pub const ALPHA_AMPLITUDE: Self = Self(23);
    /// Stored code for `AlphaFrequency`.
    pub const ALPHA_FREQUENCY: Self = Self(24);
    /// Stored code for `ParticleVelocity`.
    pub const PARTICLE_VELOCITY: Self = Self(25);
    /// Stored code for `Overlay`.
    pub const OVERLAY: Self = Self(26);
    /// Stored code for `SplineVelocity`.
    pub const SPLINE_VELOCITY: Self = Self(27);
    /// Stored code for `SplineVelocityBaseFactor`.
    pub const SPLINE_VELOCITY_BASE_FACTOR: Self = Self(28);
    /// Stored code for `SplineVelocityEndFactor`.
    pub const SPLINE_VELOCITY_END_FACTOR: Self = Self(29);
    /// Stored code for `SplineYawAmplitude`.
    pub const SPLINE_YAW_AMPLITUDE: Self = Self(30);
    /// Stored code for `SplineYawFrequency`.
    pub const SPLINE_YAW_FREQUENCY: Self = Self(31);
    /// Stored code for `SplinePitchAmplitude`.
    pub const SPLINE_PITCH_AMPLITUDE: Self = Self(32);
    /// Stored code for `SplinePitchFrequency`.
    pub const SPLINE_PITCH_FREQUENCY: Self = Self(33);
    /// Stored code for `SplineVelocityAmplitude`.
    pub const SPLINE_VELOCITY_AMPLITUDE: Self = Self(34);
    /// Stored code for `SplineVelocityFrequency`.
    pub const SPLINE_VELOCITY_FREQUENCY: Self = Self(35);
    /// Stored code for `SplineYaw`.
    pub const SPLINE_YAW: Self = Self(36);
    /// Stored code for `SplinePitch`.
    pub const SPLINE_PITCH: Self = Self(37);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(38);
}
/// Native Wc3ParticleFilter codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Wc3ParticleFilter(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for Wc3ParticleFilter {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<Wc3ParticleFilter> for u32 {
    fn from(value: Wc3ParticleFilter) -> Self {
        value.0
    }
}
impl Wc3ParticleFilter {
    /// Stored code for `Blend`.
    pub const BLEND: Self = Self(0);
    /// Stored code for `Additive`.
    pub const ADDITIVE: Self = Self(1);
    /// Stored code for `Modulate`.
    pub const MODULATE: Self = Self(2);
    /// Stored code for `Modulate2x`.
    pub const MODULATE2X: Self = Self(3);
    /// Stored code for `AlphaKey`.
    pub const ALPHA_KEY: Self = Self(4);
}
/// Native Wc3ParticleHeadOrTail codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Wc3ParticleHeadOrTail(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for Wc3ParticleHeadOrTail {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<Wc3ParticleHeadOrTail> for u32 {
    fn from(value: Wc3ParticleHeadOrTail) -> Self {
        value.0
    }
}
impl Wc3ParticleHeadOrTail {
    /// Stored code for `Head`.
    pub const HEAD: Self = Self(0);
    /// Stored code for `Tail`.
    pub const TAIL: Self = Self(1);
    /// Stored code for `Both`.
    pub const BOTH: Self = Self(2);
}
/// Native M2ParticleProperty codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct M2ParticleProperty(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for M2ParticleProperty {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<M2ParticleProperty> for u32 {
    fn from(value: M2ParticleProperty) -> Self {
        value.0
    }
}
impl M2ParticleProperty {
    /// Stored code for `Speed`.
    pub const SPEED: Self = Self(0);
    /// Stored code for `SpeedVariation`.
    pub const SPEED_VARIATION: Self = Self(1);
    /// Stored code for `VerticalRange`.
    pub const VERTICAL_RANGE: Self = Self(2);
    /// Stored code for `HorizontalRange`.
    pub const HORIZONTAL_RANGE: Self = Self(3);
    /// Stored code for `Gravity`.
    pub const GRAVITY: Self = Self(4);
    /// Stored code for `Lifespan`.
    pub const LIFESPAN: Self = Self(5);
    /// Stored code for `EmissionRate`.
    pub const EMISSION_RATE: Self = Self(6);
    /// Stored code for `Width`.
    pub const WIDTH: Self = Self(7);
    /// Stored code for `Length`.
    pub const LENGTH: Self = Self(8);
    /// Stored code for `ZSource`.
    pub const Z_SOURCE: Self = Self(9);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(10);
}
/// Native Sc2EmitterShape codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Sc2EmitterShape(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for Sc2EmitterShape {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<Sc2EmitterShape> for u32 {
    fn from(value: Sc2EmitterShape) -> Self {
        value.0
    }
}
impl Sc2EmitterShape {
    /// Stored code for `Point`.
    pub const POINT: Self = Self(0);
    /// Stored code for `Plane`.
    pub const PLANE: Self = Self(1);
    /// Stored code for `Sphere`.
    pub const SPHERE: Self = Self(2);
    /// Stored code for `Box`.
    pub const BOX: Self = Self(3);
    /// Stored code for `Cylinder`.
    pub const CYLINDER: Self = Self(4);
    /// Stored code for `Disc`.
    pub const DISC: Self = Self(5);
    /// Stored code for `Spline`.
    pub const SPLINE: Self = Self(6);
    /// Stored code for `Mesh`.
    pub const MESH: Self = Self(7);
}
/// Native Sc2ParticleInstance codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Sc2ParticleInstance(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for Sc2ParticleInstance {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<Sc2ParticleInstance> for u32 {
    fn from(value: Sc2ParticleInstance) -> Self {
        value.0
    }
}
impl Sc2ParticleInstance {
    /// Stored code for `Billboard`.
    pub const BILLBOARD: Self = Self(0);
    /// Stored code for `Tail`.
    pub const TAIL: Self = Self(1);
    /// Stored code for `FaceTravelDir`.
    pub const FACE_TRAVEL_DIR: Self = Self(2);
    /// Stored code for `FaceWorldDir`.
    pub const FACE_WORLD_DIR: Self = Self(3);
    /// Stored code for `SingleAxis`.
    pub const SINGLE_AXIS: Self = Self(4);
    /// Stored code for `TerrainOriented`.
    pub const TERRAIN_ORIENTED: Self = Self(5);
    /// Stored code for `TerrainDirOriented`.
    pub const TERRAIN_DIR_ORIENTED: Self = Self(6);
    /// Stored code for `EmitterOriented`.
    pub const EMITTER_ORIENTED: Self = Self(7);
    /// Stored code for `PhysicsOriented`.
    pub const PHYSICS_ORIENTED: Self = Self(8);
    /// Stored code for `Pinned`.
    pub const PINNED: Self = Self(9);
    /// Stored code for `Trail`.
    pub const TRAIL: Self = Self(10);
}
/// Native Sc2Smoothing codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Sc2Smoothing(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for Sc2Smoothing {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<Sc2Smoothing> for u32 {
    fn from(value: Sc2Smoothing) -> Self {
        value.0
    }
}
impl Sc2Smoothing {
    /// Stored code for `Linear`.
    pub const LINEAR: Self = Self(0);
    /// Stored code for `LinearSmooth`.
    pub const LINEAR_SMOOTH: Self = Self(1);
    /// Stored code for `Bezier`.
    pub const BEZIER: Self = Self(2);
    /// Stored code for `LinearWithHold`.
    pub const LINEAR_WITH_HOLD: Self = Self(3);
    /// Stored code for `BezierWithHold`.
    pub const BEZIER_WITH_HOLD: Self = Self(4);
}
/// Native Sc2RibbonType codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Sc2RibbonType(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for Sc2RibbonType {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<Sc2RibbonType> for u32 {
    fn from(value: Sc2RibbonType) -> Self {
        value.0
    }
}
impl Sc2RibbonType {
    /// Stored code for `Billboard`.
    pub const BILLBOARD: Self = Self(0);
    /// Stored code for `Planar`.
    pub const PLANAR: Self = Self(1);
    /// Stored code for `Cylinder`.
    pub const CYLINDER: Self = Self(2);
    /// Stored code for `Star`.
    pub const STAR: Self = Self(3);
}
/// Native NodeReference codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct NodeReference(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for NodeReference {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<NodeReference> for u8 {
    fn from(value: NodeReference) -> Self {
        value.0
    }
}
impl NodeReference {
    /// Stored code for `Influence`.
    pub const INFLUENCE: Self = Self(0);
    /// Stored code for `RigidNode`.
    pub const RIGID_NODE: Self = Self(1);
    /// Stored code for `VisibilityGate`.
    pub const VISIBILITY_GATE: Self = Self(2);
    /// Stored code for `Channel`.
    pub const CHANNEL: Self = Self(3);
    /// Stored code for `StageDriven`.
    pub const STAGE_DRIVEN: Self = Self(4);
    /// Stored code for `StageTarget`.
    pub const STAGE_TARGET: Self = Self(5);
    /// Stored code for `StageSource`.
    pub const STAGE_SOURCE: Self = Self(6);
    /// Stored code for `StageUp`.
    pub const STAGE_UP: Self = Self(7);
    /// Stored code for `PhysicsBody`.
    pub const PHYSICS_BODY: Self = Self(8);
    /// Stored code for `ClothCollider`.
    pub const CLOTH_COLLIDER: Self = Self(9);
    /// Stored code for `ClothBone`.
    pub const CLOTH_BONE: Self = Self(10);
    /// Stored code for `FractureHelper`.
    pub const FRACTURE_HELPER: Self = Self(11);
    /// Stored code for `FractureField`.
    pub const FRACTURE_FIELD: Self = Self(12);
    /// Stored code for `FractureWholeGate`.
    pub const FRACTURE_WHOLE_GATE: Self = Self(13);
    /// Stored code for `FracturePiecesGate`.
    pub const FRACTURE_PIECES_GATE: Self = Self(14);
    /// Stored code for `FractureBone`.
    pub const FRACTURE_BONE: Self = Self(15);
    /// Stored code for `FractureSkinNode`.
    pub const FRACTURE_SKIN_NODE: Self = Self(16);
    /// Stored code for `NodeLink`.
    pub const NODE_LINK: Self = Self(17);
    /// Stored code for `ClipEvent`.
    pub const CLIP_EVENT: Self = Self(18);
    /// Stored code for `ClipWorldCentre`.
    pub const CLIP_WORLD_CENTRE: Self = Self(19);
    /// Stored code for `SoftCarrier`.
    pub const SOFT_CARRIER: Self = Self(20);
    /// Stored code for `SoftNode`.
    pub const SOFT_NODE: Self = Self(21);
    /// Stored code for `SoftChain`.
    pub const SOFT_CHAIN: Self = Self(22);
    /// Stored code for `SoftAxisBone`.
    pub const SOFT_AXIS_BONE: Self = Self(23);
    /// Stored code for `SoftStrand`.
    pub const SOFT_STRAND: Self = Self(24);
}
/// Native SubdivideScheme codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SubdivideScheme(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SubdivideScheme {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SubdivideScheme> for u8 {
    fn from(value: SubdivideScheme) -> Self {
        value.0
    }
}
impl SubdivideScheme {
    /// Stored code for `CatmullClark`.
    pub const CATMULL_CLARK: Self = Self(0);
    /// Stored code for `Linear`.
    pub const LINEAR: Self = Self(1);
}
/// Native SmoothScheme codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SmoothScheme(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SmoothScheme {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SmoothScheme> for u8 {
    fn from(value: SmoothScheme) -> Self {
        value.0
    }
}
impl SmoothScheme {
    /// Stored code for `Laplacian`.
    pub const LAPLACIAN: Self = Self(0);
    /// Stored code for `Taubin`.
    pub const TAUBIN: Self = Self(1);
}
/// Native ModelRefusal codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ModelRefusal(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for ModelRefusal {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ModelRefusal> for u8 {
    fn from(value: ModelRefusal) -> Self {
        value.0
    }
}
impl ModelRefusal {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `EmptySelection`.
    pub const EMPTY_SELECTION: Self = Self(1);
    /// Stored code for `DifferentMeshes`.
    pub const DIFFERENT_MESHES: Self = Self(2);
    /// Stored code for `WouldPinch`.
    pub const WOULD_PINCH: Self = Self(3);
    /// Stored code for `CrackNeedsWeld`.
    pub const CRACK_NEEDS_WELD: Self = Self(4);
    /// Stored code for `NotAdjacent`.
    pub const NOT_ADJACENT: Self = Self(5);
    /// Stored code for `BorderEdgeRemove`.
    pub const BORDER_EDGE_REMOVE: Self = Self(6);
    /// Stored code for `BorderEdgeChamfer`.
    pub const BORDER_EDGE_CHAMFER: Self = Self(7);
    /// Stored code for `WouldRepeatVertex`.
    pub const WOULD_REPEAT_VERTEX: Self = Self(8);
    /// Stored code for `WouldDuplicateFace`.
    pub const WOULD_DUPLICATE_FACE: Self = Self(9);
    /// Stored code for `WouldFold`.
    pub const WOULD_FOLD: Self = Self(10);
    /// Stored code for `CollapseIllegal`.
    pub const COLLAPSE_ILLEGAL: Self = Self(11);
    /// Stored code for `TargetCollapseIllegal`.
    pub const TARGET_COLLAPSE_ILLEGAL: Self = Self(12);
    /// Stored code for `AllFaces`.
    pub const ALL_FACES: Self = Self(13);
    /// Stored code for `ZeroAmount`.
    pub const ZERO_AMOUNT: Self = Self(14);
    /// Stored code for `ConcaveTurn`.
    pub const CONCAVE_TURN: Self = Self(15);
    /// Stored code for `DiagonalOutside`.
    pub const DIAGONAL_OUTSIDE: Self = Self(16);
    /// Stored code for `AdjacentCorners`.
    pub const ADJACENT_CORNERS: Self = Self(17);
    /// Stored code for `NotOneFace`.
    pub const NOT_ONE_FACE: Self = Self(18);
    /// Stored code for `DiagonalExists`.
    pub const DIAGONAL_EXISTS: Self = Self(19);
    /// Stored code for `CapTooShort`.
    pub const CAP_TOO_SHORT: Self = Self(20);
    /// Stored code for `CrossesSection`.
    pub const CROSSES_SECTION: Self = Self(21);
    /// Stored code for `DissolveVertexIllegal`.
    pub const DISSOLVE_VERTEX_ILLEGAL: Self = Self(22);
    /// Stored code for `NothingToSlice`.
    pub const NOTHING_TO_SLICE: Self = Self(23);
    /// Stored code for `NoSourceSide`.
    pub const NO_SOURCE_SIDE: Self = Self(24);
    /// Stored code for `WindingDisagrees`.
    pub const WINDING_DISAGREES: Self = Self(25);
    /// Stored code for `LoopCountsDiffer`.
    pub const LOOP_COUNTS_DIFFER: Self = Self(26);
    /// Stored code for `SameLoop`.
    pub const SAME_LOOP: Self = Self(27);
    /// Stored code for `HingeNotOnBoundary`.
    pub const HINGE_NOT_ON_BOUNDARY: Self = Self(28);
    /// Stored code for `NotBuiltYet`.
    pub const NOT_BUILT_YET: Self = Self(29);
}
/// Native ExtrudeType codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ExtrudeType(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for ExtrudeType {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ExtrudeType> for u8 {
    fn from(value: ExtrudeType) -> Self {
        value.0
    }
}
impl ExtrudeType {
    /// Stored code for `Group`.
    pub const GROUP: Self = Self(0);
    /// Stored code for `LocalNormal`.
    pub const LOCAL_NORMAL: Self = Self(1);
    /// Stored code for `ByPolygon`.
    pub const BY_POLYGON: Self = Self(2);
}
/// Native InsetType codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct InsetType(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for InsetType {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<InsetType> for u8 {
    fn from(value: InsetType) -> Self {
        value.0
    }
}
impl InsetType {
    /// Stored code for `Group`.
    pub const GROUP: Self = Self(0);
    /// Stored code for `ByPolygon`.
    pub const BY_POLYGON: Self = Self(1);
}
/// Native SliceMode codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SliceMode(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SliceMode {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SliceMode> for u8 {
    fn from(value: SliceMode) -> Self {
        value.0
    }
}
impl SliceMode {
    /// Stored code for `Cut`.
    pub const CUT: Self = Self(0);
    /// Stored code for `Split`.
    pub const SPLIT: Self = Self(1);
    /// Stored code for `Remove`.
    pub const REMOVE: Self = Self(2);
}
/// Native RepairPolicy codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct RepairPolicy(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for RepairPolicy {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<RepairPolicy> for u8 {
    fn from(value: RepairPolicy) -> Self {
        value.0
    }
}
impl RepairPolicy {
    /// Stored code for `RepairAndRecord`.
    pub const REPAIR_AND_RECORD: Self = Self(0);
    /// Stored code for `RepairSilently`.
    pub const REPAIR_SILENTLY: Self = Self(1);
    /// Stored code for `Refuse`.
    pub const REFUSE: Self = Self(2);
}
/// Native TriangulationPolicy codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct TriangulationPolicy(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for TriangulationPolicy {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<TriangulationPolicy> for u8 {
    fn from(value: TriangulationPolicy) -> Self {
        value.0
    }
}
impl TriangulationPolicy {
    /// Stored code for `FanFromFirstHalfedge`.
    pub const FAN_FROM_FIRST_HALFEDGE: Self = Self(0);
    /// Stored code for `None`.
    pub const NONE: Self = Self(1);
    /// Stored code for `Authored`.
    pub const AUTHORED: Self = Self(2);
}
/// Native Weighting codes, represented by their stored u16 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Weighting(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u16,
);
impl From<u16> for Weighting {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<Weighting> for u16 {
    fn from(value: Weighting) -> Self {
        value.0
    }
}
impl Weighting {
    /// Stored code for `Even`.
    pub const EVEN: Self = Self(0);
    /// Stored code for `Angle`.
    pub const ANGLE: Self = Self(1);
    /// Stored code for `Area`.
    pub const AREA: Self = Self(2);
    /// Stored code for `AreaAngle`.
    pub const AREA_ANGLE: Self = Self(3);
}
/// Native Level codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Level(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Level {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Level> for u8 {
    fn from(value: Level) -> Self {
        value.0
    }
}
impl Level {
    /// Stored code for `Vertex`.
    pub const VERTEX: Self = Self(0);
    /// Stored code for `Edge`.
    pub const EDGE: Self = Self(1);
    /// Stored code for `Polygon`.
    pub const POLYGON: Self = Self(2);
    /// Stored code for `Mesh`.
    pub const MESH: Self = Self(3);
}
/// Native Status codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Status(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Status {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Status> for u8 {
    fn from(value: Status) -> Self {
        value.0
    }
}
impl Status {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Deleted`.
    pub const DELETED: Self = Self(1);
    /// Stored code for `Locked`.
    pub const LOCKED: Self = Self(2);
    /// Stored code for `Selected`.
    pub const SELECTED: Self = Self(4);
    /// Stored code for `Hidden`.
    pub const HIDDEN: Self = Self(8);
    /// Stored code for `Feature`.
    pub const FEATURE: Self = Self(16);
    /// Stored code for `Tagged`.
    pub const TAGGED: Self = Self(32);
}
/// Native Domain codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Domain(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Domain {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Domain> for u8 {
    fn from(value: Domain) -> Self {
        value.0
    }
}
impl Domain {
    /// Stored code for `Vertex`.
    pub const VERTEX: Self = Self(0);
    /// Stored code for `Halfedge`.
    pub const HALFEDGE: Self = Self(1);
    /// Stored code for `Edge`.
    pub const EDGE: Self = Self(2);
    /// Stored code for `Face`.
    pub const FACE: Self = Self(3);
    /// Stored code for `Mesh`.
    pub const MESH: Self = Self(4);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(5);
}
/// Native AttrType codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct AttrType(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for AttrType {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<AttrType> for u8 {
    fn from(value: AttrType) -> Self {
        value.0
    }
}
impl AttrType {
    /// Stored code for `F32`.
    pub const F32: Self = Self(0);
    /// Stored code for `F32x2`.
    pub const F32X2: Self = Self(1);
    /// Stored code for `F32x3`.
    pub const F32X3: Self = Self(2);
    /// Stored code for `F32x4`.
    pub const F32X4: Self = Self(3);
    /// Stored code for `U8x4`.
    pub const U8X4: Self = Self(4);
    /// Stored code for `U16`.
    pub const U16: Self = Self(5);
    /// Stored code for `U32`.
    pub const U32: Self = Self(6);
    /// Stored code for `I32`.
    pub const I32: Self = Self(7);
    /// Stored code for `Quat`.
    pub const QUAT: Self = Self(8);
    /// Stored code for `Bool`.
    pub const BOOL: Self = Self(9);
    /// Stored code for `U32x4`.
    pub const U32X4: Self = Self(10);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(11);
}
/// Native SectionFlags codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SectionFlags(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for SectionFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<SectionFlags> for u32 {
    fn from(value: SectionFlags) -> Self {
        value.0
    }
}
impl SectionFlags {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Hidden`.
    pub const HIDDEN: Self = Self(1);
    /// Stored code for `ClothSimulated`.
    pub const CLOTH_SIMULATED: Self = Self(2);
    /// Stored code for `ClothInfluenced`.
    pub const CLOTH_INFLUENCED: Self = Self(4);
    /// Stored code for `Billboard`.
    pub const BILLBOARD: Self = Self(8);
    /// Stored code for `ProjectedShadow`.
    pub const PROJECTED_SHADOW: Self = Self(16);
}
/// Native BuildError codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct BuildError(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for BuildError {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<BuildError> for u8 {
    fn from(value: BuildError) -> Self {
        value.0
    }
}
impl BuildError {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `NonManifoldEdge`.
    pub const NON_MANIFOLD_EDGE: Self = Self(1);
    /// Stored code for `NonManifoldVertex`.
    pub const NON_MANIFOLD_VERTEX: Self = Self(2);
    /// Stored code for `InconsistentWinding`.
    pub const INCONSISTENT_WINDING: Self = Self(3);
    /// Stored code for `DegenerateFace`.
    pub const DEGENERATE_FACE: Self = Self(4);
    /// Stored code for `VertexOutOfRange`.
    pub const VERTEX_OUT_OF_RANGE: Self = Self(5);
}
/// Native CutNote codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct CutNote(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for CutNote {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<CutNote> for u8 {
    fn from(value: CutNote) -> Self {
        value.0
    }
}
impl CutNote {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Stored`.
    pub const STORED: Self = Self(1);
    /// Stored code for `Folded`.
    pub const FOLDED: Self = Self(2);
    /// Stored code for `NoEar`.
    pub const NO_EAR: Self = Self(3);
    /// Stored code for `Degenerate`.
    pub const DEGENERATE: Self = Self(4);
}
/// Native ElementLevel codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ElementLevel(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for ElementLevel {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ElementLevel> for u8 {
    fn from(value: ElementLevel) -> Self {
        value.0
    }
}
impl ElementLevel {
    /// Stored code for `Vertex`.
    pub const VERTEX: Self = Self(0);
    /// Stored code for `Edge`.
    pub const EDGE: Self = Self(1);
    /// Stored code for `Border`.
    pub const BORDER: Self = Self(2);
    /// Stored code for `Face`.
    pub const FACE: Self = Self(3);
}
/// Native LinkDelimit codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct LinkDelimit(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for LinkDelimit {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<LinkDelimit> for u8 {
    fn from(value: LinkDelimit) -> Self {
        value.0
    }
}
impl LinkDelimit {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Seam`.
    pub const SEAM: Self = Self(1);
    /// Stored code for `Sharp`.
    pub const SHARP: Self = Self(2);
    /// Stored code for `Section`.
    pub const SECTION: Self = Self(4);
    /// Stored code for `Angle`.
    pub const ANGLE: Self = Self(8);
}
/// Native ConvertRule codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ConvertRule(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for ConvertRule {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ConvertRule> for u8 {
    fn from(value: ConvertRule) -> Self {
        value.0
    }
}
impl ConvertRule {
    /// Stored code for `Touching`.
    pub const TOUCHING: Self = Self(0);
    /// Stored code for `Enclosed`.
    pub const ENCLOSED: Self = Self(1);
}
/// Native RetopoMethod codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct RetopoMethod(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for RetopoMethod {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<RetopoMethod> for u8 {
    fn from(value: RetopoMethod) -> Self {
        value.0
    }
}
impl RetopoMethod {
    /// Stored code for `Field`.
    pub const FIELD: Self = Self(0);
    /// Stored code for `Layout`.
    pub const LAYOUT: Self = Self(1);
}
/// Native Failure codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Failure(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Failure {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Failure> for u8 {
    fn from(value: Failure) -> Self {
        value.0
    }
}
impl Failure {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Cancelled`.
    pub const CANCELLED: Self = Self(1);
    /// Stored code for `EmptyMesh`.
    pub const EMPTY_MESH: Self = Self(2);
    /// Stored code for `NotManifold`.
    pub const NOT_MANIFOLD: Self = Self(3);
    /// Stored code for `Unsolved`.
    pub const UNSOLVED: Self = Self(4);
}
/// Native Refusal codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Refusal(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Refusal {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Refusal> for u8 {
    fn from(value: Refusal) -> Self {
        value.0
    }
}
impl Refusal {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Closed`.
    pub const CLOSED: Self = Self(1);
    /// Stored code for `NoFaces`.
    pub const NO_FACES: Self = Self(2);
    /// Stored code for `TooFewPins`.
    pub const TOO_FEW_PINS: Self = Self(3);
}
/// Native ProjectShape codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ProjectShape(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for ProjectShape {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ProjectShape> for u8 {
    fn from(value: ProjectShape) -> Self {
        value.0
    }
}
impl ProjectShape {
    /// Stored code for `Planar`.
    pub const PLANAR: Self = Self(0);
    /// Stored code for `Cylinder`.
    pub const CYLINDER: Self = Self(1);
    /// Stored code for `Sphere`.
    pub const SPHERE: Self = Self(2);
    /// Stored code for `Box`.
    pub const BOX: Self = Self(3);
}
/// Native SeamsRefusal codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SeamsRefusal(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SeamsRefusal {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SeamsRefusal> for u8 {
    fn from(value: SeamsRefusal) -> Self {
        value.0
    }
}
impl SeamsRefusal {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `NoLimbs`.
    pub const NO_LIMBS: Self = Self(1);
}
/// Native NormalPacking codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct NormalPacking(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for NormalPacking {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<NormalPacking> for u8 {
    fn from(value: NormalPacking) -> Self {
        value.0
    }
}
impl NormalPacking {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `XInAlpha`.
    pub const X_IN_ALPHA: Self = Self(1);
    /// Stored code for `TwoChannel`.
    pub const TWO_CHANNEL: Self = Self(2);
}
/// Native Grain codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Grain(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Grain {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Grain> for u8 {
    fn from(value: Grain) -> Self {
        value.0
    }
}
impl Grain {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `X`.
    pub const X: Self = Self(1);
    /// Stored code for `Y`.
    pub const Y: Self = Self(2);
    /// Stored code for `Z`.
    pub const Z: Self = Self(3);
}
/// Native Parts codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Parts(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Parts {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Parts> for u8 {
    fn from(value: Parts) -> Self {
        value.0
    }
}
impl Parts {
    /// Stored code for `Overlap`.
    pub const OVERLAP: Self = Self(0);
    /// Stored code for `Touch`.
    pub const TOUCH: Self = Self(1);
}
/// Native FractureRefusal codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct FractureRefusal(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for FractureRefusal {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<FractureRefusal> for u8 {
    fn from(value: FractureRefusal) -> Self {
        value.0
    }
}
impl FractureRefusal {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Missing`.
    pub const MISSING: Self = Self(1);
    /// Stored code for `Empty`.
    pub const EMPTY: Self = Self(2);
    /// Stored code for `Cloth`.
    pub const CLOTH: Self = Self(3);
    /// Stored code for `Billboard`.
    pub const BILLBOARD: Self = Self(4);
    /// Stored code for `Repeated`.
    pub const REPEATED: Self = Self(5);
    /// Stored code for `NoPieces`.
    pub const NO_PIECES: Self = Self(6);
    /// Stored code for `TooMany`.
    pub const TOO_MANY: Self = Self(7);
}
/// Native OpenParts codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct OpenParts(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for OpenParts {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<OpenParts> for u8 {
    fn from(value: OpenParts) -> Self {
        value.0
    }
}
impl OpenParts {
    /// Stored code for `Solid`.
    pub const SOLID: Self = Self(0);
    /// Stored code for `Thicken`.
    pub const THICKEN: Self = Self(1);
}
/// Native Method codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct Method(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for Method {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<Method> for u8 {
    fn from(value: Method) -> Self {
        value.0
    }
}
impl Method {
    /// Stored code for `Fracture`.
    pub const FRACTURE: Self = Self(0);
    /// Stored code for `Slices`.
    pub const SLICES: Self = Self(1);
}
/// Native FillingUvs codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct FillingUvs(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for FillingUvs {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<FillingUvs> for u8 {
    fn from(value: FillingUvs) -> Self {
        value.0
    }
}
impl FillingUvs {
    /// Stored code for `Tiled`.
    pub const TILED: Self = Self(0);
    /// Stored code for `Square`.
    pub const SQUARE: Self = Self(1);
}
/// Native SoftWeights codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftWeights(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftWeights {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftWeights> for u8 {
    fn from(value: SoftWeights) -> Self {
        value.0
    }
}
impl SoftWeights {
    /// Stored code for `Float`.
    pub const FLOAT: Self = Self(0);
    /// Stored code for `Bytes`.
    pub const BYTES: Self = Self(1);
    /// Stored code for `EqualShares`.
    pub const EQUAL_SHARES: Self = Self(2);
}
/// Native SoftEffectKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftEffectKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftEffectKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftEffectKind> for u8 {
    fn from(value: SoftEffectKind) -> Self {
        value.0
    }
}
impl SoftEffectKind {
    /// Stored code for `Inflate`.
    pub const INFLATE: Self = Self(0);
    /// Stored code for `Bulge`.
    pub const BULGE: Self = Self(1);
    /// Stored code for `Jiggle`.
    pub const JIGGLE: Self = Self(2);
    /// Stored code for `Wobble`.
    pub const WOBBLE: Self = Self(3);
    /// Stored code for `Blobify`.
    pub const BLOBIFY: Self = Self(4);
    /// Stored code for `Drip`.
    pub const DRIP: Self = Self(5);
    /// Stored code for `SquashStretch`.
    pub const SQUASH_STRETCH: Self = Self(6);
    /// Stored code for `Melt`.
    pub const MELT: Self = Self(7);
    /// Stored code for `Ripple`.
    pub const RIPPLE: Self = Self(8);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(9);
}
/// Native SoftLumpKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftLumpKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftLumpKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftLumpKind> for u8 {
    fn from(value: SoftLumpKind) -> Self {
        value.0
    }
}
impl SoftLumpKind {
    /// Stored code for `Bone`.
    pub const BONE: Self = Self(0);
    /// Stored code for `Copy`.
    pub const COPY: Self = Self(1);
    /// Stored code for `Surface`.
    pub const SURFACE: Self = Self(2);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(3);
}
/// Native SoftAlong codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftAlong(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftAlong {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftAlong> for u8 {
    fn from(value: SoftAlong) -> Self {
        value.0
    }
}
impl SoftAlong {
    /// Stored code for `Surface`.
    pub const SURFACE: Self = Self(0);
    /// Stored code for `Bones`.
    pub const BONES: Self = Self(1);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(2);
}
/// Native SoftAxis codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftAxis(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftAxis {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftAxis> for u8 {
    fn from(value: SoftAxis) -> Self {
        value.0
    }
}
impl SoftAxis {
    /// Stored code for `Auto`.
    pub const AUTO: Self = Self(0);
    /// Stored code for `Up`.
    pub const UP: Self = Self(1);
    /// Stored code for `X`.
    pub const X: Self = Self(2);
    /// Stored code for `Y`.
    pub const Y: Self = Self(3);
    /// Stored code for `Z`.
    pub const Z: Self = Self(4);
    /// Stored code for `Bone`.
    pub const BONE: Self = Self(5);
    /// Stored code for `Motion`.
    pub const MOTION: Self = Self(6);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(7);
}
/// Native SoftComesFrom codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftComesFrom(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftComesFrom {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftComesFrom> for u8 {
    fn from(value: SoftComesFrom) -> Self {
        value.0
    }
}
impl SoftComesFrom {
    /// Stored code for `Spans`.
    pub const SPANS: Self = Self(0);
    /// Stored code for `Times`.
    pub const TIMES: Self = Self(1);
    /// Stored code for `Events`.
    pub const EVENTS: Self = Self(2);
    /// Stored code for `Contact`.
    pub const CONTACT: Self = Self(3);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(4);
}
/// Native SoftPulse codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftPulse(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftPulse {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftPulse> for u8 {
    fn from(value: SoftPulse) -> Self {
        value.0
    }
}
impl SoftPulse {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Breathe`.
    pub const BREATHE: Self = Self(1);
    /// Stored code for `Heartbeat`.
    pub const HEARTBEAT: Self = Self(2);
    /// Stored code for `Rate`.
    pub const RATE: Self = Self(3);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(4);
}
/// Native SoftBulgeShape codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftBulgeShape(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftBulgeShape {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftBulgeShape> for u8 {
    fn from(value: SoftBulgeShape) -> Self {
        value.0
    }
}
impl SoftBulgeShape {
    /// Stored code for `Round`.
    pub const ROUND: Self = Self(0);
    /// Stored code for `Ring`.
    pub const RING: Self = Self(1);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(2);
}
/// Native SoftJigglePreset codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftJigglePreset(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftJigglePreset {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftJigglePreset> for u8 {
    fn from(value: SoftJigglePreset) -> Self {
        value.0
    }
}
impl SoftJigglePreset {
    /// Stored code for `Custom`.
    pub const CUSTOM: Self = Self(0);
    /// Stored code for `Jelly`.
    pub const JELLY: Self = Self(1);
    /// Stored code for `Fat`.
    pub const FAT: Self = Self(2);
    /// Stored code for `Muscle`.
    pub const MUSCLE: Self = Self(3);
    /// Stored code for `Rubber`.
    pub const RUBBER: Self = Self(4);
    /// Stored code for `WaterBalloon`.
    pub const WATER_BALLOON: Self = Self(5);
    /// Stored code for `Water`.
    pub const WATER: Self = Self(6);
    /// Stored code for `Slime`.
    pub const SLIME: Self = Self(7);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(8);
}
/// Native SoftWobblePattern codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftWobblePattern(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftWobblePattern {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftWobblePattern> for u8 {
    fn from(value: SoftWobblePattern) -> Self {
        value.0
    }
}
impl SoftWobblePattern {
    /// Stored code for `Jelly`.
    pub const JELLY: Self = Self(0);
    /// Stored code for `Sway`.
    pub const SWAY: Self = Self(1);
    /// Stored code for `Roll`.
    pub const ROLL: Self = Self(2);
    /// Stored code for `Random`.
    pub const RANDOM: Self = Self(3);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(4);
}
/// Native SoftDripPreset codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftDripPreset(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftDripPreset {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftDripPreset> for u8 {
    fn from(value: SoftDripPreset) -> Self {
        value.0
    }
}
impl SoftDripPreset {
    /// Stored code for `Custom`.
    pub const CUSTOM: Self = Self(0);
    /// Stored code for `Wax`.
    pub const WAX: Self = Self(1);
    /// Stored code for `Slime`.
    pub const SLIME: Self = Self(2);
    /// Stored code for `Blood`.
    pub const BLOOD: Self = Self(3);
    /// Stored code for `Paint`.
    pub const PAINT: Self = Self(4);
    /// Stored code for `Chocolate`.
    pub const CHOCOLATE: Self = Self(5);
    /// Stored code for `Honey`.
    pub const HONEY: Self = Self(6);
    /// Stored code for `Water`.
    pub const WATER: Self = Self(7);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(8);
}
/// Native SoftNeckLaw codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftNeckLaw(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftNeckLaw {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftNeckLaw> for u8 {
    fn from(value: SoftNeckLaw) -> Self {
        value.0
    }
}
impl SoftNeckLaw {
    /// Stored code for `Linear`.
    pub const LINEAR: Self = Self(0);
    /// Stored code for `Snap`.
    pub const SNAP: Self = Self(1);
    /// Stored code for `Thread`.
    pub const THREAD: Self = Self(2);
    /// Stored code for `Sets`.
    pub const SETS: Self = Self(3);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(4);
}
/// Native SoftSquashMode codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftSquashMode(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftSquashMode {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftSquashMode> for u8 {
    fn from(value: SoftSquashMode) -> Self {
        value.0
    }
}
impl SoftSquashMode {
    /// Stored code for `Auto`.
    pub const AUTO: Self = Self(0);
    /// Stored code for `Keyed`.
    pub const KEYED: Self = Self(1);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(2);
}
/// Native SoftAnchor codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftAnchor(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftAnchor {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftAnchor> for u8 {
    fn from(value: SoftAnchor) -> Self {
        value.0
    }
}
impl SoftAnchor {
    /// Stored code for `Bottom`.
    pub const BOTTOM: Self = Self(0);
    /// Stored code for `Centre`.
    pub const CENTRE: Self = Self(1);
    /// Stored code for `Pivot`.
    pub const PIVOT: Self = Self(2);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(3);
}
/// Native SoftMeltOrder codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftMeltOrder(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftMeltOrder {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftMeltOrder> for u8 {
    fn from(value: SoftMeltOrder) -> Self {
        value.0
    }
}
impl SoftMeltOrder {
    /// Stored code for `Top`.
    pub const TOP: Self = Self(0);
    /// Stored code for `Bottom`.
    pub const BOTTOM: Self = Self(1);
    /// Stored code for `AllAtOnce`.
    pub const ALL_AT_ONCE: Self = Self(2);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(3);
}
/// Native SoftRipplePlays codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftRipplePlays(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftRipplePlays {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftRipplePlays> for u8 {
    fn from(value: SoftRipplePlays) -> Self {
        value.0
    }
}
impl SoftRipplePlays {
    /// Stored code for `Once`.
    pub const ONCE: Self = Self(0);
    /// Stored code for `Loops`.
    pub const LOOPS: Self = Self(1);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(2);
}
/// Native SoftThrough codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftThrough(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftThrough {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftThrough> for u8 {
    fn from(value: SoftThrough) -> Self {
        value.0
    }
}
impl SoftThrough {
    /// Stored code for `Skin`.
    pub const SKIN: Self = Self(0);
    /// Stored code for `Body`.
    pub const BODY: Self = Self(1);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(2);
}
/// Native SoftCeiling codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftCeiling(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftCeiling {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftCeiling> for u8 {
    fn from(value: SoftCeiling) -> Self {
        value.0
    }
}
impl SoftCeiling {
    /// Stored code for `Swing`.
    pub const SWING: Self = Self(0);
    /// Stored code for `Clip`.
    pub const CLIP: Self = Self(1);
    /// Stored code for `None`.
    pub const NONE: Self = Self(2);
}
/// Native SwitchWhy codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SwitchWhy(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SwitchWhy {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SwitchWhy> for u8 {
    fn from(value: SwitchWhy) -> Self {
        value.0
    }
}
impl SwitchWhy {
    /// Stored code for `Still`.
    pub const STILL: Self = Self(0);
    /// Stored code for `Rest`.
    pub const REST: Self = Self(1);
    /// Stored code for `Keyed`.
    pub const KEYED: Self = Self(2);
    /// Stored code for `Dropped`.
    pub const DROPPED: Self = Self(3);
    /// Stored code for `Death`.
    pub const DEATH: Self = Self(4);
    /// Stored code for `Carried`.
    pub const CARRIED: Self = Self(5);
    /// Stored code for `Inherited`.
    pub const INHERITED: Self = Self(6);
    /// Stored code for `Exempt`.
    pub const EXEMPT: Self = Self(7);
}
/// Native SwitchScope codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SwitchScope(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SwitchScope {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SwitchScope> for u8 {
    fn from(value: SwitchScope) -> Self {
        value.0
    }
}
impl SwitchScope {
    /// Stored code for `Clip`.
    pub const CLIP: Self = Self(0);
    /// Stored code for `Simulate`.
    pub const SIMULATE: Self = Self(1);
    /// Stored code for `Test`.
    pub const TEST: Self = Self(2);
}
/// Native PhysicsShapeKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct PhysicsShapeKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for PhysicsShapeKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<PhysicsShapeKind> for u8 {
    fn from(value: PhysicsShapeKind) -> Self {
        value.0
    }
}
impl PhysicsShapeKind {
    /// Stored code for `Box`.
    pub const BOX: Self = Self(0);
    /// Stored code for `Sphere`.
    pub const SPHERE: Self = Self(1);
    /// Stored code for `Capsule`.
    pub const CAPSULE: Self = Self(2);
    /// Stored code for `Cylinder`.
    pub const CYLINDER: Self = Self(3);
    /// Stored code for `ConvexHull`.
    pub const CONVEX_HULL: Self = Self(4);
    /// Stored code for `TriangleMesh`.
    pub const TRIANGLE_MESH: Self = Self(5);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(6);
}
/// Native BodyMotion codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct BodyMotion(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for BodyMotion {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<BodyMotion> for u8 {
    fn from(value: BodyMotion) -> Self {
        value.0
    }
}
impl BodyMotion {
    /// Stored code for `Dynamic`.
    pub const DYNAMIC: Self = Self(0);
    /// Stored code for `Kinematic`.
    pub const KINEMATIC: Self = Self(1);
    /// Stored code for `Static`.
    pub const STATIC: Self = Self(2);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(3);
}
/// Native JointKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct JointKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for JointKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<JointKind> for u8 {
    fn from(value: JointKind) -> Self {
        value.0
    }
}
impl JointKind {
    /// Stored code for `Spherical`.
    pub const SPHERICAL: Self = Self(0);
    /// Stored code for `Revolute`.
    pub const REVOLUTE: Self = Self(1);
    /// Stored code for `ConeTwist`.
    pub const CONE_TWIST: Self = Self(2);
    /// Stored code for `Weld`.
    pub const WELD: Self = Self(3);
    /// Stored code for `Prismatic`.
    pub const PRISMATIC: Self = Self(4);
    /// Stored code for `Distance`.
    pub const DISTANCE: Self = Self(5);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(6);
}
/// Native JointFriction codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct JointFriction(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for JointFriction {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<JointFriction> for u8 {
    fn from(value: JointFriction) -> Self {
        value.0
    }
}
impl JointFriction {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Torque`.
    pub const TORQUE: Self = Self(1);
    /// Stored code for `GravityHold`.
    pub const GRAVITY_HOLD: Self = Self(2);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(3);
}
/// Native JointMotorMode codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct JointMotorMode(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for JointMotorMode {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<JointMotorMode> for u8 {
    fn from(value: JointMotorMode) -> Self {
        value.0
    }
}
impl JointMotorMode {
    /// Stored code for `Off`.
    pub const OFF: Self = Self(0);
    /// Stored code for `Position`.
    pub const POSITION: Self = Self(1);
    /// Stored code for `Velocity`.
    pub const VELOCITY: Self = Self(2);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(3);
}
/// Native ClothColliderKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ClothColliderKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for ClothColliderKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ClothColliderKind> for u8 {
    fn from(value: ClothColliderKind) -> Self {
        value.0
    }
}
impl ClothColliderKind {
    /// Stored code for `Capsule`.
    pub const CAPSULE: Self = Self(0);
    /// Stored code for `Plane`.
    pub const PLANE: Self = Self(1);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(2);
}
/// Native ClothCageKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ClothCageKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for ClothCageKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ClothCageKind> for u8 {
    fn from(value: ClothCageKind) -> Self {
        value.0
    }
}
impl ClothCageKind {
    /// Stored code for `Auto`.
    pub const AUTO: Self = Self(0);
    /// Stored code for `AsModelled`.
    pub const AS_MODELLED: Self = Self(1);
    /// Stored code for `Grid`.
    pub const GRID: Self = Self(2);
    /// Stored code for `Strip`.
    pub const STRIP: Self = Self(3);
    /// Stored code for `Reduced`.
    pub const REDUCED: Self = Self(4);
    /// Stored code for `FromFaces`.
    pub const FROM_FACES: Self = Self(5);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(6);
}
/// Native ClothBakeInto codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct ClothBakeInto(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for ClothBakeInto {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ClothBakeInto> for u8 {
    fn from(value: ClothBakeInto) -> Self {
        value.0
    }
}
impl ClothBakeInto {
    /// Stored code for `Bones`.
    pub const BONES: Self = Self(0);
    /// Stored code for `FullDetail`.
    pub const FULL_DETAIL: Self = Self(1);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(2);
}
/// Native RigStart codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct RigStart(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for RigStart {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<RigStart> for u8 {
    fn from(value: RigStart) -> Self {
        value.0
    }
}
impl RigStart {
    /// Stored code for `Animated`.
    pub const ANIMATED: Self = Self(0);
    /// Stored code for `OnDeath`.
    pub const ON_DEATH: Self = Self(1);
    /// Stored code for `Always`.
    pub const ALWAYS: Self = Self(2);
    /// Stored code for `Never`.
    pub const NEVER: Self = Self(3);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(4);
}
/// Native WowPhysicsKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct WowPhysicsKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for WowPhysicsKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<WowPhysicsKind> for u8 {
    fn from(value: WowPhysicsKind) -> Self {
        value.0
    }
}
impl WowPhysicsKind {
    /// Stored code for `WornItem`.
    pub const WORN_ITEM: Self = Self(0);
    /// Stored code for `Vegetation`.
    pub const VEGETATION: Self = Self(1);
    /// Stored code for `Ragdoll`.
    pub const RAGDOLL: Self = Self(2);
    /// Stored code for `PrivateWorld`.
    pub const PRIVATE_WORLD: Self = Self(3);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(4);
}
/// Native SoftFitMethod codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct SoftFitMethod(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for SoftFitMethod {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<SoftFitMethod> for u8 {
    fn from(value: SoftFitMethod) -> Self {
        value.0
    }
}
impl SoftFitMethod {
    /// Stored code for `Fast`.
    pub const FAST: Self = Self(0);
    /// Stored code for `Dense`.
    pub const DENSE: Self = Self(1);
}
/// Native HingeSource codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct HingeSource(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for HingeSource {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<HingeSource> for u8 {
    fn from(value: HingeSource) -> Self {
        value.0
    }
}
impl HingeSource {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Joint`.
    pub const JOINT: Self = Self(1);
    /// Stored code for `Clips`.
    pub const CLIPS: Self = Self(2);
    /// Stored code for `Fallback`.
    pub const FALLBACK: Self = Self(3);
}
/// Native TPoseRefusal codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct TPoseRefusal(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for TPoseRefusal {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<TPoseRefusal> for u8 {
    fn from(value: TPoseRefusal) -> Self {
        value.0
    }
}
impl TPoseRefusal {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `NoChain`.
    pub const NO_CHAIN: Self = Self(1);
    /// Stored code for `ZeroLength`.
    pub const ZERO_LENGTH: Self = Self(2);
    /// Stored code for `NoHinge`.
    pub const NO_HINGE: Self = Self(3);
    /// Stored code for `JointLimits`.
    pub const JOINT_LIMITS: Self = Self(4);
    /// Stored code for `Unreachable`.
    pub const UNREACHABLE: Self = Self(5);
    /// Stored code for `Overlaps`.
    pub const OVERLAPS: Self = Self(6);
}
/// Native TPoseLanding codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct TPoseLanding(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for TPoseLanding {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<TPoseLanding> for u8 {
    fn from(value: TPoseLanding) -> Self {
        value.0
    }
}
impl TPoseLanding {
    /// Stored code for `Refused`.
    pub const REFUSED: Self = Self(0);
    /// Stored code for `OnTarget`.
    pub const ON_TARGET: Self = Self(1);
    /// Stored code for `Off`.
    pub const OFF: Self = Self(2);
}
/// Native TPoseScoreKind codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct TPoseScoreKind(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for TPoseScoreKind {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<TPoseScoreKind> for u8 {
    fn from(value: TPoseScoreKind) -> Self {
        value.0
    }
}
impl TPoseScoreKind {
    /// Stored code for `Aim`.
    pub const AIM: Self = Self(0);
    /// Stored code for `Heading`.
    pub const HEADING: Self = Self(1);
    /// Stored code for `Level`.
    pub const LEVEL: Self = Self(2);
    /// Stored code for `Roll`.
    pub const ROLL: Self = Self(3);
    /// Stored code for `Mirror`.
    pub const MIRROR: Self = Self(4);
    /// Stored code for `AnkleHeight`.
    pub const ANKLE_HEIGHT: Self = Self(5);
    /// Stored code for `Head`.
    pub const HEAD: Self = Self(6);
    /// Stored code for `Hips`.
    pub const HIPS: Self = Self(7);
}
/// Native RebindCase codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct RebindCase(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for RebindCase {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<RebindCase> for u8 {
    fn from(value: RebindCase) -> Self {
        value.0
    }
}
impl RebindCase {
    /// Stored code for `NonUniformScale`.
    pub const NON_UNIFORM_SCALE: Self = Self(0);
    /// Stored code for `DontInherit`.
    pub const DONT_INHERIT: Self = Self(1);
    /// Stored code for `Billboard`.
    pub const BILLBOARD: Self = Self(2);
    /// Stored code for `CollisionShape`.
    pub const COLLISION_SHAPE: Self = Self(3);
    /// Stored code for `GlobalSequence`.
    pub const GLOBAL_SEQUENCE: Self = Self(4);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(5);
}
/// Native RigRole codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct RigRole(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for RigRole {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<RigRole> for u8 {
    fn from(value: RigRole) -> Self {
        value.0
    }
}
impl RigRole {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `Body`.
    pub const BODY: Self = Self(1);
    /// Stored code for `Spine`.
    pub const SPINE: Self = Self(2);
    /// Stored code for `Neck`.
    pub const NECK: Self = Self(3);
    /// Stored code for `Head`.
    pub const HEAD: Self = Self(4);
    /// Stored code for `Clavicle`.
    pub const CLAVICLE: Self = Self(5);
    /// Stored code for `Upper`.
    pub const UPPER: Self = Self(6);
    /// Stored code for `Lower`.
    pub const LOWER: Self = Self(7);
    /// Stored code for `Hock`.
    pub const HOCK: Self = Self(8);
    /// Stored code for `End`.
    pub const END: Self = Self(9);
    /// Stored code for `Toe`.
    pub const TOE: Self = Self(10);
    /// Stored code for `Twist`.
    pub const TWIST: Self = Self(11);
    /// Stored code for `Pad`.
    pub const PAD: Self = Self(12);
    /// Stored code for `Prop`.
    pub const PROP: Self = Self(13);
}
/// Native RigSide codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct RigSide(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for RigSide {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<RigSide> for u8 {
    fn from(value: RigSide) -> Self {
        value.0
    }
}
impl RigSide {
    /// Stored code for `Centre`.
    pub const CENTRE: Self = Self(0);
    /// Stored code for `Left`.
    pub const LEFT: Self = Self(1);
    /// Stored code for `Right`.
    pub const RIGHT: Self = Self(2);
}
/// Native RigLimb codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct RigLimb(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for RigLimb {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<RigLimb> for u8 {
    fn from(value: RigLimb) -> Self {
        value.0
    }
}
impl RigLimb {
    /// Stored code for `Other`.
    pub const OTHER: Self = Self(0);
    /// Stored code for `Arm`.
    pub const ARM: Self = Self(1);
    /// Stored code for `Leg`.
    pub const LEG: Self = Self(2);
}
/// Native RigPlant codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct RigPlant(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for RigPlant {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<RigPlant> for u8 {
    fn from(value: RigPlant) -> Self {
        value.0
    }
}
impl RigPlant {
    /// Stored code for `ByRole`.
    pub const BY_ROLE: Self = Self(0);
    /// Stored code for `OnGround`.
    pub const ON_GROUND: Self = Self(1);
    /// Stored code for `Always`.
    pub const ALWAYS: Self = Self(2);
    /// Stored code for `Free`.
    pub const FREE: Self = Self(3);
}
/// Native RigSource codes, represented by their stored u8 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct RigSource(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u8,
);
impl From<u8> for RigSource {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<RigSource> for u8 {
    fn from(value: RigSource) -> Self {
        value.0
    }
}
impl RigSource {
    /// Stored code for `None`.
    pub const NONE: Self = Self(0);
    /// Stored code for `File`.
    pub const FILE: Self = Self(1);
    /// Stored code for `Name`.
    pub const NAME: Self = Self(2);
    /// Stored code for `Attachment`.
    pub const ATTACHMENT: Self = Self(3);
    /// Stored code for `Shape`.
    pub const SHAPE: Self = Self(4);
    /// Stored code for `You`.
    pub const YOU: Self = Self(5);
}
/// Native AttributeClass codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct AttributeClass(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for AttributeClass {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<AttributeClass> for u32 {
    fn from(value: AttributeClass) -> Self {
        value.0
    }
}
impl AttributeClass {
    /// Stored code for `Position`.
    pub const POSITION: Self = Self(0);
    /// Stored code for `Normal`.
    pub const NORMAL: Self = Self(1);
    /// Stored code for `Tangent`.
    pub const TANGENT: Self = Self(2);
    /// Stored code for `UV`.
    pub const UV: Self = Self(3);
    /// Stored code for `Color`.
    pub const COLOR: Self = Self(4);
    /// Stored code for `BlendIndices`.
    pub const BLEND_INDICES: Self = Self(5);
    /// Stored code for `BlendWeights`.
    pub const BLEND_WEIGHTS: Self = Self(6);
    /// Stored code for `Binormal`.
    pub const BINORMAL: Self = Self(7);
    /// Stored code for `Count`.
    pub const COUNT: Self = Self(8);
}
/// Native AttributeEncoding codes, represented by their stored u32 value.
/// Unrecognized codes remain representable during read/edit/write.
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
)]
pub struct AttributeEncoding(
    /// Uninterpreted scalar encoding, including any unknown bits.
    pub u32,
);
impl From<u32> for AttributeEncoding {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<AttributeEncoding> for u32 {
    fn from(value: AttributeEncoding) -> Self {
        value.0
    }
}
impl AttributeEncoding {
    /// Stored code for `Float32`.
    pub const FLOAT32: Self = Self(0);
    /// Stored code for `Float16`.
    pub const FLOAT16: Self = Self(1);
    /// Stored code for `SNorm8`.
    pub const S_NORM8: Self = Self(2);
    /// Stored code for `SNorm16`.
    pub const S_NORM16: Self = Self(3);
    /// Stored code for `UNorm8`.
    pub const U_NORM8: Self = Self(4);
    /// Stored code for `UNorm16`.
    pub const U_NORM16: Self = Self(5);
    /// Stored code for `UInt8`.
    pub const U_INT8: Self = Self(6);
    /// Stored code for `UInt16`.
    pub const U_INT16: Self = Self(7);
    /// Stored code for `UInt32`.
    pub const U_INT32: Self = Self(8);
    /// Stored code for `Int8`.
    pub const INT8: Self = Self(9);
    /// Stored code for `Int16`.
    pub const INT16: Self = Self(10);
    /// Stored code for `Int32`.
    pub const INT32: Self = Self(11);
}
