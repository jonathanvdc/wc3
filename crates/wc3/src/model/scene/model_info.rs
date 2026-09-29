//! Model name, bounds, and animation blending settings.
use crate::model::mdl;
use crate::model::mdl::is_zero;
use crate::model::mdx;
use crate::model::FixedText;
use crate::model::Model;
use crate::model::ModelVersion;
use crate::model::ValueError;
use crate::model::{Cursor, ModelInfoChunk};
use crate::model::{Tag, Vec3};

const SIZE: usize = 372;
const NAME_SIZE: usize = 80;
const ANIMATION_FILE_NAME_SIZE: usize = 260;

/// The model name, overall bounds, and animation cross-fade duration.
#[derive(Clone, Debug, PartialEq, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(
    block = "Model",
    validate_write = "ModelInfo::validate_mdl_write",
    write_order(blend_time, minimum_extent, maximum_extent, bounds_radius)
)]
pub struct ModelInfo {
    #[mdl(header)]
    /// Display name of the model.
    pub name: FixedText<NAME_SIZE>,
    #[mdl(skip, default)]
    /// External animation resource path. Nonempty binary data here cannot be exported to MDL.
    pub animation_file_name: FixedText<ANIMATION_FILE_NAME_SIZE>,
    #[mdl(property = "BoundsRadius", default)]
    /// Model's bounding sphere radius.
    pub bounds_radius: f32,
    #[mdl(property = "MinimumExtent", default)]
    /// Minimum XYZ extent.
    pub minimum_extent: Vec3,
    #[mdl(property = "MaximumExtent", default)]
    /// Maximum XYZ extent.
    pub maximum_extent: Vec3,
    #[mdl(property = "BlendTime", default, skip_if = "is_zero")]
    /// Animation blend time in milliseconds.
    pub blend_time: u32,
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
    /// Creates model information with the supplied name and zero bounds and blend time.
    pub fn new(name: &str) -> Result<Self, ValueError> {
        let mut info = Self::default();
        info.name.set_text(name)?;
        Ok(info)
    }
}

impl<V: ModelVersion> Model<V> {
    /// Returns an owned copy of the model information, if present.
    pub fn model_info(&self) -> Option<ModelInfo> {
        self.decoded_chunks::<ModelInfoChunk>()
            .next()
            .map(|decoded| decoded.info.clone())
    }

    /// Replaces the model information, creating it if absent.
    /// Any additional model-information chunks and their extensions are removed.
    pub fn set_model_info(&mut self, info: &ModelInfo) {
        self.replace_chunk(ModelInfoChunk::new(info.clone(), Vec::new()));
    }
}

impl mdx::Read for ModelInfo {
    fn read_mdx(cursor: &mut Cursor<'_>) -> Result<Self, mdx::ReadError> {
        let size = cursor.remaining().len();
        if size < SIZE {
            return Err(mdx::ReadError::new(
                cursor.absolute_position(),
                mdx::ReadErrorKind::UnexpectedEnd {
                    needed: SIZE,
                    remaining: size,
                },
            )
            .with_tag(Self::TAG));
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
    fn validate_mdl_write(&self) -> Result<(), mdl::WriteError> {
        if self
            .animation_file_name
            .as_bytes()
            .iter()
            .any(|&byte| byte != 0)
        {
            return Err(mdl::WriteError::Unrepresentable {
                field: "model animation file name",
            });
        }
        Ok(())
    }
}
