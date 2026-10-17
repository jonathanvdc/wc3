//! Owner-local authoring controls chosen by the graph vocabulary.
/// Diablo III ownership declarations.
pub mod diablo3;
/// Diablo IV ownership declarations.
pub mod diablo4;
/// Layered-model ownership and hierarchy semantics.
pub mod layered;
/// Model playback, shadow passes, and external category bindings.
pub mod model;
/// Controller selection and playback programs.
pub mod playback;
/// Programmed-model skeleton and owner declarations.
pub mod programmed;
/// Resource sampling producer declarations.
pub mod texture;
/// Ordered keys whose destinations are defined externally.
pub mod unbound;
/// Static-world region and environment ownership declarations.
pub mod world;
/// Static-world draw semantics.
pub mod world_draw;

use crate::model::{Link, ValidateLinks};
/// Additional texture behavior permitted by the universal vocabulary.
#[derive(Clone, Debug)]
pub enum TextureControls<'a> {
    /// World of Warcraft producer options independent of texture addressing.
    M2(texture::TextureControls<'a>),
    /// Warcraft III fallback filename and texture behavior.
    Mdlx(layered::TextureControls<'a>),
}
impl ValidateLinks for TextureControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::M2(value) => value.visit_links(visitor),
            Self::Mdlx(value) => value.visit_links(visitor),
        }
    }
}
/// Additional geometry behavior permitted by the universal vocabulary.
#[derive(Clone, Debug)]
pub enum MeshControls {
    /// Diablo III subobject behavior.
    D3(diablo3::MeshControls),
    /// Diablo IV subobject declarations and segment palettes.
    D4(diablo4::MeshControls),
    /// Warcraft III picking and sequence bounds.
    Mdlx(layered::Geometry),
    /// World of Warcraft sorting and culling frames.
    M2(model::GeometryControls),
    /// StarCraft II region identity and behavior.
    M3(programmed::MeshControls),
}
impl ValidateLinks for MeshControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::D4(value) => value.visit_links(visitor),
            Self::D3(value) => value.visit_links(visitor),

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
    Wmo(Box<world::RegionControls<'a>>),
    /// StarCraft II region and draw-batch behavior.
    M3(programmed::InstanceControls),
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
    /// Diablo III bone bind poses and attachment roles.
    D3(diablo3::NodeRole<'a>),
    /// Diablo IV hashed bones and independent bind poses.
    D4(diablo4::NodeRole),
    /// World of Warcraft skeleton behavior and external bone role.
    M2(model::BoneControls),
    /// Warcraft III bone scope, bind pose, and unresolved node behavior.
    Mdlx(layered::NodeControls),
    /// StarCraft II bone controller behavior.
    M3(programmed::NodeRole),
}
impl ValidateLinks for NodeControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::D4(value) => value.visit_links(visitor),
            Self::D3(value) => value.visit_links(visitor),

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
    M2(model::DrawControls),
    /// StarCraft II batching, priority, and animated culling.
    M3(programmed::BatchControls),
    /// Static-world lighting class, bounds, and lightmap behavior.
    Wmo(world_draw::DrawControls),
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
