//! Independently authored scene systems and genuinely model-wide controls.
use crate::model::semantic::{profiles, scene, Link, ValidateLinks};
/// Model-wide behavior chosen by the universal vocabulary.
#[derive(Clone, Debug)]
pub enum ModelControls<'a> {
    /// Diablo III appearance-wide declarations.
    D3(profiles::diablo::d3::ModelControls),
    /// Diablo IV appearance and structure declarations.
    D4(profiles::diablo::d4::ModelControls),
    /// Warcraft III animation resources and transition policy.
    Mdlx(profiles::mdlx::ModelControls<'a>),
    /// World of Warcraft model behavior and gameplay lookup programs.
    M2(profiles::m2::ModelControls),
    /// StarCraft II model behavior.
    M3(profiles::m3::ModelControls),
    /// World-model rendering and environment settings.
    Wmo(profiles::wmo::ModelControls<'a>),
    /// Editable-model saved poses and preparation settings.
    Wem(profiles::wem::ModelControls<'a>),
}
impl ValidateLinks for ModelControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::D4(value) => value.visit_links(visitor),
            Self::D3(value) => value.visit_links(visitor),
            Self::Mdlx(value) => value.visit_links(visitor),
            Self::M2(value) => value.visit_links(visitor),
            Self::M3(value) => value.visit_links(visitor),
            Self::Wmo(value) => value.visit_links(visitor),
            Self::Wem(value) => value.visit_links(visitor),
        }
    }
}
/// A scene system with its own logical relationships, independent of file tables.
#[derive(Clone, Debug)]
pub enum System<'a> {
    /// Portable simulation bodies, collision shapes, and constraints.
    Physics(scene::PhysicsSystem<'a>),
    /// World of Warcraft collision, deformation, texture animation, or physics system.
    M2(profiles::m2::System<'a>),
    /// StarCraft II collision, rigid physics, or cloth system.
    M3(profiles::m3::System<'a>),
    /// Static-world groups, portals, and environment volumes.
    Wmo(profiles::wmo::WorldSystem<'a>),
    /// Editable-model mixed-family simulation domain.
    Wem(profiles::wem::System<'a>),
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
            Self::M3(value) => value.visit_links(visitor),
            Self::Wmo(value) => value.visit_links(visitor),
            Self::Wem(value) => value.visit_links(visitor),
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
pub trait SystemPayload: profiles::ControlsPayload {
    /// Whether this system owns the specified member.
    fn contains(&self, _member: SystemMember) -> bool {
        false
    }
}
impl SystemPayload for profiles::m2::System<'_> {}
impl SystemPayload for profiles::m3::System<'_> {}
impl SystemPayload for profiles::Unavailable {}
impl SystemPayload for profiles::wmo::WorldSystem<'_> {
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
impl<A: SystemPayload, B: SystemPayload> SystemPayload for profiles::Either<A, B> {
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
            Self::Wem(value) => value.contains(member),
            _ => false,
        }
    }
}
