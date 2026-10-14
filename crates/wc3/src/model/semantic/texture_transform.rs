//! Shared, animated texture-coordinate transformations.
use crate::model::semantic::{
    Animated, Link, OwnerId, Property, Quaternion, TextureTransformId, ValidateLinks,
};
/// A shared texture-coordinate transform, independent of any surface or file table.
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
impl TextureTransformId {
    /// Animated coordinate translation of this shared transform.
    pub fn translation(self) -> Property<[f32; 3]> {
        Property::named(OwnerId::TextureTransform(self), "translation")
    }
    /// Animated coordinate rotation of this shared transform.
    pub fn rotation(self) -> Property<Quaternion> {
        Property::named(OwnerId::TextureTransform(self), "rotation")
    }
    /// Animated coordinate scale of this shared transform.
    pub fn scale(self) -> Property<[f32; 3]> {
        Property::named(OwnerId::TextureTransform(self), "scale")
    }
}
