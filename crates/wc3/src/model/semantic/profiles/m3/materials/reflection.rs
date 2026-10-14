//! Logical M3 reflection values.
use super::super::values::ReflectionFlags;
use super::{Property, TextureLayer};

/// Editable M3 Reflection values.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reflection {
    /// The purpose of this field has not been established.
    pub unknown: u32,
    /// Controls reflection strength through its animation value.
    pub reflection_strength: Property<f32>,
    /// Controls displacement strength through its animation value.
    pub displacement_strength: Property<f32>,
    /// Controls reflection offset through its animation value.
    pub reflection_offset: Property<f32>,
    /// Controls blur angle through its animation value.
    pub blur_angle: Property<f32>,
    /// Controls max blur distance through its animation value.
    pub blur_distance_max: Property<f32>,
    /// Supplies the map supplying reflected color.
    pub reflection_map: Option<TextureLayer>,
    /// Supplies the map defining displacement.
    pub displacement_map: Option<TextureLayer>,
    /// Supplies the map controlling blur.
    pub blur_map: Option<TextureLayer>,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: ReflectionFlags,
    /// The meaning of the unknown2 storage has not been established; its encoded bits are preserved.
    pub unknown2: u32,
}
