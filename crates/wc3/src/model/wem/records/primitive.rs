//! Typed primitive records for indexed WEM payloads and inline values.
use super::TPoseSource;
use crate::model::wem::{self, Record};
/// Typed character payload in its serialized field order.
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
pub struct Character {
    /// One byte of a counted CHAR string; no terminator is implied.
    pub value: u8,
}
impl Record for Character {
    const TAG: [u8; 4] = *b"CHAR";
    const VERSION: u32 = 0;
}

/// Typed byte payload in its serialized field order.
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
pub struct Byte {
    /// One octet of an opaque or packed byte stream.
    pub value: u8,
}
impl Record for Byte {
    const TAG: [u8; 4] = *b"U8__";
    const VERSION: u32 = 0;
}

/// Typed u16 payload in its serialized field order.
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
pub struct U16 {
    /// Unsigned 16-bit element; its interpretation follows the referring field.
    pub value: u16,
}
impl Record for U16 {
    const TAG: [u8; 4] = *b"U16_";
    const VERSION: u32 = 0;
}

/// Typed u32 payload in its serialized field order.
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
pub struct U32 {
    /// Unsigned 32-bit element; its interpretation follows the referring field.
    pub value: u32,
}
impl Record for U32 {
    const TAG: [u8; 4] = *b"U32_";
    const VERSION: u32 = 0;
}

/// Typed u64 payload in its serialized field order.
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
pub struct U64 {
    /// Unsigned 64-bit element; its interpretation follows the referring field.
    pub value: u64,
}
impl Record for U64 {
    const TAG: [u8; 4] = *b"U64_";
    const VERSION: u32 = 0;
}

/// Typed i16 payload in its serialized field order.
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
pub struct I16 {
    /// Signed 16-bit element; its interpretation follows the referring field.
    pub value: i16,
}
impl Record for I16 {
    const TAG: [u8; 4] = *b"I16_";
    const VERSION: u32 = 0;
}

/// Typed i32 payload in its serialized field order.
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
pub struct I32 {
    /// Signed 32-bit element; its interpretation follows the referring field.
    pub value: i32,
}
impl Record for I32 {
    const TAG: [u8; 4] = *b"I32_";
    const VERSION: u32 = 0;
}

/// Typed real payload in its serialized field order.
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
pub struct Real {
    /// Single-precision element; its units follow the referring field.
    pub value: f32,
}
impl Record for Real {
    const TAG: [u8; 4] = *b"REAL";
    const VERSION: u32 = 0;
}

/// Typed vector2 payload in its serialized field order.
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
pub struct Vector2 {
    /// Two floating-point components in serialized order.
    pub value: [f32; 2],
}
impl Record for Vector2 {
    const TAG: [u8; 4] = *b"VEC2";
    const VERSION: u32 = 0;
}

/// Typed vector3 payload in its serialized field order.
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
pub struct Vector3 {
    /// Three floating-point components in serialized order.
    pub value: [f32; 3],
}
impl Record for Vector3 {
    const TAG: [u8; 4] = *b"VEC3";
    const VERSION: u32 = 0;
}

/// Typed vector4 payload in its serialized field order.
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
pub struct Vector4 {
    /// Four floating-point components in serialized order.
    pub value: [f32; 4],
}
impl Record for Vector4 {
    const TAG: [u8; 4] = *b"VEC4";
    const VERSION: u32 = 0;
}

/// Typed quaternion record payload in its serialized field order.
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
pub struct QuaternionRecord {
    /// Quaternion components in x, y, z, w order.
    pub value: [f32; 4],
}
impl Record for QuaternionRecord {
    const TAG: [u8; 4] = *b"QUAT";
    const VERSION: u32 = 0;
}

/// Typed byte4 payload in its serialized field order.
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
pub struct Byte4 {
    /// Four byte components kept in their serialized order.
    pub value: [u8; 4],
}
impl Record for Byte4 {
    const TAG: [u8; 4] = *b"BYT4";
    const VERSION: u32 = 0;
}

/// Typed matrix44 payload in its serialized field order.
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
pub struct Matrix44 {
    /// Sixteen matrix entries grouped in their serialized four-by-four order.
    pub value: [[f32; 4]; 4],
}
impl Record for Matrix44 {
    const TAG: [u8; 4] = *b"MTX4";
    const VERSION: u32 = 1;
}

/// Typed t pose source record payload in its serialized field order.
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
pub struct TPoseSourceRecord {
    /// Provenance of the pose used to construct the bone setup.
    pub value: TPoseSource,
}
impl Record for TPoseSourceRecord {
    const TAG: [u8; 4] = *b"TPSR";
    const VERSION: u32 = 0;
}
