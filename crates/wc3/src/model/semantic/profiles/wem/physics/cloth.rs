//! Cloth cages and bindings reference actual mesh groups and simulation bodies.
use super::super::SectionId;
use super::{BodyId, ClothBakeInto, ClothCageKind, ClothColliderKind, Physics, ShapeId};
use crate::model::semantic::{Error, Id, Link, MeshId, NodeId, ValidateLinks};
use std::borrow::Cow;

/// Identity marker for a cloth collision primitive.
pub enum ClothColliderObject {}
/// Cloth collision primitive owned by a simulation domain.
pub type ClothColliderId = Id<ClothColliderObject>;
/// Identity marker for an independently simulated cloth.
pub enum ClothObject {}
/// Cloth object owned by a simulation domain.
pub type ClothId = Id<ClothObject>;

/// One actual grouping within a mesh, independent of physical draw ordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SectionRef {
    /// Mesh containing the section.
    pub mesh: MeshId,
    /// Identity allocated by that mesh's geometry.
    pub section: SectionId,
}
impl ValidateLinks for SectionRef {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::Mesh(self.mesh));
    }
}

/// Collision geometry used by cloth, optionally driven by a rigid-body shape.
#[derive(Clone, Debug)]
pub struct ClothCollider {
    /// Scene anchor; absence places the primitive in model coordinates.
    pub node: Option<NodeId>,
    /// Capsule or plane interpretation.
    pub kind: ClothColliderKind,
    /// Primitive-to-anchor transform.
    pub transform: [[f32; 4]; 4],
    /// Radial dimension for capsules.
    pub radius: f32,
    /// Longitudinal dimension for capsules.
    pub length: f32,
    /// Body and actual shape driving the primitive, when selected.
    pub follows: Option<(BodyId, ShapeId)>,
}
impl ValidateLinks for ClothCollider {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.node.visit_links(visitor);
    }
}

/// Additional StarCraft II cloth response and skinned-surface contact.
#[derive(Clone, Debug, Default)]
pub struct Sc2Cloth {
    /// Strength of tracking toward anchors.
    pub tracking: f32,
    /// Resistance of horizontal constraints.
    pub horizontal_stiffness: f32,
    /// Resistance of shear constraints.
    pub shear_stiffness: f32,
    /// Multiplier for explosion forces.
    pub explosion_scale: f32,
    /// Multiplier for wind forces.
    pub wind_scale: f32,
    /// Aerodynamic drag coefficient.
    pub drag_factor: f32,
    /// Aerodynamic lift coefficient.
    pub lift_factor: f32,
    /// Response stiffness for sphere collisions.
    pub sphere_stiffness: f32,
    /// Enables flattening of simulated cloth.
    pub flatten: bool,
    /// Enables contact with the skinned mesh.
    pub use_skin_collision: bool,
    /// Surface offset for skinned-mesh contact.
    pub skin_offset: f32,
    /// Exponent of skinned-mesh contact response.
    pub skin_exponent: f32,
    /// Stiffness of skinned-mesh contact response.
    pub skin_stiffness: f32,
}

/// Cage generation, anchor selection, and animation baking declarations.
#[derive(Clone, Debug)]
pub struct ClothRecipe<'a> {
    /// Name of the preparation recipe.
    pub name: Cow<'a, str>,
    /// Nodes selected as candidate anchors.
    pub bones: Vec<NodeId>,
    /// Minimum selected-bone skin weight for a particle to remain movable.
    pub threshold: f32,
    /// Cage generation convention.
    pub cage: ClothCageKind,
    /// Requested generated particle count.
    pub particles: u32,
    /// Maximum binding distance relative to cloth size.
    pub reach: f32,
    /// Uses manually authored pinned particles.
    pub pins_by_hand: bool,
    /// Bone-animation or full-detail deformation output.
    pub bake_into: ClothBakeInto,
    /// Original geometry groups associated with the bindings.
    pub binding_origins: Vec<SectionRef>,
    /// Original cage group; absence denotes a procedural cage.
    pub cage_origin: Option<SectionRef>,
}

/// Authored cloth simulation and actual geometry receiving its deformation.
#[derive(Clone, Debug)]
pub struct Cloth<'a> {
    /// Geometry group providing the simulated cage.
    pub cage: SectionRef,
    /// Geometry groups receiving deformation from the cage.
    pub bindings: Vec<SectionRef>,
    /// Collision primitives participating in contact.
    pub colliders: Vec<ClothColliderId>,
    /// Initial activation state.
    pub active: bool,
    /// Assigned mass density.
    pub density: f32,
    /// Attenuation of motion.
    pub damping: f32,
    /// Contact friction.
    pub friction: f32,
    /// Resistance to stretching.
    pub stretch_stiffness: f32,
    /// Resistance to bending.
    pub bend_stiffness: f32,
    /// Gravity contribution multiplier.
    pub gravity_scale: f32,
    /// Assigned wind vector.
    pub wind: [f32; 3],
    /// Additional StarCraft II response parameters.
    pub sc2: Option<Sc2Cloth>,
    /// Optional preparation declarations.
    pub recipe: Option<ClothRecipe<'a>>,
}
impl ValidateLinks for Cloth<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.cage.visit_links(visitor);
        self.bindings.visit_links(visitor);
        if let Some(recipe) = &self.recipe {
            recipe.bones.visit_links(visitor);
            recipe.binding_origins.visit_links(visitor);
            recipe.cage_origin.visit_links(visitor);
        }
    }
}

impl<'a> Physics<'a> {
    /// Adds an independently authored cloth collision primitive.
    pub fn add_cloth_collider(&mut self, value: ClothCollider) -> ClothColliderId {
        let id = self.scope.allocate();
        self.cloth_colliders.push((id, value));
        id
    }
    /// Replaces contact parameters while preserving every cloth relationship to this primitive.
    pub fn set_cloth_collider(
        &mut self,
        id: ClothColliderId,
        value: ClothCollider,
    ) -> Result<(), Error> {
        let entry = self
            .cloth_colliders
            .iter_mut()
            .find(|(key, _)| *key == id)
            .ok_or_else(|| {
                Error::new(
                    "wem.physics.cloth_collider",
                    "primitive belongs to another simulation domain",
                )
            })?;
        entry.1 = value;
        Ok(())
    }
    /// Borrows a collision primitive owned by this simulation domain.
    pub fn cloth_collider(&self, id: ClothColliderId) -> Option<&ClothCollider> {
        self.cloth_colliders
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Iterates cloth collision primitives in authored order.
    pub fn cloth_colliders(
        &self,
    ) -> impl ExactSizeIterator<Item = (ClothColliderId, &ClothCollider)> {
        self.cloth_colliders.iter().map(|(id, value)| (*id, value))
    }
    /// Adds an independently authored cloth simulation.
    pub fn add_cloth(&mut self, value: Cloth<'a>) -> ClothId {
        let id = self.scope.allocate();
        self.cloth.push((id, value));
        id
    }
    /// Replaces simulation settings while preserving the cloth's identity and bake references.
    pub fn set_cloth(&mut self, id: ClothId, value: Cloth<'a>) -> Result<(), Error> {
        let entry = self
            .cloth
            .iter_mut()
            .find(|(key, _)| *key == id)
            .ok_or_else(|| {
                Error::new(
                    "wem.physics.cloth",
                    "cloth belongs to another simulation domain",
                )
            })?;
        entry.1 = value;
        Ok(())
    }
    /// Borrows cloth owned by this simulation domain.
    pub fn cloth(&self, id: ClothId) -> Option<&Cloth<'a>> {
        self.cloth
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Iterates simulated cloth in authored order.
    pub fn cloths(&self) -> impl ExactSizeIterator<Item = (ClothId, &Cloth<'a>)> {
        self.cloth.iter().map(|(id, value)| (*id, value))
    }
    pub(super) fn validate_cloth(&self) -> Result<(), Error> {
        for (_, collider) in &self.cloth_colliders {
            if collider
                .transform
                .iter()
                .flatten()
                .chain([&collider.radius, &collider.length])
                .any(|value| !value.is_finite())
            {
                return Err(Error::new(
                    "wem.physics.cloth_collider",
                    "nonfinite collision primitive",
                ));
            }
            if let Some((body, shape)) = collider.follows {
                if !self
                    .body(body)
                    .is_some_and(|body| body.shapes.contains(&shape))
                {
                    return Err(Error::new(
                        "wem.physics.cloth_collider.follows",
                        "shape is not attached to the selected body",
                    ));
                }
            }
        }
        for (_, cloth) in &self.cloth {
            if cloth
                .colliders
                .iter()
                .any(|id| self.cloth_collider(*id).is_none())
            {
                return Err(Error::new(
                    "wem.physics.cloth.colliders",
                    "collision primitive belongs to another simulation domain",
                ));
            }
            let finite = [
                &cloth.density,
                &cloth.damping,
                &cloth.friction,
                &cloth.stretch_stiffness,
                &cloth.bend_stiffness,
                &cloth.gravity_scale,
            ]
            .into_iter()
            .chain(&cloth.wind)
            .all(|value| value.is_finite());
            let sc2_finite = cloth.sc2.as_ref().is_none_or(|value| {
                [
                    value.tracking,
                    value.horizontal_stiffness,
                    value.shear_stiffness,
                    value.explosion_scale,
                    value.wind_scale,
                    value.drag_factor,
                    value.lift_factor,
                    value.sphere_stiffness,
                    value.skin_offset,
                    value.skin_exponent,
                    value.skin_stiffness,
                ]
                .iter()
                .all(|value| value.is_finite())
            });
            let recipe_finite = cloth
                .recipe
                .as_ref()
                .is_none_or(|value| value.threshold.is_finite() && value.reach.is_finite());
            if !finite || !sc2_finite || !recipe_finite {
                return Err(Error::new(
                    "wem.physics.cloth",
                    "nonfinite simulation response",
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Body, ForceChannels, Motion};
    use super::*;
    use crate::model::semantic::{Animated, Scope};

    fn body() -> Body {
        Body {
            node: Scope::default().allocate(),
            motion: Motion::default(),
            simulates: Animated::constant(true),
            shapes: vec![],
            damping: [0.0; 2],
            inertia_scale: 1.0,
            gravity_scale: 1.0,
            inherit_dynamic: false,
            exempt_from_ragdoll: false,
            force_channels: ForceChannels::default(),
            sc2: None,
            wow: None,
            locked: false,
        }
    }
    #[test]
    fn driven_cloth_collision_requires_a_shape_on_the_selected_body() {
        let mut physics = Physics::default();
        let own = physics.add_body(body());
        let id = physics.add_cloth_collider(ClothCollider {
            node: None,
            kind: ClothColliderKind::default(),
            transform: [[0.0; 4]; 4],
            radius: 1.0,
            length: 2.0,
            follows: Some((own, Scope::default().allocate())),
        });
        assert!(physics.validate().is_err());
        physics
            .cloth_colliders
            .iter_mut()
            .find(|(key, _)| *key == id)
            .unwrap()
            .1
            .follows = None;
        physics.validate().unwrap();
        physics
            .cloth_colliders
            .iter_mut()
            .find(|(key, _)| *key == id)
            .unwrap()
            .1
            .radius = f32::NAN;
        assert!(physics.validate().is_err());
    }
}
