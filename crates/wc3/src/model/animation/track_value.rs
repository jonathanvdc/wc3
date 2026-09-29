//! Scalar and vector values supported by animation tracks.
use crate::model::mdx;
use crate::model::{Quaternion, Vec3};
use std::fmt::Debug;

/// A scalar or vector value supported by MDX keyframe tracks.
pub trait TrackValue: mdx::Read + mdx::Write + Copy + Default + PartialEq + Debug {
    const COMPONENTS: usize;
}

impl TrackValue for f32 {
    const COMPONENTS: usize = 1;
}
impl TrackValue for u32 {
    const COMPONENTS: usize = 1;
}
impl TrackValue for Vec3 {
    const COMPONENTS: usize = 3;
}
impl TrackValue for Quaternion {
    const COMPONENTS: usize = 4;
}
