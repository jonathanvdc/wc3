//! Bounds, vertices, and skin draw records.
use super::SkinSectionLayout;
use crate::model::{common::Ownership, m2};
use m2::ModelLayout;
use std::array::from_fn;
use zerovec::ule::AsULE;

/// Axis-aligned box and bounding sphere used for model or sequence bounds.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct Extent {
    /// Lower corner of the axis-aligned bounding box in model coordinates.
    pub minimum: [f32; 3],
    /// Upper corner of the axis-aligned bounding box in model coordinates.
    pub maximum: [f32; 3],
    /// Bounding-sphere radius in model distance units.
    pub radius: f32,
}
/// Skinned model vertex with four bone influences and two UV sets.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct Vertex {
    /// Vertex position in the model's Z-up coordinate system.
    pub position: [f32; 3],
    /// Four skinning weights, normalized by 255; nonzero weights should sum to 255.
    pub bone_weights: [u8; 4],
    /// Bone indices paired with `bone_weights`, referencing the model skeleton.
    pub bone_indices: [u8; 4],
    /// Surface normal in model coordinates.
    pub normal: [f32; 3],
    /// Two UV coordinate sets used by the selected material shader.
    pub texture_coordinates: [[f32; 2]; 2],
}
/// Section sorting sphere present in later skin layouts.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct SkinSort {
    /// Center used to sort the section for transparency rendering.
    pub center: [f32; 3],
    /// Radius of the section's sorting sphere in model distance units.
    pub radius: f32,
}
/// Mesh section referencing a vertex range, triangle range, and bone palette.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct SkinSection<V: ModelLayout> {
    /// Geoset group ID used to select visible mesh variants, such as hairstyles.
    pub id: u16,
    /// High bits of the triangle-index start in extended index lists: `(level << 16) | index_start`.
    pub level: u16,
    /// First entry in the skin profile's vertex lookup for this section.
    pub vertex_start: u16,
    /// Number of vertex lookup entries belonging to this section.
    pub vertex_count: u16,
    /// Low 16 bits of the first triangle-index entry in the skin profile.
    pub index_start: u16,
    /// Number of triangle-index entries, rather than the number of triangles.
    pub index_count: u16,
    /// Number of bones in this section's palette.
    pub bone_count: u16,
    /// First entry of the section's palette in the model's bone-combination table.
    pub bone_combo_index: u16,
    /// Maximum number of bone influences per vertex, normally one through four.
    pub bone_influences: u16,
    /// Bone index used when calculating the section's distance from the camera.
    pub center_bone_index: u16,
    /// Center of the section in model coordinates.
    pub center_position: [f32; 3],
    /// Sorting center and radius in layouts that include them; `()` in older layouts.
    pub sort_center: <V as SkinSectionLayout>::Tail,
}
/// Material draw call for a skin section.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct Batch {
    /// Draw-batch flags; `0x04` identifies a projected-texture batch.
    pub flags: u8,
    /// Signed priority used when ordering draw batches.
    pub priority_plane: i8,
    /// Shader selection code; some legacy layouts instead index texture-combiner combinations.
    pub shader_id: u16,
    /// Index of the skin section drawn by this batch.
    pub skin_section_index: u16,
    /// Auxiliary section/flags word whose interpretation varies with the client version.
    pub geoset_index: u16,
    /// Index of the model's color animation, or `-1` to omit color modulation.
    pub color_index: i16,
    /// Index of the model material supplying render flags and blending.
    pub material_index: u16,
    /// Material layer used when ordering passes on the section.
    pub material_layer: u16,
    /// Number of texture slots used by this draw call.
    pub texture_count: u16,
    /// First entry in the model's texture-combination table for this batch.
    pub texture_combo_index: u16,
    /// Texture-coordinate combination slot; actual UV selection also depends on the shader.
    pub texture_coordinate_combo_index: u16,
    /// First entry in the texture-weight lookup for this batch's texture slots.
    pub texture_weight_combo_index: u16,
    /// First entry in the texture-transform lookup for this batch's texture slots.
    pub texture_transform_combo_index: u16,
}
/// Additional draw-call references for the section's shadow pass.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct ShadowBatch {
    /// Shadow-pass render flags.
    pub flags: u8,
    /// Additional shadow-pass flags; `0x08` selects EDGF edge-fade data.
    pub flags2: u8,
    /// Unresolved shadow-pass parameter.
    pub unknown: u16,
    /// Section reference used by the shadow pass.
    pub submesh_id: u16,
    /// Texture reference used by the shadow pass.
    pub texture_id: u16,
    /// Native shadow-pass color reference; the target table remains unresolved.
    pub color_id: u16,
    /// Native shadow-pass transparency reference; the target table remains unresolved.
    pub transparency_id: u16,
}
impl AsULE for Vertex {
    type ULE = <[u8; 48] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let mut bytes = [0; 48];
        for (index, value) in self.position.iter().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes[12..16].copy_from_slice(&self.bone_weights);
        bytes[16..20].copy_from_slice(&self.bone_indices);
        for (index, value) in self.normal.iter().enumerate() {
            bytes[20 + index * 4..24 + index * 4].copy_from_slice(&value.to_le_bytes());
        }
        for (index, value) in self.texture_coordinates.iter().flatten().enumerate() {
            bytes[32 + index * 4..36 + index * 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes.to_unaligned()
    }
    fn from_unaligned(unaligned: Self::ULE) -> Self {
        let bytes = <[u8; 48]>::from_unaligned(unaligned);
        let float =
            |at| f32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
        Self {
            position: from_fn(|index| float(index * 4)),
            bone_weights: from_fn(|index| bytes[12 + index]),
            bone_indices: from_fn(|index| bytes[16 + index]),
            normal: from_fn(|index| float(20 + index * 4)),
            texture_coordinates: from_fn(|uv| {
                from_fn(|component| float(32 + uv * 8 + component * 4))
            }),
        }
    }
}
