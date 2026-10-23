//! Composable draw state with independently selected operation domains.
use crate::model::common::{Detached, IntoOwned};
use crate::model::{Link, ValidateLinks};

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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, IntoOwned)]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, IntoOwned)]
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
impl<'owned, B, R, O> Detached<'owned> for PassState<B, R, O>
where
    B: Detached<'owned>,
    R: Detached<'owned>,
    O: Detached<'owned>,
{
    type Detached = PassState<B::Detached, R::Detached, O::Detached>;
    fn into_detached(self) -> Self::Detached {
        PassState {
            blend: self.blend.into_detached(),
            rasterization: self.rasterization.into_detached(),
            ordering: self.ordering.into_detached(),
        }
    }
}
impl<'owned, D, F, L, C> Detached<'owned> for Rasterization<D, F, L, C>
where
    D: Detached<'owned>,
    F: Detached<'owned>,
    L: Detached<'owned>,
    C: Detached<'owned>,
{
    type Detached = Rasterization<D::Detached, F::Detached, L::Detached, C::Detached>;
    fn into_detached(self) -> Self::Detached {
        Rasterization {
            depth: self.depth.into_detached(),
            faces: self.faces.into_detached(),
            lighting: self.lighting.into_detached(),
            controls: self.controls.into_detached(),
        }
    }
}
impl<'owned, T, W, B> Detached<'owned> for DepthState<T, W, B>
where
    T: Detached<'owned>,
    W: Detached<'owned>,
    B: Detached<'owned>,
{
    type Detached = DepthState<T::Detached, W::Detached, B::Detached>;
    fn into_detached(self) -> Self::Detached {
        DepthState {
            test: self.test.into_detached(),
            write: self.write.into_detached(),
            bias: self.bias.into_detached(),
        }
    }
}
impl<'owned, E, O, F, C> Detached<'owned> for BlendEquation<E, O, F, C>
where
    E: Detached<'owned>,
    O: Detached<'owned>,
    F: Detached<'owned>,
    C: Detached<'owned>,
{
    type Detached = BlendEquation<E::Detached, O::Detached, F::Detached, C::Detached>;
    fn into_detached(self) -> Self::Detached {
        BlendEquation {
            enabled: self.enabled.into_detached(),
            operation: self.operation.into_detached(),
            source: self.source.into_detached(),
            destination: self.destination.into_detached(),
            constant: self.constant.into_detached(),
        }
    }
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
/// use wc3::model::materials::{state::BlendOperations, wmo};
/// let _: wmo::Blend = BlendOperations::AdditiveAlpha(());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BlendOperations<E, Cutout = (), AlphaAdd = (), Unknown = u32> {
    /// Replace destination color.
    #[default]
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

impl<'owned, E, C, A, U> Detached<'owned> for BlendOperations<E, C, A, U>
where
    E: Detached<'owned>,
    C: Detached<'owned>,
    A: Detached<'owned>,
    U: Detached<'owned>,
{
    type Detached = BlendOperations<E::Detached, C::Detached, A::Detached, U::Detached>;
    fn into_detached(self) -> Self::Detached {
        match self {
            Self::Opaque => BlendOperations::Opaque,
            Self::Alpha => BlendOperations::Alpha,
            Self::Additive => BlendOperations::Additive,
            Self::Modulate => BlendOperations::Modulate,
            Self::Modulate2x => BlendOperations::Modulate2x,
            Self::AlphaKey(value) => BlendOperations::AlphaKey(value.into_detached()),
            Self::AdditiveAlpha(value) => BlendOperations::AdditiveAlpha(value.into_detached()),
            Self::Extension(value) => BlendOperations::Extension(value.into_detached()),
            Self::Unknown(value) => BlendOperations::Unknown(value.into_detached()),
        }
    }
}
