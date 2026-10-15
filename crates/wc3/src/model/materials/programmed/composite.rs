//! Logical M3 composite values.
use super::Property;
use crate::model::MaterialId;

/// Editable M3 CompositeSection values.
#[derive(Clone, Debug, PartialEq)]
pub struct CompositeSection {
    /// Selects an entry in the material array chosen by `material_type`.
    pub material: MaterialId,
    /// Controls blend weight through its animation value.
    pub map_multiplier: Property<f32>,
}

/// Editable M3 Composite values.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Composite {
    /// Sets render priority (lower = earlier).
    pub priority: u32,
    /// Sets sub-material sections (CMS_).
    pub sections: Vec<CompositeSection>,
}
