//! Owner-local authoring controls chosen by the graph vocabulary.

/// Layered-model ownership and hierarchy semantics.
pub mod layered;
/// Model playback, shadow passes, and external category bindings.
pub mod model;

/// Static-world region and environment ownership declarations.
pub mod world;
/// Static-world draw semantics.
pub mod world_draw;

use crate::model::{Link, ValidateLinks};
/// Additional geometry behavior permitted by the universal vocabulary.
#[derive(Clone, Debug)]
pub enum MeshControls {
    /// Warcraft III picking and sequence bounds.
    Mdlx(layered::Geometry),
    /// World of Warcraft sorting and culling frames.
    M2(model::GeometryControls),
}
impl ValidateLinks for MeshControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Mdlx(value) => value.visit_links(visitor),
            Self::M2(value) => value.center_bone.visit_links(visitor),
        }
    }
}
/// Additional placement behavior permitted by the universal vocabulary.
#[derive(Clone, Debug)]
pub enum InstanceControls<'a> {
    /// Static-world placed region behavior and relationships.
    Wmo(Box<world::RegionControls<'a>>),
}
impl ValidateLinks for InstanceControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Wmo(value) => value.visit_links(visitor),
        }
    }
}
impl ValidateLinks for () {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}

/// Additional hierarchy behavior chosen by a semantic vocabulary.
#[derive(Clone, Debug)]
pub enum NodeControls {
    /// World of Warcraft skeleton behavior and external bone role.
    M2(model::BoneControls),
    /// Warcraft III bone scope, bind pose, and unresolved node behavior.
    Mdlx(layered::NodeControls),
}
impl ValidateLinks for NodeControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::M2(value) => value.visit_links(visitor),
            Self::Mdlx(value) => value.visit_links(visitor),
        }
    }
}

/// Behavioral controls attached directly to one draw pass.
#[derive(Clone, Debug)]
pub enum DrawControls {
    /// World of Warcraft color or shadow pass behavior.
    M2(model::DrawControls),

    /// Static-world lighting class, bounds, and lightmap behavior.
    Wmo(world_draw::DrawControls),
}
impl ValidateLinks for DrawControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::M2(value) => value.visit_links(visitor),

            Self::Wmo(value) => value.visit_links(visitor),
        }
    }
}

/// Native billboard declarations preserved independently of effective node behavior.
/// MDLX and M2 assign different precedence and activation rules to these declarations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BillboardFlags {
    /// Native full-facing / activation flag.
    pub enabled: bool,
    /// Native X lock declaration.
    pub x: bool,
    /// Native Y lock declaration.
    pub y: bool,
    /// Native Z lock declaration.
    pub z: bool,
}
