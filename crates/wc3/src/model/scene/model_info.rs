//! Fixed-size `MODL` model information.
use crate::model::mdl;
use crate::model::mdl::{is_zero, WriteError};
use crate::model::mdx;
use crate::model::ModelVersion;
use crate::model::ValueError;
use crate::model::{Tag, Vec3};

use crate::model::{Cursor, ModelInfoChunk};

use std::borrow::Cow;

use crate::model::FixedText;
use crate::model::{Model, ReadError};

const SIZE: usize = 372;
const NAME_SIZE: usize = 80;
const ANIMATION_FILE_NAME_SIZE: usize = 260;

/// The 372-byte `MODL` record with separate name and animation-file fields.
#[derive(Clone, Debug, PartialEq, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(
    block = "Model",
    validate_write = "ModelInfo::validate_mdl_write",
    write_order(blend_time, minimum_extent, maximum_extent, bounds_radius)
)]
pub struct ModelInfo {
    #[mdl(header)]
    name: FixedText<NAME_SIZE>,
    #[mdl(skip, default)]
    animation_file_name: FixedText<ANIMATION_FILE_NAME_SIZE>,
    #[mdl(property = "BoundsRadius", default)]
    bounds_radius: f32,
    #[mdl(property = "MinimumExtent", default)]
    minimum_extent: Vec3,
    #[mdl(property = "MaximumExtent", default)]
    maximum_extent: Vec3,
    #[mdl(property = "BlendTime", default, skip_if = "is_zero")]
    blend_time: u32,
}

impl Default for ModelInfo {
    fn default() -> Self {
        Self {
            name: FixedText::default(),
            animation_file_name: FixedText::default(),
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

    /// Returns the model name up to the first NUL, replacing invalid UTF-8.
    pub fn name(&self) -> Cow<'_, str> {
        self.name.text()
    }

    /// Sets the model name, clearing the rest of its fixed-width field.
    pub fn set_name(&mut self, name: &str) -> Result<(), ValueError> {
        self.name.set_text(name)
    }

    /// Returns the animation-file path, replacing invalid UTF-8.
    pub fn animation_file_name(&self) -> Cow<'_, str> {
        self.animation_file_name.text()
    }

    /// Sets the animation-file path without changing the model name.
    pub fn set_animation_file_name(&mut self, path: &str) -> Result<(), ValueError> {
        self.animation_file_name.set_text(path)
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

impl<V: ModelVersion> Model<V> {
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

impl mdx::Read for ModelInfo {
    fn read_mdx(cursor: &mut Cursor<'_>) -> Result<Self, ReadError> {
        let size = cursor.remaining().len();
        if size < SIZE {
            return Err(ReadError::MalformedChunk {
                tag: Self::TAG,
                size,
                expected: SIZE,
            });
        }
        let name = cursor.read()?;
        let animation_file_name = cursor.read()?;
        let bounds_radius = cursor.read()?;
        let minimum_extent = cursor.read()?;
        let maximum_extent = cursor.read()?;
        let blend_time = cursor.read()?;
        Ok(Self {
            name,
            animation_file_name,
            bounds_radius,
            minimum_extent,
            maximum_extent,
            blend_time,
        })
    }
}

impl ModelInfo {
    /// The tag of the chunk containing this record.
    pub const TAG: Tag = *b"MODL";
}

impl ModelInfo {
    fn validate_mdl_write(&self) -> Result<(), WriteError> {
        if self
            .animation_file_name
            .as_bytes()
            .iter()
            .any(|&byte| byte != 0)
        {
            return Err(WriteError::Unsupported("model animation file name"));
        }
        Ok(())
    }
}
