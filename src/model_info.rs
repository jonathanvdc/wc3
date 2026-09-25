//! Fixed-size `MODL` model information.

use crate::Record;
use std::borrow::Cow;

use crate::utils::field;
use crate::{Error, Model};

const SIZE: usize = 372;
const NAME_SIZE: usize = 336;

/// The 372-byte `MODL` record. Reserved bytes remain intact on edit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelInfo {
    name: [u8; NAME_SIZE],
    reserved: [u8; 4],
    bounds_radius: u32,
    minimum_extent: [u32; 3],
    maximum_extent: [u32; 3],
    blend_time: u32,
}

impl Default for ModelInfo {
    fn default() -> Self {
        Self {
            name: [0; NAME_SIZE],
            reserved: [0; 4],
            bounds_radius: 0,
            minimum_extent: [0; 3],
            maximum_extent: [0; 3],
            blend_time: 0,
        }
    }
}

impl ModelInfo {
    /// Creates a zero-initialized model record with a name.
    pub fn new(name: &str) -> Result<Self, Error> {
        let mut info = Self::default();
        info.set_name(name)?;
        Ok(info)
    }

    pub(crate) fn parse(data: &[u8]) -> Result<Self, Error> {
        Self::decode_one(&mut crate::Cursor::new(data), 0)
    }

    /// Returns the raw fixed-width record.
    pub fn as_bytes(&self) -> [u8; SIZE] {
        self.encode()
            .expect("fixed-size record")
            .try_into()
            .expect("fixed-size record")
    }

    /// Returns the model name up to the first NUL, replacing invalid UTF-8.
    pub fn name(&self) -> Cow<'_, str> {
        field::text(&self.name)
    }

    /// Sets the model name, clearing the rest of its fixed-width field.
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        field::set_text(&mut self.name, name)
    }

    /// Returns the model's bounding sphere radius.
    pub fn bounds_radius(&self) -> f32 {
        f32::from_bits(self.bounds_radius)
    }

    /// Sets the model's bounding sphere radius.
    pub fn set_bounds_radius(&mut self, radius: f32) {
        self.bounds_radius = radius.to_bits();
    }

    /// Returns the minimum XYZ extent.
    pub fn minimum_extent(&self) -> [f32; 3] {
        self.minimum_extent.map(f32::from_bits)
    }

    /// Sets the minimum XYZ extent.
    pub fn set_minimum_extent(&mut self, extent: [f32; 3]) {
        self.minimum_extent = extent.map(f32::to_bits);
    }

    /// Returns the maximum XYZ extent.
    pub fn maximum_extent(&self) -> [f32; 3] {
        self.maximum_extent.map(f32::from_bits)
    }

    /// Sets the maximum XYZ extent.
    pub fn set_maximum_extent(&mut self, extent: [f32; 3]) {
        self.maximum_extent = extent.map(f32::to_bits);
    }

    /// Returns the animation blend time in milliseconds.
    pub fn blend_time(&self) -> u32 {
        self.blend_time
    }

    /// Sets the animation blend time in milliseconds.
    pub fn set_blend_time(&mut self, time: u32) {
        self.blend_time = time;
    }
}

impl Model {
    /// Reads the first `MODL` record, if present.
    pub fn model_info(&self) -> Result<Option<ModelInfo>, Error> {
        self.chunks()
            .iter()
            .find(|chunk| chunk.tag() == ModelInfo::TAG)
            .map(|chunk| match chunk {
                crate::ModelChunk::ModelInfo(decoded) => Ok(decoded.info.clone()),
                crate::ModelChunk::Malformed(malformed) => Err(malformed.error.clone()),
                crate::ModelChunk::Unknown(raw) => ModelInfo::parse(&raw.data),
                _ => unreachable!("MODL tag matched another typed chunk"),
            })
            .transpose()
    }

    /// Replaces the first `MODL` record or appends one. Any extension bytes
    /// after the standard record are retained.
    pub fn set_model_info(&mut self, info: &ModelInfo) {
        if let Some(chunk) = self.chunk_mut(ModelInfo::TAG) {
            match chunk {
                crate::ModelChunk::ModelInfo(current) => current.info = info.clone(),
                _ => {
                    let raw = chunk.to_raw().expect("model info chunk can be encoded");
                    let extension = raw.data.get(SIZE..).unwrap_or_default().to_vec();
                    *chunk = crate::ModelChunk::ModelInfo(crate::ModelInfoChunk::new(
                        info.clone(),
                        extension,
                    ));
                }
            }
        } else {
            self.push(crate::ModelChunk::ModelInfo(crate::ModelInfoChunk::new(
                info.clone(),
                Vec::new(),
            )));
        }
    }
}

impl Record for ModelInfo {
    fn decode_one(cursor: &mut crate::Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let size = cursor.remaining().len();
        if size < SIZE {
            return Err(Error::MalformedChunk {
                tag: Self::TAG,
                size,
                expected: SIZE,
            });
        }
        let name = cursor
            .read_exact(NAME_SIZE)?
            .try_into()
            .expect("fixed-width name");
        let reserved = cursor.read_exact(4)?.try_into().expect("fixed-width field");
        let bounds_radius = cursor.read_u32()?;
        let mut minimum_extent = [0; 3];
        let mut maximum_extent = [0; 3];
        for value in &mut minimum_extent {
            *value = cursor.read_u32()?;
        }
        for value in &mut maximum_extent {
            *value = cursor.read_u32()?;
        }
        let blend_time = cursor.read_u32()?;
        Ok(Self {
            name,
            reserved,
            bounds_radius,
            minimum_extent,
            maximum_extent,
            blend_time,
        })
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = Vec::with_capacity(SIZE);
        bytes.extend_from_slice(&self.name);
        bytes.extend_from_slice(&self.reserved);
        bytes.extend_from_slice(&self.bounds_radius.to_le_bytes());
        for value in self.minimum_extent.into_iter().chain(self.maximum_extent) {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&self.blend_time.to_le_bytes());
        Ok(bytes)
    }
}

impl ModelInfo {
    /// The tag of the chunk containing this record.
    pub const TAG: [u8; 4] = *b"MODL";
}
