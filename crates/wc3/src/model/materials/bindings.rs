//! Independently selected image, coordinates, sampling, and shader contribution.
use crate::model::common::Detached;
use crate::model::{Construct, Property, PropertyValue, PropertyValues};
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
    /// PropertyValues controlling this input's contribution to the surface.
    pub contribution: V,
}

/// A coordinate source paired with a selected transform representation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Coordinates<S, T = ()> {
    /// Mesh coordinate set or a typed generated-coordinate selector.
    pub source: S,
    /// Selected coordinate transform.
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

impl<'owned, I, C, S, V> Detached<'owned> for TextureBinding<I, C, S, V>
where
    I: Detached<'owned>,
    C: Detached<'owned>,
    S: Detached<'owned>,
    V: Detached<'owned>,
{
    type Detached = TextureBinding<I::Detached, C::Detached, S::Detached, V::Detached>;
    fn into_detached(self) -> Self::Detached {
        TextureBinding {
            image: self.image.into_detached(),
            coordinates: self.coordinates.into_detached(),
            sampling: self.sampling.into_detached(),
            contribution: self.contribution.into_detached(),
        }
    }
}
impl<'owned, S: Detached<'owned>, T: Detached<'owned>> Detached<'owned> for Coordinates<S, T> {
    type Detached = Coordinates<S::Detached, T::Detached>;
    fn into_detached(self) -> Self::Detached {
        Coordinates {
            source: self.source.into_detached(),
            transform: self.transform.into_detached(),
        }
    }
}
impl<'owned, S: Detached<'owned>> Detached<'owned> for ImageSelection<S> {
    type Detached = ImageSelection<S::Detached>;
    fn into_detached(self) -> Self::Detached {
        ImageSelection {
            source: self.source.into_detached(),
            texture: self.texture,
        }
    }
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

impl<T: PropertyValue + Default, C: Construct, S: Construct, V: Construct> Construct
    for TextureBinding<Property<T>, C, S, V>
{
    fn construct(values: &mut PropertyValues) -> Self {
        Self {
            image: values.add(T::default()),
            coordinates: C::construct(values),
            sampling: S::construct(values),
            contribution: V::construct(values),
        }
    }
}
impl<S: Default> Construct for Coordinates<S, crate::model::TextureTransform> {
    fn construct(values: &mut PropertyValues) -> Self {
        Self {
            source: S::default(),
            transform: crate::model::TextureTransform::construct(values),
        }
    }
}
