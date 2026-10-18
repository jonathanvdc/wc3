//! Owner-local authoring controls chosen by the graph vocabulary.

/// Layered-model ownership and hierarchy semantics.
pub mod layered;
/// Model playback, shadow passes, and external category bindings.
pub mod model;
/// Resource sampling producer declarations.
pub mod texture;

/// Static-world region and environment ownership declarations.
pub mod world;
/// Static-world draw semantics.
pub mod world_draw;

use crate::model::{ImageResourceId, Link, ValidateLinks};
use zerovec::ule::AsULE;
/// Additional texture behavior permitted by the universal vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureControls {
    /// World of Warcraft producer options independent of texture addressing.
    M2(texture::TextureControls),
    /// Warcraft III fallback filename and texture behavior.
    Mdlx(layered::TextureControls),
}
impl ValidateLinks for TextureControls {
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

#[doc(hidden)]
#[allow(missing_docs)]
#[zerovec::make_ule(TextureControlsKindULE)]
#[zerovec::skip_derive(Ord)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureControlsKind {
    M2 = 0,
    Mdlx = 1,
}
#[doc(hidden)]
#[allow(missing_docs)]
#[zerovec::make_ule(TextureControlsEncodingULE)]
#[zerovec::skip_derive(Ord)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureControlsEncoding {
    kind: TextureControlsKind,
    path: Option<ImageResourceId>,
    flags: u32,
}
impl AsULE for TextureControls {
    type ULE = TextureControlsEncodingULE;
    fn to_unaligned(self) -> Self::ULE {
        let value = match self {
            Self::M2(value) => TextureControlsEncoding {
                kind: TextureControlsKind::M2,
                path: value.path,
                flags: value.flags.bits(),
            },
            Self::Mdlx(value) => TextureControlsEncoding {
                kind: TextureControlsKind::Mdlx,
                path: value.fallback_path,
                flags: value.unclassified.bits(),
            },
        };
        value.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        let value = TextureControlsEncoding::from_unaligned(value);
        match value.kind {
            TextureControlsKind::M2 => Self::M2(texture::TextureControls {
                path: value.path,
                flags: texture::SupplementalTextureFlags(value.flags),
            }),
            TextureControlsKind::Mdlx => Self::Mdlx(layered::TextureControls {
                fallback_path: value.path,
                unclassified: value.flags.into(),
            }),
        }
    }
}
impl From<texture::TextureControls> for TextureControls {
    fn from(value: texture::TextureControls) -> Self {
        Self::M2(value)
    }
}
impl From<layered::TextureControls> for TextureControls {
    fn from(value: layered::TextureControls) -> Self {
        Self::Mdlx(value)
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
