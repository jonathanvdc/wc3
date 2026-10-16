//! Immutable GPU trail records and rendering.
mod records;
mod render;
pub(crate) use records::{RibbonInstances, RibbonSection, RibbonSegment, RibbonUniform};
pub(crate) use render::RibbonRenderPlugin;
