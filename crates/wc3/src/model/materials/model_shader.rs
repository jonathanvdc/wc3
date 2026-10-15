//! World of Warcraft shader inputs and draw-state semantics.
use super::bindings::{Coordinates, ImageSelection, TextureBinding};
use super::state::{DepthState, Lighting, Operation, PassState, Rasterization};
use crate::model::profiles::m2::properties::{Target, TextureUnitField};
use crate::model::profiles::SurfacePayload;
use crate::model::values::model::{BatchFlags, BlendMode, MaterialFlags, TextureSource};
use crate::model::{Animated, Error, Link, PropertyIdentityKey, PropertyPayload, ValidateLinks};
use std::{borrow::Cow, fmt::Debug};

pub use crate::model::TextureTransform;
/// Input-local legacy texture-frame playback.
#[derive(Clone, Debug, Default)]
pub struct FrameSampling {
    /// Animated legacy texture frame selection.
    pub frame: Animated<u16>,
}
impl ValidateLinks for FrameSampling {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.frame.visit_links(visitor);
    }
}
/// One shader input composed from independently selected coordinate behavior.
pub type TextureUnit<T = TextureTransform> = TextureBinding<
    ImageSelection<TextureSource>,
    Coordinates<i16, T>,
    FrameSampling,
    Animated<f32>,
>;
/// Complete logical shader pass. Table addresses are replaced with graph bindings.
#[derive(Clone, Debug)]
pub struct ShaderPass<'a, I = TextureUnit> {
    /// Shader family identifier interpreted by the game shader implementation.
    pub shader: u16,
    /// Geometry classification used by this draw.
    pub geoset: u16,
    /// Layer order of this pass within its geometry section.
    pub layer: u16,
    /// Established draw behavior and retained extension bits.
    pub batch_flags: BatchFlags,
    /// Color composition, rasterization, and authored pass ordering.
    pub state: PassState<BlendMode, ModelRasterization, i8>,
    /// Animated surface color multiplier.
    pub color: Animated<[f32; 3]>,
    /// Animated surface opacity multiplier.
    pub opacity: Animated<f32>,
    /// Ordered shader texture inputs.
    pub textures: Vec<I>,
    /// Optional user-authored shader label.
    pub label: Cow<'a, str>,
}
impl<I: ValidateLinks> ValidateLinks for ShaderPass<'_, I> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.state.visit_links(v);
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
                    .coordinates
                    .transform
                    .supports_property(CoordinateProperty::Translation),
                TextureUnitField::Rotation => self
                    .coordinates
                    .transform
                    .supports_property(CoordinateProperty::Rotation),
                TextureUnitField::Scale => self
                    .coordinates
                    .transform
                    .supports_property(CoordinateProperty::Scale),
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
            geoset: 0,
            layer: 0,
            batch_flags: BatchFlags::default(),
            state: PassState::default(),
            color: Animated::default(),
            opacity: Animated::default(),
            textures: Vec::new(),
            label: Cow::Borrowed(""),
        }
    }
}

/// Model shaders select common depth, face, and lighting operations with retained extension flags.
pub type ModelRasterization =
    Rasterization<DepthState<Operation, Operation>, bool, Lighting, MaterialFlags>;
/// Separates established model render flags from extension controls.
pub fn rasterization(flags: MaterialFlags) -> ModelRasterization {
    let mut controls = flags;
    controls.set_unlit(false);
    controls.set_unfogged(false);
    controls.set_two_sided(false);
    controls.set_no_depth_test(false);
    controls.set_no_depth_write(false);
    Rasterization {
        depth: DepthState {
            test: Operation::bypassed(flags.no_depth_test()),
            write: Operation::bypassed(flags.no_depth_write()),
            bias: (),
        },
        faces: flags.two_sided(),
        lighting: Lighting {
            unlit: flags.unlit(),
            unfogged: flags.unfogged(),
        },
        controls,
    }
}
/// Reconstructs the selected model flag domain, preserving unresolved controls.
pub fn material_flags(state: &ModelRasterization) -> MaterialFlags {
    let mut flags = state.controls;
    flags.set_unlit(state.lighting.unlit);
    flags.set_unfogged(state.lighting.unfogged);
    flags.set_two_sided(state.faces);
    flags.set_no_depth_test(state.depth.test.is_bypassed());
    flags.set_no_depth_write(state.depth.write.is_bypassed());
    flags
}

impl ValidateLinks for MaterialFlags {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}

impl ValidateLinks for TextureSource {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
