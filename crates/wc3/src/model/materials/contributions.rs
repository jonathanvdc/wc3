//! Surface inputs composed with the parameters that govern their contribution.
use crate::model::{Animatable, Construct, Values};
use crate::model::{Link, ValidateLinks};

/// An input together with its authored response parameters.
///
/// The input may be a texture binding, a sparse set of shader slots, or `()`
/// for a constant contribution. Parameter types preserve the chosen shader model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SurfaceContribution<Input, Parameters = ()> {
    /// Source of the contribution.
    pub input: Input,
    /// Authored response, tint, strength, or compositing operation.
    pub parameters: Parameters,
}
impl<Input: ValidateLinks, Parameters: ValidateLinks> ValidateLinks
    for SurfaceContribution<Input, Parameters>
{
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.input.visit_links(visitor);
        self.parameters.visit_links(visitor);
    }
}

/// Two independently optional inputs whose named slots retain their meaning.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PairedInputs<Input> {
    /// First authored slot.
    pub primary: Input,
    /// Second authored slot.
    pub secondary: Input,
}
impl<Input: ValidateLinks> ValidateLinks for PairedInputs<Input> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.primary.visit_links(visitor);
        self.secondary.visit_links(visitor);
    }
}

/// An input and its independently optional influence mask.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaskedInput<Input> {
    /// Contributing input.
    pub input: Input,
    /// Mask controlling its influence.
    pub mask: Input,
}
impl<Input: ValidateLinks> ValidateLinks for MaskedInput<Input> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.input.visit_links(visitor);
        self.mask.visit_links(visitor);
    }
}

/// Base normal input, two masked additions, and independently authored weights.
///
/// Weights remain a separate collection because their source length is not
/// constrained to the number of available additional texture slots.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NormalContributions<Input, Weights> {
    /// Base surface-normal input.
    pub base: Input,
    /// First independently masked normal input.
    pub primary: MaskedInput<Input>,
    /// Second independently masked normal input.
    pub secondary: MaskedInput<Input>,
    /// Authored weight collection.
    pub weights: Weights,
}
impl<Input: ValidateLinks, Weights: ValidateLinks> ValidateLinks
    for NormalContributions<Input, Weights>
{
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.base.visit_links(visitor);
        self.primary.visit_links(visitor);
        self.secondary.visit_links(visitor);
        self.weights.visit_links(visitor);
    }
}

/// Distinct specular-color and gloss inputs to a highlight response.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpecularInputs<Input> {
    /// Color or intensity of the highlight response.
    pub color: Input,
    /// Gloss input interpreted by the selected response model.
    pub gloss: Input,
}
impl<Input: ValidateLinks> ValidateLinks for SpecularInputs<Input> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.color.visit_links(visitor);
        self.gloss.visit_links(visitor);
    }
}

impl<I: Construct, T: Animatable + Default> Construct
    for SurfaceContribution<I, crate::model::animation::programmed::Property<T>>
{
    fn construct(values: &mut Values) -> Self {
        Self {
            input: I::construct(values),
            parameters: Construct::construct(values),
        }
    }
}
