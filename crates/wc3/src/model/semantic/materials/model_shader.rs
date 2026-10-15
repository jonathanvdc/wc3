//! World of Warcraft shader inputs and draw-state semantics.
use crate::model::semantic::profiles::m2::properties::{Target, TextureUnitField};
use crate::model::semantic::profiles::SurfacePayload;
use crate::model::semantic::values::model::{BatchFlags, BlendMode, MaterialFlags, TextureSource};
use crate::model::semantic::{
    Animated, Error, Link, PropertyIdentityKey, PropertyPayload, TextureId, ValidateLinks,
};
use std::{borrow::Cow, fmt::Debug};

pub use crate::model::semantic::TextureTransform;
/// One shader input, with logical resource and animation bindings.
#[derive(Clone, Debug)]
pub struct TextureUnit<T = TextureTransform> {
    /// Runtime texture source category.
    pub source: TextureSource,
    /// Referenced texture resource, when supplied by the asset.
    pub texture: Option<TextureId>,
    /// Texture-coordinate set; negative values select generated coordinates.
    pub coordinate_set: i16,
    /// Animated input opacity.
    pub weight: Animated<f32>,
    /// Texture-coordinate transform.
    pub transform: T,
    /// Legacy texture frame selection.
    pub frame: Animated<u16>,
}
/// Complete logical shader pass. Table addresses are replaced with graph bindings.
#[derive(Clone, Debug)]
pub struct ShaderPass<'a, I = TextureUnit> {
    /// Shader family identifier interpreted by the game shader implementation.
    pub shader: u16,
    /// Shader pass ordering plane.
    pub priority_plane: i8,
    /// Geometry classification used by this draw.
    pub geoset: u16,
    /// Layer order of this pass within its geometry section.
    pub layer: u16,
    /// Established draw behavior and retained extension bits.
    pub batch_flags: BatchFlags,
    /// Native-family render behavior.
    pub flags: MaterialFlags,
    /// Native-family blend semantics, including unknown choices.
    pub blend: BlendMode,
    /// Animated surface color multiplier.
    pub color: Animated<[f32; 3]>,
    /// Animated surface opacity multiplier.
    pub opacity: Animated<f32>,
    /// Ordered shader texture inputs.
    pub textures: Vec<I>,
    /// Optional user-authored shader label.
    pub label: Cow<'a, str>,
}
impl<T: ValidateLinks> ValidateLinks for TextureUnit<T> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.texture.visit_links(v);
        self.weight.visit_links(v);
        self.transform.visit_links(v);
        self.frame.visit_links(v)
    }
}
impl<I: ValidateLinks> ValidateLinks for ShaderPass<'_, I> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.color.visit_links(v);
        self.opacity.visit_links(v);
        self.textures.visit_links(v)
    }
}

/// A complete independently animated shader-input vocabulary.
pub trait ShaderInput: Clone + Debug + Send + Sync + ValidateLinks {
    /// Whether this input exposes a structural destination at its ordinal.
    fn supports_property(&self, _key: &PropertyIdentityKey, _ordinal: usize) -> bool {
        false
    }
    /// Checks input values before graph publication.
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}
/// An animated destination supplied by a texture-coordinate representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoordinateProperty {
    /// Coordinate translation.
    Translation,
    /// Coordinate rotation.
    Rotation,
    /// Coordinate scaling.
    Scale,
}
/// Coordinate behavior admitted by a shader input's selected transform type.
pub trait ShaderCoordinates: Clone + Debug + Send + Sync + ValidateLinks {
    /// Whether this coordinate form exposes the requested animated destination.
    fn supports_property(&self, property: CoordinateProperty) -> bool;
}
impl ShaderCoordinates for TextureTransform {
    fn supports_property(&self, _property: CoordinateProperty) -> bool {
        true
    }
}
impl ShaderCoordinates for () {
    fn supports_property(&self, _property: CoordinateProperty) -> bool {
        false
    }
}
impl<T: ShaderCoordinates> ShaderInput for TextureUnit<T> {
    fn supports_property(&self, key: &PropertyIdentityKey, ordinal: usize) -> bool {
        match Target::from_identity(key) {
            Some(Target::TextureUnit { index, field }) if index == ordinal => match field {
                TextureUnitField::Weight => true,
                TextureUnitField::Translation => self
                    .transform
                    .supports_property(CoordinateProperty::Translation),
                TextureUnitField::Rotation => self
                    .transform
                    .supports_property(CoordinateProperty::Rotation),
                TextureUnitField::Scale => {
                    self.transform.supports_property(CoordinateProperty::Scale)
                }
            },
            _ => false,
        }
    }
}
impl<I: ShaderInput> SurfacePayload for ShaderPass<'_, I> {
    fn supports_property(&self, key: &PropertyIdentityKey) -> bool {
        matches!(
            Target::from_identity(key),
            Some(Target::Color | Target::Opacity)
        ) || self
            .textures
            .iter()
            .enumerate()
            .any(|(ordinal, input)| input.supports_property(key, ordinal))
    }
    fn validate(&self) -> Result<(), Error> {
        self.textures.iter().try_for_each(ShaderInput::validate)
    }
}

impl<I> Default for ShaderPass<'_, I> {
    fn default() -> Self {
        Self {
            shader: 0,
            priority_plane: 0,
            geoset: 0,
            layer: 0,
            batch_flags: BatchFlags::default(),
            flags: MaterialFlags::default(),
            blend: BlendMode::default(),
            color: Animated::default(),
            opacity: Animated::default(),
            textures: Vec::new(),
            label: Cow::Borrowed(""),
        }
    }
}
