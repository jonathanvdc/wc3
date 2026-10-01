//! Ribbon trails: CPU birth-time sampling and GPU geometry, UVs, and ballistic motion.
//!
//! Cross-sections use local Y for height and remain in world space after birth.
//! Gravity is world -Z with displacement `0.5 * gravity * age²`, deliberately
//! independent of simulation FPS.
mod render;
mod simulation;

pub(crate) use render::{RibbonInstances, RibbonRenderPlugin};
pub(crate) use simulation::{update_ribbons, RibbonLayer, RibbonState};
