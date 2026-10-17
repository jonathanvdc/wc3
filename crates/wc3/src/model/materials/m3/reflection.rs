//! Logical M3 reflection values.
use super::{Property, TextureLayer};
use crate::model::materials::contributions::SurfaceContribution;
use crate::model::values::programmed::ReflectionFlags;
use crate::model::Construct;

/// Editable M3 ReflectionMaterialStep values.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct ReflectionMaterialStep {
    /// The purpose of this field has not been established.
    pub unknown: u32,
    /// Controls reflection offset through its animation value.
    pub reflection_offset: Property<f32>,
    /// Controls blur angle through its animation value.
    pub blur_angle: Property<f32>,
    /// Controls max blur distance through its animation value.
    pub blur_distance_max: Property<f32>,
    /// Supplies the map controlling blur.
    pub blur_map: Option<TextureLayer>,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: ReflectionFlags,
    /// The meaning of the unknown2 storage has not been established; its encoded bits are preserved.
    pub unknown2: u32,
    /// Authored reflection input and response.
    pub reflection: SurfaceContribution<Option<TextureLayer>, Property<f32>>,

    /// Authored displacement input and response.
    pub displacement: SurfaceContribution<Option<TextureLayer>, Property<f32>>,
}
