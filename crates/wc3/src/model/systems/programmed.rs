//! Independent systems describe collision geometry and simulation relationships.
//! Their node and mesh links refer to ordinary graph objects. A system never
//! introduces a hidden skeleton or owns an encoded model document.
use crate::model::controls::playback::ControllerSelection;
use crate::model::systems::programmed_physics::{Body, BodyId, Cloth, Constraint, Joint};
use crate::model::values::programmed::BoneSetFlags;
use crate::model::{
    Error, FeatureId, FeatureKind, GeometryKind, Link, MeshId, ValidateLinks, ZeroVec,
};
use std::borrow::Cow;
use std::collections::HashSet;
/// A collision surface independent of deforming display geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct CollisionSystem {
    /// Mesh supplying collision positions and connectivity.
    pub mesh: MeshId,
}
/// A rigid-body world and its constraints.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PhysicsSystem<'a> {
    /// Bodies and their authored shape and simulation controls.
    pub bodies: Vec<Body<'a>>,
    /// Constraints linking the world's bodies or skeleton joints.
    pub joints: Vec<Joint>,
    /// Constraints linking body identities within this world.
    pub constraints: Vec<Constraint>,
}
/// A collection of coupled cloth solvers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClothSystem<'a> {
    /// Solvers, proxy relationships, and their particle bindings.
    pub cloths: Vec<Cloth<'a>>,
}
/// A named turret whose parts are independently authored bone-attached features.
#[derive(Clone, Debug, PartialEq)]
pub struct TurretGroup<'a> {
    /// Authored group name.
    pub name: Cow<'a, str>,
    /// Parts participating in this turret.
    pub parts: Vec<FeatureId>,
}
impl ValidateLinks for TurretGroup<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for id in &self.parts {
            visitor(Link::TypedFeature {
                id: *id,
                kind: FeatureKind::M3Turret,
            });
        }
    }
}
/// Authored animation splitting declaration with unresolved selector semantics.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationPartition<'a> {
    /// Authored declaration name.
    pub name: Cow<'a, str>,
    /// Authored split-control flags.
    pub flags: BoneSetFlags,
    /// Primary selector; its namespace has not been established.
    pub selector: u16,
    /// Fallback selector in the same unresolved namespace.
    pub fallback_selector: u16,
    /// Ordered authored split-item controls, without an invented bone interpretation.
    pub split_items: ZeroVec<'a, u16>,
}
/// Independent M3 scene systems whose ownership is separate from node features.
#[derive(Clone, Debug, PartialEq)]
pub enum System<'a> {
    /// Named declarations governing animation splitting.
    AnimationPartitions(Vec<AnimationPartition<'a>>),
    /// Authored controller selections not associated with a playback clip.
    ControllerSelections(Vec<ControllerSelection<'a>>),
    /// Named turret groups referencing authored turret parts.
    TurretGroups(Vec<TurretGroup<'a>>),
    /// Static collision geometry.
    Collision(CollisionSystem),
    /// Rigid-body simulation and constraints.
    Physics(PhysicsSystem<'a>),
    /// Coupled cloth simulation.
    Cloth(ClothSystem<'a>),
}
impl ValidateLinks for CollisionSystem {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::TypedMesh {
            id: self.mesh,
            kind: GeometryKind::Collision,
        })
    }
}
impl<'a> PhysicsSystem<'a> {
    /// Adds a body and returns its checked identity in this world.
    pub fn add_body(&mut self, body: Body<'a>) -> Result<BodyId, Error> {
        let id = body.id();
        if self.bodies.iter().any(|value| value.id() == id) {
            return Err(Error::new("m3.physics.bodies", "duplicate body identity"));
        }
        self.bodies.push(body);
        Ok(id)
    }
}
impl ValidateLinks for PhysicsSystem<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        let ids: HashSet<_> = self.bodies.iter().map(Body::id).collect();
        if ids.len() != self.bodies.len() {
            visitor(Link::InvalidValue);
        }
        for constraint in &self.constraints {
            if !constraint.break_force.is_finite()
                || [constraint.body_a, constraint.body_b]
                    .into_iter()
                    .chain(constraint.dependents.iter().copied())
                    .any(|id| !ids.contains(&id))
            {
                visitor(Link::InvalidValue);
            }
        }
        for body in &self.bodies {
            body.visit_links(visitor)
        }
        for joint in &self.joints {
            joint.visit_links(visitor)
        }
    }
}
impl ValidateLinks for ClothSystem<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.validate().is_err() {
            visitor(Link::InvalidValue)
        }
        for cloth in &self.cloths {
            cloth.visit_links(visitor)
        }
    }
}
impl ValidateLinks for System<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::ControllerSelections(values) => values.visit_links(visitor),
            Self::AnimationPartitions(_) => {}
            Self::Collision(value) => value.visit_links(visitor),
            Self::Physics(value) => value.visit_links(visitor),
            Self::Cloth(value) => value.visit_links(visitor),
            Self::TurretGroups(groups) => {
                for group in groups {
                    group.visit_links(visitor);
                }
            }
        }
    }
}
