//! Size-bounded geoset records and basic mesh accessors.

use std::borrow::Cow;

use crate::{Error, Model};

const TAG: [u8; 4] = *b"GEOS";

/// A geoset record, retaining all version-specific fields and unknown data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Geoset {
    bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
struct Layout {
    vertex_groups: (usize, usize),
    matrix_group_sizes: (usize, usize),
    matrix_indices: (usize, usize),
    properties: usize,
    extent: usize,
    sequence_extents: (usize, usize),
    uv_start: usize,
    tangents: Option<(usize, usize)>,
    skin_weights: Option<(usize, usize)>,
    skin_bone_indices: Option<(usize, usize)>,
    uv_sets: Vec<(usize, usize)>,
}

/// Bounding volume stored on a geoset or one of its sequence extents.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeosetExtent {
    pub bounds_radius: f32,
    pub minimum: [f32; 3],
    pub maximum: [f32; 3],
}

impl Geoset {
    /// Builds a basic geoset with one matrix group and one UV set.
    pub fn new(
        version: u32,
        vertices: &[[f32; 3]],
        normals: &[[f32; 3]],
        faces: &[u16],
    ) -> Result<Self, Error> {
        if vertices.len() != normals.len()
            || vertices.len() > u32::MAX as usize
            || faces.len() > u32::MAX as usize
        {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        let mut bytes = vec![0; 4];
        {
            let mut section = |tag: [u8; 4], count: usize, data: &[u8]| {
                bytes.extend_from_slice(&tag);
                bytes.extend_from_slice(&(count as u32).to_le_bytes());
                bytes.extend_from_slice(data);
            };
            let mut coordinates = Vec::with_capacity(vertices.len() * 12);
            for vertex in vertices {
                for value in vertex {
                    coordinates.extend_from_slice(&value.to_le_bytes());
                }
            }
            section(*b"VRTX", vertices.len(), &coordinates);
            coordinates.clear();
            for normal in normals {
                for value in normal {
                    coordinates.extend_from_slice(&value.to_le_bytes());
                }
            }
            section(*b"NRMS", normals.len(), &coordinates);
            section(*b"PTYP", 1, &4u32.to_le_bytes());
            section(*b"PCNT", 1, &(faces.len() as u32).to_le_bytes());
            let mut indices = Vec::with_capacity(faces.len() * 2);
            for index in faces {
                indices.extend_from_slice(&index.to_le_bytes());
            }
            section(*b"PVTX", faces.len(), &indices);
            section(*b"GNDX", vertices.len(), &vec![0; vertices.len()]);
            section(*b"MTGC", 1, &1u32.to_le_bytes());
            section(*b"MATS", 1, &0u32.to_le_bytes());
        }
        bytes.extend_from_slice(&[0; 12]);
        if version >= 900 {
            bytes.extend_from_slice(&[0; 84]);
        }
        bytes.extend_from_slice(&[0; 28]);
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(b"UVAS");
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(b"UVBS");
        bytes.extend_from_slice(&(vertices.len() as u32).to_le_bytes());
        bytes.resize(bytes.len() + vertices.len() * 8, 0);
        if bytes.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: bytes.len(),
            });
        }
        let size = bytes.len() as u32;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        Ok(Self { bytes })
    }

    /// Wraps one inclusive-size geoset record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 4 {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        let size = u32::from_le_bytes(bytes[..4].try_into().expect("four-byte size"));
        if size as usize != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete record, including its inclusive size field.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the XYZ vertices from the leading `VRTX` section.
    pub fn vertices(&self) -> Result<Vec<[f32; 3]>, Error> {
        let (data, _) = self.section(4, *b"VRTX", 12)?;
        Ok(data
            .chunks_exact(12)
            .map(|vertex| {
                std::array::from_fn(|index| {
                    let start = index * 4;
                    f32::from_le_bytes(
                        vertex[start..start + 4]
                            .try_into()
                            .expect("four-byte field"),
                    )
                })
            })
            .collect())
    }

    /// Changes one XYZ vertex while leaving all other geoset bytes intact.
    pub fn set_vertex(&mut self, index: usize, vertex: [f32; 3]) -> Result<(), Error> {
        let (_, end) = self.section(4, *b"VRTX", 12)?;
        let start = 12usize
            .checked_add(index.checked_mul(12).ok_or(Error::MalformedRecord {
                tag: TAG,
                offset: index,
            })?)
            .ok_or(Error::MalformedRecord {
                tag: TAG,
                offset: index,
            })?;
        if start.checked_add(12).map_or(true, |next| next > end) {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: index,
            });
        }
        for (coordinate, value) in vertex.into_iter().enumerate() {
            let offset = start + coordinate * 4;
            self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }

    /// Returns XYZ normals from the `NRMS` section.
    pub fn normals(&self) -> Result<Vec<[f32; 3]>, Error> {
        let (_, vertex_end) = self.section(4, *b"VRTX", 12)?;
        let (data, _) = self.section(vertex_end, *b"NRMS", 12)?;
        Ok(data
            .chunks_exact(12)
            .map(|normal| {
                std::array::from_fn(|index| {
                    let start = index * 4;
                    f32::from_le_bytes(
                        normal[start..start + 4]
                            .try_into()
                            .expect("four-byte field"),
                    )
                })
            })
            .collect())
    }

    /// Changes one normal without altering other sections.
    pub fn set_normal(&mut self, index: usize, normal: [f32; 3]) -> Result<(), Error> {
        let (_, vertex_end) = self.section(4, *b"VRTX", 12)?;
        let (_, normal_end) = self.section(vertex_end, *b"NRMS", 12)?;
        let start = vertex_end
            .checked_add(8)
            .and_then(|n| index.checked_mul(12).and_then(|m| n.checked_add(m)))
            .filter(|&start| start.checked_add(12).is_some_and(|end| end <= normal_end))
            .ok_or(Error::MalformedRecord {
                tag: TAG,
                offset: index,
            })?;
        for (i, value) in normal.into_iter().enumerate() {
            self.bytes[start + i * 4..start + i * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }

    /// Returns face vertex indices from `PVTX`.
    pub fn face_indices(&self) -> Result<Vec<u16>, Error> {
        let (_, vertex_end) = self.section(4, *b"VRTX", 12)?;
        let (_, normal_end) = self.section(vertex_end, *b"NRMS", 12)?;
        let (_, primitive_end) = self.section(normal_end, *b"PTYP", 4)?;
        let (_, counts_end) = self.section(primitive_end, *b"PCNT", 4)?;
        let (data, _) = self.section(counts_end, *b"PVTX", 2)?;
        Ok(data
            .chunks_exact(2)
            .map(|bytes| u16::from_le_bytes(bytes.try_into().expect("two-byte field")))
            .collect())
    }

    /// Returns one matrix group index per vertex.
    pub fn vertex_groups(&self, version: u32) -> Result<&[u8], Error> {
        let layout = self.layout(version)?;
        Ok(&self.bytes[layout.vertex_groups.0..layout.vertex_groups.1])
    }

    /// Replaces the matrix group index assigned to each vertex.
    pub fn set_vertex_groups(&mut self, version: u32, groups: &[u8]) -> Result<(), Error> {
        if groups.len() != self.word(8) as usize {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: groups.len(),
            });
        }
        let layout = self.layout(version)?;
        let mut data = Vec::with_capacity(groups.len() + 8);
        data.extend_from_slice(b"GNDX");
        data.extend_from_slice(&(groups.len() as u32).to_le_bytes());
        data.extend_from_slice(groups);
        self.replace_range(layout.vertex_groups.0 - 8..layout.vertex_groups.1, &data)
    }

    /// Returns the number of matrix entries in each geoset group.
    pub fn matrix_group_sizes(&self, version: u32) -> Result<Vec<u32>, Error> {
        let layout = self.layout(version)?;
        Ok(self.words(layout.matrix_group_sizes))
    }

    /// Returns flattened matrix indices for all geoset groups.
    pub fn matrix_indices(&self, version: u32) -> Result<Vec<u32>, Error> {
        let layout = self.layout(version)?;
        Ok(self.words(layout.matrix_indices))
    }

    /// Replaces matrix groups and their flattened matrix references.
    pub fn set_matrix_groups(&mut self, version: u32, groups: &[Vec<u32>]) -> Result<(), Error> {
        if groups.len() > u32::MAX as usize
            || groups.iter().any(|group| group.len() > u32::MAX as usize)
        {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: usize::MAX,
            });
        }
        let layout = self.layout(version)?;
        let mut data = Vec::new();
        data.extend_from_slice(b"MTGC");
        data.extend_from_slice(&(groups.len() as u32).to_le_bytes());
        for group in groups {
            data.extend_from_slice(&(group.len() as u32).to_le_bytes());
        }
        data.extend_from_slice(b"MATS");
        let total = groups
            .iter()
            .try_fold(0usize, |sum, group| sum.checked_add(group.len()))
            .filter(|&n| n <= u32::MAX as usize)
            .ok_or(Error::ChunkTooLarge {
                tag: TAG,
                size: usize::MAX,
            })?;
        data.extend_from_slice(&(total as u32).to_le_bytes());
        for group in groups {
            for index in group {
                data.extend_from_slice(&index.to_le_bytes());
            }
        }
        self.replace_range(
            layout.matrix_group_sizes.0 - 8..layout.matrix_indices.1,
            &data,
        )
    }

    /// Returns the material index used by this geoset.
    pub fn material_id(&self, version: u32) -> Result<u32, Error> {
        Ok(self.word(self.layout(version)?.properties))
    }

    /// Changes the material index without altering the other mesh sections.
    pub fn set_material_id(&mut self, version: u32, id: u32) -> Result<(), Error> {
        let offset = self.layout(version)?.properties;
        self.bytes[offset..offset + 4].copy_from_slice(&id.to_le_bytes());
        Ok(())
    }

    /// Returns the selection group index.
    pub fn selection_group(&self, version: u32) -> Result<u32, Error> {
        Ok(self.word(self.layout(version)?.properties + 4))
    }

    /// Changes the selection group index.
    pub fn set_selection_group(&mut self, version: u32, group: u32) -> Result<(), Error> {
        let offset = self.layout(version)?.properties + 4;
        self.bytes[offset..offset + 4].copy_from_slice(&group.to_le_bytes());
        Ok(())
    }

    /// Returns the raw unselectable flag word.
    pub fn unselectable(&self, version: u32) -> Result<bool, Error> {
        Ok(self.word(self.layout(version)?.properties + 8) != 0)
    }

    /// Changes the unselectable flag while preserving other fields.
    pub fn set_unselectable(&mut self, version: u32, value: bool) -> Result<(), Error> {
        let offset = self.layout(version)?.properties + 8;
        self.bytes[offset..offset + 4].copy_from_slice(&u32::from(value).to_le_bytes());
        Ok(())
    }

    /// Returns the level of detail present since version 900.
    pub fn level_of_detail(&self, version: u32) -> Result<Option<u32>, Error> {
        let layout = self.layout(version)?;
        Ok((version >= 900).then(|| self.word(layout.properties + 12)))
    }

    /// Changes the level of detail in a version 900 or later geoset.
    pub fn set_level_of_detail(&mut self, version: u32, level: u32) -> Result<(), Error> {
        if version < 900 {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        let offset = self.layout(version)?.properties + 12;
        self.bytes[offset..offset + 4].copy_from_slice(&level.to_le_bytes());
        Ok(())
    }

    /// Returns the version 900 or later geoset name, if present.
    pub fn name(&self, version: u32) -> Result<Option<Cow<'_, str>>, Error> {
        if version < 900 {
            return Ok(None);
        }
        let offset = self.layout(version)?.properties + 16;
        let field = &self.bytes[offset..offset + 80];
        let end = field.iter().position(|&byte| byte == 0).unwrap_or(80);
        Ok(Some(String::from_utf8_lossy(&field[..end])))
    }

    /// Changes the version 900 or later geoset name.
    pub fn set_name(&mut self, version: u32, name: &str) -> Result<(), Error> {
        if version < 900 {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        if name.len() >= 80 || name.as_bytes().contains(&0) {
            return Err(Error::InvalidString { max_bytes: 79 });
        }
        let offset = self.layout(version)?.properties + 16;
        self.bytes[offset..offset + 80].fill(0);
        self.bytes[offset..offset + name.len()].copy_from_slice(name.as_bytes());
        Ok(())
    }

    /// Returns the geoset's bounding volume.
    pub fn extent(&self, version: u32) -> Result<GeosetExtent, Error> {
        Ok(self.extent_at(self.layout(version)?.extent))
    }

    /// Changes the geoset's bounding volume.
    pub fn set_extent(&mut self, version: u32, extent: GeosetExtent) -> Result<(), Error> {
        let offset = self.layout(version)?.extent;
        self.set_extent_at(offset, extent);
        Ok(())
    }

    /// Returns per-sequence bounding volumes.
    pub fn sequence_extents(&self, version: u32) -> Result<Vec<GeosetExtent>, Error> {
        let layout = self.layout(version)?;
        Ok((layout.sequence_extents.0..layout.sequence_extents.1)
            .step_by(28)
            .map(|offset| self.extent_at(offset))
            .collect())
    }

    /// Changes one per-sequence bounding volume.
    pub fn set_sequence_extent(
        &mut self,
        version: u32,
        index: usize,
        extent: GeosetExtent,
    ) -> Result<(), Error> {
        let layout = self.layout(version)?;
        let offset = index
            .checked_mul(28)
            .and_then(|n| layout.sequence_extents.0.checked_add(n))
            .filter(|&offset| {
                offset
                    .checked_add(28)
                    .is_some_and(|end| end <= layout.sequence_extents.1)
            })
            .ok_or(Error::MalformedRecord {
                tag: TAG,
                offset: index,
            })?;
        self.set_extent_at(offset, extent);
        Ok(())
    }

    /// Replaces all per-sequence bounding volumes.
    pub fn set_sequence_extents(
        &mut self,
        version: u32,
        extents: &[GeosetExtent],
    ) -> Result<(), Error> {
        if extents.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: extents.len(),
            });
        }
        let layout = self.layout(version)?;
        let mut data = Vec::new();
        data.extend_from_slice(&(extents.len() as u32).to_le_bytes());
        for extent in extents {
            data.extend_from_slice(&extent.bounds_radius.to_le_bytes());
            for value in extent.minimum.into_iter().chain(extent.maximum) {
                data.extend_from_slice(&value.to_le_bytes());
            }
        }
        self.replace_range(
            layout.sequence_extents.0 - 4..layout.sequence_extents.1,
            &data,
        )
    }

    /// Returns optional Reforged XYZW tangent vectors.
    pub fn tangents(&self, version: u32) -> Result<Option<Vec<[f32; 4]>>, Error> {
        let layout = self.layout(version)?;
        Ok(layout.tangents.map(|(start, end)| {
            self.bytes[start..end]
                .chunks_exact(16)
                .map(|bytes| {
                    std::array::from_fn(|i| {
                        f32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().expect("tangent"))
                    })
                })
                .collect()
        }))
    }

    /// Replaces or removes the optional Reforged tangent section.
    pub fn set_tangents(
        &mut self,
        version: u32,
        tangents: Option<&[[f32; 4]]>,
    ) -> Result<(), Error> {
        if version < 900 {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        let layout = self.layout(version)?;
        let range = if let Some((start, end)) = layout.tangents {
            start - 8..end
        } else {
            let start = layout
                .skin_weights
                .map_or(layout.uv_start, |(start, _)| start - 8);
            start..start
        };
        let mut data = Vec::new();
        if let Some(tangents) = tangents {
            if tangents.len() > u32::MAX as usize {
                return Err(Error::ChunkTooLarge {
                    tag: TAG,
                    size: tangents.len(),
                });
            }
            data.extend_from_slice(b"TANG");
            data.extend_from_slice(&(tangents.len() as u32).to_le_bytes());
            for tangent in tangents {
                for value in tangent {
                    data.extend_from_slice(&value.to_le_bytes());
                }
            }
        }
        self.replace_range(range, &data)
    }

    /// Returns optional Reforged skin weight and bone index bytes.
    pub fn skin_weights(&self, version: u32) -> Result<Option<&[u8]>, Error> {
        let layout = self.layout(version)?;
        Ok(layout
            .skin_weights
            .map(|(start, end)| &self.bytes[start..end]))
    }

    /// Returns the additional packed bone-index bytes found after skin weights in newer files.
    pub fn skin_bone_indices(&self, version: u32) -> Result<Option<&[u8]>, Error> {
        let layout = self.layout(version)?;
        Ok(layout
            .skin_bone_indices
            .map(|(start, end)| &self.bytes[start..end]))
    }

    /// Replaces or removes the optional skin data. Newer layouts may include a
    /// second equally sized packed bone-index array after the weights.
    pub fn set_skin_data(
        &mut self,
        version: u32,
        weights: Option<&[u8]>,
        bone_indices: Option<&[u8]>,
    ) -> Result<(), Error> {
        if version < 900 {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        let layout = self.layout(version)?;
        let range = if let Some((start, end)) = layout.skin_weights {
            start - 8..layout.skin_bone_indices.map_or(end, |(_, end)| end)
        } else {
            layout.uv_start..layout.uv_start
        };
        if bone_indices.is_some()
            && (version < 1200
                || weights.is_none()
                || weights.unwrap().len() != bone_indices.unwrap().len())
        {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: range.start,
            });
        }
        let mut data = Vec::new();
        if let Some(weights) = weights {
            if weights.len() > u32::MAX as usize {
                return Err(Error::ChunkTooLarge {
                    tag: TAG,
                    size: weights.len(),
                });
            }
            data.extend_from_slice(b"SKIN");
            data.extend_from_slice(&(weights.len() as u32).to_le_bytes());
            data.extend_from_slice(weights);
            if let Some(indices) = bone_indices {
                data.extend_from_slice(indices);
            }
        }
        self.replace_range(range, &data)
    }

    /// Returns all UV coordinate sets.
    pub fn uv_sets(&self, version: u32) -> Result<Vec<Vec<[f32; 2]>>, Error> {
        let layout = self.layout(version)?;
        Ok(layout
            .uv_sets
            .into_iter()
            .map(|(start, end)| {
                self.bytes[start..end]
                    .chunks_exact(8)
                    .map(|bytes| {
                        [
                            f32::from_le_bytes(bytes[..4].try_into().expect("u coordinate")),
                            f32::from_le_bytes(bytes[4..8].try_into().expect("v coordinate")),
                        ]
                    })
                    .collect()
            })
            .collect())
    }

    /// Changes one UV coordinate in an existing set.
    pub fn set_uv(
        &mut self,
        version: u32,
        set: usize,
        index: usize,
        uv: [f32; 2],
    ) -> Result<(), Error> {
        let layout = self.layout(version)?;
        let (start, end) = *layout.uv_sets.get(set).ok_or(Error::MalformedRecord {
            tag: TAG,
            offset: set,
        })?;
        let offset = index
            .checked_mul(8)
            .and_then(|n| start.checked_add(n))
            .filter(|&offset| {
                offset
                    .checked_add(8)
                    .is_some_and(|end_offset| end_offset <= end)
            })
            .ok_or(Error::MalformedRecord {
                tag: TAG,
                offset: index,
            })?;
        self.bytes[offset..offset + 4].copy_from_slice(&uv[0].to_le_bytes());
        self.bytes[offset + 4..offset + 8].copy_from_slice(&uv[1].to_le_bytes());
        Ok(())
    }

    /// Replaces every UV coordinate set.
    pub fn set_uv_sets(&mut self, version: u32, sets: &[Vec<[f32; 2]>]) -> Result<(), Error> {
        if sets.len() > u32::MAX as usize || sets.iter().any(|set| set.len() > u32::MAX as usize) {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: usize::MAX,
            });
        }
        let layout = self.layout(version)?;
        let mut data = Vec::new();
        data.extend_from_slice(b"UVAS");
        data.extend_from_slice(&(sets.len() as u32).to_le_bytes());
        for set in sets {
            data.extend_from_slice(b"UVBS");
            data.extend_from_slice(&(set.len() as u32).to_le_bytes());
            for uv in set {
                for value in uv {
                    data.extend_from_slice(&value.to_le_bytes());
                }
            }
        }
        self.replace_range(layout.uv_start..self.bytes.len(), &data)
    }

    fn layout(&self, version: u32) -> Result<Layout, Error> {
        let (_, end) = self.section(4, *b"VRTX", 12)?;
        let (_, end) = self.section(end, *b"NRMS", 12)?;
        let (_, end) = self.section(end, *b"PTYP", 4)?;
        let (_, end) = self.section(end, *b"PCNT", 4)?;
        let (_, end) = self.section(end, *b"PVTX", 2)?;
        let (data, end) = self.section(end, *b"GNDX", 1)?;
        let vertex_groups = (end - data.len(), end);
        let (data, end) = self.section(end, *b"MTGC", 4)?;
        let matrix_group_sizes = (end - data.len(), end);
        let (data, end) = self.section(end, *b"MATS", 4)?;
        let matrix_indices = (end - data.len(), end);
        let properties = end;
        let metadata_size = if version >= 900 { 96 } else { 12 };
        let extent = end
            .checked_add(metadata_size)
            .filter(|&next| next <= self.bytes.len())
            .ok_or(Error::MalformedRecord {
                tag: TAG,
                offset: end,
            })?;
        let sequence_count_offset = extent
            .checked_add(28)
            .filter(|&next| next + 4 <= self.bytes.len())
            .ok_or(Error::MalformedRecord {
                tag: TAG,
                offset: extent,
            })?;
        let count = self.word(sequence_count_offset) as usize;
        let sequence_start = sequence_count_offset + 4;
        let mut offset = count
            .checked_mul(28)
            .and_then(|n| sequence_start.checked_add(n))
            .filter(|&next| next <= self.bytes.len())
            .ok_or(Error::MalformedRecord {
                tag: TAG,
                offset: sequence_start,
            })?;
        let sequence_extents = (sequence_start, offset);
        let mut tangents = None;
        let mut skin_weights = None;
        let mut skin_bone_indices = None;
        if version >= 900 {
            loop {
                let tag = self
                    .bytes
                    .get(offset..offset + 4)
                    .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
                if tag == b"UVAS" {
                    break;
                }
                let kind = if tag == b"TANG" {
                    0
                } else if tag == b"SKIN" {
                    1
                } else {
                    return Err(Error::MalformedRecord { tag: TAG, offset });
                };
                let (_, next) = self.section(
                    offset,
                    if kind == 0 { *b"TANG" } else { *b"SKIN" },
                    if kind == 0 { 16 } else { 1 },
                )?;
                let range = (offset + 8, next);
                if kind == 0 {
                    if tangents.replace(range).is_some() {
                        return Err(Error::MalformedRecord { tag: TAG, offset });
                    }
                } else if skin_weights.replace(range).is_some() {
                    return Err(Error::MalformedRecord { tag: TAG, offset });
                }
                offset = next;
                if kind == 1
                    && version >= 1200
                    && self.bytes.get(offset..offset + 4) != Some(b"UVAS")
                {
                    let end = offset
                        .checked_add(range.1 - range.0)
                        .filter(|&end| end <= self.bytes.len())
                        .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
                    skin_bone_indices = Some((offset, end));
                    offset = end;
                }
            }
        }
        let uv_start = offset;
        if self.bytes.get(offset..offset + 4) != Some(b"UVAS") {
            return Err(Error::MalformedRecord { tag: TAG, offset });
        }
        if self.bytes.get(offset + 4..offset + 8).is_none() {
            return Err(Error::MalformedRecord { tag: TAG, offset });
        }
        let uv_count = self.word(offset + 4) as usize;
        offset += 8;
        let mut uv_sets = Vec::new();
        for _ in 0..uv_count {
            let (data, end) = self.section(offset, *b"UVBS", 8)?;
            uv_sets.push((end - data.len(), end));
            offset = end;
        }
        if offset != self.bytes.len() {
            return Err(Error::MalformedRecord { tag: TAG, offset });
        }
        Ok(Layout {
            vertex_groups,
            matrix_group_sizes,
            matrix_indices,
            properties,
            extent,
            sequence_extents,
            uv_start,
            tangents,
            skin_weights,
            skin_bone_indices,
            uv_sets,
        })
    }

    fn word(&self, offset: usize) -> u32 {
        u32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("checked field"),
        )
    }

    fn words(&self, (start, end): (usize, usize)) -> Vec<u32> {
        self.bytes[start..end]
            .chunks_exact(4)
            .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("matrix word")))
            .collect()
    }

    fn extent_at(&self, offset: usize) -> GeosetExtent {
        GeosetExtent {
            bounds_radius: f32::from_bits(self.word(offset)),
            minimum: std::array::from_fn(|i| f32::from_bits(self.word(offset + 4 + i * 4))),
            maximum: std::array::from_fn(|i| f32::from_bits(self.word(offset + 16 + i * 4))),
        }
    }

    fn set_extent_at(&mut self, offset: usize, extent: GeosetExtent) {
        self.bytes[offset..offset + 4].copy_from_slice(&extent.bounds_radius.to_le_bytes());
        for (i, value) in extent.minimum.into_iter().chain(extent.maximum).enumerate() {
            self.bytes[offset + 4 + i * 4..offset + 8 + i * 4]
                .copy_from_slice(&value.to_le_bytes());
        }
    }

    fn replace_range(&mut self, range: std::ops::Range<usize>, data: &[u8]) -> Result<(), Error> {
        let size = self
            .bytes
            .len()
            .checked_sub(range.len())
            .and_then(|n| n.checked_add(data.len()))
            .filter(|&n| n <= u32::MAX as usize)
            .ok_or(Error::ChunkTooLarge {
                tag: TAG,
                size: usize::MAX,
            })?;
        self.bytes.splice(range, data.iter().copied());
        self.bytes[..4].copy_from_slice(&(size as u32).to_le_bytes());
        Ok(())
    }

    fn section(&self, offset: usize, tag: [u8; 4], stride: usize) -> Result<(&[u8], usize), Error> {
        let header = self
            .bytes
            .get(offset..offset.saturating_add(8))
            .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
        if header[..4] != tag {
            return Err(Error::MalformedRecord { tag: TAG, offset });
        }
        let count = u32::from_le_bytes(header[4..8].try_into().expect("four-byte count"));
        let start = offset + 8;
        let end = (count as usize)
            .checked_mul(stride)
            .and_then(|size| start.checked_add(size))
            .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
        let data = self
            .bytes
            .get(start..end)
            .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
        Ok((data, end))
    }
}

impl Model {
    /// Decodes size-bounded geoset records from all `GEOS` chunks.
    pub fn geosets(&self) -> Result<Vec<Geoset>, Error> {
        let mut geosets = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            let mut offset = 0;
            while offset < chunk.data.len() {
                let size_bytes = chunk
                    .data
                    .get(offset..offset.saturating_add(4))
                    .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
                let size =
                    u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize;
                if size < 4 {
                    return Err(Error::MalformedRecord { tag: TAG, offset });
                }
                let end = offset
                    .checked_add(size)
                    .filter(|&end| end <= chunk.data.len())
                    .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
                geosets.push(Geoset::from_bytes(&chunk.data[offset..end])?);
                offset = end;
            }
        }
        Ok(geosets)
    }

    /// Replaces all geoset records in the first `GEOS` chunk.
    pub fn set_geosets(&mut self, geosets: &[Geoset]) -> Result<(), Error> {
        let size = geosets.iter().try_fold(0usize, |sum, geoset| {
            sum.checked_add(geoset.bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for geoset in geosets {
            data.extend_from_slice(geoset.as_bytes());
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
