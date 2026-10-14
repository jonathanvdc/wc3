//! Owner-local authoring controls chosen by the graph vocabulary.
use crate::model::semantic::{profiles, Link, ValidateLinks};
/// Additional texture behavior permitted by the universal vocabulary.
#[derive(Clone, Debug)]
pub enum TextureControls<'a> {
    /// World of Warcraft producer options independent of texture addressing.
    M2(profiles::m2::TextureControls<'a>),
    /// Warcraft III fallback filename and texture behavior.
    Mdlx(profiles::mdlx::TextureControls<'a>),
    /// Editable-model image-role and alternate resource declarations.
    Wem(profiles::wem::TextureControls<'a>),
}
impl ValidateLinks for TextureControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Wem(value) => value.visit_links(visitor),
            Self::M2(value) => value.visit_links(visitor),
            Self::Mdlx(value) => value.visit_links(visitor),
        }
    }
}
/// Additional geometry behavior permitted by the universal vocabulary.
#[derive(Clone, Debug)]
pub enum MeshControls {
    /// Warcraft III picking and sequence bounds.
    Mdlx(profiles::mdlx::Geometry),
    /// World of Warcraft sorting and culling frames.
    M2(profiles::m2::GeometryControls),
    /// StarCraft II region identity and behavior.
    M3(profiles::m3::MeshControls),
    /// Editable-model LOD selection.
    Wem(profiles::wem::MeshControls),
}
impl ValidateLinks for MeshControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Wem(value) => value.visit_links(visitor),
            Self::Mdlx(value) => value.visit_links(visitor),
            Self::M2(value) => value.center_bone.visit_links(visitor),
            Self::M3(value) => value.visit_links(visitor),
        }
    }
}
/// Additional placement behavior permitted by the universal vocabulary.
#[derive(Clone, Debug)]
pub enum InstanceControls<'a> {
    /// Static-world placed region behavior and relationships.
    Wmo(Box<profiles::wmo::RegionControls<'a>>),
    /// StarCraft II region and draw-batch behavior.
    M3(profiles::m3::InstanceControls),
}
impl ValidateLinks for InstanceControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::M3(value) => value.visit_links(visitor),
            Self::Wmo(value) => value.visit_links(visitor),
        }
    }
}
impl ValidateLinks for () {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}

/// Additional hierarchy behavior chosen by a semantic vocabulary.
#[derive(Clone, Debug)]
pub enum NodeControls<'a> {
    /// World of Warcraft skeleton behavior and external bone role.
    M2(profiles::m2::BoneControls),
    /// Warcraft III bone scope, bind pose, and unresolved node behavior.
    Mdlx(profiles::mdlx::NodeControls),
    /// StarCraft II bone controller behavior.
    M3(profiles::m3::NodeRole),
    /// Editable-model saved poses and declared hierarchy role.
    Wem(profiles::wem::NodeControls<'a>),
}
impl ValidateLinks for NodeControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Wem(value) => value.visit_links(visitor),
            Self::M2(value) => value.visit_links(visitor),
            Self::Mdlx(value) => value.visit_links(visitor),
            Self::M3(value) => value.visit_links(visitor),
        }
    }
}

/// Behavioral controls attached directly to one draw pass.
#[derive(Clone, Debug)]
pub enum DrawControls {
    /// World of Warcraft color or shadow pass behavior.
    M2(profiles::m2::DrawControls),
    /// StarCraft II batching, priority, and animated culling.
    M3(profiles::m3::BatchControls),
    /// Static-world lighting class, bounds, and lightmap behavior.
    Wmo(profiles::wmo::DrawControls),
}
impl ValidateLinks for DrawControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::M2(value) => value.visit_links(visitor),
            Self::M3(value) => value.visit_links(visitor),
            Self::Wmo(value) => value.visit_links(visitor),
        }
    }
}
