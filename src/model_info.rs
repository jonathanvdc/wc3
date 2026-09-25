//! Fixed-size `MODL` model information.

use std::borrow::Cow;

use crate::utils::field;
use crate::{Error, Model};

const TAG: [u8; 4] = *b"MODL";
const SIZE: usize = 372;
const NAME_SIZE: usize = 336;

/// The 372-byte `MODL` record. Reserved bytes remain intact on edit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelInfo {
    bytes: [u8; SIZE],
}

impl Default for ModelInfo {
    fn default() -> Self {
        Self { bytes: [0; SIZE] }
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
        let bytes = data.get(..SIZE).ok_or(Error::MalformedChunk {
            tag: TAG,
            size: data.len(),
            expected: SIZE,
        })?;
        Ok(Self {
            bytes: bytes.try_into().expect("fixed-size record"),
        })
    }

    /// Returns the raw fixed-width record.
    pub fn as_bytes(&self) -> &[u8; SIZE] {
        &self.bytes
    }

    /// Returns the model name up to the first NUL, replacing invalid UTF-8.
    pub fn name(&self) -> Cow<'_, str> {
        field::text(&self.bytes[..NAME_SIZE])
    }

    /// Sets the model name, clearing the rest of its fixed-width field.
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        field::set_text(&mut self.bytes[..NAME_SIZE], name)
    }

    /// Returns the model's bounding sphere radius.
    pub fn bounds_radius(&self) -> f32 {
        f32::from_le_bytes(self.bytes[340..344].try_into().expect("fixed-size field"))
    }

    /// Sets the model's bounding sphere radius.
    pub fn set_bounds_radius(&mut self, radius: f32) {
        self.bytes[340..344].copy_from_slice(&radius.to_le_bytes());
    }

    /// Returns the minimum XYZ extent.
    pub fn minimum_extent(&self) -> [f32; 3] {
        self.extent_at(344)
    }

    /// Sets the minimum XYZ extent.
    pub fn set_minimum_extent(&mut self, extent: [f32; 3]) {
        self.set_extent_at(344, extent);
    }

    /// Returns the maximum XYZ extent.
    pub fn maximum_extent(&self) -> [f32; 3] {
        self.extent_at(356)
    }

    /// Sets the maximum XYZ extent.
    pub fn set_maximum_extent(&mut self, extent: [f32; 3]) {
        self.set_extent_at(356, extent);
    }

    /// Returns the animation blend time in milliseconds.
    pub fn blend_time(&self) -> u32 {
        u32::from_le_bytes(self.bytes[368..372].try_into().expect("fixed-size field"))
    }

    /// Sets the animation blend time in milliseconds.
    pub fn set_blend_time(&mut self, time: u32) {
        self.bytes[368..372].copy_from_slice(&time.to_le_bytes());
    }

    fn extent_at(&self, start: usize) -> [f32; 3] {
        std::array::from_fn(|index| {
            let offset = start + index * 4;
            f32::from_le_bytes(
                self.bytes[offset..offset + 4]
                    .try_into()
                    .expect("fixed-size field"),
            )
        })
    }

    fn set_extent_at(&mut self, start: usize, extent: [f32; 3]) {
        for (index, value) in extent.into_iter().enumerate() {
            let offset = start + index * 4;
            self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
    }
}

impl Model {
    /// Reads the first `MODL` record, if present.
    pub fn model_info(&self) -> Result<Option<ModelInfo>, Error> {
        self.chunk(TAG)
            .map(|chunk| ModelInfo::parse(&chunk.data))
            .transpose()
    }

    /// Replaces the first `MODL` record or appends one. Any extension bytes
    /// after the standard record are retained.
    pub fn set_model_info(&mut self, info: &ModelInfo) {
        if let Some(chunk) = self.chunk_mut(TAG) {
            if chunk.data.len() >= SIZE {
                chunk.data[..SIZE].copy_from_slice(info.as_bytes());
            } else {
                chunk.data = info.as_bytes().to_vec();
            }
        } else {
            self.push(crate::Chunk::new(TAG, info.as_bytes().to_vec()));
        }
    }
}
