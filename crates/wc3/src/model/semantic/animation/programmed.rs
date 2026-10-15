//! Logical M3 properties values.
use crate::model::semantic;
use crate::model::semantic::values::programmed::{AnimationFlags, Interpolation};

/// M3 property values and their semantic animation binding.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Property<T> {
    /// Value used before a curve overrides this property.
    pub initial: T,
    /// Value used when the property resets.
    pub reset: T,
    /// Editable native animation controls, independent of storage layout.
    pub control: AnimationControl,
    /// Common graph destination when this property participates in animation.
    pub binding: Option<semantic::Property<T>>,
}

/// M3 animation-reference controls with no storage addresses.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AnimationControl {
    /// Declared native interpolation selector.
    pub interpolation: Interpolation,
    /// Full animation control word; individual bit meanings are unresolved.
    pub flags: AnimationFlags,
    /// Source property identifier, independent of arena handles.
    pub source_id: u32,
    /// Unresolved animation-reference control word.
    pub reserved: i32,
}
