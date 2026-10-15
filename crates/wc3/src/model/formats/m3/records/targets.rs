//! Section-family markers for references whose target revision is selected independently.
pub use crate::model::formats::m3::Any;
use crate::model::formats::m3::Target;
/// Target family `CHAR`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Character;
impl Target for Character {
    const TAG: Option<[u8; 4]> = Some(*b"CHAR");
}
/// Target family `U8__`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Byte;
impl Target for Byte {
    const TAG: Option<[u8; 4]> = Some(*b"U8__");
}
/// Target family `U16_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct U16;
impl Target for U16 {
    const TAG: Option<[u8; 4]> = Some(*b"U16_");
}
/// Target family `U32_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct U32;
impl Target for U32 {
    const TAG: Option<[u8; 4]> = Some(*b"U32_");
}
/// Target family `U64_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct U64;
impl Target for U64 {
    const TAG: Option<[u8; 4]> = Some(*b"U64_");
}
/// Target family `I16_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct I16;
impl Target for I16 {
    const TAG: Option<[u8; 4]> = Some(*b"I16_");
}
/// Target family `I32_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct I32;
impl Target for I32 {
    const TAG: Option<[u8; 4]> = Some(*b"I32_");
}
/// Target family `REAL`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Real;
impl Target for Real {
    const TAG: Option<[u8; 4]> = Some(*b"REAL");
}
/// Target family `FLAG`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flag;
impl Target for Flag {
    const TAG: Option<[u8; 4]> = Some(*b"FLAG");
}
/// Target family `VEC2`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Vector2;
impl Target for Vector2 {
    const TAG: Option<[u8; 4]> = Some(*b"VEC2");
}
/// Target family `VEC3`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Vector3;
impl Target for Vector3 {
    const TAG: Option<[u8; 4]> = Some(*b"VEC3");
}
/// Target family `VEC4`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Vector4;
impl Target for Vector4 {
    const TAG: Option<[u8; 4]> = Some(*b"VEC4");
}
/// Target family `QUAT`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Quaternion;
impl Target for Quaternion {
    const TAG: Option<[u8; 4]> = Some(*b"QUAT");
}
/// Target family `NULCOL`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color;
impl Target for Color {
    const TAG: Option<[u8; 4]> = Some(*b"\0COL");
}
/// Target family `BNDS`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Extent;
impl Target for Extent {
    const TAG: Option<[u8; 4]> = Some(*b"BNDS");
}
/// Target family `SDEV`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationBlockEvent;
impl Target for AnimationBlockEvent {
    const TAG: Option<[u8; 4]> = Some(*b"SDEV");
}
/// Target family `SD2V`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationBlockVector2;
impl Target for AnimationBlockVector2 {
    const TAG: Option<[u8; 4]> = Some(*b"SD2V");
}
/// Target family `SD3V`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationBlockVector3;
impl Target for AnimationBlockVector3 {
    const TAG: Option<[u8; 4]> = Some(*b"SD3V");
}
/// Target family `SDR3`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationBlockReal;
impl Target for AnimationBlockReal {
    const TAG: Option<[u8; 4]> = Some(*b"SDR3");
}
/// Target family `SD4Q`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationBlockQuaternion;
impl Target for AnimationBlockQuaternion {
    const TAG: Option<[u8; 4]> = Some(*b"SD4Q");
}
/// Target family `SDCC`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationBlockColor;
impl Target for AnimationBlockColor {
    const TAG: Option<[u8; 4]> = Some(*b"SDCC");
}
/// Target family `SDU8`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationBlockByte;
impl Target for AnimationBlockByte {
    const TAG: Option<[u8; 4]> = Some(*b"SDU8");
}
/// Target family `SDS6`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationBlockI16;
impl Target for AnimationBlockI16 {
    const TAG: Option<[u8; 4]> = Some(*b"SDS6");
}
/// Target family `SDU6`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationBlockU16;
impl Target for AnimationBlockU16 {
    const TAG: Option<[u8; 4]> = Some(*b"SDU6");
}
/// Target family `SDS3`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationBlockI32;
impl Target for AnimationBlockI32 {
    const TAG: Option<[u8; 4]> = Some(*b"SDS3");
}
/// Target family `SDU3`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationBlockU32;
impl Target for AnimationBlockU32 {
    const TAG: Option<[u8; 4]> = Some(*b"SDU3");
}
/// Target family `SDFG`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationBlockFlag;
impl Target for AnimationBlockFlag {
    const TAG: Option<[u8; 4]> = Some(*b"SDFG");
}
/// Target family `SDMB`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationBlockExtent;
impl Target for AnimationBlockExtent {
    const TAG: Option<[u8; 4]> = Some(*b"SDMB");
}
/// Target family `SS32`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationValueI32;
impl Target for AnimationValueI32 {
    const TAG: Option<[u8; 4]> = Some(*b"SS32");
}
/// Target family `SU32`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationValueU32;
impl Target for AnimationValueU32 {
    const TAG: Option<[u8; 4]> = Some(*b"SU32");
}
/// Target family `SR32`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationValueReal;
impl Target for AnimationValueReal {
    const TAG: Option<[u8; 4]> = Some(*b"SR32");
}
/// Target family `SVC3`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationValueVector3;
impl Target for AnimationValueVector3 {
    const TAG: Option<[u8; 4]> = Some(*b"SVC3");
}
/// Target family `SVC2`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationValueVector2;
impl Target for AnimationValueVector2 {
    const TAG: Option<[u8; 4]> = Some(*b"SVC2");
}
/// Target family `MT16`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MeshTriangle16;
impl Target for MeshTriangle16 {
    const TAG: Option<[u8; 4]> = Some(*b"MT16");
}
/// Target family `MT32`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MeshTriangle32;
impl Target for MeshTriangle32 {
    const TAG: Option<[u8; 4]> = Some(*b"MT32");
}
/// Target family `SCHR`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StringReference;
impl Target for StringReference {
    const TAG: Option<[u8; 4]> = Some(*b"SCHR");
}
/// Target family `EVNT`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Event;
impl Target for Event {
    const TAG: Option<[u8; 4]> = Some(*b"EVNT");
}
/// Target family `SEQS`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sequence;
impl Target for Sequence {
    const TAG: Option<[u8; 4]> = Some(*b"SEQS");
}
/// Target family `STC_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SubTrackContainer;
impl Target for SubTrackContainer {
    const TAG: Option<[u8; 4]> = Some(*b"STC_");
}
/// Target family `STG_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationGroup;
impl Target for AnimationGroup {
    const TAG: Option<[u8; 4]> = Some(*b"STG_");
}
/// Target family `STS_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationState;
impl Target for AnimationState {
    const TAG: Option<[u8; 4]> = Some(*b"STS_");
}
/// Target family `BSET`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoneAnimationSet;
impl Target for BoneAnimationSet {
    const TAG: Option<[u8; 4]> = Some(*b"BSET");
}
/// Target family `BONE`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bone;
impl Target for Bone {
    const TAG: Option<[u8; 4]> = Some(*b"BONE");
}
/// Target family `REGN`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Region;
impl Target for Region {
    const TAG: Option<[u8; 4]> = Some(*b"REGN");
}
/// Target family `BAT_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Batch;
impl Target for Batch {
    const TAG: Option<[u8; 4]> = Some(*b"BAT_");
}
/// Target family `MSEC`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MeshSection;
impl Target for MeshSection {
    const TAG: Option<[u8; 4]> = Some(*b"MSEC");
}
/// Target family `DIV_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MeshDivision;
impl Target for MeshDivision {
    const TAG: Option<[u8; 4]> = Some(*b"DIV_");
}
/// Target family `IREF`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InitialReference;
impl Target for InitialReference {
    const TAG: Option<[u8; 4]> = Some(*b"IREF");
}
/// Target family `ATT_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AttachmentPoint;
impl Target for AttachmentPoint {
    const TAG: Option<[u8; 4]> = Some(*b"ATT_");
}
/// Target family `MATM`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MaterialMap;
impl Target for MaterialMap {
    const TAG: Option<[u8; 4]> = Some(*b"MATM");
}
/// Target family `LAYR`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextureLayer;
impl Target for TextureLayer {
    const TAG: Option<[u8; 4]> = Some(*b"LAYR");
}
/// Target family `MAT_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StandardMaterial;
impl Target for StandardMaterial {
    const TAG: Option<[u8; 4]> = Some(*b"MAT_");
}
/// Target family `DIS_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DisplacementMaterial;
impl Target for DisplacementMaterial {
    const TAG: Option<[u8; 4]> = Some(*b"DIS_");
}
/// Target family `CMS_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompositeSection;
impl Target for CompositeSection {
    const TAG: Option<[u8; 4]> = Some(*b"CMS_");
}
/// Target family `CMP_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompositeMaterial;
impl Target for CompositeMaterial {
    const TAG: Option<[u8; 4]> = Some(*b"CMP_");
}
/// Target family `TER_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TerrainMaterial;
impl Target for TerrainMaterial {
    const TAG: Option<[u8; 4]> = Some(*b"TER_");
}
/// Target family `VOL_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VolumeMaterial;
impl Target for VolumeMaterial {
    const TAG: Option<[u8; 4]> = Some(*b"VOL_");
}
/// Target family `HAI_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HairMaterial;
impl Target for HairMaterial {
    const TAG: Option<[u8; 4]> = Some(*b"HAI_");
}
/// Target family `VON_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VolumeNoiseMaterial;
impl Target for VolumeNoiseMaterial {
    const TAG: Option<[u8; 4]> = Some(*b"VON_");
}
/// Target family `CREP`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CreepMaterial;
impl Target for CreepMaterial {
    const TAG: Option<[u8; 4]> = Some(*b"CREP");
}
/// Target family `STBM`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct STBMaterial;
impl Target for STBMaterial {
    const TAG: Option<[u8; 4]> = Some(*b"STBM");
}
/// Target family `REF_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReflectionMaterial;
impl Target for ReflectionMaterial {
    const TAG: Option<[u8; 4]> = Some(*b"REF_");
}
/// Target family `LFSB`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SubFlare;
impl Target for SubFlare {
    const TAG: Option<[u8; 4]> = Some(*b"LFSB");
}
/// Target family `LFLR`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LensFlare;
impl Target for LensFlare {
    const TAG: Option<[u8; 4]> = Some(*b"LFLR");
}
/// Target family `MADD`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DataDrivenMaterial;
impl Target for DataDrivenMaterial {
    const TAG: Option<[u8; 4]> = Some(*b"MADD");
}
/// Target family `PAR_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParticleEmitter;
impl Target for ParticleEmitter {
    const TAG: Option<[u8; 4]> = Some(*b"PAR_");
}
/// Target family `PARC`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParticleEmitterCopy;
impl Target for ParticleEmitterCopy {
    const TAG: Option<[u8; 4]> = Some(*b"PARC");
}
/// Target family `SRIB`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SplineRibbon;
impl Target for SplineRibbon {
    const TAG: Option<[u8; 4]> = Some(*b"SRIB");
}
/// Target family `RIB_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RibbonEmitter;
impl Target for RibbonEmitter {
    const TAG: Option<[u8; 4]> = Some(*b"RIB_");
}
/// Target family `PROJ`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Projector;
impl Target for Projector {
    const TAG: Option<[u8; 4]> = Some(*b"PROJ");
}
/// Target family `SSGS`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HitTestShape;
impl Target for HitTestShape {
    const TAG: Option<[u8; 4]> = Some(*b"SSGS");
}
/// Target family `ATVL`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AttachmentVolume;
impl Target for AttachmentVolume {
    const TAG: Option<[u8; 4]> = Some(*b"ATVL");
}
/// Target family `TRGD`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TurretGroup;
impl Target for TurretGroup {
    const TAG: Option<[u8; 4]> = Some(*b"TRGD");
}
/// Target family `PATU`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TurretBehavior;
impl Target for TurretBehavior {
    const TAG: Option<[u8; 4]> = Some(*b"PATU");
}
/// Target family `BBSC`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BillboardBehavior;
impl Target for BillboardBehavior {
    const TAG: Option<[u8; 4]> = Some(*b"BBSC");
}
/// Target family `IKJT`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IKJoint;
impl Target for IKJoint {
    const TAG: Option<[u8; 4]> = Some(*b"IKJT");
}
/// Target family `IK2J`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IKTwoJoint;
impl Target for IKTwoJoint {
    const TAG: Option<[u8; 4]> = Some(*b"IK2J");
}
/// Target family `IKCC`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IKCCD;
impl Target for IKCCD {
    const TAG: Option<[u8; 4]> = Some(*b"IKCC");
}
/// Target family `PAOB`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OneBoneSolver;
impl Target for OneBoneSolver {
    const TAG: Option<[u8; 4]> = Some(*b"PAOB");
}
/// Target family `SHBX`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShadowBox;
impl Target for ShadowBox {
    const TAG: Option<[u8; 4]> = Some(*b"SHBX");
}
/// Target family `VVOL`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewVolume;
impl Target for ViewVolume {
    const TAG: Option<[u8; 4]> = Some(*b"VVOL");
}
/// Target family `TMD_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TrailingModel;
impl Target for TrailingModel {
    const TAG: Option<[u8; 4]> = Some(*b"TMD_");
}
/// Target family `FOR_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Force;
impl Target for Force {
    const TAG: Option<[u8; 4]> = Some(*b"FOR_");
}
/// Target family `WRP_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Warp;
impl Target for Warp {
    const TAG: Option<[u8; 4]> = Some(*b"WRP_");
}
/// Target family `DMSE`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConvexHullHalfEdge;
impl Target for ConvexHullHalfEdge {
    const TAG: Option<[u8; 4]> = Some(*b"DMSE");
}
/// Target family `DMMN`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicsMeshBvhNode;
impl Target for PhysicsMeshBvhNode {
    const TAG: Option<[u8; 4]> = Some(*b"DMMN");
}
/// Target family `DMMT`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicsMeshTriangle;
impl Target for PhysicsMeshTriangle {
    const TAG: Option<[u8; 4]> = Some(*b"DMMT");
}
/// Target family `DMME`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicsMeshEdge;
impl Target for PhysicsMeshEdge {
    const TAG: Option<[u8; 4]> = Some(*b"DMME");
}
/// Target family `PHSH`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicsShape;
impl Target for PhysicsShape {
    const TAG: Option<[u8; 4]> = Some(*b"PHSH");
}
/// Target family `PHRB`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RigidBody;
impl Target for RigidBody {
    const TAG: Option<[u8; 4]> = Some(*b"PHRB");
}
/// Target family `PHYJ`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicsJoint;
impl Target for PhysicsJoint {
    const TAG: Option<[u8; 4]> = Some(*b"PHYJ");
}
/// Target family `PHCC`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClothCollider;
impl Target for ClothCollider {
    const TAG: Option<[u8; 4]> = Some(*b"PHCC");
}
/// Target family `PHAC`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClothProxy;
impl Target for ClothProxy {
    const TAG: Option<[u8; 4]> = Some(*b"PHAC");
}
/// Target family `PHCL`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClothPhysics;
impl Target for ClothPhysics {
    const TAG: Option<[u8; 4]> = Some(*b"PHCL");
}
/// Target family `LITE`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Light;
impl Target for Light {
    const TAG: Option<[u8; 4]> = Some(*b"LITE");
}
/// Target family `CAM_`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Camera;
impl Target for Camera {
    const TAG: Option<[u8; 4]> = Some(*b"CAM_");
}
/// Target family `MODL`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Document;
impl Target for Document {
    const TAG: Option<[u8; 4]> = Some(*b"MODL");
}
/// Target family `PHCT`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicsConstraint;
impl Target for PhysicsConstraint {
    const TAG: Option<[u8; 4]> = Some(*b"PHCT");
}
