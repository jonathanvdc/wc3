//! Independently selected image, coordinates, sampling, and shader contribution.
use crate::model::{Animatable, Animated, Construct, Values};
use crate::model::{Link, Texture, ValidateLinks};

/// One image input, independently composed from its coordinate and shader behavior.
///
/// A binding supplies a shader input; it does not imply an additional draw pass.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureBinding<I, C = (), S = (), V = ()> {
    /// Image selection, including any selected resource namespace.
    pub image: I,
    /// Coordinate generation and transformation.
    pub coordinates: C,
    /// Input-local sampling and playback choices.
    pub sampling: S,
    /// Values controlling this input's contribution to the surface.
    pub contribution: V,
}

/// A coordinate source paired with a selected transform representation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Coordinates<S, T = ()> {
    /// Mesh coordinate set or a typed generated-coordinate selector.
    pub source: S,
    /// Inline, referenced, or programmed coordinate transform.
    pub transform: T,
}

/// Static sampled-image selection; absence remains an authored empty slot.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImageSelection<S = ()> {
    /// Runtime resource-source category, when required by the shader family.
    pub source: S,
    /// Complete sampled-image value owned by this input.
    pub texture: Option<Texture>,
}

impl<I: ValidateLinks, C: ValidateLinks, S: ValidateLinks, V: ValidateLinks> ValidateLinks
    for TextureBinding<I, C, S, V>
{
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.image.visit_links(visitor);
        self.coordinates.visit_links(visitor);
        self.sampling.visit_links(visitor);
        self.contribution.visit_links(visitor);
    }
}
impl<S: ValidateLinks, T: ValidateLinks> ValidateLinks for Coordinates<S, T> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.source.visit_links(visitor);
        self.transform.visit_links(visitor);
    }
}
impl<S: ValidateLinks> ValidateLinks for ImageSelection<S> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.source.visit_links(visitor);
        self.texture.visit_links(visitor);
    }
}

impl<T: Animatable + Default, C: Construct, S: Construct, V: Construct> Construct
    for TextureBinding<Animated<T>, C, S, V>
{
    fn construct(values: &mut Values) -> Self {
        Self {
            image: values.add(T::default()),
            coordinates: C::construct(values),
            sampling: S::construct(values),
            contribution: V::construct(values),
        }
    }
}
impl<S: Default> Construct for Coordinates<S, crate::model::TextureTransform> {
    fn construct(values: &mut Values) -> Self {
        Self {
            source: S::default(),
            transform: crate::model::TextureTransform::construct(values),
        }
    }
}
