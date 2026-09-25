//! Size-bounded material and layer records.

use crate::{Error, Model};

const TAG: [u8; 4] = *b"MTLS";

/// A material record, including all version-specific bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Material {
    bytes: Vec<u8>,
}

/// A material layer record, including animation tracks and extensions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Layer {
    bytes: Vec<u8>,
}

fn sized_records<'a>(data: &'a [u8], tag: [u8; 4]) -> Result<Vec<&'a [u8]>, Error> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        let size_bytes = data
            .get(offset..offset.saturating_add(4))
            .ok_or(Error::MalformedRecord { tag, offset })?;
        let size = u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize;
        if size < 4 {
            return Err(Error::MalformedRecord { tag, offset });
        }
        let end = offset
            .checked_add(size)
            .filter(|&end| end <= data.len())
            .ok_or(Error::MalformedRecord { tag, offset })?;
        records.push(&data[offset..end]);
        offset = end;
    }
    Ok(records)
}

impl Material {
    /// Wraps one inclusive-size material record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if sized_records(bytes, TAG)?.len() != 1 {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete record, including its size field.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the material priority plane.
    pub fn priority_plane(&self) -> Result<u32, Error> {
        self.u32_at(4)
    }

    /// Sets the material priority plane.
    pub fn set_priority_plane(&mut self, value: u32) -> Result<(), Error> {
        self.set_u32_at(4, value)
    }

    /// Returns raw render mode flags.
    pub fn render_mode(&self) -> Result<u32, Error> {
        self.u32_at(8)
    }

    /// Sets raw render mode flags.
    pub fn set_render_mode(&mut self, value: u32) -> Result<(), Error> {
        self.set_u32_at(8, value)
    }

    /// Returns the bounded layer records. Versions 900 through 1099 have an
    /// additional 80-byte shader field before `LAYS`.
    pub fn layers(&self, version: u32) -> Result<Vec<Layer>, Error> {
        let offset = if (900..1100).contains(&version) {
            92
        } else {
            12
        };
        let header = self
            .bytes
            .get(offset..offset + 8)
            .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
        if &header[..4] != b"LAYS" {
            return Err(Error::MalformedRecord { tag: TAG, offset });
        }
        let count = u32::from_le_bytes(header[4..8].try_into().expect("four-byte count")) as usize;
        let records = sized_records(&self.bytes[offset + 8..], *b"LAYS")?;
        if records.len() != count {
            return Err(Error::MalformedRecord { tag: TAG, offset });
        }
        Ok(records
            .into_iter()
            .map(|bytes| Layer {
                bytes: bytes.to_vec(),
            })
            .collect())
    }

    fn u32_at(&self, offset: usize) -> Result<u32, Error> {
        let bytes = self
            .bytes
            .get(offset..offset + 4)
            .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
        Ok(u32::from_le_bytes(
            bytes.try_into().expect("four-byte field"),
        ))
    }

    fn set_u32_at(&mut self, offset: usize, value: u32) -> Result<(), Error> {
        let bytes = self
            .bytes
            .get_mut(offset..offset + 4)
            .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }
}

impl Layer {
    /// Wraps one inclusive-size layer record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if sized_records(bytes, *b"LAYS")?.len() != 1 {
            return Err(Error::MalformedRecord {
                tag: *b"LAYS",
                offset: 0,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete layer record.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the filter mode.
    pub fn filter_mode(&self) -> Result<u32, Error> {
        self.u32_at(4)
    }

    /// Returns raw shading flags.
    pub fn shading_flags(&self) -> Result<u32, Error> {
        self.u32_at(8)
    }

    /// Returns the texture index.
    pub fn texture_id(&self) -> Result<u32, Error> {
        self.u32_at(12)
    }

    /// Sets the texture index.
    pub fn set_texture_id(&mut self, id: u32) -> Result<(), Error> {
        self.set_u32_at(12, id)
    }

    /// Returns the base alpha value.
    pub fn alpha(&self) -> Result<f32, Error> {
        Ok(f32::from_bits(self.u32_at(24)?))
    }

    /// Sets the base alpha value.
    pub fn set_alpha(&mut self, alpha: f32) -> Result<(), Error> {
        self.set_u32_at(24, alpha.to_bits())
    }

    fn u32_at(&self, offset: usize) -> Result<u32, Error> {
        let bytes = self
            .bytes
            .get(offset..offset + 4)
            .ok_or(Error::MalformedRecord {
                tag: *b"LAYS",
                offset,
            })?;
        Ok(u32::from_le_bytes(
            bytes.try_into().expect("four-byte field"),
        ))
    }

    fn set_u32_at(&mut self, offset: usize, value: u32) -> Result<(), Error> {
        let bytes = self
            .bytes
            .get_mut(offset..offset + 4)
            .ok_or(Error::MalformedRecord {
                tag: *b"LAYS",
                offset,
            })?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }
}

impl Model {
    /// Decodes all `MTLS` records in file order.
    pub fn materials(&self) -> Result<Vec<Material>, Error> {
        let mut materials = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            materials.extend(
                sized_records(&chunk.data, TAG)?
                    .into_iter()
                    .map(|bytes| Material {
                        bytes: bytes.to_vec(),
                    }),
            );
        }
        Ok(materials)
    }

    /// Replaces all material records in the first `MTLS` chunk.
    pub fn set_materials(&mut self, materials: &[Material]) -> Result<(), Error> {
        let size = materials.iter().try_fold(0usize, |sum, material| {
            sum.checked_add(material.bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for material in materials {
            data.extend_from_slice(material.as_bytes());
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
