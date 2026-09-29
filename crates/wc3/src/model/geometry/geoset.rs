//! Mesh geometry, material references, bounds, and skinning.
use crate::model::conversion::ConversionContext;
use crate::model::ConversionError;
use crate::model::Encoder;
use crate::model::KnownChunk;
use crate::model::ValueError;
use crate::model::WriteError;
use crate::model::{mdl, mdx};
use crate::model::{ModelVersion, SupportsReforgedChunks, Tag, Vec3, Version};

use std::fmt::Debug;
use std::marker::PhantomData;

use crate::model::Cursor;
use crate::model::GeosetsChunk;
use std::borrow::Cow;

use crate::model::FixedText;
use crate::model::{Model, ReadError};

/// A geoset's bounding volume, also used for each sequence extent.
#[derive(Clone, Copy, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(fields, write_order(minimum, maximum, bounds_radius))]
pub struct GeosetExtent {
    #[mdl(property = "BoundsRadius", default)]
    pub bounds_radius: f32,
    #[mdl(property = "MinimumExtent", default)]
    pub minimum: Vec3,
    #[mdl(property = "MaximumExtent", default)]
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

/// Four bone influences for one vertex. Weights use the range 0..=255
/// and represent fractions of full influence when divided by 255.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkinWeights {
    pub bone_indices: [u16; 4],
    pub weights: [u8; 4],
}

/// Optional sections retain their original order for serialization.
#[derive(Clone, Debug, PartialEq)]
enum GeosetExtraSection {
    Tangents(Vec<[f32; 4]>),
    Skin { weights: Vec<SkinWeights> },
}

/// Storage and serialization of the optional TANG and SKIN sections for a version.
pub trait GeosetExtraSections: Default + Clone + Debug + PartialEq {
    fn read<V: ModelVersion>(_: &mut Cursor<'_>) -> Result<Self, ReadError> {
        Ok(Self::default())
    }
    fn write<V: ModelVersion>(&self, _: &mut Encoder<'_>) -> Result<(), WriteError> {
        Ok(())
    }
    fn reforged(&self) -> Option<&ReforgedGeosetExtraSections> {
        None
    }
    fn reforged_mut(&mut self) -> Option<&mut ReforgedGeosetExtraSections> {
        None
    }
}

/// Classic geosets have no optional TANG or SKIN sections.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct NoGeosetExtraSections;

impl GeosetExtraSections for NoGeosetExtraSections {}

/// Optional TANG and SKIN sections, retained in file order.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ReforgedGeosetExtraSections {
    sections: Vec<GeosetExtraSection>,
}

impl GeosetExtraSections for ReforgedGeosetExtraSections {
    fn read<V: ModelVersion>(cursor: &mut Cursor<'_>) -> Result<Self, ReadError> {
        Ok(Self {
            sections: read_extra_sections::<V>(cursor)?,
        })
    }
    fn write<V: ModelVersion>(&self, bytes: &mut Encoder<'_>) -> Result<(), WriteError> {
        write_extra_sections::<V>(bytes, &self.sections)
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
    Default + mdx::Read + mdx::Write + Clone + Debug + PartialEq
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
    type LevelOfDetail: GeosetLevelOfDetail;
    type ExtraSections: GeosetExtraSections;
}

use crate::model::{V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900};
impl GeosetLayout for V800 {
    type LevelOfDetail = NoGeosetLevelOfDetail;
    type ExtraSections = NoGeosetExtraSections;
}
impl GeosetLayout for V900 {
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections = ReforgedGeosetExtraSections;
}
impl GeosetLayout for V1000 {
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections = ReforgedGeosetExtraSections;
}
impl GeosetLayout for V1100 {
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections = ReforgedGeosetExtraSections;
}
impl GeosetLayout for V1200 {
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections = ReforgedGeosetExtraSections;
}
impl GeosetLayout for V1300 {
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections = ReforgedGeosetExtraSections;
}
impl GeosetLayout for V1400 {
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections = ReforgedGeosetExtraSections;
}
impl GeosetLayout for V1600 {
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections = ReforgedGeosetExtraSections;
}
impl GeosetLayout for V1800 {
    type LevelOfDetail = GeosetLevelOfDetailFields;
    type ExtraSections = ReforgedGeosetExtraSections;
}

mod mdl_codec;
use mdl_codec::{
    AnimExtent, Faces, Groups, List, OptionalList, Selection, SkinRow, Uncounted, UvSet,
};

/// A mesh rendered with one material.
///
/// Vertex positions, normals, UV sets, and skin influences must agree in count.
/// Use the editing methods to keep those arrays consistent. Material and bone
/// references are model indices; changing collection order requires updating
/// the affected references.
#[derive(Clone, Debug, PartialEq, mdl::Read, mdl::Write)]
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
        #[mdl(property = "LevelOfDetailName", delegate, get = "Self::mdl_lod_name", set = "Self::set_mdl_lod_name")]
        lod_name: Option<FixedText<80>>,
        #[mdl(repeated = "Anim", get = "Self::mdl_bounds", set = "Self::set_mdl_bounds")]
        bounds: Vec<AnimExtent>,
        #[mdl(property = "Tangents", delegate, get = "Self::mdl_tangents", set = "Self::set_mdl_tangents")]
        tangents: OptionalList<[f32; 4]>,
        #[mdl(property = "SkinWeights", delegate, get = "Self::mdl_skin", set = "Self::set_mdl_skin")]
        skin: OptionalList<SkinRow>,
    )
)]
pub struct Geoset<V: ModelVersion> {
    #[mdl(skip, default)]
    version: PhantomData<V>,
    #[mdl(skip, default)]
    vertices: Vec<Vec3>,
    #[mdl(skip, default)]
    normals: Vec<Vec3>,
    #[mdl(skip, default)]
    primitive_types: Vec<u32>,
    #[mdl(skip, default)]
    primitive_counts: Vec<u32>,
    #[mdl(skip, default)]
    faces: Vec<u16>,
    #[mdl(skip, default)]
    vertex_groups: Vec<u8>,
    #[mdl(skip, default)]
    matrix_group_sizes: Vec<u32>,
    #[mdl(skip, default)]
    matrix_indices: Vec<u32>,
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
    pub sequence_extents: Vec<GeosetExtent>,
    #[mdl(skip, default)]
    extra_sections: V::ExtraSections,
    #[mdl(skip, default)]
    uv_sets: Vec<Vec<[f32; 2]>>,
}

impl<V: ModelVersion> Geoset<V> {
    /// Builds a basic geoset with one matrix group and one UV set.
    pub fn new(vertices: &[Vec3], normals: &[Vec3], faces: &[u16]) -> Result<Self, ValueError> {
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
        Ok(Self {
            version: PhantomData,
            vertices: vertices.to_vec(),
            normals: normals.to_vec(),
            primitive_types: vec![4],
            primitive_counts: vec![faces.len() as u32],
            faces: faces.to_vec(),
            vertex_groups: vec![0; vertices.len()],
            matrix_group_sizes: vec![1],
            matrix_indices: vec![0],
            material_id: 0,
            selection_group: 0,
            unselectable_raw: 0,
            level_of_detail: V::LevelOfDetail::default(),
            extent: GeosetExtent::default(),
            sequence_extents: Vec::new(),
            extra_sections: V::ExtraSections::default(),
            uv_sets: vec![vec![[0.0; 2]; vertices.len()]],
        })
    }

    /// The MDX version used to interpret this record.
    pub fn version(&self) -> Version {
        V::NUMBER
    }

    /// Vertex positions in model coordinates.
    pub fn vertices(&self) -> &[Vec3] {
        &self.vertices
    }
    /// Mutably borrows vertex positions for bulk edits.
    pub fn vertices_mut(&mut self) -> &mut [Vec3] {
        &mut self.vertices
    }
    /// Borrows all vertex normals.
    pub fn normals(&self) -> &[Vec3] {
        &self.normals
    }
    /// Mutably borrows normals for bulk edits.
    pub fn normals_mut(&mut self) -> &mut [Vec3] {
        &mut self.normals
    }
    /// Primitive type for each face group; triangle groups use type 4.
    pub fn primitive_types(&self) -> &[u32] {
        &self.primitive_types
    }
    /// Number of vertex indices in each primitive group.
    pub fn primitive_counts(&self) -> &[u32] {
        &self.primitive_counts
    }
    /// Face indices into the vertex-position array.
    pub fn face_indices(&self) -> &[u16] {
        &self.faces
    }
    /// Mutably borrows face indices for bulk edits.
    pub fn face_indices_mut(&mut self) -> &mut [u16] {
        &mut self.faces
    }
    /// Borrows the matrix group assigned to each vertex.
    pub fn vertex_groups(&self) -> &[u8] {
        &self.vertex_groups
    }
    /// Number of bone references in each matrix group.
    pub fn matrix_group_sizes(&self) -> &[u32] {
        &self.matrix_group_sizes
    }
    /// Bone object IDs, concatenated in matrix-group order.
    pub fn matrix_indices(&self) -> &[u32] {
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
    pub fn uv_sets(&self) -> &[Vec<[f32; 2]>] {
        &self.uv_sets
    }
    /// Mutably borrows one UV set for bulk edits.
    pub fn uv_set_mut(&mut self, index: usize) -> Option<&mut [[f32; 2]]> {
        self.uv_sets.get_mut(index).map(Vec::as_mut_slice)
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
    pub fn try_tangents(&self) -> Result<Option<&[[f32; 4]]>, ValueError> {
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
                GeosetExtraSection::Tangents(values) => Some(values.as_slice()),
                _ => None,
            }))
    }

    /// Borrows the four bone influences for each skinned vertex.
    pub fn try_skin_weights(&self) -> Result<Option<&[SkinWeights]>, ValueError> {
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
                GeosetExtraSection::Skin { weights, .. } => Some(weights.as_slice()),
                _ => None,
            }))
    }

    /// Changes one vertex position.
    pub fn set_vertex(&mut self, index: usize, vertex: Vec3) -> Result<(), ValueError> {
        let len = self.vertices.len();
        *self
            .vertices
            .get_mut(index)
            .ok_or(ValueError::IndexOutOfBounds {
                tag: GeosetsChunk::<V>::TAG,
                index,
                len,
            })? = vertex;
        Ok(())
    }

    /// Changes one vertex normal.
    pub fn set_normal(&mut self, index: usize, normal: Vec3) -> Result<(), ValueError> {
        let len = self.normals.len();
        *self
            .normals
            .get_mut(index)
            .ok_or(ValueError::IndexOutOfBounds {
                tag: GeosetsChunk::<V>::TAG,
                index,
                len,
            })? = normal;
        Ok(())
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
        self.vertex_groups = groups.to_vec();
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
        let len = self.sequence_extents.len();
        *self
            .sequence_extents
            .get_mut(index)
            .ok_or(ValueError::IndexOutOfBounds {
                tag: GeosetsChunk::<V>::TAG,
                index,
                len,
            })? = extent;
        Ok(())
    }

    /// Replaces or removes the Reforged tangent section, retaining its order.
    pub fn try_set_tangents(&mut self, tangents: Option<&[[f32; 4]]>) -> Result<(), ValueError> {
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
                sections[index] = GeosetExtraSection::Tangents(values.to_vec());
            } else {
                sections.remove(index);
            }
        } else if let Some(values) = tangents {
            sections.insert(0, GeosetExtraSection::Tangents(values.to_vec()));
        }
        Ok(())
    }

    /// Replaces or removes per-vertex skin influences, retaining section order.
    pub fn try_set_skin_weights(
        &mut self,
        weights: Option<&[SkinWeights]>,
    ) -> Result<(), ValueError> {
        if V::NUMBER < 900 {
            return Err(ValueError::UnsupportedVersion {
                tag: *b"GEOS",
                minimum: 900,
                actual: V::NUMBER,
            });
        }
        if let Some(weights) = weights {
            if weights.len() > u32::MAX as usize / 8 {
                return Err(ValueError::CountTooLarge {
                    tag: *b"SKIN",
                    count: weights.len().saturating_mul(8),
                });
            }
            if V::NUMBER < 1400
                && weights.iter().any(|vertex| {
                    vertex
                        .bone_indices
                        .iter()
                        .any(|&index| index > u8::MAX as u16)
                })
            {
                return Err(ValueError::UnsupportedVersion {
                    tag: *b"SKIN",
                    minimum: 1400,
                    actual: V::NUMBER,
                });
            }
        }
        let sections = &mut self
            .extra_sections
            .reforged_mut()
            .expect("supported version")
            .sections;
        let new_part = weights.map(|weights| GeosetExtraSection::Skin {
            weights: weights.to_vec(),
        });
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
        let len = coordinates.len();
        *coordinates
            .get_mut(index)
            .ok_or(ValueError::IndexOutOfBounds {
                tag: GeosetsChunk::<V>::TAG,
                index,
                len,
            })? = uv;
        Ok(())
    }

    /// Replaces every UV coordinate set.
    pub fn set_uv_sets(&mut self, sets: &[Vec<[f32; 2]>]) {
        self.uv_sets = sets.to_vec();
    }
}

fn peek_tag(cursor: &Cursor<'_>) -> Result<Tag, ReadError> {
    Ok(cursor.peek_exact(4)?.try_into().expect("four-byte tag"))
}

fn section<'a>(cursor: &mut Cursor<'a>, tag: Tag, stride: usize) -> Result<&'a [u8], ReadError> {
    let offset = cursor.absolute_position();
    if cursor.read_exact(4)? != tag {
        return Err(ReadError::MalformedRecord {
            tag: *b"GEOS",
            offset,
        });
    }
    let count = cursor.read::<u32>()? as usize;
    let size = count
        .checked_mul(stride)
        .ok_or(ReadError::MalformedRecord {
            tag: *b"GEOS",
            offset,
        })?;
    cursor.read_exact(size)
}

fn decode_values<T: mdx::Read>(bytes: &[u8], width: usize) -> Result<Vec<T>, ReadError> {
    let mut cursor = Cursor::new(bytes);
    let values = (0..bytes.len() / width)
        .map(|_| cursor.read())
        .collect::<Result<Vec<_>, _>>()?;
    cursor.finish()?;
    Ok(values)
}

fn read_skin_weights<V: ModelVersion>(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<SkinWeights>, ReadError> {
    let offset = cursor.absolute_position();
    let wide = V::NUMBER >= 1400;
    let data = section(cursor, *b"SKIN", if wide { 2 } else { 1 })?;
    let stride = if wide { 16 } else { 8 };
    let malformed = || ReadError::MalformedRecord {
        tag: *b"SKIN",
        offset,
    };
    if data.len() % stride != 0 {
        return Err(malformed());
    }
    let mut cursor = Cursor::new(data);
    (0..data.len() / stride)
        .map(|_| {
            let bone_indices = if wide {
                cursor.read::<[u16; 4]>()?
            } else {
                cursor.read::<[u8; 4]>()?.map(u16::from)
            };
            let weights = if wide {
                let values = cursor.read::<[u16; 4]>()?;
                if values.iter().any(|&weight| weight > u8::MAX as u16) {
                    return Err(malformed());
                }
                values.map(|weight| weight as u8)
            } else {
                cursor.read::<[u8; 4]>()?
            };
            Ok(SkinWeights {
                bone_indices,
                weights,
            })
        })
        .collect()
}

fn read_extra_sections<V: ModelVersion>(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<GeosetExtraSection>, ReadError> {
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
                let weights = read_skin_weights::<V>(cursor)?;
                extra_sections.push(GeosetExtraSection::Skin { weights });
            }
            _ => {
                return Err(ReadError::MalformedRecord {
                    tag: GeosetsChunk::<V>::TAG,
                    offset,
                })
            }
        }
    }
    Ok(extra_sections)
}

fn write_extra_sections<V: ModelVersion>(
    bytes: &mut Encoder<'_>,
    sections: &[GeosetExtraSection],
) -> Result<(), WriteError> {
    for extension in sections {
        match extension {
            GeosetExtraSection::Tangents(tangents) => write_vectors(bytes, *b"TANG", tangents)?,
            GeosetExtraSection::Skin { weights } => {
                write_section_header(bytes, *b"SKIN", weights.len() * 8)?;
                for vertex in weights {
                    for &index in &vertex.bone_indices {
                        if V::NUMBER >= 1400 {
                            bytes.write(&index)?;
                        } else {
                            let index =
                                u8::try_from(index).map_err(|_| WriteError::MalformedRecord {
                                    tag: *b"SKIN",
                                    offset: bytes.position(),
                                })?;
                            bytes.write(&index)?;
                        }
                    }
                    for &weight in &vertex.weights {
                        if V::NUMBER >= 1400 {
                            bytes.write(&u16::from(weight))?;
                        } else {
                            bytes.write(&weight)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn write_count(bytes: &mut Encoder<'_>, count: usize) -> Result<(), WriteError> {
    let count = u32::try_from(count).map_err(|_| WriteError::ChunkTooLarge {
        tag: *b"GEOS",
        size: count,
    })?;
    bytes.write(&(count))?;
    Ok(())
}

fn write_section_header(bytes: &mut Encoder<'_>, tag: Tag, count: usize) -> Result<(), WriteError> {
    bytes.write_bytes(&tag);
    write_count(bytes, count)
}

fn write_words(bytes: &mut Encoder<'_>, tag: Tag, words: &[u32]) -> Result<(), WriteError> {
    write_section_header(bytes, tag, words.len())?;
    for word in words {
        bytes.write(word)?;
    }
    Ok(())
}

fn write_vectors<const N: usize>(
    bytes: &mut Encoder<'_>,
    tag: Tag,
    vectors: &[[f32; N]],
) -> Result<(), WriteError> {
    write_section_header(bytes, tag, vectors.len())?;
    bytes.write(vectors)?;
    Ok(())
}

impl<V: ModelVersion> Model<V> {
    /// Returns owned geosets in chunk and record order.
    pub fn geosets(&self) -> Vec<Geoset<V>> {
        self.collect_chunk_records::<GeosetsChunk<V>>()
    }

    /// Replaces geosets.
    pub fn set_geosets(&mut self, geosets: &[Geoset<V>]) {
        self.replace_chunk(GeosetsChunk::new(geosets.to_vec()));
    }
}

impl<V: ModelVersion> mdx::Read for Geoset<V> {
    fn read_mdx(source: &mut Cursor<'_>) -> Result<Self, ReadError> {
        let mut cursor = source.slice_u32_sized()?;

        let value = {
            let vertices = decode_values::<[f32; 3]>(section(&mut cursor, *b"VRTX", 12)?, 12)?;
            let normals = decode_values::<[f32; 3]>(section(&mut cursor, *b"NRMS", 12)?, 12)?;
            let primitive_types = decode_values::<u32>(section(&mut cursor, *b"PTYP", 4)?, 4)?;
            let primitive_counts = decode_values::<u32>(section(&mut cursor, *b"PCNT", 4)?, 4)?;
            let faces = decode_values::<u16>(section(&mut cursor, *b"PVTX", 2)?, 2)?;
            let vertex_groups = section(&mut cursor, *b"GNDX", 1)?.to_vec();
            let matrix_group_sizes = decode_values::<u32>(section(&mut cursor, *b"MTGC", 4)?, 4)?;
            let matrix_indices = decode_values::<u32>(section(&mut cursor, *b"MATS", 4)?, 4)?;
            let material_id = cursor.read()?;
            let selection_group = cursor.read()?;
            let unselectable_raw = cursor.read()?;
            let level_of_detail = cursor.read::<V::LevelOfDetail>()?;
            let extent = cursor.read()?;
            let sequence_count = cursor.read::<u32>()? as usize;
            let mut sequence_extents = Vec::new();
            for _ in 0..sequence_count {
                sequence_extents.push(cursor.read()?);
            }
            let extra_sections = V::ExtraSections::read::<V>(&mut cursor)?;
            if cursor.read_exact(4)? != b"UVAS" {
                return Err(ReadError::MalformedRecord {
                    tag: GeosetsChunk::<V>::TAG,
                    offset: cursor.absolute_position() - 4,
                });
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

impl<V: ModelVersion> mdx::Write for Geoset<V> {
    fn write_mdx(&self, bytes: &mut Encoder<'_>) -> Result<(), WriteError> {
        let start = bytes.position();
        let marker = bytes.begin_sized();
        write_vectors(bytes, *b"VRTX", &self.vertices)?;
        write_vectors(bytes, *b"NRMS", &self.normals)?;
        write_words(bytes, *b"PTYP", &self.primitive_types)?;
        write_words(bytes, *b"PCNT", &self.primitive_counts)?;
        write_section_header(bytes, *b"PVTX", self.faces.len())?;
        for face in &self.faces {
            bytes.write(face)?;
        }
        write_section_header(bytes, *b"GNDX", self.vertex_groups.len())?;
        bytes.write_bytes(&self.vertex_groups);
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
        for extent in &self.sequence_extents {
            bytes.write(extent)?;
        }
        self.extra_sections.write::<V>(bytes)?;
        bytes.write_bytes(b"UVAS");
        write_count(bytes, self.uv_sets.len())?;
        for uv_set in &self.uv_sets {
            write_vectors(bytes, *b"UVBS", uv_set)?;
        }
        if bytes.position() - start > u32::MAX as usize {
            return Err(WriteError::ChunkTooLarge {
                tag: GeosetsChunk::<V>::TAG,
                size: bytes.position() - start,
            });
        }
        bytes.finish_sized(marker, GeosetsChunk::<V>::TAG)?;
        Ok(())
    }
}

impl<V: SupportsReforgedChunks> Geoset<V> {
    pub fn level_of_detail(&self) -> u32 {
        self.level_of_detail
            .level_of_detail()
            .expect("supported version")
    }
    pub fn name(&self) -> Cow<'_, str> {
        self.level_of_detail.name().expect("supported version")
    }
    pub fn tangents(&self) -> Option<&[[f32; 4]]> {
        self.extra_sections
            .reforged()
            .expect("supported version")
            .sections
            .iter()
            .find_map(|part| match part {
                GeosetExtraSection::Tangents(values) => Some(values.as_slice()),
                _ => None,
            })
    }
    pub fn skin_weights(&self) -> Option<&[SkinWeights]> {
        self.extra_sections
            .reforged()
            .expect("supported version")
            .sections
            .iter()
            .find_map(|part| match part {
                GeosetExtraSection::Skin { weights, .. } => Some(weights.as_slice()),
                _ => None,
            })
    }
    pub fn set_level_of_detail(&mut self, level: u32) {
        self.try_set_level_of_detail(level)
            .expect("supported version")
    }
    pub fn set_name(&mut self, name: &str) -> Result<(), ValueError> {
        self.try_set_name(name)
    }
    pub fn set_tangents(&mut self, tangents: Option<&[[f32; 4]]>) {
        self.try_set_tangents(tangents).expect("supported version")
    }
    pub fn set_skin_weights(&mut self, weights: Option<&[SkinWeights]>) -> Result<(), ValueError> {
        self.try_set_skin_weights(weights)
    }
}

impl<V: ModelVersion> Geoset<V> {
    pub(crate) fn convert_with<T: ModelVersion>(
        &self,
        context: &mut ConversionContext<'_>,
        path: &str,
    ) -> Result<Geoset<T>, ConversionError> {
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
        let mut extra_sections = T::ExtraSections::default();
        if let Some(source) = self.extra_sections.reforged() {
            for (index, section) in source.sections.iter().enumerate() {
                let section_path = format!("{path}.extra_sections[{index}]");
                let supported = T::NUMBER >= 900;
                let fits = match section {
                    GeosetExtraSection::Skin { weights } => {
                        T::NUMBER >= 1400
                            || weights
                                .iter()
                                .all(|vertex| vertex.bone_indices.iter().all(|&bone| bone <= 255))
                    }
                    _ => true,
                };
                if supported && fits {
                    extra_sections
                        .reforged_mut()
                        .expect("supported extra sections")
                        .sections
                        .push(section.clone());
                } else {
                    context.drop(&section_path, if !supported { "geoset section is not supported by the target" } else { "skin bone indices exceed the target's 8-bit range; conversion requires dropping the whole skin section" })?;
                }
            }
        }
        Ok(Geoset {
            version: PhantomData,
            level_of_detail,
            extra_sections,
            vertices: self.vertices.clone(),
            normals: self.normals.clone(),
            primitive_types: self.primitive_types.clone(),
            primitive_counts: self.primitive_counts.clone(),
            faces: self.faces.clone(),
            vertex_groups: self.vertex_groups.clone(),
            matrix_group_sizes: self.matrix_group_sizes.clone(),
            matrix_indices: self.matrix_indices.clone(),
            material_id: self.material_id,
            selection_group: self.selection_group,
            unselectable_raw: self.unselectable_raw,
            extent: self.extent,
            sequence_extents: self.sequence_extents.clone(),
            uv_sets: self.uv_sets.clone(),
        })
    }
}
