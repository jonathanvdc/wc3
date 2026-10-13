//! Mesh divisions connect index buffers, regions, material batches, and animated bounds.
//!
//! Each record type selects its own section revision. Region vertex ranges refer
//! to the model packed vertex data; batch material indices address MATM entries.
use super::dialect::*;
use super::values::{BatchFlags, RegionFlags};
use super::{targets, AnimationValue, Extent};
use crate::model::m3::{self, Record, Ref};
use core::fmt::Debug;
/// Native field selections for independent `Region` section revisions.
pub trait RegionDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `unknown`; `()` omits the field.
    type Unknown: NativeField;
    /// Field representation for `first_vertex`; `()` omits the field.
    type FirstVertex: NativeField;
    /// Field representation for `vertex_count`; `()` omits the field.
    type VertexCount: NativeField;
    /// Field representation for `first_index`; `()` omits the field.
    type FirstIndex: NativeField;
    /// Field representation for `index_count`; `()` omits the field.
    type IndexCount: NativeField;
    /// Field representation for `flags`; `()` omits the field.
    type Flags: NativeField;
    /// Field representation for `uv_scale`; `()` omits the field.
    type UvScale: NativeField;
    /// Field representation for `uv_offset`; `()` omits the field.
    type UvOffset: NativeField;
}

impl RegionDialect for V0 {
    const VERSION: u32 = 0;
    type Unknown = ();
    type FirstVertex = u16;
    type VertexCount = u16;
    type FirstIndex = ();
    type IndexCount = ();
    type Flags = ();
    type UvScale = ();
    type UvOffset = ();
}

impl RegionDialect for V1 {
    const VERSION: u32 = 1;
    type Unknown = ();
    type FirstVertex = u16;
    type VertexCount = u16;
    type FirstIndex = ();
    type IndexCount = ();
    type Flags = ();
    type UvScale = ();
    type UvOffset = ();
}

impl RegionDialect for V2 {
    const VERSION: u32 = 2;
    type Unknown = ();
    type FirstVertex = u16;
    type VertexCount = u16;
    type FirstIndex = u32;
    type IndexCount = u32;
    type Flags = ();
    type UvScale = ();
    type UvOffset = ();
}

impl RegionDialect for V3 {
    const VERSION: u32 = 3;
    type Unknown = u32;
    type FirstVertex = u32;
    type VertexCount = u32;
    type FirstIndex = u32;
    type IndexCount = u32;
    type Flags = ();
    type UvScale = ();
    type UvOffset = ();
}

impl RegionDialect for V4 {
    const VERSION: u32 = 4;
    type Unknown = u32;
    type FirstVertex = u32;
    type VertexCount = u32;
    type FirstIndex = u32;
    type IndexCount = u32;
    type Flags = RegionFlags;
    type UvScale = ();
    type UvOffset = ();
}

impl RegionDialect for V5 {
    const VERSION: u32 = 5;
    type Unknown = u32;
    type FirstVertex = u32;
    type VertexCount = u32;
    type FirstIndex = u32;
    type IndexCount = u32;
    type Flags = RegionFlags;
    type UvScale = f32;
    type UvOffset = f32;
}

/// Fields of the native `REGN` section, in serialized order.
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
pub struct Region<D: RegionDialect> {
    /// Identifier stored in this region record.
    pub index: u32,
    /// Additional region word before the vertex range; its purpose is unresolved.
    pub unknown: D::Unknown,
    /// Starting element of the region in the packed vertex buffer.
    pub first_vertex: D::FirstVertex,
    /// Length of the vertex range beginning at `first_vertex`.
    pub vertex_count: D::VertexCount,
    /// Starting element of this region in the division face-index array.
    pub first_index: D::FirstIndex,
    /// Length of the face-index range; each triangle consumes three indices.
    pub index_count: D::IndexCount,
    /// Number of bones associated with this region.
    pub bone_count: u16,
    /// Starting element of this region in the model bone-lookup array.
    pub first_bone_lookup: u16,
    /// Length of the bone-lookup range beginning at `first_bone_lookup`.
    pub bone_lookup_count: u16,
    /// Two bytes separating the bone lookup metadata from the per-vertex lane counts.
    pub padding: u16,
    /// Number of weight pairs encoded for each vertex in this region.
    pub bone_weight_pairs: u8,
    /// Uninterpreted byte following the per-vertex lookup count; reference models commonly store one.
    pub unknown_byte: u8,
    /// BONE record index identifying the region root.
    pub root_bone: u16,
    /// Region behavior bits, including visibility and cloth participation.
    pub flags: D::Flags,
    /// Multiplicative adjustment applied to the region texture coordinates.
    pub uv_scale: D::UvScale,
    /// Additive adjustment applied to the region texture coordinates.
    pub uv_offset: D::UvOffset,
}

impl<D: RegionDialect> Record for Region<D> {
    const TAG: [u8; 4] = *b"REGN";
    const VERSION: u32 = D::VERSION;
}

/// Native field selections for independent `Batch` section revisions.
pub trait BatchDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Native representation of the batch behavior flags.
    type Flags: NativeField;
    /// Native representation of the draw priority plane.
    type PriorityPlane: NativeField;
}

impl BatchDialect for V0 {
    const VERSION: u32 = 0;
    type Flags = BatchFlags;
    type PriorityPlane = u16;
}

impl BatchDialect for V1 {
    const VERSION: u32 = 1;
    type Flags = BatchFlags;
    type PriorityPlane = u16;
}

/// Fields of the native `BAT_` section, in serialized order.
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
pub struct Batch<D: BatchDialect> {
    /// Batch behavior flags.
    pub flags: D::Flags,
    /// Draw-order priority plane.
    pub priority_plane: D::PriorityPlane,
    /// REGN record selected for this draw batch.
    pub region_index: u16,
    /// Index of the MSEC bounds control for this batch.
    pub bounds_index: u16,
    /// Native color-control index associated with the batch.
    pub color_index: u16,
    /// MATM record selected for this draw batch.
    pub material_index: u16,
    /// BONE index gating batch visibility; `0xffff` disables the gate.
    pub visibility_bone: u16,
}

impl<D: BatchDialect> Record for Batch<D> {
    const TAG: [u8; 4] = *b"BAT_";
    const VERSION: u32 = D::VERSION;
}

/// Native field selections for independent `MeshSection` section revisions.
pub trait MeshSectionDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `node_index`; `()` omits the field.
    type NodeIndex: NativeField;
}

impl MeshSectionDialect for V0 {
    const VERSION: u32 = 0;
    type NodeIndex = u32;
}

impl MeshSectionDialect for V1 {
    const VERSION: u32 = 1;
    type NodeIndex = u32;
}

/// Fields of the native `MSEC` section, in serialized order.
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
pub struct MeshSection<D: MeshSectionDialect> {
    /// BONE record whose mesh extent is described by this section.
    pub node_index: D::NodeIndex,
    /// Bounding box and radius with an animation identifier and default/reset values.
    pub bounds: AnimationValue<Extent>,
}

impl<D: MeshSectionDialect> Record for MeshSection<D> {
    const TAG: [u8; 4] = *b"MSEC";
    const VERSION: u32 = D::VERSION;
}

/// Native field selections for independent `MeshDivision` section revisions.
pub trait MeshDivisionDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `faces`; `()` omits the field.
    type Faces: NativeField;
}

impl MeshDivisionDialect for V0 {
    const VERSION: u32 = 0;
    type Faces = Ref<targets::U16>;
}

impl MeshDivisionDialect for V1 {
    const VERSION: u32 = 1;
    type Faces = Ref<targets::U16>;
}

impl MeshDivisionDialect for V2 {
    const VERSION: u32 = 2;
    type Faces = Ref<targets::U16>;
}

/// Fields of the native `DIV_` section, in serialized order.
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
pub struct MeshDivision<D: MeshDivisionDialect> {
    /// U16_ array containing the division triangle vertex indices.
    pub faces: D::Faces,
    /// REGN records partitioning the division into vertex and face-index ranges.
    pub regions: Ref<targets::Region>,
    /// BAT_ records assigning materials to the division regions.
    pub batches: Ref<targets::Batch>,
    /// MSEC records providing animated bounds for nodes in this division.
    pub msec: Ref<targets::MeshSection>,
    /// Instance count recorded for this mesh division.
    pub instances: u32,
}

impl<D: MeshDivisionDialect> Record for MeshDivision<D> {
    const TAG: [u8; 4] = *b"DIV_";
    const VERSION: u32 = D::VERSION;
}
