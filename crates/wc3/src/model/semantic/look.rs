//! Complete authored material assignments for instance draw bindings.
use crate::model::semantic::{DrawId, InstanceId, MaterialId};
use std::borrow::Cow;
/// A material assignment for one draw of a mesh placement.
#[derive(Clone, Debug)]
pub struct LookBinding {
    /// Mesh placement receiving this assignment.
    pub instance: InstanceId,
    /// Stable draw binding identity within that placement.
    pub draw: DrawId,
    /// Material assigned to this draw.
    pub material: MaterialId,
}
/// A named complete material assignment for every instance draw.
#[derive(Clone, Debug, Default)]
pub struct Look<'a> {
    /// Authoring name of the material alternative.
    pub name: Cow<'a, str>,
    /// Exactly one material assignment for each instance draw.
    pub bindings: Vec<LookBinding>,
}
