//! Section identities used by typed references.
use crate::model::wem::Target;
/// Reference target for the `CHAR` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Character;
impl Target for Character {
    const TAG: Option<[u8; 4]> = Some(*b"CHAR");
}
/// Reference target for the `U8__` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Byte;
impl Target for Byte {
    const TAG: Option<[u8; 4]> = Some(*b"U8__");
}
/// Reference target for the `U16_` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct U16;
impl Target for U16 {
    const TAG: Option<[u8; 4]> = Some(*b"U16_");
}
/// Reference target for the `U32_` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct U32;
impl Target for U32 {
    const TAG: Option<[u8; 4]> = Some(*b"U32_");
}
/// Reference target for the `U64_` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct U64;
impl Target for U64 {
    const TAG: Option<[u8; 4]> = Some(*b"U64_");
}
/// Reference target for the `I16_` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct I16;
impl Target for I16 {
    const TAG: Option<[u8; 4]> = Some(*b"I16_");
}
/// Reference target for the `I32_` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct I32;
impl Target for I32 {
    const TAG: Option<[u8; 4]> = Some(*b"I32_");
}
/// Reference target for the `REAL` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Real;
impl Target for Real {
    const TAG: Option<[u8; 4]> = Some(*b"REAL");
}
/// Reference target for the `VEC2` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Vector2;
impl Target for Vector2 {
    const TAG: Option<[u8; 4]> = Some(*b"VEC2");
}
/// Reference target for the `VEC3` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Vector3;
impl Target for Vector3 {
    const TAG: Option<[u8; 4]> = Some(*b"VEC3");
}
/// Reference target for the `VEC4` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Vector4;
impl Target for Vector4 {
    const TAG: Option<[u8; 4]> = Some(*b"VEC4");
}
/// Reference target for the `QUAT` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QuaternionRecord;
impl Target for QuaternionRecord {
    const TAG: Option<[u8; 4]> = Some(*b"QUAT");
}
/// Reference target for the `BNDS` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Extent;
impl Target for Extent {
    const TAG: Option<[u8; 4]> = Some(*b"BNDS");
}
/// Reference target for the `BYT4` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Byte4;
impl Target for Byte4 {
    const TAG: Option<[u8; 4]> = Some(*b"BYT4");
}
/// Reference target for the `WDOC` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Document;
impl Target for Document {
    const TAG: Option<[u8; 4]> = Some(*b"WDOC");
}
/// Reference target for the `MODL` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Model;
impl Target for Model {
    const TAG: Option<[u8; 4]> = Some(*b"MODL");
}
/// Reference target for the `PROF` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProfileMaterialSet;
impl Target for ProfileMaterialSet {
    const TAG: Option<[u8; 4]> = Some(*b"PROF");
}
/// Reference target for the `SBND` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SlotBinding;
impl Target for SlotBinding {
    const TAG: Option<[u8; 4]> = Some(*b"SBND");
}
/// Reference target for the `MESH` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mesh;
impl Target for Mesh {
    const TAG: Option<[u8; 4]> = Some(*b"MESH");
}
/// Reference target for the `SECT` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MeshSection;
impl Target for MeshSection {
    const TAG: Option<[u8; 4]> = Some(*b"SECT");
}
/// Reference target for the `ATTR` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AttrLayer;
impl Target for AttrLayer {
    const TAG: Option<[u8; 4]> = Some(*b"ATTR");
}
/// Reference target for the `INFL` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Influence;
impl Target for Influence {
    const TAG: Option<[u8; 4]> = Some(*b"INFL");
}
/// Reference target for the `VSPL` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VertexSplit;
impl Target for VertexSplit {
    const TAG: Option<[u8; 4]> = Some(*b"VSPL");
}
/// Reference target for the `FDRP` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FaceRecord;
impl Target for FaceRecord {
    const TAG: Option<[u8; 4]> = Some(*b"FDRP");
}
/// Reference target for the `XFRM` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Transform;
impl Target for Transform {
    const TAG: Option<[u8; 4]> = Some(*b"XFRM");
}
/// Reference target for the `MTX4` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Matrix44;
impl Target for Matrix44 {
    const TAG: Option<[u8; 4]> = Some(*b"MTX4");
}
/// Reference target for the `NODE` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Node;
impl Target for Node {
    const TAG: Option<[u8; 4]> = Some(*b"NODE");
}
/// Reference target for the `TPSR` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TPoseSourceRecord;
impl Target for TPoseSourceRecord {
    const TAG: Option<[u8; 4]> = Some(*b"TPSR");
}
/// Reference target for the `PDLT` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoseDelta;
impl Target for PoseDelta {
    const TAG: Option<[u8; 4]> = Some(*b"PDLT");
}
/// Reference target for the `TPOS` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TestPose;
impl Target for TestPose {
    const TAG: Option<[u8; 4]> = Some(*b"TPOS");
}
/// Reference target for the `AKEY` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AssetKey;
impl Target for AssetKey {
    const TAG: Option<[u8; 4]> = Some(*b"AKEY");
}
/// Reference target for the `S2P3` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sc2PropertyVector3;
impl Target for Sc2PropertyVector3 {
    const TAG: Option<[u8; 4]> = Some(*b"S2P3");
}
/// Reference target for the `S2RS` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sc2RibbonSplinePoint;
impl Target for Sc2RibbonSplinePoint {
    const TAG: Option<[u8; 4]> = Some(*b"S2RS");
}
/// Reference target for the `NBAG` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NativeEntry;
impl Target for NativeEntry {
    const TAG: Option<[u8; 4]> = Some(*b"NBAG");
}
/// Reference target for the `PSCH` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoseSchema;
impl Target for PoseSchema {
    const TAG: Option<[u8; 4]> = Some(*b"PSCH");
}
/// Reference target for the `MATL` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Material;
impl Target for Material {
    const TAG: Option<[u8; 4]> = Some(*b"MATL");
}
/// Reference target for the `MFEA` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MaterialFeature;
impl Target for MaterialFeature {
    const TAG: Option<[u8; 4]> = Some(*b"MFEA");
}
/// Reference target for the `FFRS` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FresnelFeature;
impl Target for FresnelFeature {
    const TAG: Option<[u8; 4]> = Some(*b"FFRS");
}
/// Reference target for the `FUVA` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UvAnimationFeature;
impl Target for UvAnimationFeature {
    const TAG: Option<[u8; 4]> = Some(*b"FUVA");
}
/// Reference target for the `FLSH` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayerShadingFeature;
impl Target for LayerShadingFeature {
    const TAG: Option<[u8; 4]> = Some(*b"FLSH");
}
/// Reference target for the `MKCP` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompositeBody;
impl Target for CompositeBody {
    const TAG: Option<[u8; 4]> = Some(*b"MKCP");
}
/// Reference target for the `MKCL` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompositeLayer;
impl Target for CompositeLayer {
    const TAG: Option<[u8; 4]> = Some(*b"MKCL");
}
/// Reference target for the `MKCB` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CombinersBody;
impl Target for CombinersBody {
    const TAG: Option<[u8; 4]> = Some(*b"MKCB");
}
/// Reference target for the `MKCS` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CombinerStage;
impl Target for CombinerStage {
    const TAG: Option<[u8; 4]> = Some(*b"MKCS");
}
/// Reference target for the `MKLD` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LegacyDeferredBody;
impl Target for LegacyDeferredBody {
    const TAG: Option<[u8; 4]> = Some(*b"MKLD");
}
/// Reference target for the `MKPB` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PbrDeferredBody;
impl Target for PbrDeferredBody {
    const TAG: Option<[u8; 4]> = Some(*b"MKPB");
}
/// Reference target for the `TEXR` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextureRef;
impl Target for TextureRef {
    const TAG: Option<[u8; 4]> = Some(*b"TEXR");
}
/// Reference target for the `LOOK` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Look;
impl Target for Look {
    const TAG: Option<[u8; 4]> = Some(*b"LOOK");
}
/// Reference target for the `ACHN` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimChannel;
impl Target for AnimChannel {
    const TAG: Option<[u8; 4]> = Some(*b"ACHN");
}
/// Reference target for the `STRK` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SubTrack;
impl Target for SubTrack {
    const TAG: Option<[u8; 4]> = Some(*b"STRK");
}
/// Reference target for the `STCC` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SubTrackContainer;
impl Target for SubTrackContainer {
    const TAG: Option<[u8; 4]> = Some(*b"STCC");
}
/// Reference target for the `CLEV` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClipEvent;
impl Target for ClipEvent {
    const TAG: Option<[u8; 4]> = Some(*b"CLEV");
}
/// Reference target for the `CLIP` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Clip;
impl Target for Clip {
    const TAG: Option<[u8; 4]> = Some(*b"CLIP");
}
/// Reference target for the `CPWF` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BakeWorldForce;
impl Target for BakeWorldForce {
    const TAG: Option<[u8; 4]> = Some(*b"CPWF");
}
/// Reference target for the `TSET` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TrackSet;
impl Target for TrackSet {
    const TAG: Option<[u8; 4]> = Some(*b"TSET");
}
/// Reference target for the `PSTG` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoseStage;
impl Target for PoseStage {
    const TAG: Option<[u8; 4]> = Some(*b"PSTG");
}
/// Reference target for the `PSRC` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StageSource;
impl Target for StageSource {
    const TAG: Option<[u8; 4]> = Some(*b"PSRC");
}
/// Reference target for the `PBDY` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicsBody;
impl Target for PhysicsBody {
    const TAG: Option<[u8; 4]> = Some(*b"PBDY");
}
/// Reference target for the `PSHP` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicsShape;
impl Target for PhysicsShape {
    const TAG: Option<[u8; 4]> = Some(*b"PSHP");
}
/// Reference target for the `PJNT` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicsJoint;
impl Target for PhysicsJoint {
    const TAG: Option<[u8; 4]> = Some(*b"PJNT");
}
/// Reference target for the `PCOL` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClothCollider;
impl Target for ClothCollider {
    const TAG: Option<[u8; 4]> = Some(*b"PCOL");
}
/// Reference target for the `CLTH` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cloth;
impl Target for Cloth {
    const TAG: Option<[u8; 4]> = Some(*b"CLTH");
}
/// Reference target for the `CBND` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClothBinding;
impl Target for ClothBinding {
    const TAG: Option<[u8; 4]> = Some(*b"CBND");
}
/// Reference target for the `PRIG` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicsRig;
impl Target for PhysicsRig {
    const TAG: Option<[u8; 4]> = Some(*b"PRIG");
}
/// Reference target for the `SBDY` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoftBody;
impl Target for SoftBody {
    const TAG: Option<[u8; 4]> = Some(*b"SBDY");
}
/// Reference target for the `SLMP` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoftLump;
impl Target for SoftLump {
    const TAG: Option<[u8; 4]> = Some(*b"SLMP");
}
/// Reference target for the `SEFX` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoftEffect;
impl Target for SoftEffect {
    const TAG: Option<[u8; 4]> = Some(*b"SEFX");
}
/// Reference target for the `SPRT` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoftPart;
impl Target for SoftPart {
    const TAG: Option<[u8; 4]> = Some(*b"SPRT");
}
/// Reference target for the `CTSU` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClipTrackSet;
impl Target for ClipTrackSet {
    const TAG: Option<[u8; 4]> = Some(*b"CTSU");
}
/// Reference target for the `ATAG` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimTag;
impl Target for AnimTag {
    const TAG: Option<[u8; 4]> = Some(*b"ATAG");
}
/// Reference target for the `ASET` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimSet;
impl Target for AnimSet {
    const TAG: Option<[u8; 4]> = Some(*b"ASET");
}
/// Reference target for the `NMDX` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MdxMaterial;
impl Target for MdxMaterial {
    const TAG: Option<[u8; 4]> = Some(*b"NMDX");
}
/// Reference target for the `NM2_` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct M2Material;
impl Target for M2Material {
    const TAG: Option<[u8; 4]> = Some(*b"NM2_");
}
/// Reference target for the `ND3_` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct D3Material;
impl Target for D3Material {
    const TAG: Option<[u8; 4]> = Some(*b"ND3_");
}
/// Reference target for the `NM3_` section family.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct M3Material;
impl Target for M3Material {
    const TAG: Option<[u8; 4]> = Some(*b"NM3_");
}
