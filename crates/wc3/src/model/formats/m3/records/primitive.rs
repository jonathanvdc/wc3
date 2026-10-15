//! Small section elements carry scalar data, vectors, colors, and animation arrays.
//!
//! References from other records determine how generic numeric elements are used.
//! Animation blocks pair a timestamp section with a key section without resolving
//! either array eagerly.
use super::values::{AnimationBlockFlags, ConstraintFlags, FlagBits};
use super::{targets, AnimationValue, Extent};
use crate::model::formats::m3::{self, Record, Ref};
/// Fields of the native `CHAR` section, in serialized order.
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
pub struct Character {
    /// One CHAR byte; strings reference a run of these records including their terminator.
    pub value: u8,
}

impl Record for Character {
    const TAG: [u8; 4] = *b"CHAR";
    const VERSION: u32 = 0;
}

/// Fields of the native `U8__` section, in serialized order.
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
pub struct Byte {
    /// Unsigned byte element; its interpretation is supplied by the referring field.
    pub value: u8,
}

impl Record for Byte {
    const TAG: [u8; 4] = *b"U8__";
    const VERSION: u32 = 0;
}

/// Fields of the native `U16_` section, in serialized order.
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
pub struct U16 {
    /// Unsigned 16-bit array element.
    pub value: u16,
}

impl Record for U16 {
    const TAG: [u8; 4] = *b"U16_";
    const VERSION: u32 = 0;
}

/// Fields of the native `U32_` section, in serialized order.
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
pub struct U32 {
    /// Unsigned 32-bit array element.
    pub value: u32,
}

impl Record for U32 {
    const TAG: [u8; 4] = *b"U32_";
    const VERSION: u32 = 0;
}

/// Fields of the native `U64_` section, in serialized order.
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
pub struct U64 {
    /// Unsigned 64-bit array element.
    pub value: u64,
}

impl Record for U64 {
    const TAG: [u8; 4] = *b"U64_";
    const VERSION: u32 = 0;
}

/// Fields of the native `I16_` section, in serialized order.
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
pub struct I16 {
    /// Signed 16-bit array element.
    pub value: i16,
}

impl Record for I16 {
    const TAG: [u8; 4] = *b"I16_";
    const VERSION: u32 = 0;
}

/// Fields of the native `I32_` section, in serialized order.
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
pub struct I32 {
    /// Signed 32-bit array element.
    pub value: i32,
}

impl Record for I32 {
    const TAG: [u8; 4] = *b"I32_";
    const VERSION: u32 = 0;
}

/// Fields of the native `REAL` section, in serialized order.
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
pub struct Real {
    /// Single-precision floating-point array element.
    pub value: f32,
}

impl Record for Real {
    const TAG: [u8; 4] = *b"REAL";
    const VERSION: u32 = 0;
}

/// Fields of the native `FLAG` section, in serialized order.
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
pub struct Flag {
    /// One 32-bit bitmask element.
    pub value: FlagBits,
}

impl Record for Flag {
    const TAG: [u8; 4] = *b"FLAG";
    const VERSION: u32 = 0;
}

/// Fields of the native `VEC2` section, in serialized order.
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
pub struct Vector2 {
    /// 2 floating-point components; their coordinate meaning comes from the referring field.
    pub value: [f32; 2],
}

impl Record for Vector2 {
    const TAG: [u8; 4] = *b"VEC2";
    const VERSION: u32 = 0;
}

/// Fields of the native `VEC3` section, in serialized order.
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
pub struct Vector3 {
    /// 3 floating-point components; their coordinate meaning comes from the referring field.
    pub value: [f32; 3],
}

impl Record for Vector3 {
    const TAG: [u8; 4] = *b"VEC3";
    const VERSION: u32 = 0;
}

/// Fields of the native `VEC4` section, in serialized order.
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
pub struct Vector4 {
    /// 4 floating-point components; their coordinate meaning comes from the referring field.
    pub value: [f32; 4],
}

impl Record for Vector4 {
    const TAG: [u8; 4] = *b"VEC4";
    const VERSION: u32 = 0;
}

/// Fields of the native `QUAT` section, in serialized order.
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
pub struct Quaternion {
    /// Four quaternion components in the encoded component order.
    pub value: [f32; 4],
}

impl Record for Quaternion {
    const TAG: [u8; 4] = *b"QUAT";
    const VERSION: u32 = 0;
}

/// Fields of the native `\0COL` section, in serialized order.
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
pub struct Color {
    /// Four color bytes ordered blue, green, red, then alpha.
    pub value: [u8; 4],
}

impl Record for Color {
    const TAG: [u8; 4] = *b"\0COL";
    const VERSION: u32 = 0;
}

/// Fields of the native `BNDS` section, in serialized order.
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
pub struct ExtentRecord {
    /// Bounding-box corners and radius for this extent array element.
    pub value: Extent,
}

impl Record for ExtentRecord {
    const TAG: [u8; 4] = *b"BNDS";
    const VERSION: u32 = 0;
}

/// Fields of the native `SDEV` section, in serialized order.
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
pub struct AnimationBlockEvent {
    /// I32_ time coordinates corresponding to the entries in `keys`.
    pub timestamps: Ref<targets::I32>,
    /// Keyframe-block control bits.
    pub flags: AnimationBlockFlags,
    /// Ending frame coordinate recorded for the animation block.
    pub end_frame: u32,
    /// Key values selected in parallel with the block timestamp array.
    pub keys: Ref<targets::Event>,
}

impl Record for AnimationBlockEvent {
    const TAG: [u8; 4] = *b"SDEV";
    const VERSION: u32 = 0;
}

/// Fields of the native `SD2V` section, in serialized order.
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
pub struct AnimationBlockVector2 {
    /// I32_ time coordinates corresponding to the entries in `keys`.
    pub timestamps: Ref<targets::I32>,
    /// Keyframe-block control bits.
    pub flags: AnimationBlockFlags,
    /// Ending frame coordinate recorded for the animation block.
    pub end_frame: u32,
    /// Key values selected in parallel with the block timestamp array.
    pub keys: Ref<targets::Vector2>,
}

impl Record for AnimationBlockVector2 {
    const TAG: [u8; 4] = *b"SD2V";
    const VERSION: u32 = 0;
}

/// Fields of the native `SD3V` section, in serialized order.
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
pub struct AnimationBlockVector3 {
    /// I32_ time coordinates corresponding to the entries in `keys`.
    pub timestamps: Ref<targets::I32>,
    /// Keyframe-block control bits.
    pub flags: AnimationBlockFlags,
    /// Ending frame coordinate recorded for the animation block.
    pub end_frame: u32,
    /// Key values selected in parallel with the block timestamp array.
    pub keys: Ref<targets::Vector3>,
}

impl Record for AnimationBlockVector3 {
    const TAG: [u8; 4] = *b"SD3V";
    const VERSION: u32 = 0;
}

/// Fields of the native `SDR3` section, in serialized order.
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
pub struct AnimationBlockReal {
    /// I32_ time coordinates corresponding to the entries in `keys`.
    pub timestamps: Ref<targets::I32>,
    /// Keyframe-block control bits.
    pub flags: AnimationBlockFlags,
    /// Ending frame coordinate recorded for the animation block.
    pub end_frame: u32,
    /// Key values selected in parallel with the block timestamp array.
    pub keys: Ref<targets::Real>,
}

impl Record for AnimationBlockReal {
    const TAG: [u8; 4] = *b"SDR3";
    const VERSION: u32 = 0;
}

/// Fields of the native `SD4Q` section, in serialized order.
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
pub struct AnimationBlockQuaternion {
    /// I32_ time coordinates corresponding to the entries in `keys`.
    pub timestamps: Ref<targets::I32>,
    /// Keyframe-block control bits.
    pub flags: AnimationBlockFlags,
    /// Ending frame coordinate recorded for the animation block.
    pub end_frame: u32,
    /// Key values selected in parallel with the block timestamp array.
    pub keys: Ref<targets::Quaternion>,
}

impl Record for AnimationBlockQuaternion {
    const TAG: [u8; 4] = *b"SD4Q";
    const VERSION: u32 = 0;
}

/// Fields of the native `SDCC` section, in serialized order.
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
pub struct AnimationBlockColor {
    /// I32_ time coordinates corresponding to the entries in `keys`.
    pub timestamps: Ref<targets::I32>,
    /// Keyframe-block control bits.
    pub flags: AnimationBlockFlags,
    /// Ending frame coordinate recorded for the animation block.
    pub end_frame: u32,
    /// Key values selected in parallel with the block timestamp array.
    pub keys: Ref<targets::Color>,
}

impl Record for AnimationBlockColor {
    const TAG: [u8; 4] = *b"SDCC";
    const VERSION: u32 = 0;
}

/// Fields of the native `SDU8` section, in serialized order.
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
pub struct AnimationBlockByte {
    /// I32_ time coordinates corresponding to the entries in `keys`.
    pub timestamps: Ref<targets::I32>,
    /// Keyframe-block control bits.
    pub flags: AnimationBlockFlags,
    /// Ending frame coordinate recorded for the animation block.
    pub end_frame: u32,
    /// Key values selected in parallel with the block timestamp array.
    pub keys: Ref<targets::Byte>,
}

impl Record for AnimationBlockByte {
    const TAG: [u8; 4] = *b"SDU8";
    const VERSION: u32 = 0;
}

/// Fields of the native `SDS6` section, in serialized order.
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
pub struct AnimationBlockI16 {
    /// I32_ time coordinates corresponding to the entries in `keys`.
    pub timestamps: Ref<targets::I32>,
    /// Keyframe-block control bits.
    pub flags: AnimationBlockFlags,
    /// Ending frame coordinate recorded for the animation block.
    pub end_frame: u32,
    /// Key values selected in parallel with the block timestamp array.
    pub keys: Ref<targets::I16>,
}

impl Record for AnimationBlockI16 {
    const TAG: [u8; 4] = *b"SDS6";
    const VERSION: u32 = 0;
}

/// Fields of the native `SDU6` section, in serialized order.
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
pub struct AnimationBlockU16 {
    /// I32_ time coordinates corresponding to the entries in `keys`.
    pub timestamps: Ref<targets::I32>,
    /// Keyframe-block control bits.
    pub flags: AnimationBlockFlags,
    /// Ending frame coordinate recorded for the animation block.
    pub end_frame: u32,
    /// Key values selected in parallel with the block timestamp array.
    pub keys: Ref<targets::U16>,
}

impl Record for AnimationBlockU16 {
    const TAG: [u8; 4] = *b"SDU6";
    const VERSION: u32 = 0;
}

/// Fields of the native `SDS3` section, in serialized order.
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
pub struct AnimationBlockI32 {
    /// I32_ time coordinates corresponding to the entries in `keys`.
    pub timestamps: Ref<targets::I32>,
    /// Keyframe-block control bits.
    pub flags: AnimationBlockFlags,
    /// Ending frame coordinate recorded for the animation block.
    pub end_frame: u32,
    /// Key values selected in parallel with the block timestamp array.
    pub keys: Ref<targets::I32>,
}

impl Record for AnimationBlockI32 {
    const TAG: [u8; 4] = *b"SDS3";
    const VERSION: u32 = 0;
}

/// Fields of the native `SDU3` section, in serialized order.
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
pub struct AnimationBlockU32 {
    /// I32_ time coordinates corresponding to the entries in `keys`.
    pub timestamps: Ref<targets::I32>,
    /// Keyframe-block control bits.
    pub flags: AnimationBlockFlags,
    /// Ending frame coordinate recorded for the animation block.
    pub end_frame: u32,
    /// Key values selected in parallel with the block timestamp array.
    pub keys: Ref<targets::U32>,
}

impl Record for AnimationBlockU32 {
    const TAG: [u8; 4] = *b"SDU3";
    const VERSION: u32 = 0;
}

/// Fields of the native `SDFG` section, in serialized order.
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
pub struct AnimationBlockFlag {
    /// I32_ time coordinates corresponding to the entries in `keys`.
    pub timestamps: Ref<targets::I32>,
    /// Keyframe-block control bits.
    pub flags: AnimationBlockFlags,
    /// Ending frame coordinate recorded for the animation block.
    pub end_frame: u32,
    /// Key values selected in parallel with the block timestamp array.
    pub keys: Ref<targets::Flag>,
}

impl Record for AnimationBlockFlag {
    const TAG: [u8; 4] = *b"SDFG";
    const VERSION: u32 = 0;
}

/// Fields of the native `SDMB` section, in serialized order.
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
pub struct AnimationBlockExtent {
    /// I32_ time coordinates corresponding to the entries in `keys`.
    pub timestamps: Ref<targets::I32>,
    /// Keyframe-block control bits.
    pub flags: AnimationBlockFlags,
    /// Ending frame coordinate recorded for the animation block.
    pub end_frame: u32,
    /// Key values selected in parallel with the block timestamp array.
    pub keys: Ref<targets::Extent>,
}

impl Record for AnimationBlockExtent {
    const TAG: [u8; 4] = *b"SDMB";
    const VERSION: u32 = 0;
}

/// Fields of the native `SS32` section, in serialized order.
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
pub struct AnimationValueI32 {
    /// Inline animated-property metadata with its initial and reset values.
    pub value: AnimationValue<i32>,
}

impl Record for AnimationValueI32 {
    const TAG: [u8; 4] = *b"SS32";
    const VERSION: u32 = 0;
}

/// Fields of the native `SU32` section, in serialized order.
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
pub struct AnimationValueU32 {
    /// Inline animated-property metadata with its initial and reset values.
    pub value: AnimationValue<u32>,
}

impl Record for AnimationValueU32 {
    const TAG: [u8; 4] = *b"SU32";
    const VERSION: u32 = 0;
}

/// Fields of the native `SR32` section, in serialized order.
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
pub struct AnimationValueReal {
    /// Inline animated-property metadata with its initial and reset values.
    pub value: AnimationValue<f32>,
}

impl Record for AnimationValueReal {
    const TAG: [u8; 4] = *b"SR32";
    const VERSION: u32 = 0;
}

/// Fields of the native `SVC3` section, in serialized order.
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
pub struct AnimationValueVector3 {
    /// Inline animated-property metadata with its initial and reset values.
    pub value: AnimationValue<[f32; 3]>,
}

impl Record for AnimationValueVector3 {
    const TAG: [u8; 4] = *b"SVC3";
    const VERSION: u32 = 0;
}

/// Fields of the native `SVC2` section, in serialized order.
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
pub struct AnimationValueVector2 {
    /// Inline animated-property metadata with its initial and reset values.
    pub value: AnimationValue<[f32; 2]>,
}

impl Record for AnimationValueVector2 {
    const TAG: [u8; 4] = *b"SVC2";
    const VERSION: u32 = 0;
}

/// Fields of the native `MT16` section, in serialized order.
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
pub struct MeshTriangle16 {
    /// Seven packed face lanes; their per-lane interpretation is not exposed as separate fields.
    pub value: [u16; 7],
}

impl Record for MeshTriangle16 {
    const TAG: [u8; 4] = *b"MT16";
    const VERSION: u32 = 0;
}

/// Fields of the native `MT32` section, in serialized order.
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
pub struct MeshTriangle32 {
    /// Seven packed face lanes; their per-lane interpretation is not exposed as separate fields.
    pub value: [u32; 7],
}

impl Record for MeshTriangle32 {
    const TAG: [u8; 4] = *b"MT32";
    const VERSION: u32 = 0;
}

/// Fields of the native `SCHR` section, in serialized order.
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
pub struct StringReference {
    /// CHAR link describing one string, with its terminator included in the entry count.
    pub value: Ref<targets::Character>,
}

impl Record for StringReference {
    const TAG: [u8; 4] = *b"SCHR";
    const VERSION: u32 = 0;
}

/// Fields of the native `PHCT` section, in serialized order.
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
pub struct PhysicsConstraint {
    /// U16_ array listing rigid bodies dependent on this constraint.
    pub dependents: Ref<targets::U16>,
    /// Index of the first rigid body participating in the constraint.
    pub rigid_body_1: u16,
    /// Index of the second rigid body participating in the constraint.
    pub rigid_body_2: u16,
    /// Constraint control bits.
    pub flags: ConstraintFlags,
    /// Force threshold recorded for breaking the constraint.
    pub break_force: f32,
}

impl Record for PhysicsConstraint {
    const TAG: [u8; 4] = *b"PHCT";
    const VERSION: u32 = 0;
}
