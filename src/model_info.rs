//! Fixed-size `MODL` model information.
use crate::EncodeError;
use crate::Encoder;
use crate::ValueError;
use crate::{Tag, Vec3};

use crate::{Cursor, ModelInfoChunk};
use crate::{Decodable, Encodable};
use std::borrow::Cow;

use crate::utils::field;
use crate::{DecodeError, Model};

const SIZE: usize = 372;
const NAME_SIZE: usize = 336;

/// The 372-byte `MODL` record. Reserved bytes remain intact on edit.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelInfo {
    name: [u8; NAME_SIZE],
    reserved: Tag,
    bounds_radius: f32,
    minimum_extent: Vec3,
    maximum_extent: Vec3,
    blend_time: u32,
}

impl Default for ModelInfo {
    fn default() -> Self {
        Self {
            name: [0; NAME_SIZE],
            reserved: [0; 4],
            bounds_radius: 0.0,
            minimum_extent: [0.0; 3],
            maximum_extent: [0.0; 3],
            blend_time: 0,
        }
    }
}

impl ModelInfo {
    /// Creates a zero-initialized model record with a name.
    pub fn new(name: &str) -> Result<Self, ValueError> {
        let mut info = Self::default();
        info.set_name(name)?;
        Ok(info)
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
    pub fn set_name(&mut self, name: &str) -> Result<(), ValueError> {
        field::set_text(&mut self.name, name)
    }

    /// Returns the model's bounding sphere radius.
    pub fn bounds_radius(&self) -> f32 {
        self.bounds_radius
    }

    /// Sets the model's bounding sphere radius.
    pub fn set_bounds_radius(&mut self, radius: f32) {
        self.bounds_radius = radius;
    }

    /// Returns the minimum XYZ extent.
    pub fn minimum_extent(&self) -> Vec3 {
        self.minimum_extent
    }

    /// Sets the minimum XYZ extent.
    pub fn set_minimum_extent(&mut self, extent: Vec3) {
        self.minimum_extent = extent;
    }

    /// Returns the maximum XYZ extent.
    pub fn maximum_extent(&self) -> Vec3 {
        self.maximum_extent
    }

    /// Sets the maximum XYZ extent.
    pub fn set_maximum_extent(&mut self, extent: Vec3) {
        self.maximum_extent = extent;
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
    /// Returns the first decoded `MODL` record, if present.
    pub fn model_info(&self) -> Option<ModelInfo> {
        self.decoded_chunks::<ModelInfoChunk>()
            .next()
            .map(|decoded| decoded.info.clone())
    }

    /// Replaces all `MODL` chunks with one decoded chunk at the first one's
    /// position, or appends one if none exists.
    pub fn set_model_info(&mut self, info: &ModelInfo) {
        self.replace_chunk(ModelInfoChunk::new(info.clone(), Vec::new()));
    }
}

impl Decodable for ModelInfo {
    fn decode_one(cursor: &mut Cursor<'_>, _version: u32) -> Result<Self, DecodeError> {
        let size = cursor.remaining().len();
        if size < SIZE {
            return Err(DecodeError::MalformedChunk {
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
        let bounds_radius = cursor.read()?;
        let minimum_extent = cursor.read()?;
        let maximum_extent = cursor.read()?;
        let blend_time = cursor.read()?;
        Ok(Self {
            name,
            reserved,
            bounds_radius,
            minimum_extent,
            maximum_extent,
            blend_time,
        })
    }
}

impl Encodable for ModelInfo {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        bytes.write_bytes(&self.name);
        bytes.write_bytes(&self.reserved);
        bytes.write(self.bounds_radius);
        bytes.write(self.minimum_extent);
        bytes.write(self.maximum_extent);
        bytes.write(self.blend_time);
        Ok(())
    }
}

impl ModelInfo {
    /// The tag of the chunk containing this record.
    pub const TAG: Tag = *b"MODL";
}
