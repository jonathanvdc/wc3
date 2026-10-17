//! Input-owned, animated texture-coordinate transformations.
use crate::model::{Animated, Link, Quaternion, ValidateLinks};
/// An animated coordinate transform owned by one material input.
#[derive(Clone, Debug, PartialEq)]
pub struct TextureTransform {
    /// Translation in texture-coordinate space.
    pub translation: Animated<[f32; 3]>,
    /// Rotation about the texture-coordinate origin.
    pub rotation: Animated<Quaternion>,
    /// Coordinate scaling before sampling.
    pub scale: Animated<[f32; 3]>,
}
impl Default for TextureTransform {
    fn default() -> Self {
        Self {
            translation: Animated::constant([0.; 3]),
            rotation: Animated::constant(Quaternion::IDENTITY),
            scale: Animated::constant([1.; 3]),
        }
    }
}
impl ValidateLinks for TextureTransform {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.translation.visit_links(visitor);
        self.rotation.visit_links(visitor);
        self.scale.visit_links(visitor);
    }
}
