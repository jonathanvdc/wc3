//! Logical M3 properties values.
use crate::model as semantic;
use crate::model::values::programmed::{AnimationFlags, Interpolation};
use crate::model::Construct;

/// M3 animated slots with native reset values and control metadata.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct Property<T: semantic::Animatable> {
    /// Value used before a curve overrides this property.
    pub value: semantic::Animated<T>,
    /// Value used when the property resets.
    pub reset: T,
    /// Editable native animation controls, independent of storage layout.
    pub control: AnimationControl,
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
