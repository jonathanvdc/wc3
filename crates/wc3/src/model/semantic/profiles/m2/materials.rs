//! World of Warcraft shader inputs and draw-state semantics.
use super::values::{BatchFlags, BlendMode, MaterialFlags, TextureSource};
use crate::model::semantic::{Animated, Link, Quaternion, TextureId, ValidateLinks};
use std::borrow::Cow;

/// A logical texture-coordinate transform shared by shader inputs.
#[derive(Clone, Debug)]
pub struct TextureTransform {
    /// Translation in texture coordinates.
    pub translation: Animated<[f32; 3]>,
    /// Rotation quaternion in x/y/z/w order.
    pub rotation: Animated<Quaternion>,
    /// Texture-coordinate scale.
    pub scale: Animated<[f32; 3]>,
}
impl Default for TextureTransform {
    fn default() -> Self {
        Self {
            translation: Animated::constant([0.; 3]),
            rotation: Animated::constant(Quaternion::default()),
            scale: Animated::constant([1.; 3]),
        }
    }
}
/// One shader input, with logical resource and animation bindings.
#[derive(Clone, Debug)]
pub struct TextureUnit {
    /// Runtime texture source category.
    pub source: TextureSource,
    /// Referenced texture resource, when supplied by the asset.
    pub texture: Option<TextureId>,
    /// Texture-coordinate set; negative values select generated coordinates.
    pub coordinate_set: i16,
    /// Animated input opacity.
    pub weight: Animated<f32>,
    /// Texture-coordinate transform.
    pub transform: TextureTransform,
    /// Legacy texture frame selection.
    pub frame: Animated<u16>,
}
/// Complete logical shader pass. Table addresses are replaced with graph bindings.
#[derive(Clone, Debug, Default)]
pub struct Material<'a> {
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
    pub textures: Vec<TextureUnit>,
    /// Optional user-authored shader label.
    pub label: Cow<'a, str>,
}
impl ValidateLinks for TextureTransform {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.translation.visit_links(v);
        self.rotation.visit_links(v);
        self.scale.visit_links(v)
    }
}
impl ValidateLinks for TextureUnit {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.texture.visit_links(v);
        self.weight.visit_links(v);
        self.transform.visit_links(v);
        self.frame.visit_links(v)
    }
}
impl ValidateLinks for Material<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.color.visit_links(v);
        self.opacity.visit_links(v);
        self.textures.visit_links(v)
    }
}
