//! Authored alternatives to the default draw-material assignment.
use crate::model::semantic::{InstanceId, MaterialId};
use std::borrow::Cow;
/// A material assignment for one draw of a mesh placement.
#[derive(Clone, Debug)]
pub struct LookBinding {
    /// Mesh placement receiving this assignment.
    pub instance: InstanceId,
    /// Draw ordinal within that placement.
    pub draw: u32,
    /// Material overriding the draw's base assignment.
    pub material: MaterialId,
}
/// A named set of material overrides. Unmentioned draws retain their default material.
#[derive(Clone, Debug, Default)]
pub struct Look<'a> {
    /// Authoring name of the material alternative.
    pub name: Cow<'a, str>,
    /// Overrides; unmentioned draws retain their base materials.
    pub bindings: Vec<LookBinding>,
}
