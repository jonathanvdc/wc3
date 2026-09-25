//! Size-bounded geoset records and basic mesh accessors.

use crate::{Error, Model};

const TAG: [u8; 4] = *b"GEOS";

/// A geoset record, retaining all version-specific fields and unknown data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Geoset {
    bytes: Vec<u8>,
}

impl Geoset {
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
