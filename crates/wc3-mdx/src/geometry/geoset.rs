//! Typed geoset sections and lossless MDX serialization.
use crate::EncodeError;
use crate::Encoder;
use crate::KnownChunk;
use crate::ValueError;
use crate::{ModelVersion, Tag, Vec3, Version};
use crate::{Readable, Writable};
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::Cursor;
use crate::GeosetsChunk;
use crate::{Decodable, Encodable};
use std::borrow::Cow;

use crate::FixedText;
use crate::{DecodeError, Model};

/// A geoset's bounding volume, also used for each sequence extent.
#[derive(Clone, Copy, Debug, PartialEq, Readable, Writable)]
pub struct GeosetExtent {
    pub bounds_radius: f32,
    pub minimum: Vec3,
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

/// An optional TANG or SKIN section. Their order is retained because both
/// orderings occur in valid files. Skin's second array has no tag or count.
#[derive(Clone, Debug, PartialEq)]
enum GeosetExtraSection<V: ModelVersion> {
    Tangents(Vec<[f32; 4]>),
    Skin {
        weights: Vec<u8>,
        bone_indices: V::SkinBones,
    },
}

/// Fixed header fields added in version 900. Both fields are always present
/// together, and the exact name bytes are retained for round-trip encoding.
#[derive(Clone, Debug, PartialEq, Readable, Writable)]
pub struct GeosetHeaderExtension {
    level_of_detail: u32,
    name: FixedText<80>,
}

/// The fixed geoset header selected by a model version.
pub trait GeosetHeader: Clone + Debug + PartialEq {
    fn empty() -> Self;
    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError>;
    fn encode(&self, output: &mut Encoder<'_>);
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

#[derive(Clone, Debug, PartialEq)]
pub struct ClassicGeosetHeader;

impl GeosetHeader for ClassicGeosetHeader {
    fn empty() -> Self {
        Self
    }
    fn decode(_: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        Ok(Self)
    }
    fn encode(&self, _: &mut Encoder<'_>) {}
}

impl GeosetHeader for GeosetHeaderExtension {
    fn empty() -> Self {
        Self {
            level_of_detail: 0,
            name: FixedText::default(),
        }
    }
    fn decode(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        cursor.read()
    }
    fn encode(&self, output: &mut Encoder<'_>) {
        output.write(self);
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
    type Header: GeosetHeader;
    type SkinBones: SkinBoneField;
}

/// Bone indices following the SKIN weights in layouts that support them.
pub trait SkinBoneField: Clone + Debug + PartialEq {
    fn from_indices(indices: Option<&[u8]>) -> Result<Self, ValueError>;
    fn decode(cursor: &mut Cursor<'_>, weight_count: usize) -> Result<Self, DecodeError>;
    fn indices(&self) -> Option<&[u8]>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct NoSkinBones;

impl SkinBoneField for NoSkinBones {
    fn from_indices(indices: Option<&[u8]>) -> Result<Self, ValueError> {
        if indices.is_some() {
            Err(ValueError::UnavailableField {
                tag: *b"GEOS",
                field: "skin bone indices",
            })
        } else {
            Ok(Self)
        }
    }
    fn decode(_: &mut Cursor<'_>, _: usize) -> Result<Self, DecodeError> {
        Ok(Self)
    }
    fn indices(&self) -> Option<&[u8]> {
        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OptionalSkinBones(Option<Vec<u8>>);

impl SkinBoneField for OptionalSkinBones {
    fn from_indices(indices: Option<&[u8]>) -> Result<Self, ValueError> {
        Ok(Self(indices.map(ToOwned::to_owned)))
    }
    fn decode(cursor: &mut Cursor<'_>, weight_count: usize) -> Result<Self, DecodeError> {
        let present = !matches!(&peek_tag(cursor)?, b"UVAS" | b"TANG");
        Ok(Self(if present {
            Some(cursor.read_exact(weight_count)?.to_vec())
        } else {
            None
        }))
    }
    fn indices(&self) -> Option<&[u8]> {
        self.0.as_deref()
    }
}

use crate::{V1000, V1100, V1200, V1800, V800, V900};
impl GeosetLayout for V800 {
    type Header = ClassicGeosetHeader;
    type SkinBones = NoSkinBones;
}
impl GeosetLayout for V900 {
    type Header = GeosetHeaderExtension;
    type SkinBones = NoSkinBones;
}
impl GeosetLayout for V1000 {
    type Header = GeosetHeaderExtension;
    type SkinBones = NoSkinBones;
}
impl GeosetLayout for V1100 {
    type Header = GeosetHeaderExtension;
    type SkinBones = NoSkinBones;
}
impl GeosetLayout for V1200 {
    type Header = GeosetHeaderExtension;
    type SkinBones = OptionalSkinBones;
}
impl GeosetLayout for V1800 {
    type Header = GeosetHeaderExtension;
    type SkinBones = OptionalSkinBones;
}

/// A geoset as typed sections. Uninterpreted packed skin bytes and the exact
/// fixed-width name field are retained for byte-for-byte serialization.
#[derive(Clone, Debug, PartialEq)]
pub struct Geoset<V: ModelVersion> {
    version: PhantomData<V>,
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    primitive_types: Vec<u32>,
    primitive_counts: Vec<u32>,
    faces: Vec<u16>,
    vertex_groups: Vec<u8>,
    matrix_group_sizes: Vec<u32>,
    matrix_indices: Vec<u32>,
    material_id: u32,
    selection_group: u32,
    unselectable_raw: u32,
    header_extension: V::Header,
    extent: GeosetExtent,
    sequence_extents: Vec<GeosetExtent>,
    extensions: Vec<GeosetExtraSection<V>>,
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
            header_extension: V::Header::empty(),
            extent: GeosetExtent::default(),
            sequence_extents: Vec::new(),
            extensions: Vec::new(),
            uv_sets: vec![vec![[0.0; 2]; vertices.len()]],
        })
    }

    /// The MDX version used to interpret this record.
    pub fn version(&self) -> Version {
        V::NUMBER
    }

    /// Borrows all vertex positions without decoding or allocating.
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
    /// Borrows primitive type identifiers from `PTYP`.
    pub fn primitive_types(&self) -> &[u32] {
        &self.primitive_types
    }
    /// Borrows primitive counts from `PCNT`.
    pub fn primitive_counts(&self) -> &[u32] {
        &self.primitive_counts
    }
    /// Borrows vertex indices from `PVTX`.
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
    /// Borrows matrix group sizes from `MTGC`.
    pub fn matrix_group_sizes(&self) -> &[u32] {
        &self.matrix_group_sizes
    }
    /// Borrows flattened matrix indices from `MATS`.
    pub fn matrix_indices(&self) -> &[u32] {
        &self.matrix_indices
    }
    /// Returns the referenced material index.
    pub fn material_id(&self) -> u32 {
        self.material_id
    }
    /// Returns the selection group index.
    pub fn selection_group(&self) -> u32 {
        self.selection_group
    }
    /// Returns the unselectable flag as a boolean.
    pub fn unselectable(&self) -> bool {
        self.unselectable_raw != 0
    }
    /// Returns the exact unselectable field, including nonstandard bits.
    pub fn raw_unselectable(&self) -> u32 {
        self.unselectable_raw
    }
    /// Returns the Reforged level of detail, if that field exists.
    pub fn level_of_detail(&self) -> Option<u32> {
        self.header_extension.level_of_detail()
    }
    /// Returns the overall geoset bounds.
    pub fn extent(&self) -> GeosetExtent {
        self.extent
    }
    /// Borrows per-sequence bounding volumes.
    pub fn sequence_extents(&self) -> &[GeosetExtent] {
        &self.sequence_extents
    }
    /// Borrows every UV coordinate set.
    pub fn uv_sets(&self) -> &[Vec<[f32; 2]>] {
        &self.uv_sets
    }
    /// Mutably borrows one UV set for bulk edits.
    pub fn uv_set_mut(&mut self, index: usize) -> Option<&mut [[f32; 2]]> {
        self.uv_sets.get_mut(index).map(Vec::as_mut_slice)
    }

    /// Returns the fixed-width name without changing nonzero padding bytes.
    pub fn name(&self) -> Option<Cow<'_, str>> {
        self.header_extension.name()
    }

    /// Borrows optional Reforged tangent vectors.
    pub fn tangents(&self) -> Option<&[[f32; 4]]> {
        self.extensions.iter().find_map(|part| match part {
            GeosetExtraSection::Tangents(values) => Some(values.as_slice()),
            _ => None,
        })
    }

    /// Borrows opaque packed skin weights.
    pub fn skin_weights(&self) -> Option<&[u8]> {
        self.extensions.iter().find_map(|part| match part {
            GeosetExtraSection::Skin { weights, .. } => Some(weights.as_slice()),
            _ => None,
        })
    }

    /// Borrows the extra untagged bone index array in newer files.
    pub fn skin_bone_indices(&self) -> Option<&[u8]> {
        self.extensions.iter().find_map(|part| match part {
            GeosetExtraSection::Skin { bone_indices, .. } => bone_indices.indices(),
            _ => None,
        })
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

    /// Changes the material reference.
    pub fn set_material_id(&mut self, id: u32) {
        self.material_id = id;
    }
    /// Changes the selection group.
    pub fn set_selection_group(&mut self, group: u32) {
        self.selection_group = group;
    }
    /// Sets the unselectable field to zero or one.
    pub fn set_unselectable(&mut self, value: bool) {
        self.unselectable_raw = u32::from(value);
    }
    /// Sets the exact unselectable field.
    pub fn set_raw_unselectable(&mut self, value: u32) {
        self.unselectable_raw = value;
    }

    /// Changes the Reforged level of detail.
    pub fn set_level_of_detail(&mut self, level: u32) -> Result<(), ValueError> {
        *self
            .header_extension
            .level_of_detail_mut()
            .ok_or(ValueError::UnavailableField {
                tag: GeosetsChunk::<V>::TAG,
                field: "level of detail",
            })? = level;
        Ok(())
    }

    /// Changes the Reforged name and clears unused name bytes.
    pub fn set_name(&mut self, name: &str) -> Result<(), ValueError> {
        self.header_extension
            .name_mut()
            .ok_or(ValueError::UnavailableField {
                tag: GeosetsChunk::<V>::TAG,
                field: "name",
            })?
            .set_text(name)?;
        Ok(())
    }

    /// Changes the overall geoset bounds.
    pub fn set_extent(&mut self, extent: GeosetExtent) {
        self.extent = extent;
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

    /// Replaces all per-sequence bounds.
    pub fn set_sequence_extents(&mut self, extents: &[GeosetExtent]) {
        self.sequence_extents = extents.to_vec();
    }

    /// Replaces or removes the Reforged tangent section, retaining its order.
    pub fn set_tangents(&mut self, tangents: Option<&[[f32; 4]]>) -> Result<(), ValueError> {
        if V::NUMBER < 900 {
            return Err(ValueError::UnavailableField {
                tag: GeosetsChunk::<V>::TAG,
                field: "tangents",
            });
        }
        if let Some(index) = self
            .extensions
            .iter()
            .position(|part| matches!(part, GeosetExtraSection::Tangents(_)))
        {
            if let Some(values) = tangents {
                self.extensions[index] = GeosetExtraSection::Tangents(values.to_vec());
            } else {
                self.extensions.remove(index);
            }
        } else if let Some(values) = tangents {
            self.extensions
                .insert(0, GeosetExtraSection::Tangents(values.to_vec()));
        }
        Ok(())
    }

    /// Replaces or removes packed skin data, retaining its section order.
    pub fn set_skin_data(
        &mut self,
        weights: Option<&[u8]>,
        bone_indices: Option<&[u8]>,
    ) -> Result<(), ValueError> {
        if V::NUMBER < 900 {
            return Err(ValueError::UnavailableField {
                tag: GeosetsChunk::<V>::TAG,
                field: "skin data",
            });
        }
        if let Some(indices) = bone_indices {
            if V::NUMBER < 1200 {
                return Err(ValueError::UnavailableField {
                    tag: GeosetsChunk::<V>::TAG,
                    field: "skin bone indices",
                });
            }
            let weights = weights.ok_or(ValueError::MissingField {
                tag: GeosetsChunk::<V>::TAG,
                field: "skin weights",
            })?;
            if weights.len() != indices.len() {
                return Err(ValueError::LengthMismatch {
                    tag: GeosetsChunk::<V>::TAG,
                    expected: weights.len(),
                    actual: indices.len(),
                });
            }
        }
        let skin_bones = V::SkinBones::from_indices(bone_indices)?;
        let new_part = weights.map(|weights| GeosetExtraSection::Skin {
            weights: weights.to_vec(),
            bone_indices: skin_bones,
        });
        if let Some(index) = self
            .extensions
            .iter()
            .position(|part| matches!(part, GeosetExtraSection::Skin { .. }))
        {
            if let Some(part) = new_part {
                self.extensions[index] = part;
            } else {
                self.extensions.remove(index);
            }
        } else if let Some(part) = new_part {
            self.extensions.push(part);
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

fn peek_tag(cursor: &Cursor<'_>) -> Result<Tag, DecodeError> {
    Ok(cursor.peek_exact(4)?.try_into().expect("four-byte tag"))
}

fn section<'a>(cursor: &mut Cursor<'a>, tag: Tag, stride: usize) -> Result<&'a [u8], DecodeError> {
    let offset = cursor.absolute_position();
    if cursor.read_exact(4)? != tag {
        return Err(DecodeError::MalformedRecord {
            tag: *b"GEOS",
            offset,
        });
    }
    let count = cursor.read::<u32>()? as usize;
    let size = count
        .checked_mul(stride)
        .ok_or(DecodeError::MalformedRecord {
            tag: *b"GEOS",
            offset,
        })?;
    cursor.read_exact(size)
}

fn decode_values<T: Readable>(bytes: &[u8], width: usize) -> Result<Vec<T>, DecodeError> {
    let mut cursor = Cursor::new(bytes);
    let values = (0..bytes.len() / width)
        .map(|_| cursor.read())
        .collect::<Result<Vec<_>, _>>()?;
    cursor.finish()?;
    Ok(values)
}

fn write_count(bytes: &mut Encoder<'_>, count: usize) -> Result<(), EncodeError> {
    let count = u32::try_from(count).map_err(|_| EncodeError::ChunkTooLarge {
        tag: *b"GEOS",
        size: count,
    })?;
    bytes.write(count);
    Ok(())
}

fn write_section_header(
    bytes: &mut Encoder<'_>,
    tag: Tag,
    count: usize,
) -> Result<(), EncodeError> {
    bytes.write_bytes(&tag);
    write_count(bytes, count)
}

fn write_words(bytes: &mut Encoder<'_>, tag: Tag, words: &[u32]) -> Result<(), EncodeError> {
    write_section_header(bytes, tag, words.len())?;
    for word in words {
        bytes.write(word);
    }
    Ok(())
}

fn write_vectors<const N: usize>(
    bytes: &mut Encoder<'_>,
    tag: Tag,
    vectors: &[[f32; N]],
) -> Result<(), EncodeError> {
    write_section_header(bytes, tag, vectors.len())?;
    bytes.write(vectors);
    Ok(())
}

impl<V: ModelVersion> Model<V> {
    /// Decodes geosets from every `GEOS` chunk in file order.
    pub fn geosets(&self) -> Vec<Geoset<V>> {
        self.collect_chunk_records::<GeosetsChunk<V>>()
    }

    /// Replaces geosets.
    pub fn set_geosets(&mut self, geosets: &[Geoset<V>]) {
        self.replace_chunk(GeosetsChunk::new(geosets.to_vec()));
    }
}

impl<V: ModelVersion> Decodable for Geoset<V> {
    fn decode_one(source: &mut Cursor<'_>, _version: Version) -> Result<Self, DecodeError> {
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
            let header_extension = V::Header::decode(&mut cursor)?;
            let extent = cursor.read()?;
            let sequence_count = cursor.read::<u32>()? as usize;
            let mut sequence_extents = Vec::new();
            for _ in 0..sequence_count {
                sequence_extents.push(cursor.read()?);
            }
            let mut extensions = Vec::new();
            if V::NUMBER >= 900 {
                while peek_tag(&cursor)? != *b"UVAS" {
                    let offset = cursor.absolute_position();
                    let tag = peek_tag(&cursor)?;
                    match &tag {
                        b"TANG"
                            if !extensions
                                .iter()
                                .any(|part| matches!(part, GeosetExtraSection::Tangents(_))) =>
                        {
                            extensions.push(GeosetExtraSection::Tangents(
                                decode_values::<[f32; 4]>(section(&mut cursor, *b"TANG", 16)?, 16)?,
                            ));
                        }
                        b"SKIN"
                            if !extensions
                                .iter()
                                .any(|part| matches!(part, GeosetExtraSection::Skin { .. })) =>
                        {
                            let weights = section(&mut cursor, *b"SKIN", 1)?.to_vec();
                            let bone_indices = V::SkinBones::decode(&mut cursor, weights.len())?;
                            extensions.push(GeosetExtraSection::Skin {
                                weights,
                                bone_indices,
                            });
                        }
                        _ => {
                            return Err(DecodeError::MalformedRecord {
                                tag: GeosetsChunk::<V>::TAG,
                                offset,
                            })
                        }
                    }
                }
            }
            if cursor.read_exact(4)? != b"UVAS" {
                return Err(DecodeError::MalformedRecord {
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
                header_extension,
                extent,
                sequence_extents,
                extensions,
                uv_sets,
            })
        }?;
        cursor.finish()?;
        Ok(value)
    }
}

impl<V: ModelVersion> Encodable for Geoset<V> {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let start = bytes.position();
        let marker = bytes.begin_sized();
        write_vectors(bytes, *b"VRTX", &self.vertices)?;
        write_vectors(bytes, *b"NRMS", &self.normals)?;
        write_words(bytes, *b"PTYP", &self.primitive_types)?;
        write_words(bytes, *b"PCNT", &self.primitive_counts)?;
        write_section_header(bytes, *b"PVTX", self.faces.len())?;
        for face in &self.faces {
            bytes.write(face);
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
            bytes.write(word);
        }
        self.header_extension.encode(bytes);
        bytes.write(&self.extent);
        write_count(bytes, self.sequence_extents.len())?;
        for extent in &self.sequence_extents {
            bytes.write(extent);
        }
        for extension in &self.extensions {
            match extension {
                GeosetExtraSection::Tangents(tangents) => write_vectors(bytes, *b"TANG", tangents)?,
                GeosetExtraSection::Skin {
                    weights,
                    bone_indices,
                } => {
                    write_section_header(bytes, *b"SKIN", weights.len())?;
                    bytes.write_bytes(weights);
                    if let Some(indices) = bone_indices.indices() {
                        if V::NUMBER < 1200 || indices.len() != weights.len() {
                            return Err(EncodeError::MalformedRecord {
                                tag: GeosetsChunk::<V>::TAG,
                                offset: bytes.position() - start,
                            });
                        }
                        bytes.write_bytes(indices);
                    }
                }
            }
        }
        bytes.write_bytes(b"UVAS");
        write_count(bytes, self.uv_sets.len())?;
        for uv_set in &self.uv_sets {
            write_vectors(bytes, *b"UVBS", uv_set)?;
        }
        if bytes.position() - start > u32::MAX as usize {
            return Err(EncodeError::ChunkTooLarge {
                tag: GeosetsChunk::<V>::TAG,
                size: bytes.position() - start,
            });
        }
        bytes.finish_sized(marker, GeosetsChunk::<V>::TAG)?;
        Ok(())
    }
}
