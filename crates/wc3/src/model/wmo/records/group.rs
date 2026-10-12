//! Fixed group headers, draw batches, collision nodes and group-local lights.
use crate::model::{common::Ownership, wmo};
use std::fmt::Debug;
use wmo::{Dialect, Read, RecordLayout, Validate, Write};

/// Dialect-selected fields in group headers and rendering batches.
pub trait GroupLayout {
    /// Reserved header word or split-group links, always four bytes.
    type HeaderTail: HeaderTailMetadata
        + Validate
        + Copy
        + Clone
        + Debug
        + PartialEq
        + Default
        + for<'data> Read<'data>
        + Write
        + RecordLayout;
    /// Rounded culling bounds or extended material metadata, always twelve bytes.
    type BatchPrefix: BatchMetadata
        + Validate
        + Copy
        + Clone
        + Debug
        + PartialEq
        + Default
        + for<'data> Read<'data>
        + Write
        + RecordLayout;
}
/// Parent/child links occupying the header's last word in 9.2 and later.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct SplitGroupLinks {
    /// Split parent for a child, or first child for a parent; -1 means none.
    pub parent_or_first_child: i16,
    /// Next sibling in the split-child chain; -1 means none.
    pub next_child: i16,
}
/// Rounded vertex bounds in the leading twelve bytes of a pre-Legion batch.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct CullingBox {
    /// Minimum model-space corner, rounded to signed integers.
    pub minimum: [i16; 3],
    /// Maximum model-space corner, rounded to signed integers.
    pub maximum: [i16; 3],
}
/// Legion and later batch metadata replacing the rounded culling box.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct LargeMaterialPrefix {
    /// Retained old box bytes; bytes 0 and 1 extend vertex indices when flag 4 is set.
    pub reserved: [u8; 10],
    /// Root MOMT material index selected by batch flag 2.
    pub material: u16,
}
/// The 68-byte MOGP header preceding the nested group chunk stream.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct GroupHeader<D: Dialect> {
    /// Byte offset of the group name in the root MOGN string block.
    pub name_offset: u32,
    /// Byte offset of the descriptive group name in MOGN.
    pub descriptive_name_offset: u32,
    /// Native group behavior and stream-presence flags; unknown bits are retained.
    pub flags: u32,
    /// Axis-aligned minimum corner in model coordinates.
    pub bounds_minimum: [f32; 3],
    /// Axis-aligned maximum corner in model coordinates.
    pub bounds_maximum: [f32; 3],
    /// First portal reference in the root MOPR array.
    pub portal_start: u16,
    /// Number of portal references belonging to this group.
    pub portal_count: u16,
    /// Number of transition batches at the beginning of MOBA.
    pub transition_batch_count: u16,
    /// Historical interior-batch count, retained independently of group lighting flags.
    pub interior_batch_count: u16,
    /// Historical exterior-batch count.
    pub exterior_batch_count: u16,
    /// Fourth batch-count slot, unused by established clients.
    pub reserved_batch_count: u16,
    /// Root MFOG indices; a modern MFVR chunk can replace this list.
    pub fog_indices: [u8; 4],
    /// Native liquid-type code, whose interpretation also depends on root flags.
    pub liquid_type: u32,
    /// WMOAreaTable group identifier.
    pub unique_id: u32,
    /// Extended flags, including terrain cutting and split-group relationships.
    pub flags2: u32,
    /// Reserved word or typed split-group links selected by the dialect.
    pub tail: <D as GroupLayout>::HeaderTail,
}
/// Two-byte MOPY triangle flags and root material index.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct Triangle {
    /// Collision, rendering, transition and query flags.
    pub flags: u8,
    /// Index into root MOMT; 0xff marks a collision-only triangle.
    pub material: u8,
}
/// Four-byte MPY2 replacement for MOPY, retaining extended material indices.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct Triangle2 {
    /// Triangle flags; 0x100 selects a ground-type entry from MOQG.
    pub flags: u16,
    /// Root MOMT material index; 0xffff marks a collision-only triangle.
    pub material: u16,
}
/// A 24-byte MOBA draw batch with dialect-selected prefix metadata.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct Batch<D: Dialect> {
    /// Rounded culling box or extended material and vertex-index metadata.
    pub prefix: D::BatchPrefix,
    /// First index in MOVI or MOVX, counted in index elements rather than triangles.
    pub start_index: u32,
    /// Number of consecutive index elements used by this batch.
    pub index_count: u16,
    /// Low sixteen bits of the first referenced vertex index.
    pub minimum_vertex: u16,
    /// Low sixteen bits of the last referenced vertex index, inclusive.
    pub maximum_vertex: u16,
    /// Batch flags; modern bit 2 selects the large material and bit 4 extends vertex indices.
    pub flags: u8,
    /// Root MOMT material index when the large-material flag is clear.
    pub material: u8,
}
/// An eight-byte MORB override of the matching draw batch's strip range.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct BatchOverride {
    /// First index in the MORI triangle-strip stream.
    pub start_index: u32,
    /// Number of strip index elements.
    pub index_count: u16,
    /// Preserved alignment padding.
    pub padding: u16,
}
/// A 24-byte MOBS shadow-pass batch with retained undocumented metadata.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct ShadowBatch {
    /// Undeciphered prefix bytes.
    pub reserved_prefix: [u8; 10],
    /// Root MOMT material index selected by flag 2.
    pub large_material: u16,
    /// Native start value, divided by three by clients to obtain a face index.
    pub start: i32,
    /// Native count value, divided by three to obtain a face count.
    pub count: i16,
    /// Undeciphered middle bytes.
    pub reserved_middle: [u8; 4],
    /// Native shadow batch flags.
    pub flags: u8,
    /// Root MOMT material index when flag 2 is clear.
    pub material: u8,
}
/// A sixteen-byte MOBN collision BSP node.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct BspNode {
    /// Split axis 0–2, or leaf bit 4; remaining flags are retained.
    pub flags: u16,
    /// Negative-side child node index, or -1 when absent.
    pub negative_child: i16,
    /// Positive-side child node index, or -1 when absent.
    pub positive_child: i16,
    /// Number of entries in the node's MOBR face window.
    pub face_count: u16,
    /// First entry in the MOBR face-index array.
    pub first_face: u32,
    /// Split-plane coordinate in model-space units.
    pub plane_distance: f32,
}
/// A window into one of the group-local light or light-animation arrays.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct LightSetRange {
    /// First element in the corresponding light array.
    pub offset: u32,
    /// Number of consecutive light elements.
    pub count: u32,
}
/// A 44-byte MOLP point light.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct PointLight {
    /// Preserved header word with unresolved semantics.
    pub reserved_header: u32,
    /// Blue, green, red and alpha bytes.
    pub color_bgra: [u8; 4],
    /// Light position in model coordinates.
    pub position: [f32; 3],
    /// Native intensity multiplier.
    pub intensity: f32,
    /// Distance where attenuation begins, in model-space units.
    pub attenuation_start: f32,
    /// Distance where attenuation ends, in model-space units.
    pub attenuation_end: f32,
    /// Preserved floating-point slot with unresolved semantics.
    pub reserved_float: f32,
    /// Two preserved trailing words with unresolved semantics.
    pub reserved_tail: [u32; 2],
}
/// A 96-byte MOP2 animated point-light record.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct PointLightAnimation {
    /// Index of the point light receiving this animation.
    pub light_index: u32,
    /// Blue, green, red and alpha bytes.
    pub color_bgra: [u8; 4],
    /// Light position in model coordinates.
    pub position: [f32; 3],
    /// Attenuation start distance in model-space units.
    pub attenuation_start: f32,
    /// Attenuation end distance in model-space units.
    pub attenuation_end: f32,
    /// Native intensity multiplier.
    pub intensity: f32,
    /// Euler rotation in radians.
    pub rotation: [f32; 3],
    /// Intensity variation used by the flicker effect.
    pub flicker_intensity: f32,
    /// Native flicker speed.
    pub flicker_speed: f32,
    /// Flicker mode: 0 off, 1 sine, 2 noise curve, 3 noise step.
    pub flicker_mode: i32,
    /// Four preserved words preceding the cookie texture reference.
    pub reserved: [i32; 4],
    /// FileDataID of the light-cookie texture.
    pub texture_file_id: u32,
    /// Five preserved trailing words with unresolved semantics.
    pub reserved_tail: [i32; 5],
}
/// A 56-byte MOLS spot light whose field layout remains unresolved.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct SpotLight {
    /// Exact native record bytes.
    pub bytes: [u8; 56],
}
/// A 108-byte MOS2 animated spot-light record with unresolved fields.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct SpotLightAnimation {
    /// Exact native record bytes.
    pub bytes: [u8; 108],
}
/// A 24-byte MOPB prepass batch with an unresolved field layout.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct PrepassBatch {
    /// Exact native record bytes.
    pub bytes: [u8; 24],
}
/// A sixteen-byte model-space plane used by terrain-cutting MOPL chunks.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Validate,
)]
pub struct Plane {
    /// Plane normal; points satisfy dot(normal, point) + distance = 0.
    pub normal: [f32; 3],
    /// Signed plane constant in model-space units.
    pub distance: f32,
}

/// Common inspection of dialect-selected group links without changing their representation.
pub trait HeaderTailMetadata {
    /// Split parent/first-child and next-sibling indices, when the dialect defines them.
    fn split_links(&self) -> Option<(i16, i16)>;
}
impl HeaderTailMetadata for u32 {
    fn split_links(&self) -> Option<(i16, i16)> {
        None
    }
}
impl HeaderTailMetadata for SplitGroupLinks {
    fn split_links(&self) -> Option<(i16, i16)> {
        Some((self.parent_or_first_child, self.next_child))
    }
}
/// Access to draw ranges shared by both batch prefix layouts.
pub trait BatchMetadata {
    /// Selects the active root material index from batch flags.
    fn material(&self, flags: u8, small: u8) -> u16;
    /// Reconstructs the first and last vertex indices from batch flags and low words.
    fn vertex_range(&self, flags: u8, minimum: u16, maximum: u16) -> (u32, u32);
}
impl BatchMetadata for CullingBox {
    fn material(&self, _: u8, small: u8) -> u16 {
        u16::from(small)
    }
    fn vertex_range(&self, _: u8, minimum: u16, maximum: u16) -> (u32, u32) {
        (minimum.into(), maximum.into())
    }
}
impl BatchMetadata for LargeMaterialPrefix {
    fn material(&self, flags: u8, small: u8) -> u16 {
        if flags & 2 != 0 {
            self.material
        } else {
            small.into()
        }
    }
    fn vertex_range(&self, flags: u8, minimum: u16, maximum: u16) -> (u32, u32) {
        if flags & 4 != 0 {
            (
                (u32::from(self.reserved[0]) << 16) | u32::from(minimum),
                (u32::from(self.reserved[1]) << 16) | u32::from(maximum),
            )
        } else {
            (minimum.into(), maximum.into())
        }
    }
}
impl<D: Dialect> Batch<D> {
    /// Returns the selected root material index.
    pub fn material_index(&self) -> u16 {
        self.prefix.material(self.flags, self.material)
    }
    /// Returns inclusive first and last vertex indices, including modern extension bytes.
    pub fn vertex_range(&self) -> (u32, u32) {
        self.prefix
            .vertex_range(self.flags, self.minimum_vertex, self.maximum_vertex)
    }
}

impl GroupLayout for wmo::Classic {
    type HeaderTail = u32;
    type BatchPrefix = CullingBox;
}
impl GroupLayout for wmo::Legion {
    type HeaderTail = u32;
    type BatchPrefix = LargeMaterialPrefix;
}
impl GroupLayout for wmo::Bfa {
    type HeaderTail = u32;
    type BatchPrefix = LargeMaterialPrefix;
}
impl GroupLayout for wmo::ShadowlandsPre92 {
    type HeaderTail = u32;
    type BatchPrefix = LargeMaterialPrefix;
}
impl GroupLayout for wmo::Shadowlands {
    type HeaderTail = SplitGroupLinks;
    type BatchPrefix = LargeMaterialPrefix;
}
impl GroupLayout for wmo::Dragonflight {
    type HeaderTail = SplitGroupLinks;
    type BatchPrefix = LargeMaterialPrefix;
}
impl GroupLayout for wmo::WarWithin {
    type HeaderTail = SplitGroupLinks;
    type BatchPrefix = LargeMaterialPrefix;
}

impl Default for SplitGroupLinks {
    fn default() -> Self {
        Self {
            parent_or_first_child: -1,
            next_child: -1,
        }
    }
}
impl Default for BspNode {
    fn default() -> Self {
        Self {
            flags: 0,
            negative_child: -1,
            positive_child: -1,
            face_count: 0,
            first_face: 0,
            plane_distance: 0.0,
        }
    }
}
