//! Polygon loops, draw-section metadata, attributes, and skinning remain separate tables.
//!
//! Attribute domains distinguish geometric vertices from shading corners. Section
//! membership is carried by face attributes rather than by contiguous face ranges.
//! Skinning and explicit triangulation use boundary arrays to partition variable
//! per-vertex or per-face data; repair records retain earlier geometry identities.
use super::*;
use crate::model::wem::{self, InlineOption, Record, Ref};
/// A vertex duplication recorded by geometry repair.
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
pub struct VertexSplit {
    /// Vertex identity that existed before the repair split.
    pub original: u32,
    /// Vertex identity allocated as a copy of `original` during repair.
    pub created: u32,
}
impl Record for VertexSplit {
    const TAG: [u8; 4] = *b"VSPL";
    const VERSION: u32 = 1;
}

/// A removed polygon with enough metadata to restore its original position.
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
pub struct FaceRecord {
    /// Ordered vertex identities of a removed face.
    pub corners: Ref<targets::U32>,
    /// Draw-section index originally assigned to the removed face.
    pub section: u32,
    /// Original face position before removal; this supports reconstructing its ordering.
    pub index: u32,
}
impl Record for FaceRecord {
    const TAG: [u8; 4] = *b"FDRP";
    const VERSION: u32 = 1;
}

/// The vertex and face changes retained from geometry repair.
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
pub struct RepairLog {
    /// Vertex duplication records ordered by when the repair applied them.
    pub splits: Ref<targets::VertexSplit>,
    /// Removed faces, including their vertex loops and original section assignments.
    pub dropped_faces: Ref<targets::FaceRecord>,
}

/// One named attribute array over a mesh domain.
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
pub struct AttrLayer {
    /// CHAR bytes naming the attribute layer.
    pub name: Ref<targets::Character>,
    /// Domain code: vertex, half-edge, edge, face, or whole mesh.
    pub domain: Domain,
    /// Element interpretation, including scalar/vector width and quaternion or boolean semantics.
    pub type_: AttrType,
    /// Encoding preference for render/export views; it does not change the unpacked bytes in `data`.
    pub storage: AttributeEncoding,
    /// Unpacked attribute bytes, with one element per member of the selected domain.
    pub data: Ref<targets::Byte>,
}
impl Record for AttrLayer {
    const TAG: [u8; 4] = *b"ATTR";
    const VERSION: u32 = 1;
}

/// Domain sizes and the attribute layers indexed by those domains.
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
pub struct AttributeSet {
    /// Element counts ordered as vertex, half-edge, edge, face, then mesh.
    pub domain_counts: [u32; 5],
    /// Named attribute layers sharing these domain counts.
    pub layers: Ref<targets::AttrLayer>,
}

/// Draw metadata assigned through per-face section attributes.
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
pub struct MeshSection {
    /// CHAR bytes naming the draw section.
    pub name: Ref<targets::Character>,
    /// Document material-slot index; `0xffffffff` means no material has been assigned.
    pub material_slot: u32,
    /// Bitmask selecting the native profiles in which the section participates.
    pub profiles: u32,
    /// Optional document node index binding every section vertex at full weight.
    pub rigid_node: InlineOption<u32>,
    /// Native selection-group identity retained for profile-specific output.
    pub selection_group: u16,
    /// Section behavior bits for hidden geometry, cloth participation, billboarding, and projected shadows.
    pub flags: SectionFlags,
    /// Box and radius associated with the draw section.
    pub bounds: Extent,
    /// Profile-specific section metadata stored outside the common geometry fields.
    pub native: NativeBag,
}
impl Record for MeshSection {
    const TAG: [u8; 4] = *b"SECT";
    const VERSION: u32 = 1;
}

/// Static field selections for `Mesh` section or inline context revisions.
pub trait MeshDialect: Revision {
    /// Independent section revision.
    const VERSION: u32;
    /// Representation of `triangulation`; `()` omits this slot.
    type Triangulation: NativeField;
}

impl MeshDialect for V1 {
    const VERSION: u32 = 1;
    type Triangulation = ();
}

impl MeshDialect for V2 {
    const VERSION: u32 = 2;
    type Triangulation = FaceTriangulation;
}

/// Polygon geometry, domain attributes, skinning, and draw-section metadata.
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
pub struct Mesh<D: MeshDialect> {
    /// CHAR bytes naming the mesh.
    pub name: Ref<targets::Character>,
    /// Level-of-detail identity; `0xffffffff` denotes geometry shared by all levels.
    pub lod_level: u32,
    /// Polygon topology expressed by face valences and concatenated vertex indices.
    pub face_set: FaceSet,
    /// Domain counts and named attribute arrays, including the per-face section assignment.
    pub attributes: AttributeSet,
    /// Variable-length bone influences associated with mesh vertices.
    pub skin: SkinBinding,
    /// Draw-section metadata; the face section attribute identifies membership without requiring face ordering.
    pub sections: Ref<targets::MeshSection>,
    /// Vertex splits and removed faces retained from geometry repair.
    pub repair_log: RepairLog,
    /// Box and radius enclosing the mesh geometry.
    pub bounds: Extent,
    /// Optional per-face triangle decompositions selected by the mesh dialect.
    pub triangulation: D::Triangulation,
}
impl<D: MeshDialect> Record for Mesh<D> {
    const TAG: [u8; 4] = *b"MESH";
    const VERSION: u32 = D::VERSION;
}

/// One node contribution to a vertex skinning blend.
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
pub struct Influence {
    /// Global document node index receiving this skinning contribution.
    pub bone: u32,
    /// Contribution of this node to a vertex skinning blend.
    pub weight: f32,
}
impl Record for Influence {
    const TAG: [u8; 4] = *b"INFL";
    const VERSION: u32 = 1;
}

/// Per-vertex variable-length influence ranges.
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
pub struct SkinBinding {
    /// U32_ influence boundaries; a populated table has vertex count plus one entries.
    pub offsets: Ref<targets::U32>,
    /// Consecutive influence rows, partitioned per vertex by neighboring `offsets` entries.
    pub influences: Ref<targets::Influence>,
}

/// Polygon loops described by corner counts and vertex identities.
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
pub struct FaceSet {
    /// Number of vertex identities available to the face loops.
    pub vertex_count: u32,
    /// Corner count for each polygon, in face order.
    pub face_valence: Ref<targets::U32>,
    /// Concatenated polygon vertex indices; `face_valence` partitions the array into loops.
    pub corner_vertex: Ref<targets::U32>,
}

/// Per-face explicit triangle decompositions stored through boundary and vertex arrays.
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
pub struct FaceTriangulation {
    /// Per-face boundaries into `vertices`, with a final end boundary; an empty table stores no explicit triangulations.
    pub offsets: Ref<targets::U32>,
    /// Vertex indices grouped in triples; an empty face range leaves that face to automatic triangulation.
    pub vertices: Ref<targets::U32>,
}
