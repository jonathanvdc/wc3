//! Input-owned, animated texture-coordinate transformations.
use crate::model::{Construct, PropertyValues};
use crate::model::{Link, Property, Quaternion, ValidateLinks};
/// An animated coordinate transform owned by one material input.
#[derive(Clone, Debug, PartialEq)]
pub struct TextureTransform {
    /// Translation in texture-coordinate space.
    pub translation: Property<[f32; 3]>,
    /// Rotation about the texture-coordinate origin.
    pub rotation: Property<Quaternion>,
    /// Coordinate scaling before sampling.
    pub scale: Property<[f32; 3]>,
}
impl Construct for TextureTransform {
    fn construct(values: &mut PropertyValues) -> Self {
        Self {
            translation: values.add([0.; 3]),
            rotation: values.add(Quaternion::IDENTITY),
            scale: values.add([1.; 3]),
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
