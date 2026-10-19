//! World of Warcraft shader inputs and draw-state semantics.
use super::bindings::{Coordinates, ImageSelection, TextureBinding};
use super::state::{DepthState, Lighting, Operation, Rasterization};
use crate::model::values::model::{MaterialFlags, TextureSource};
use crate::model::{Construct, PropertyValues};
use crate::model::{Error, Link, Property, ValidateLinks};
use glam::Vec3;
use std::fmt::Debug;

pub use crate::model::TextureTransform;
/// Input-local legacy texture-frame playback.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct FrameSampling {
    /// Animated legacy texture frame selection.
    pub frame: Property<u16>,
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
    Property<f32>,
>;
/// Application-supplied linear RGB multipliers for legacy guild programmes.
///
/// These colours are external presentation settings, not values authored in an
/// M2 file. Native imports use white; native export cannot retain a changed palette.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GuildPalette {
    /// Cloth background multiplier.
    pub background: Vec3,
    /// Emblem multiplier, blended by the emblem texture alpha.
    pub emblem: Vec3,
    /// Border multiplier, blended by the border texture alpha.
    pub border: Vec3,
}
impl GuildPalette {
    /// Whether all linear RGB multipliers are finite.
    pub fn is_finite(self) -> bool {
        self.background
            .to_array()
            .iter()
            .chain(&self.emblem.to_array())
            .chain(&self.border.to_array())
            .all(|value| value.is_finite())
    }
}
impl Default for GuildPalette {
    fn default() -> Self {
        Self {
            background: Vec3::from_array([1.; 3]),
            emblem: Vec3::from_array([1.; 3]),
            border: Vec3::from_array([1.; 3]),
        }
    }
}
/// A complete independently animated shader-input vocabulary.
pub trait ShaderInput: Clone + Debug + Send + Sync + ValidateLinks {
    /// Checks input values before graph publication.
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

impl<T: Clone + Debug + Send + Sync + ValidateLinks> ShaderInput for TextureUnit<T> {}
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

mod steps;
pub use steps::*;

impl Construct for TextureUnit {
    fn construct(values: &mut PropertyValues) -> Self {
        Self {
            image: Default::default(),
            coordinates: Construct::construct(values),
            sampling: Construct::construct(values),
            contribution: values.add(1.),
        }
    }
}
