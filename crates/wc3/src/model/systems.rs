//! Independently authored scene systems and model-wide metadata.
use crate::model::common::IntoOwned;
/// Model collision, customization, and texture program ownership.
pub mod model;
/// Model-family bodies, constraints, and environmental response.
pub mod model_physics;

use crate::model::profiles::{m2, mdlx, wmo};
use crate::model::profiles::{Either, PropertiesPayload, Unavailable};
use crate::model::{scene, Link, ValidateLinks};
/// Model-wide metadata chosen by the universal vocabulary.
#[derive(Clone, Debug, IntoOwned)]
pub enum ModelMetadata<'a> {
    /// Warcraft III animation resources and transition policy.
    Mdlx(mdlx::ModelMetadata<'a>),
    /// World of Warcraft model behavior and gameplay lookup programs.
    M2(m2::ModelMetadata<'a>),

    /// World-model rendering and environment settings.
    Wmo(wmo::ModelMetadata),
}
impl ValidateLinks for ModelMetadata<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Mdlx(value) => value.visit_links(visitor),
            Self::M2(value) => value.visit_links(visitor),

            Self::Wmo(value) => value.visit_links(visitor),
        }
    }
}
/// A scene system with its own logical relationships, independent of file tables.
#[derive(Clone, Debug, IntoOwned)]
pub enum System<'a> {
    /// Portable simulation bodies, collision shapes, and constraints.
    Physics(scene::PhysicsSystem<'a>),
    /// World of Warcraft collision, deformation, texture animation, or physics system.
    M2(model::System<'a>),

    /// Static-world groups, portals, and environment volumes.
    Wmo(wmo::WorldSystem<'a>),
}
impl ValidateLinks for System<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Physics(value) => {
                value.visit_links(visitor);
                if value.validate().is_err() {
                    visitor(Link::InvalidValue);
                }
            }
            Self::M2(value) => value.visit_links(visitor),

            Self::Wmo(value) => value.visit_links(visitor),
        }
    }
}

/// A relationship to a collection owned by a scene system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemMember {
    /// Static-world visibility group.
    WmoGroup(u32),
    /// Static-world portal boundary.
    WmoPortal(u32),
    /// Static-world doodad placement set.
    WmoDoodadSet(u32),
    /// Static-world fog volume.
    WmoFog(u32),
    /// Static-world ambient illumination volume.
    WmoAmbient(u32),
}
/// A scene system exposes its authored local identity domains for validation.
pub trait SystemPayload: PropertiesPayload {
    /// Whether this system owns the specified member.
    fn contains(&self, _member: SystemMember) -> bool {
        false
    }
}
impl SystemPayload for model::System<'_> {}

impl SystemPayload for Unavailable {}
impl SystemPayload for wmo::WorldSystem<'_> {
    fn contains(&self, member: SystemMember) -> bool {
        match member {
            SystemMember::WmoGroup(index) => (index as usize) < self.groups.len(),
            SystemMember::WmoPortal(index) => (index as usize) < self.portals.len(),
            SystemMember::WmoDoodadSet(index) => (index as usize) < self.doodad_sets.len(),
            SystemMember::WmoFog(index) => (index as usize) < self.fogs.len(),
            SystemMember::WmoAmbient(index) => (index as usize) < self.ambient_volumes.len(),
        }
    }
}
impl<A: SystemPayload, B: SystemPayload> SystemPayload for Either<A, B> {
    fn contains(&self, member: SystemMember) -> bool {
        match self {
            Self::Left(value) => value.contains(member),
            Self::Right(value) => value.contains(member),
        }
    }
}
impl SystemPayload for System<'_> {
    fn contains(&self, member: SystemMember) -> bool {
        match self {
            Self::Wmo(value) => value.contains(member),

            _ => false,
        }
    }
}
