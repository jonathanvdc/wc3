//! Node-local force volumes and geometric deformation controls.
use super::physics::ForceChannels;
use super::{CollisionShapeKind, ForceKind, ForceVolume};
use crate::model::semantic::{Animated, Bounds, Link, MeshId, ValidateLinks};

/// An independently authored force volume in its node's coordinate frame.
#[derive(Clone, Debug)]
pub struct Force {
    /// Directional, radial, drag, or vortex operation.
    pub kind: ForceKind,
    /// Influence volume geometry.
    pub volume: ForceVolume,
    /// Enables spatial attenuation of the force.
    pub falloff: bool,
    /// Enables the height-gradient contribution.
    pub height_gradient: bool,
    /// Disables volume bounds when selected.
    pub unbounded: bool,
    /// Applies the field to particles.
    pub affects_particles: bool,
    /// Applies the field to rigid bodies.
    pub affects_bodies: bool,
    /// Local and world force-channel selections.
    pub channels: ForceChannels,
    /// Producer-defined scope selection; zero identifies the model itself.
    pub scope: u32,
    /// Force strength and optional animation destination.
    pub strength: Animated<f32>,
    /// Volume width, height, and length.
    pub dimensions: [Animated<f32>; 3],
    /// Field-to-node transform.
    pub transform: [[f32; 4]; 4],
}
impl ValidateLinks for Force {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.strength.visit_links(visitor);
        for value in &self.dimensions {
            value.visit_links(visitor);
        }
        if self
            .transform
            .iter()
            .flatten()
            .any(|value| !value.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}

/// Deformation operation whose producer-defined categories remain unresolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WarpMode(
    /// Unresolved authoring mode identifier.
    pub u32,
);
/// Radial, axial, and angular deformation settings belonging to a node.
#[derive(Clone, Debug)]
pub struct VertexWarp {
    /// Producer-defined interpretation of the parameter set.
    pub mode: WarpMode,
    /// Producer-defined control word whose meaning is unresolved.
    pub unclassified_control: u32,
    /// Influence radius.
    pub radius: Animated<f32>,
    /// Influence height.
    pub height: Animated<f32>,
    /// Overall deformation strength.
    pub strength: Animated<f32>,
    /// Angular deformation coefficient.
    pub angular: Animated<f32>,
    /// Axial deformation coefficient.
    pub axial: Animated<f32>,
    /// Radial deformation coefficient.
    pub radial: Animated<f32>,
}
impl ValidateLinks for VertexWarp {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for value in [
            &self.radius,
            &self.height,
            &self.strength,
            &self.angular,
            &self.axial,
            &self.radial,
        ] {
            value.visit_links(visitor);
        }
    }
}

/// Authored collision primitives retain the unused alternative dimensions for editing.
#[derive(Clone, Debug)]
pub struct Collision {
    /// Active primitive interpretation.
    pub kind: CollisionShapeKind,
    /// Authored box and accompanying sphere-radius bounds.
    pub bounds: Bounds,
    /// Sphere center in node coordinates.
    pub sphere_center: [f32; 3],
    /// Authored sphere or capsule radius.
    pub sphere_radius: f32,
    /// Cylinder or capsule height along its local axis.
    pub height: f32,
}
impl ValidateLinks for Collision {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.bounds.validate().is_err()
            || self
                .sphere_center
                .iter()
                .chain([&self.sphere_radius, &self.height])
                .any(|value| !value.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}

/// Bone-specific spatial declarations attached to the actual bone node.
#[derive(Clone, Debug)]
pub struct Bone {
    /// Authored axis-aligned bone bounds.
    pub bounds: Bounds,
    /// Authored bone bounding-sphere center.
    pub sphere_center: [f32; 3],
    /// Authored bone bounding-sphere radius.
    pub sphere_radius: f32,
    /// Mesh whose visibility this bone controls, when declared.
    pub visibility_mesh: Option<MeshId>,
}
impl ValidateLinks for Bone {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.visibility_mesh.visit_links(visitor);
        if self.bounds.validate().is_err()
            || self
                .sphere_center
                .iter()
                .chain([&self.sphere_radius])
                .any(|value| !value.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}
