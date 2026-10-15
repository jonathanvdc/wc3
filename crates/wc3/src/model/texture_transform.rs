//! Shared, animated texture-coordinate transformations.
use crate::model::profiles::Profile;
use crate::model::{
    Animated, Link, Property, PropertyTarget, Quaternion, TextureTransformId, ValidateLinks,
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
    pub fn translation<P: Profile>(self) -> Property<[f32; 3], P> {
        Property::new(PropertyTarget::TextureTranslation(self))
    }
    /// Animated coordinate rotation of this shared transform.
    pub fn rotation<P: Profile>(self) -> Property<Quaternion, P> {
        Property::new(PropertyTarget::TextureRotation(self))
    }
    /// Animated coordinate scale of this shared transform.
    pub fn scale<P: Profile>(self) -> Property<[f32; 3], P> {
        Property::new(PropertyTarget::TextureScale(self))
    }
}
