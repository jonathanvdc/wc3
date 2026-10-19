//! Owner-local authoring properties chosen by the graph vocabulary.

use crate::model::profiles::{m2, mdlx, wmo};
use crate::model::{Link, ValidateLinks};
/// Additional geometry behavior permitted by the universal vocabulary.
#[derive(Clone, Debug)]
pub enum MeshProperties {
    /// Warcraft III picking and sequence bounds.
    Mdlx(mdlx::MeshProperties),
    /// World of Warcraft sorting and culling frames.
    M2(m2::MeshProperties),
}
impl ValidateLinks for MeshProperties {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Mdlx(value) => value.visit_links(visitor),
            Self::M2(value) => value.visit_links(visitor),
        }
    }
}
/// Additional placement behavior permitted by the universal vocabulary.
#[derive(Clone, Debug)]
pub enum InstanceProperties<'a> {
    /// Static-world placed region behavior and relationships.
    Wmo(Box<wmo::Region<'a>>),
}
impl ValidateLinks for InstanceProperties<'_> {
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
pub enum NodeProperties {
    /// World of Warcraft skeleton behavior and external bone role.
    M2(m2::BoneProperties),
    /// Warcraft III bone scope, bind pose, and unresolved node behavior.
    Mdlx(mdlx::NodeProperties),
}
impl ValidateLinks for NodeProperties {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::M2(value) => value.visit_links(visitor),
            Self::Mdlx(value) => value.visit_links(visitor),
        }
    }
}

/// Behavioral controls attached directly to one draw pass.
#[derive(Clone, Debug)]
pub enum DrawProperties {
    /// World of Warcraft color or shadow pass behavior.
    M2(m2::DrawProperties),

    /// Static-world lighting class, bounds, and lightmap behavior.
    Wmo(wmo::draw::DrawProperties),
}
impl ValidateLinks for DrawProperties {
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
