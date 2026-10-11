//! Mesh geometry, material references, bounds, and skinning.
use crate::model::conversion::ConversionContext;
use crate::model::mdx::read_array;
use crate::model::mdx::read_array_layout;
use crate::model::mdx::WireValue;
use crate::model::ConversionError;
use crate::model::Cursor;
use crate::model::Encoder;
use crate::model::FixedText;
use crate::model::GeosetsChunk;
use crate::model::KnownChunk;
use crate::model::ModelDialect;
use crate::model::ValueError;
use crate::model::{mdl, mdx};
use crate::model::{Borrowed, Detached, ModelOwnership, ModelPartialEq};
use crate::model::{Model, ZeroVec};
use crate::model::{ModelVersion, SupportsReforgedChunks, Tag, Vec3, Version};
use std::borrow::Cow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::take;

/// A geoset's bounding volume, also used for each sequence extent.
#[derive(Clone, Copy, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(fields, write_order(minimum, maximum, bounds_radius))]
pub struct GeosetExtent {
    #[mdl(property = "BoundsRadius", default)]
    /// Authored bounding-sphere radius in model units.
    pub bounds_radius: f32,
    #[mdl(property = "MinimumExtent", default)]
    /// Minimum corner of the model-space axis-aligned bounds.
    pub minimum: Vec3,
    #[mdl(property = "MaximumExtent", default)]
    /// Maximum corner of the model-space axis-aligned bounds.
    pub maximum: Vec3,
}

impl Default for GeosetExtent {
    fn default() -> Self {
        Self {
            bounds_radius: 0.0,
            minimum: [0.0; 3],
            maximum: [0.0; 3],
        }
    }
}

/// Four bone influences in their version-selected integer representation.
/// Versions 900 through 1300 use `SkinWeights<u8>`; versions 1400 and later
/// use `SkinWeights<u16>`. Influence weights must remain in the range 0..=255.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, mdx::Read, mdx::Write)]
pub struct SkinWeights<T = u16> {
    /// Four node indices corresponding to the stored weights.
    pub bone_indices: [T; 4],
    /// Four integer influence weights, expressed as fractions of 255.
    pub weights: [T; 4],
}

/// Common access to the two version-selected skin-weight layouts.
pub trait SkinWeight:
    AsULE + Copy + Debug + Eq + mdx::Write + for<'input> mdx::Read<'input> + 'static
{
    /// Node indices widened to a common representation without allocating.
    fn bone_indices(self) -> [u16; 4];
    /// Influence weights widened to a common representation without allocating.
    fn weights(self) -> [u16; 4];
    /// Constructs a row, checking the selected layout and the weight range.
    fn from_influences(bone_indices: [u16; 4], weights: [u16; 4]) -> Option<Self>;
    #[doc(hidden)]
    fn into_narrow<'data>(values: ZeroVec<'data, Self>) -> Option<ZeroVec<'data, SkinWeights<u8>>>;
    #[doc(hidden)]
    fn into_wide<'data>(values: ZeroVec<'data, Self>) -> ZeroVec<'data, SkinWeights<u16>>;
    #[doc(hidden)]
    fn convert_array<'data, S: SkinWeight>(
        values: ZeroVec<'data, S>,
    ) -> Option<ZeroVec<'data, Self>>;
}

impl SkinWeight for SkinWeights<u8> {
    fn bone_indices(self) -> [u16; 4] {
        self.bone_indices.map(u16::from)
    }
    fn weights(self) -> [u16; 4] {
        self.weights.map(u16::from)
    }
    fn from_influences(bone_indices: [u16; 4], weights: [u16; 4]) -> Option<Self> {
        if bone_indices
            .iter()
            .chain(&weights)
            .any(|&value| value > 255)
        {
            return None;
        }
        Some(Self {
            bone_indices: bone_indices.map(|value| value as u8),
            weights: weights.map(|value| value as u8),
        })
    }
    fn into_narrow<'data>(values: ZeroVec<'data, Self>) -> Option<ZeroVec<'data, Self>> {
        Some(values)
    }
    fn into_wide<'data>(values: ZeroVec<'data, Self>) -> ZeroVec<'data, SkinWeights<u16>> {
        values
            .iter()
            .map(|row| SkinWeights {
                bone_indices: row.bone_indices(),
                weights: row.weights(),
            })
            .collect()
    }
    fn convert_array<'data, S: SkinWeight>(
        values: ZeroVec<'data, S>,
    ) -> Option<ZeroVec<'data, Self>> {
        S::into_narrow(values)
    }
}
impl SkinWeight for SkinWeights<u16> {
    fn bone_indices(self) -> [u16; 4] {
        self.bone_indices
    }
    fn weights(self) -> [u16; 4] {
        self.weights
    }
    fn from_influences(bone_indices: [u16; 4], weights: [u16; 4]) -> Option<Self> {
        weights.iter().all(|&value| value <= 255).then_some(Self {
            bone_indices,
            weights,
        })
    }
    fn into_narrow<'data>(values: ZeroVec<'data, Self>) -> Option<ZeroVec<'data, SkinWeights<u8>>> {
        values
            .iter()
            .map(|row| SkinWeights::<u8>::from_influences(row.bone_indices, row.weights))
            .collect()
    }
    fn into_wide<'data>(values: ZeroVec<'data, Self>) -> ZeroVec<'data, Self> {
        values
    }
    fn convert_array<'data, S: SkinWeight>(
        values: ZeroVec<'data, S>,
    ) -> Option<ZeroVec<'data, Self>> {
        Some(S::into_wide(values))
    }
}

/// Optional sections retain their original order for serialization.
#[derive(Clone, Debug, PartialEq, ModelOwnership)]
enum GeosetExtraSection<'data, S: SkinWeight> {
    Tangents(ZeroVec<'data, [f32; 4]>),
    Skin { weights: ZeroVec<'data, S> },
}

/// Storage and serialization of the optional TANG and SKIN sections for a version.
pub trait GeosetExtraSections<'data, S: SkinWeight>: Default + Clone + Debug + PartialEq {
    /// Reads the optional sections supported by the selected model version.
    fn read(_: &mut Cursor<'data>) -> Result<Self, mdx::ReadError> {
        Ok(Self::default())
    }
    /// Writes the stored optional sections for the selected model version.
    fn write(&self, _: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        Ok(())
    }
    /// Borrows Reforged sections, or returns `None` for layouts without them.
    fn reforged(&self) -> Option<&ReforgedGeosetExtraSections<'data, S>> {
        None
    }
    /// Mutably borrows Reforged sections, or returns `None` for layouts without them.
    fn reforged_mut(&mut self) -> Option<&mut ReforgedGeosetExtraSections<'data, S>> {
        None
    }
}

/// Classic geosets have no optional TANG or SKIN sections.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct NoGeosetExtraSections;

impl<'data, S: SkinWeight> GeosetExtraSections<'data, S> for NoGeosetExtraSections {}

/// Optional TANG and SKIN sections, retained in file order.
#[derive(Clone, Debug, PartialEq, ModelOwnership)]
pub struct ReforgedGeosetExtraSections<'data, S: SkinWeight = SkinWeights> {
    sections: Vec<GeosetExtraSection<'data, S>>,
}

impl<S: SkinWeight> Default for ReforgedGeosetExtraSections<'_, S> {
    fn default() -> Self {
        Self {
            sections: Vec::new(),
        }
    }
}

impl<'data, S: SkinWeight> GeosetExtraSections<'data, S> for ReforgedGeosetExtraSections<'data, S> {
    fn read(cursor: &mut Cursor<'data>) -> Result<Self, mdx::ReadError> {
        Ok(Self {
            sections: read_extra_sections::<S>(cursor)?,
        })
    }
    fn write(&self, bytes: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        write_extra_sections::<S>(bytes, &self.sections)
    }
    fn reforged(&self) -> Option<&Self> {
        Some(self)
    }
    fn reforged_mut(&mut self) -> Option<&mut Self> {
        Some(self)
    }
}

/// Level-of-detail index and name added in version 900. Both fields are present
/// together, and the exact name bytes are retained for round-trip encoding.
#[derive(Clone, Debug, PartialEq, Default, mdx::Read, mdx::Write)]
pub struct GeosetLevelOfDetailFields {
    pub level_of_detail: u32,
    pub name: FixedText<80>,
}

/// The level-of-detail fields selected by a model version.
pub trait GeosetLevelOfDetail:
    Default + for<'data> mdx::Read<'data> + mdx::Write + Clone + Debug + PartialEq
{
    fn fixed_name(&self) -> Option<&FixedText<80>> {
        None
    }
    fn level_of_detail(&self) -> Option<u32> {
        None
    }
    fn name(&self) -> Option<Cow<'_, str>> {
        None
    }
    fn level_of_detail_mut(&mut self) -> Option<&mut u32> {
        None
    }
    fn name_mut(&mut self) -> Option<&mut FixedText<80>> {
        None
    }
}

#[derive(Clone, Debug, PartialEq, Default, mdx::Read, mdx::Write)]
pub struct NoGeosetLevelOfDetail;

impl GeosetLevelOfDetail for NoGeosetLevelOfDetail {}

impl GeosetLevelOfDetail for GeosetLevelOfDetailFields {
    fn fixed_name(&self) -> Option<&FixedText<80>> {
        Some(&self.name)
    }
    fn level_of_detail(&self) -> Option<u32> {
        Some(self.level_of_detail)
    }
    fn name(&self) -> Option<Cow<'_, str>> {
        Some(self.name.text())
    }
    fn level_of_detail_mut(&mut self) -> Option<&mut u32> {
        Some(&mut self.level_of_detail)
    }
    fn name_mut(&mut self) -> Option<&mut FixedText<80>> {
        Some(&mut self.name)
    }
}

/// Chooses geoset fields for a model version.
pub trait GeosetLayout {
    /// Storage for the authored LOD number and name.
    type LevelOfDetail: GeosetLevelOfDetail;
    /// Integer layout used by skin-weight rows.
    type SkinWeights: SkinWeight;
    /// Storage for optional tangent and skinning sections.
    type ExtraSections<'data>: GeosetExtraSections<'data, Self::SkinWeights>
        + for<'view> Borrowed<'view, View = Self::ExtraSections<'view>>
        + for<'owned> Detached<'owned, Detached = Self::ExtraSections<'owned>>;
}
use crate::model::{V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900};
impl GeosetLayout for V800 {
    type SkinWeights = SkinWeights<u8>;
    type LevelOfDetail = NoGeosetLevelOfDetail;
    type ExtraSections<'data> = NoGeosetExtraSections;
}
impl GeosetLayout for V900 {
    type SkinWeights = SkinWeights<u8>;
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections<'data> = ReforgedGeosetExtraSections<'data, SkinWeights<u8>>;
}
impl GeosetLayout for V1000 {
    type SkinWeights = SkinWeights<u8>;
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections<'data> = ReforgedGeosetExtraSections<'data, SkinWeights<u8>>;
}
impl GeosetLayout for V1100 {
    type SkinWeights = SkinWeights<u8>;
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections<'data> = ReforgedGeosetExtraSections<'data, SkinWeights<u8>>;
}
impl GeosetLayout for V1200 {
    type SkinWeights = SkinWeights<u8>;
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections<'data> = ReforgedGeosetExtraSections<'data, SkinWeights<u8>>;
}
impl GeosetLayout for V1300 {
    type SkinWeights = SkinWeights<u8>;
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections<'data> = ReforgedGeosetExtraSections<'data, SkinWeights<u8>>;
}
impl GeosetLayout for V1400 {
    type SkinWeights = SkinWeights<u16>;
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections<'data> = ReforgedGeosetExtraSections<'data, SkinWeights<u16>>;
}
impl GeosetLayout for V1600 {
    type SkinWeights = SkinWeights<u16>;
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections<'data> = ReforgedGeosetExtraSections<'data, SkinWeights<u16>>;
}
impl GeosetLayout for V1800 {
    type SkinWeights = SkinWeights<u16>;
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections<'data> = ReforgedGeosetExtraSections<'data, SkinWeights<u16>>;
}

mod mdl_codec;
use mdl_codec::{
    AnimExtent, Faces, Groups, List, OptionalList, Selection, SkinRow, Uncounted, UvSet,
};

/// A mesh rendered with one material.
///
/// Vertex positions, normals, UV sets, and skin influences must agree in count.
/// Legacy vertex groups may be empty when Reforged skin weights cover every vertex.
/// Use the editing methods to keep those arrays consistent. Material and bone
/// references are model indices; changing collection order requires updating
/// the affected references.
#[derive(Clone, Debug, ModelPartialEq, mdl::Read, mdl::Write, ModelOwnership)]
#[mdl(block = "Geoset", validate_read = "Self::validate_mdl_read",
    validate_write = "Self::validate_mdl_write",
    write_order(positions, directions, uvs, tangents, skin, vertex_indices,
        triangles, matrices, material_id, selection_group, selection, lod, lod_name, extent, bounds),
    virtual_fields(
        #[mdl(property = "Vertices", delegate, get = "Self::mdl_positions", set = "Self::set_mdl_positions")]
        positions: List<Vec3>,
        #[mdl(property = "Normals", delegate, get = "Self::mdl_directions", set = "Self::set_mdl_directions")]
        directions: List<Vec3>,
        #[mdl(repeated = "TVertices", get = "Self::mdl_uvs", set = "Self::set_mdl_uvs")]
        uvs: Vec<UvSet>,
        #[mdl(property = "VertexGroup", delegate, get = "Self::mdl_vertex_indices", set = "Self::set_mdl_vertex_indices")]
        vertex_indices: Uncounted<u8>,
        #[mdl(property = "Faces", delegate, get = "Self::mdl_triangles", set = "Self::set_mdl_triangles")]
        triangles: Faces,
        #[mdl(property = "Groups", delegate, get = "Self::mdl_matrices", set = "Self::set_mdl_matrices")]
        matrices: Groups,
        #[mdl(flatten, get = "Self::mdl_selection", set = "Self::set_mdl_selection")]
        selection: Selection,
        #[mdl(property = "LevelOfDetail", default, get = "Self::mdl_lod", slot = "Self::mdl_lod_mut")]
        lod: u32,
        #[mdl(property = "Name", hive_name = "LevelOfDetailName", delegate, get = "Self::mdl_lod_name", set = "Self::set_mdl_lod_name")]
        lod_name: Option<FixedText<80>>,
        #[mdl(repeated = "Anim", get = "Self::mdl_bounds", set = "Self::set_mdl_bounds")]
        bounds: ZeroVec<'data, AnimExtent>,
        #[mdl(property = "Tangents", delegate, get = "Self::mdl_tangents", set = "Self::set_mdl_tangents")]
        tangents: OptionalList<[f32; 4]>,
        #[mdl(property = "SkinWeights", delegate, get = "Self::mdl_skin", set = "Self::set_mdl_skin")]
        skin: OptionalList<SkinRow<V::SkinWeights>>,
    )
)]
pub struct Geoset<'data, V: ModelVersion> {
    #[mdl(skip, default)]
    version: PhantomData<V>,
    #[mdl(skip, default)]
    vertices: ZeroVec<'data, Vec3>,
    #[mdl(skip, default)]
    normals: ZeroVec<'data, Vec3>,
    #[mdl(skip, default)]
    primitive_types: ZeroVec<'data, u32>,
    #[mdl(skip, default)]
    primitive_counts: ZeroVec<'data, u32>,
    #[mdl(skip, default)]
    faces: ZeroVec<'data, u16>,
    #[mdl(skip, default)]
    vertex_groups: ZeroVec<'data, u8>,
    #[mdl(skip, default)]
    matrix_group_sizes: ZeroVec<'data, u32>,
    #[mdl(skip, default)]
    matrix_indices: ZeroVec<'data, u32>,
    #[mdl(property = "MaterialID", default)]
    /// Referenced material index.
    pub material_id: u32,
    #[mdl(property = "SelectionGroup", default)]
    /// Selection group index.
    pub selection_group: u32,
    #[mdl(skip, default)]
    unselectable_raw: u32,
    #[mdl(skip, default)]
    level_of_detail: V::LevelOfDetail,
    #[mdl(flatten)]
    /// Overall geoset bounds.
    pub extent: GeosetExtent,
    #[mdl(skip, default)]
    /// Per-sequence bounding volumes.
    pub sequence_extents: ZeroVec<'data, GeosetExtent>,
    #[mdl(skip, default)]
    extra_sections: V::ExtraSections<'data>,
    #[mdl(skip, default)]
    uv_sets: Vec<ZeroVec<'data, [f32; 2]>>,
}

impl<'data, V: ModelVersion> Geoset<'data, V> {
    /// Builds a basic geoset with one matrix group and one UV set.
    pub fn new(vertices: &[Vec3], normals: &[Vec3], faces: &[u16]) -> Result<Self, ValueError> {
        Self::from_arrays(
            ZeroVec::alloc_from_slice(vertices),
            ZeroVec::alloc_from_slice(normals),
            ZeroVec::alloc_from_slice(faces),
        )
    }

    /// Creates geometry by moving already encoded arrays into the model.
    pub fn from_arrays(
        vertices: ZeroVec<'data, Vec3>,
        normals: ZeroVec<'data, Vec3>,
        faces: ZeroVec<'data, u16>,
    ) -> Result<Self, ValueError> {
        if vertices.len() != normals.len() {
            return Err(ValueError::LengthMismatch {
                tag: GeosetsChunk::<V>::TAG,
                expected: vertices.len(),
                actual: normals.len(),
            });
        }
        if faces.len() > u32::MAX as usize {
            return Err(ValueError::CountTooLarge {
                tag: GeosetsChunk::<V>::TAG,
                count: faces.len(),
            });
        }
        let vertex_count = vertices.len();
        let index_count = faces.len();
        Ok(Self {
            version: PhantomData,
            vertices,
            normals,
            primitive_types: [4].into_iter().collect(),
            primitive_counts: [index_count as u32].into_iter().collect(),
            faces,
            vertex_groups: (0..vertex_count).map(|_| 0u8).collect(),
            matrix_group_sizes: [1].into_iter().collect(),
            matrix_indices: [0].into_iter().collect(),
            material_id: 0,
            selection_group: 0,
            unselectable_raw: 0,
            level_of_detail: V::LevelOfDetail::default(),
            extent: GeosetExtent::default(),
            sequence_extents: ZeroVec::default(),
            extra_sections: V::ExtraSections::<'data>::default(),
            uv_sets: vec![(0..vertex_count).map(|_| [0.0; 2]).collect()],
        })
    }

    /// The MDX version used to interpret this record.
    pub fn version(&self) -> Version {
        V::NUMBER
    }

    /// Vertex positions in model coordinates.
    pub fn vertices(&self) -> &ZeroVec<'data, Vec3> {
        &self.vertices
    }
    /// Mutably borrows vertex positions for bulk edits.
    pub fn vertices_mut(&mut self) -> &mut ZeroVec<'data, Vec3> {
        &mut self.vertices
    }
    /// Borrows all vertex normals.
    pub fn normals(&self) -> &ZeroVec<'data, Vec3> {
        &self.normals
    }
    /// Mutably borrows normals for bulk edits.
    pub fn normals_mut(&mut self) -> &mut ZeroVec<'data, Vec3> {
        &mut self.normals
    }
    /// Primitive type for each face group; triangle groups use type 4.
    pub fn primitive_types(&self) -> &ZeroVec<'data, u32> {
        &self.primitive_types
    }
    /// Number of vertex indices in each primitive group.
    pub fn primitive_counts(&self) -> &ZeroVec<'data, u32> {
        &self.primitive_counts
    }
    /// Face indices into the vertex-position array.
    pub fn face_indices(&self) -> &ZeroVec<'data, u16> {
        &self.faces
    }
    /// Mutably borrows face indices for bulk edits.
    pub fn face_indices_mut(&mut self) -> &mut ZeroVec<'data, u16> {
        &mut self.faces
    }
    /// Borrows the matrix group assigned to each vertex.
    pub fn vertex_groups(&self) -> &ZeroVec<'data, u8> {
        &self.vertex_groups
    }
    /// Number of bone references in each matrix group.
    pub fn matrix_group_sizes(&self) -> &ZeroVec<'data, u32> {
        &self.matrix_group_sizes
    }
    /// Bone object IDs, concatenated in matrix-group order.
    pub fn matrix_indices(&self) -> &ZeroVec<'data, u32> {
        &self.matrix_indices
    }

    /// Returns the unselectable flag as a boolean.
    pub fn unselectable(&self) -> bool {
        self.unselectable_raw & 4 != 0
    }
    /// Returns the exact unselectable field, including nonstandard bits.
    pub fn raw_unselectable(&self) -> u32 {
        self.unselectable_raw
    }
    /// Returns the Reforged level of detail, if that field exists.
    pub fn try_level_of_detail(&self) -> Result<u32, ValueError> {
        if V::NUMBER < 900 {
            return Err(ValueError::UnsupportedVersion {
                tag: *b"GEOS",
                minimum: 900,
                actual: V::NUMBER,
            });
        }
        Ok(self
            .level_of_detail
            .level_of_detail()
            .expect("supported version"))
    }

    /// Borrows every UV coordinate set.
    pub fn uv_sets(&self) -> &[ZeroVec<'data, [f32; 2]>] {
        &self.uv_sets
    }
    /// Mutably borrows one UV set for bulk edits.
    pub fn uv_set_mut(&mut self, index: usize) -> Option<&mut ZeroVec<'data, [f32; 2]>> {
        self.uv_sets.get_mut(index)
    }

    /// Returns the level-of-detail name, if this version supports it.
    pub fn try_name(&self) -> Result<Cow<'_, str>, ValueError> {
        if V::NUMBER < 900 {
            return Err(ValueError::UnsupportedVersion {
                tag: *b"GEOS",
                minimum: 900,
                actual: V::NUMBER,
            });
        }
        Ok(self.level_of_detail.name().expect("supported version"))
    }

    /// Borrows optional Reforged tangent vectors.
    pub fn try_tangents(&self) -> Result<Option<&ZeroVec<'data, [f32; 4]>>, ValueError> {
        if V::NUMBER < 900 {
            return Err(ValueError::UnsupportedVersion {
                tag: *b"GEOS",
                minimum: 900,
                actual: V::NUMBER,
            });
        }
        Ok(self
            .extra_sections
            .reforged()
            .expect("supported version")
            .sections
            .iter()
            .find_map(|part| match part {
                GeosetExtraSection::Tangents(values) => Some(values),
                _ => None,
            }))
    }

    /// Borrows the four bone influences for each skinned vertex.
    pub fn try_skin_weights(&self) -> Result<Option<&ZeroVec<'data, V::SkinWeights>>, ValueError> {
        if V::NUMBER < 900 {
            return Err(ValueError::UnsupportedVersion {
                tag: *b"GEOS",
                minimum: 900,
                actual: V::NUMBER,
            });
        }
        Ok(self
            .extra_sections
            .reforged()
            .expect("supported version")
            .sections
            .iter()
            .find_map(|part| match part {
                GeosetExtraSection::Skin { weights, .. } => Some(weights),
                _ => None,
            }))
    }

    /// Changes one vertex position.
    pub fn set_vertex(&mut self, index: usize, vertex: Vec3) -> Result<(), ValueError> {
        set_array_value(&mut self.vertices, index, vertex)
    }

    /// Changes one vertex normal.
    pub fn set_normal(&mut self, index: usize, normal: Vec3) -> Result<(), ValueError> {
        set_array_value(&mut self.normals, index, normal)
    }

    /// Replaces the per-vertex matrix group indices.
    pub fn set_vertex_groups(&mut self, groups: &[u8]) -> Result<(), ValueError> {
        if groups.len() != self.vertices.len() {
            return Err(ValueError::LengthMismatch {
                tag: GeosetsChunk::<V>::TAG,
                expected: self.vertices.len(),
                actual: groups.len(),
            });
        }
        self.vertex_groups = ZeroVec::alloc_from_slice(groups);
        Ok(())
    }

    /// Replaces matrix groups and their flattened indices.
    pub fn set_matrix_groups(&mut self, groups: &[Vec<u32>]) -> Result<(), ValueError> {
        if let Some(group) = groups.iter().find(|group| group.len() > u32::MAX as usize) {
            return Err(ValueError::CountTooLarge {
                tag: GeosetsChunk::<V>::TAG,
                count: group.len(),
            });
        }
        self.matrix_group_sizes = groups.iter().map(|group| group.len() as u32).collect();
        self.matrix_indices = groups
            .iter()
            .flat_map(|group| group.iter().copied())
            .collect();
        Ok(())
    }

    /// Enables or disables unselectability while preserving other selection flags.
    pub fn set_unselectable(&mut self, value: bool) {
        self.unselectable_raw = (self.unselectable_raw & !4) | (u32::from(value) * 4);
    }
    /// Sets the exact unselectable field.
    pub fn set_raw_unselectable(&mut self, value: u32) {
        self.unselectable_raw = value;
    }

    /// Changes the Reforged level of detail.
    pub fn try_set_level_of_detail(&mut self, level: u32) -> Result<(), ValueError> {
        *self
            .level_of_detail
            .level_of_detail_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: GeosetsChunk::<V>::TAG,
                minimum: 900,
                actual: V::NUMBER,
            })? = level;
        Ok(())
    }

    /// Changes the Reforged name and clears unused name bytes.
    pub fn try_set_name(&mut self, name: &str) -> Result<(), ValueError> {
        self.level_of_detail
            .name_mut()
            .ok_or(ValueError::UnsupportedVersion {
                tag: GeosetsChunk::<V>::TAG,
                minimum: 900,
                actual: V::NUMBER,
            })?
            .set_text(name)?;
        Ok(())
    }

    /// Changes one per-sequence bound.
    pub fn set_sequence_extent(
        &mut self,
        index: usize,
        extent: GeosetExtent,
    ) -> Result<(), ValueError> {
        set_array_value(&mut self.sequence_extents, index, extent)
    }

    /// Replaces or removes the Reforged tangent section, retaining its order.
    pub fn try_set_tangents(
        &mut self,
        tangents: Option<ZeroVec<'data, [f32; 4]>>,
    ) -> Result<(), ValueError> {
        if V::NUMBER < 900 {
            return Err(ValueError::UnsupportedVersion {
                tag: GeosetsChunk::<V>::TAG,
                minimum: 900,
                actual: V::NUMBER,
            });
        }
        let sections = &mut self
            .extra_sections
            .reforged_mut()
            .expect("supported version")
            .sections;
        if let Some(index) = sections
            .iter()
            .position(|part| matches!(part, GeosetExtraSection::Tangents(_)))
        {
            if let Some(values) = tangents {
                sections[index] = GeosetExtraSection::Tangents(values);
            } else {
                sections.remove(index);
            }
        } else if let Some(values) = tangents {
            sections.insert(0, GeosetExtraSection::Tangents(values));
        }
        Ok(())
    }

    /// Replaces or removes per-vertex skin influences, retaining section order.
    pub fn try_set_skin_weights(
        &mut self,
        weights: Option<ZeroVec<'data, V::SkinWeights>>,
    ) -> Result<(), ValueError> {
        if V::NUMBER < 900 {
            return Err(ValueError::UnsupportedVersion {
                tag: *b"GEOS",
                minimum: 900,
                actual: V::NUMBER,
            });
        }
        if let Some(weights) = &weights {
            if weights.len() > u32::MAX as usize / 8 {
                return Err(ValueError::CountTooLarge {
                    tag: *b"SKIN",
                    count: weights.len().saturating_mul(8),
                });
            }
            if let Some(weight) = weights
                .iter()
                .flat_map(|row| row.weights())
                .find(|&weight| weight > 255)
            {
                return Err(ValueError::InvalidSkinWeight { weight });
            }
        }
        let sections = &mut self
            .extra_sections
            .reforged_mut()
            .expect("supported version")
            .sections;
        let new_part = weights.map(|weights| GeosetExtraSection::Skin { weights });
        if let Some(index) = sections
            .iter()
            .position(|part| matches!(part, GeosetExtraSection::Skin { .. }))
        {
            if let Some(part) = new_part {
                sections[index] = part;
            } else {
                sections.remove(index);
            }
        } else if let Some(part) = new_part {
            sections.push(part);
        }
        Ok(())
    }

    /// Changes one UV coordinate.
    pub fn set_uv(&mut self, set: usize, index: usize, uv: [f32; 2]) -> Result<(), ValueError> {
        let sets_len = self.uv_sets.len();
        let coordinates = self
            .uv_sets
            .get_mut(set)
            .ok_or(ValueError::IndexOutOfBounds {
                tag: GeosetsChunk::<V>::TAG,
                index: set,
                len: sets_len,
            })?;
        set_array_value(coordinates, index, uv)
    }

    /// Replaces every UV coordinate set.
    pub fn set_uv_sets(&mut self, sets: Vec<ZeroVec<'data, [f32; 2]>>) {
        self.uv_sets = sets;
    }
}

fn set_array_value<T: AsULE>(
    values: &mut ZeroVec<'_, T>,
    index: usize,
    value: T,
) -> Result<(), ValueError> {
    let len = values.len();
    if index >= len {
        return Err(ValueError::IndexOutOfBounds {
            tag: *b"GEOS",
            index,
            len,
        });
    }
    values.to_mut_slice()[index] = value.to_unaligned();
    Ok(())
}

fn peek_tag<'data>(cursor: &Cursor<'data>) -> Result<Tag, mdx::ReadError> {
    Ok(cursor.peek_bytes(4)?.try_into().expect("four-byte tag"))
}

fn section<'a>(
    cursor: &mut Cursor<'a>,
    tag: Tag,
    stride: usize,
) -> Result<Cursor<'a>, mdx::ReadError> {
    let offset = cursor.absolute_position();
    let actual = cursor.read()?;
    if actual != tag {
        return Err(mdx::ReadError::new(
            offset,
            mdx::ReadErrorKind::UnexpectedTag {
                expected: tag,
                actual,
            },
        )
        .with_tag(*b"GEOS"));
    }
    let count = cursor.read::<u32>()? as usize;
    let size = count
        .checked_mul(stride)
        .ok_or(mdx::ReadError::new(offset, mdx::ReadErrorKind::SizeOverflow).with_tag(*b"GEOS"))?;
    cursor.subcursor(size)
}

fn decode_values<'data, T: WireValue>(
    cursor: Cursor<'data>,
    width: usize,
) -> Result<ZeroVec<'data, T>, mdx::ReadError> {
    read_array_layout(cursor, width)
}

fn read_skin_weights<'data, S: SkinWeight>(
    cursor: &mut Cursor<'data>,
) -> Result<ZeroVec<'data, S>, mdx::ReadError> {
    let offset = cursor.absolute_position();
    let width = size_of::<S::ULE>();
    let data = section(cursor, *b"SKIN", width / 8)?;
    let values = read_array_layout::<S>(data, width).map_err(|error| error.with_tag(*b"SKIN"))?;
    if values
        .iter()
        .any(|row| row.weights().iter().any(|&weight| weight > 255))
    {
        return Err(mdx::ReadError::new(
            offset,
            mdx::ReadErrorKind::InvalidValue {
                field: "skin weights",
            },
        )
        .with_tag(*b"SKIN"));
    }
    Ok(values)
}

fn read_extra_sections<'data, S: SkinWeight>(
    cursor: &mut Cursor<'data>,
) -> Result<Vec<GeosetExtraSection<'data, S>>, mdx::ReadError> {
    let mut extra_sections = Vec::new();
    while peek_tag(cursor)? != *b"UVAS" {
        let offset = cursor.absolute_position();
        let tag = peek_tag(cursor)?;
        match &tag {
            b"TANG"
                if !extra_sections
                    .iter()
                    .any(|part| matches!(part, GeosetExtraSection::Tangents(_))) =>
            {
                extra_sections.push(GeosetExtraSection::Tangents(decode_values::<[f32; 4]>(
                    section(cursor, *b"TANG", 16)?,
                    16,
                )?));
            }
            b"SKIN"
                if !extra_sections
                    .iter()
                    .any(|part| matches!(part, GeosetExtraSection::Skin { .. })) =>
            {
                let weights = read_skin_weights::<S>(cursor)?;
                extra_sections.push(GeosetExtraSection::Skin { weights });
            }
            _ => {
                return Err(mdx::ReadError::new(
                    offset,
                    mdx::ReadErrorKind::UnknownTag { actual: tag },
                )
                .with_tag(*b"GEOS"))
            }
        }
    }
    Ok(extra_sections)
}

fn write_extra_sections<'data, S: SkinWeight>(
    bytes: &mut Encoder<'_>,
    sections: &[GeosetExtraSection<'data, S>],
) -> Result<(), mdx::WriteError> {
    for extension in sections {
        match extension {
            GeosetExtraSection::Tangents(tangents) => write_vectors(bytes, *b"TANG", tangents)?,
            GeosetExtraSection::Skin { weights } => {
                write_section_header(bytes, *b"SKIN", weights.len() * 8)?;
                bytes.write_bytes(weights.as_bytes());
            }
        }
    }
    Ok(())
}

fn write_count(bytes: &mut Encoder<'_>, count: usize) -> Result<(), mdx::WriteError> {
    let count = u32::try_from(count).map_err(|_| mdx::WriteError::SizeOverflow {
        field: "section count",
        tag: *b"GEOS",
        size: count,
    })?;
    bytes.write(&(count))?;
    Ok(())
}

fn write_section_header(
    bytes: &mut Encoder<'_>,
    tag: Tag,
    count: usize,
) -> Result<(), mdx::WriteError> {
    bytes.write_bytes(&tag);
    write_count(bytes, count)
}

fn write_words<'data>(
    bytes: &mut Encoder<'_>,
    tag: Tag,
    words: &ZeroVec<'data, u32>,
) -> Result<(), mdx::WriteError> {
    write_section_header(bytes, tag, words.len())?;
    bytes.write_bytes(words.as_bytes());
    Ok(())
}

fn write_vectors<'data, const N: usize>(
    bytes: &mut Encoder<'_>,
    tag: Tag,
    vectors: &ZeroVec<'data, [f32; N]>,
) -> Result<(), mdx::WriteError> {
    write_section_header(bytes, tag, vectors.len())?;
    bytes.write_bytes(vectors.as_bytes());
    Ok(())
}

impl<'data, D: ModelDialect> Model<'data, D> {
    /// Borrows records in chunk and record order.
    pub fn geosets<'s>(
        &'s self,
    ) -> impl Iterator<Item = &'s Geoset<'data, D::Version>> + use<'data, 's, D> {
        self.records::<GeosetsChunk<'_, D::Version>>()
    }

    /// Replaces geosets.
    pub fn set_geosets(&mut self, geosets: &[Geoset<'data, D::Version>]) {
        self.replace_chunk(GeosetsChunk::new(geosets.to_vec()));
    }
}

impl<'data, V: ModelVersion> mdx::Read<'data> for Geoset<'data, V> {
    fn read_mdx(source: &mut Cursor<'data>) -> Result<Self, mdx::ReadError> {
        let mut cursor = source.subcursor_u32_sized()?;

        let value = {
            let vertices = decode_values::<[f32; 3]>(section(&mut cursor, *b"VRTX", 12)?, 12)?;
            let normals = decode_values::<[f32; 3]>(section(&mut cursor, *b"NRMS", 12)?, 12)?;
            let primitive_types = decode_values::<u32>(section(&mut cursor, *b"PTYP", 4)?, 4)?;
            let primitive_counts = decode_values::<u32>(section(&mut cursor, *b"PCNT", 4)?, 4)?;
            let faces = decode_values::<u16>(section(&mut cursor, *b"PVTX", 2)?, 2)?;
            let vertex_groups = read_array(section(&mut cursor, *b"GNDX", 1)?, 1)?;
            let matrix_group_sizes = decode_values::<u32>(section(&mut cursor, *b"MTGC", 4)?, 4)?;
            let matrix_indices = decode_values::<u32>(section(&mut cursor, *b"MATS", 4)?, 4)?;
            let material_id = cursor.read()?;
            let selection_group = cursor.read()?;
            let unselectable_raw = cursor.read()?;
            let level_of_detail = cursor.read::<V::LevelOfDetail>()?;
            let extent = cursor.read()?;
            let sequence_count = cursor.read::<u32>()? as usize;
            let size = sequence_count.checked_mul(28).ok_or_else(|| {
                mdx::ReadError::new(cursor.absolute_position(), mdx::ReadErrorKind::SizeOverflow)
            })?;
            let sequence_extents = read_array(cursor.subcursor(size)?, 28)?;
            let extra_sections = V::ExtraSections::<'data>::read(&mut cursor)?;
            let actual = cursor.read()?;
            if actual != *b"UVAS" {
                return Err(mdx::ReadError::new(
                    cursor.absolute_position() - 4,
                    mdx::ReadErrorKind::UnexpectedTag {
                        expected: *b"UVAS",
                        actual,
                    },
                )
                .with_tag(GeosetsChunk::<V>::TAG));
            }
            let uv_count = cursor.read::<u32>()? as usize;
            let mut uv_sets = Vec::new();
            for _ in 0..uv_count {
                uv_sets.push(decode_values::<[f32; 2]>(
                    section(&mut cursor, *b"UVBS", 8)?,
                    8,
                )?);
            }
            Ok(Self {
                version: PhantomData,
                vertices,
                normals,
                primitive_types,
                primitive_counts,
                faces,
                vertex_groups,
                matrix_group_sizes,
                matrix_indices,
                material_id,
                selection_group,
                unselectable_raw,
                level_of_detail,
                extent,
                sequence_extents,
                extra_sections,
                uv_sets,
            })
        }?;
        cursor.finish()?;
        Ok(value)
    }
}

impl<'data, V: ModelVersion> mdx::Write for Geoset<'data, V> {
    fn write_mdx(&self, bytes: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        let start = bytes.position();
        let marker = bytes.begin_sized();
        write_vectors(bytes, *b"VRTX", &self.vertices)?;
        write_vectors(bytes, *b"NRMS", &self.normals)?;
        write_words(bytes, *b"PTYP", &self.primitive_types)?;
        write_words(bytes, *b"PCNT", &self.primitive_counts)?;
        write_section_header(bytes, *b"PVTX", self.faces.len())?;
        bytes.write_bytes(self.faces.as_bytes());
        write_section_header(bytes, *b"GNDX", self.vertex_groups.len())?;
        bytes.write_bytes(self.vertex_groups.as_bytes());
        write_words(bytes, *b"MTGC", &self.matrix_group_sizes)?;
        write_words(bytes, *b"MATS", &self.matrix_indices)?;
        for word in [
            self.material_id,
            self.selection_group,
            self.unselectable_raw,
        ] {
            bytes.write(&(word))?;
        }
        bytes.write(&self.level_of_detail)?;
        bytes.write(&self.extent)?;
        write_count(bytes, self.sequence_extents.len())?;
        bytes.write_bytes(self.sequence_extents.as_bytes());
        self.extra_sections.write(bytes)?;
        bytes.write_bytes(b"UVAS");
        write_count(bytes, self.uv_sets.len())?;
        for uv_set in &self.uv_sets {
            write_vectors(bytes, *b"UVBS", uv_set)?;
        }
        if bytes.position() - start > u32::MAX as usize {
            return Err(mdx::WriteError::SizeOverflow {
                field: "encoded size",
                tag: GeosetsChunk::<V>::TAG,
                size: bytes.position() - start,
            });
        }
        bytes.finish_sized(marker, GeosetsChunk::<V>::TAG)?;
        Ok(())
    }
}

impl<'data, V: SupportsReforgedChunks> Geoset<'data, V> {
    /// Returns the authored level-of-detail number.
    pub fn level_of_detail(&self) -> u32 {
        self.level_of_detail
            .level_of_detail()
            .expect("supported version")
    }
    /// Returns the geoset name, decoding invalid UTF-8 lossily.
    pub fn name(&self) -> Cow<'_, str> {
        self.level_of_detail.name().expect("supported version")
    }
    /// Borrows optional tangent vectors and their handedness components.
    pub fn tangents(&self) -> Option<&ZeroVec<'data, [f32; 4]>> {
        self.extra_sections
            .reforged()
            .expect("supported version")
            .sections
            .iter()
            .find_map(|part| match part {
                GeosetExtraSection::Tangents(values) => Some(values),
                _ => None,
            })
    }
    /// Borrows optional per-vertex skin influences.
    pub fn skin_weights(&self) -> Option<&ZeroVec<'data, V::SkinWeights>> {
        self.extra_sections
            .reforged()
            .expect("supported version")
            .sections
            .iter()
            .find_map(|part| match part {
                GeosetExtraSection::Skin { weights, .. } => Some(weights),
                _ => None,
            })
    }
    /// Sets the authored level-of-detail number.
    pub fn set_level_of_detail(&mut self, level: u32) {
        self.try_set_level_of_detail(level)
            .expect("supported version")
    }
    /// Sets the geoset name; rejects embedded NUL and names beyond the fixed capacity.
    pub fn set_name(&mut self, name: &str) -> Result<(), ValueError> {
        self.try_set_name(name)
    }
    /// Replaces tangents, or removes the section when passed `None`.
    pub fn set_tangents(&mut self, tangents: Option<ZeroVec<'data, [f32; 4]>>) {
        self.try_set_tangents(tangents).expect("supported version")
    }
    /// Replaces skin influences, or removes the section when passed `None`.
    /// Rejects influence weights above 255; the index type is selected by the version.
    pub fn set_skin_weights(
        &mut self,
        weights: Option<ZeroVec<'data, V::SkinWeights>>,
    ) -> Result<(), ValueError> {
        self.try_set_skin_weights(weights)
    }
}

impl<'data, V: ModelVersion> Geoset<'data, V> {
    pub(crate) fn convert_with<T: ModelVersion>(
        mut self,
        context: &mut ConversionContext<'_>,
        path: &str,
    ) -> Result<Geoset<'data, T>, ConversionError> {
        let mut level_of_detail = T::LevelOfDetail::default();
        context.field(
            self.level_of_detail.level_of_detail(),
            level_of_detail.level_of_detail_mut(),
            0,
            &format!("{path}.level_of_detail"),
        )?;
        context.field(
            self.level_of_detail.fixed_name().copied(),
            level_of_detail.name_mut(),
            FixedText::default(),
            &format!("{path}.name"),
        )?;
        let mut extra_sections = T::ExtraSections::<'data>::default();
        if let Some(source) = self.extra_sections.reforged_mut() {
            for (index, section) in take(&mut source.sections).into_iter().enumerate() {
                let section_path = format!("{path}.extra_sections[{index}]");
                let supported = T::NUMBER >= 900;
                let fits = match &section {
                    GeosetExtraSection::Skin { weights } => weights.iter().all(|vertex| {
                        vertex.weights().iter().all(|&weight| weight <= 255)
                            && (T::NUMBER >= 1400
                                || vertex.bone_indices().iter().all(|&bone| bone <= 255))
                    }),
                    _ => true,
                };
                if supported && fits {
                    extra_sections
                        .reforged_mut()
                        .expect("supported extra sections")
                        .sections
                        .push(match section {
                            GeosetExtraSection::Tangents(values) => {
                                GeosetExtraSection::Tangents(values)
                            }
                            GeosetExtraSection::Skin { weights } => GeosetExtraSection::Skin {
                                weights: T::SkinWeights::convert_array(weights)
                                    .expect("checked skin influences"),
                            },
                        });
                } else {
                    context.drop(&section_path, if !supported { "geoset section is not supported by the target" } else { "skin influences exceed the target's supported range; conversion requires dropping the whole skin section" })?;
                }
            }
        }
        Ok(Geoset {
            version: PhantomData,
            level_of_detail,
            extra_sections,
            vertices: self.vertices,
            normals: self.normals,
            primitive_types: self.primitive_types,
            primitive_counts: self.primitive_counts,
            faces: self.faces,
            vertex_groups: self.vertex_groups,
            matrix_group_sizes: self.matrix_group_sizes,
            matrix_indices: self.matrix_indices,
            material_id: self.material_id,
            selection_group: self.selection_group,
            unselectable_raw: self.unselectable_raw,
            extent: self.extent,
            sequence_extents: self.sequence_extents,
            uv_sets: self.uv_sets,
        })
    }
}

use zerovec::ule::AsULE;
impl AsULE for GeosetExtent {
    type ULE = <(f32, Vec3, Vec3) as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        (self.bounds_radius, self.minimum, self.maximum).to_unaligned()
    }
    fn from_unaligned(raw: Self::ULE) -> Self {
        let (bounds_radius, minimum, maximum) = <(f32, Vec3, Vec3)>::from_unaligned(raw);
        Self {
            bounds_radius,
            minimum,
            maximum,
        }
    }
}
macro_rules! skin_ule {
    ($component:ty) => {
        impl AsULE for SkinWeights<$component> {
            type ULE = <([$component; 4], [$component; 4]) as AsULE>::ULE;
            fn to_unaligned(self) -> Self::ULE {
                (self.bone_indices, self.weights).to_unaligned()
            }
            fn from_unaligned(value: Self::ULE) -> Self {
                let (bone_indices, weights) =
                    <([$component; 4], [$component; 4])>::from_unaligned(value);
                Self {
                    bone_indices,
                    weights,
                }
            }
        }
    };
}
skin_ule!(u8);
skin_ule!(u16);
