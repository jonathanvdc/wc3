//! Logical M3 composite values.
use super::PropertyBinding;
use crate::model::Construct;
use crate::model::MaterialId;

/// Editable M3 CompositeSection values.
#[derive(Clone, Debug, PartialEq)]
pub struct CompositeSection {
    /// Selects an entry in the material array chosen by `material_type`.
    pub material: MaterialId,
    /// Controls blend weight through its animation value.
    pub map_multiplier: PropertyBinding<f32>,
}

/// Editable M3 CompositeMaterialStep values.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct CompositeMaterialStep {
    /// Sets render priority (lower = earlier).
    pub priority: u32,
    /// Sets sub-material sections (CMS_).
    pub sections: Vec<CompositeSection>,
}
