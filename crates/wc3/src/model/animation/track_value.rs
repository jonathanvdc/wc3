//! Scalar and vector values supported by animation tracks.
use crate::model::mdx;
use crate::model::{Quaternion, Vec3};
use std::fmt::Debug;
use zerovec::ule::AsULE;

/// A scalar or vector value supported by MDX keyframe tracks.
/// Its [`AsULE`] representation must match its fixed-width MDX encoding.
pub trait TrackValue:
    AsULE + 'static + for<'data> mdx::Read<'data> + mdx::Write + Copy + Default + PartialEq + Debug
{
    /// Number of scalar components serialized for each value.
    const COMPONENTS: usize;
    /// Whether every bit pattern in the fixed-width wire value is valid.
    /// Keep the default for custom values with semantic validation; ordinary
    /// integers and floats can skip per-element validation when loading arrays.
    const ALL_BITS_VALID: bool = false;
}

impl TrackValue for f32 {
    const ALL_BITS_VALID: bool = true;
    const COMPONENTS: usize = 1;
}
impl TrackValue for u32 {
    const ALL_BITS_VALID: bool = true;
    const COMPONENTS: usize = 1;
}
impl TrackValue for Vec3 {
    const ALL_BITS_VALID: bool = true;
    const COMPONENTS: usize = 3;
}
impl TrackValue for Quaternion {
    const ALL_BITS_VALID: bool = true;
    const COMPONENTS: usize = 4;
}
