//! Composable draw state with independently selected operation domains.
use crate::model::semantic::{Link, ValidateLinks};

/// State applied to a draw pass, independent of its shader inputs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PassState<B, R, O = ()> {
    /// Color composition, retaining the selected operation vocabulary.
    pub blend: B,
    /// Depth, face, illumination, and additional rasterization controls.
    pub rasterization: R,
    /// Authored ordering within the owning rendering program.
    pub ordering: O,
}
/// Independently selected rasterization domains.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rasterization<D, F, L, C = ()> {
    /// Depth testing, writes, and offsets.
    pub depth: D,
    /// Face selection or culling.
    pub faces: F,
    /// Lighting and fog behavior.
    pub lighting: L,
    /// Additional controls belonging to this rasterization vocabulary.
    pub controls: C,
}
/// Depth controls retain the testing, write, and bias representations independently.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DepthState<T, W, B = ()> {
    /// Depth comparison policy.
    pub test: T,
    /// Depth-buffer update policy.
    pub write: W,
    /// Authored depth offset.
    pub bias: B,
}
/// Explicit control of a rendering operation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Operation {
    /// Apply the operation.
    #[default]
    Enabled,
    /// Bypass the operation.
    Disabled,
}
impl Operation {
    /// Selects whether an operation is bypassed.
    pub const fn bypassed(bypassed: bool) -> Self {
        if bypassed {
            Self::Disabled
        } else {
            Self::Enabled
        }
    }
    /// Whether the operation is bypassed.
    pub const fn is_bypassed(self) -> bool {
        matches!(self, Self::Disabled)
    }
}
/// Lighting and fog can be bypassed independently.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Lighting {
    /// Bypass scene illumination.
    pub unlit: bool,
    /// Bypass scene fog.
    pub unfogged: bool,
}
/// Source and destination blend equation with an independently selected control vocabulary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlendEquation<E, O, F, C> {
    /// Whether blending is active.
    pub enabled: E,
    /// Operation combining the factored colors.
    pub operation: O,
    /// Source contribution factor.
    pub source: F,
    /// Destination contribution factor.
    pub destination: F,
    /// Constant color supplied to the operation.
    pub constant: C,
}
impl<B: ValidateLinks, R: ValidateLinks, O: ValidateLinks> ValidateLinks for PassState<B, R, O> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.blend.visit_links(v);
        self.rasterization.visit_links(v);
        self.ordering.visit_links(v);
    }
}
impl<D: ValidateLinks, F: ValidateLinks, L: ValidateLinks, C: ValidateLinks> ValidateLinks
    for Rasterization<D, F, L, C>
{
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.depth.visit_links(v);
        self.faces.visit_links(v);
        self.lighting.visit_links(v);
        self.controls.visit_links(v);
    }
}
impl<T: ValidateLinks, W: ValidateLinks, B: ValidateLinks> ValidateLinks for DepthState<T, W, B> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.test.visit_links(v);
        self.write.visit_links(v);
        self.bias.visit_links(v);
    }
}
impl ValidateLinks for Operation {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for Lighting {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl<E: ValidateLinks, O: ValidateLinks, F: ValidateLinks, C: ValidateLinks> ValidateLinks
    for BlendEquation<E, O, F, C>
{
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.enabled.visit_links(v);
        self.operation.visit_links(v);
        self.source.visit_links(v);
        self.destination.visit_links(v);
        self.constant.visit_links(v);
    }
}

/// Established color operations with independently selected cutout, alpha-add, and extensions.
///
/// A world-surface selection cannot author an opacity-weighted additive operation:
///
/// ```compile_fail
/// use wc3::model::semantic::materials::{state::BlendOperations, world};
/// let _: world::Blend = BlendOperations::AdditiveAlpha(());
/// ```
///
/// Programmed blending cannot author a cutout operation:
///
/// ```compile_fail
/// use wc3::model::semantic::{materials::state::BlendOperations, values::programmed};
/// let _: programmed::BlendMode = BlendOperations::AlphaKey(());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlendOperations<E, Cutout = (), AlphaAdd = (), Unknown = u32> {
    /// Replace destination color.
    Opaque,
    /// Blend using source opacity.
    Alpha,
    /// Add source color without opacity modulation.
    Additive,
    /// Multiply destination color.
    Modulate,
    /// Multiply destination color and double the result.
    Modulate2x,
    /// Discard transparent texels using the selected cutout declaration.
    AlphaKey(Cutout),
    /// Add source color weighted by source opacity.
    AdditiveAlpha(AlphaAdd),
    /// An additional operation in the selected shader domain.
    Extension(E),
    /// Retain an unresolved selector without assigning it an operation.
    Unknown(Unknown),
}
impl<E: ValidateLinks, C: ValidateLinks, A: ValidateLinks, U: ValidateLinks> ValidateLinks
    for BlendOperations<E, C, A, U>
{
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::AlphaKey(value) => value.visit_links(visitor),
            Self::AdditiveAlpha(value) => value.visit_links(visitor),
            Self::Extension(value) => value.visit_links(visitor),
            Self::Unknown(value) => value.visit_links(visitor),
            _ => {}
        }
    }
}

impl<E, C, A, U> Default for BlendOperations<E, C, A, U> {
    fn default() -> Self {
        Self::Opaque
    }
}
