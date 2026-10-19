//! Input-owned, animated texture-coordinate transformations.
use crate::model::{Construct, PropertyValues};
use crate::model::{Link, Property, Quat, ValidateLinks};
use glam::Vec3;
/// An animated coordinate transform owned by one material input.
#[derive(Clone, Debug, PartialEq)]
pub struct TextureTransform {
    /// Translation in texture-coordinate space.
    pub translation: Property<Vec3>,
    /// Rotation about the texture-coordinate origin.
    pub rotation: Property<Quat>,
    /// Coordinate scaling before sampling.
    pub scale: Property<Vec3>,
}
impl Construct for TextureTransform {
    fn construct(values: &mut PropertyValues) -> Self {
        Self {
            translation: values.add(Vec3::from_array([0.; 3])),
            rotation: values.add(Quat::IDENTITY),
            scale: values.add(Vec3::from_array([1.; 3])),
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
