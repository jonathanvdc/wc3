//! One simulation domain permits common, WoW, and SC2 bodies to constrain each other.
/// Soft deformation bodies, effects, pivots, and preparation settings.
pub mod soft;
pub use soft::{SoftBody, SoftBodyId, SoftBodyObject, SoftEffectId, SoftLumpId};
mod rig;
pub use rig::*;
mod values;
pub use values::*;
mod cloth;
pub use cloth::*;
mod bodies;
use crate::model::semantic::profiles::m2::values::PhysicsGameFlags;
use crate::model::semantic::scene::ContactMaterial;
use crate::model::semantic::Scope;
use crate::model::semantic::{Error, Id, Link, ValidateLinks, Values};
pub use bodies::*;
use std::collections::BTreeSet;
/// Identity marker for authored collision geometry.
pub enum ShapeObject {}
/// Collision geometry owned by one simulation domain.
pub type ShapeId = Id<ShapeObject>;
/// Identity marker for one body in a simulation domain.
pub enum BodyObject {}
/// Body owned by one simulation domain, independent of its game extension.
pub type BodyId = Id<BodyObject>;
/// Identity marker for a constraint in one simulation domain.
pub enum JointObject {}
/// Stable constraint identity independent of authored order.
pub type JointId = Id<JointObject>;
/// Authored collider parameters; changing its interpretation retains other preparation data.
#[derive(Clone, Debug)]
pub struct Shape<'a> {
    /// Selected collision interpretation.
    pub kind: ShapeKind,
    /// Shape-to-body transform.
    pub transform: [[f32; 4]; 4],
    /// Box half dimensions in shape coordinates.
    pub half_extents: [f32; 3],
    /// Sphere, capsule, or cylinder radius.
    pub radius: f32,
    /// Cylinder length or capsule cap-center separation.
    pub length: f32,
    /// Convex hull points or explicitly authored capsule centers.
    pub points: Values<'a, [f32; 3]>,
    /// Triangle-mesh positions in shape coordinates.
    pub vertices: Values<'a, [f32; 3]>,
    /// Triangle-mesh connectivity.
    pub triangles: Values<'a, [u32; 3]>,
    /// Contact response assigned to this geometry.
    pub material: ContactMaterial,
    /// World of Warcraft contact-callback controls, including unnamed bits.
    pub contact_flags: PhysicsGameFlags,
}
/// One authored two-body constraint, including motors and springs.
#[derive(Clone, Debug)]
pub struct Joint {
    /// Bodies on either side of the constraint.
    pub bodies: [BodyId; 2],
    /// Constraint interpretation.
    pub kind: JointKind,
    /// Joint frames relative to the corresponding body node.
    pub frames: [[[f32; 4]; 4]; 2],
    /// Permits collisions between connected bodies.
    pub collide_connected: bool,
    /// Activates stored movement limits.
    pub limit_enabled: bool,
    /// Lower and upper angular or translational limits.
    pub limits: [f32; 2],
    /// Cone-twist angular limit.
    pub cone: f32,
    /// Friction convention.
    pub friction: Friction,
    /// Friction strength.
    pub friction_amount: f32,
    /// Angular spring frequency and damping.
    pub angular_spring: [f32; 2],
    /// Translational spring frequency and damping.
    pub linear_spring: [f32; 2],
    /// Resting separation for distance constraints.
    pub rest_length: f32,
    /// Breaking-force and breaking-torque thresholds; zero disables the corresponding threshold.
    pub breaking: [f32; 2],
    /// Motor convention, when one is authored.
    pub motor: Option<Motor>,
    /// Motor force or torque limit.
    pub max_motor_force: Option<f32>,
    /// Motor target speed.
    pub motor_speed: Option<f32>,
    /// Reference translation for prismatic constraints.
    pub reference_translation: Option<f32>,
    /// Protects manual tuning from bulk preparation.
    pub locked: bool,
}
/// Checked simulation domain with stable collider and body identities.
#[derive(Clone, Debug, Default)]
pub struct Physics<'a> {
    scope: Scope,
    shapes: Vec<(ShapeId, Shape<'a>)>,
    bodies: Vec<(BodyId, Body)>,
    cloth_colliders: Vec<(ClothColliderId, ClothCollider)>,
    cloth: Vec<(ClothId, Cloth<'a>)>,
    rigs: Vec<(RigId, Rig<'a>)>,
    soft_bodies: Vec<(SoftBodyId, SoftBody<'a>)>,
    reserved_soft_bodies: BTreeSet<SoftBodyId>,
    joints: Vec<(JointId, Joint)>,
}
impl<'a> Physics<'a> {
    /// Reserves a deformation identity for forward relationships in an unpublished draft.
    pub fn reserve_soft_body(&mut self) -> SoftBodyId {
        let id = self.scope.allocate();
        self.reserved_soft_bodies.insert(id);
        id
    }
    /// Adds a complete deformation definition.
    pub fn add_soft_body(&mut self, value: SoftBody<'a>) -> SoftBodyId {
        let id = self.scope.allocate();
        self.soft_bodies.push((id, value));
        id
    }
    /// Fills a reservation or updates an existing deformation definition without changing its identity.
    pub fn set_soft_body(&mut self, id: SoftBodyId, value: SoftBody<'a>) -> Result<(), Error> {
        if self.reserved_soft_bodies.remove(&id) {
            self.soft_bodies.push((id, value));
            return Ok(());
        }
        let row = self
            .soft_bodies
            .iter_mut()
            .find(|(key, _)| *key == id)
            .ok_or_else(|| {
                Error::new(
                    "wem.physics.soft",
                    "deformation definition belongs to another domain",
                )
            })?;
        row.1 = value;
        Ok(())
    }
    /// Borrows one domain-owned deformation definition.
    pub fn soft_body(&self, id: SoftBodyId) -> Option<&SoftBody<'a>> {
        self.soft_bodies
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Iterates complete deformation definitions in authored order.
    pub fn soft_bodies(&self) -> impl ExactSizeIterator<Item = (SoftBodyId, &SoftBody<'a>)> {
        self.soft_bodies.iter().map(|(id, value)| (*id, value))
    }
    /// Adds a constraint between domain-owned bodies.
    pub fn add_joint(&mut self, joint: Joint) -> JointId {
        let id = self.scope.allocate();
        self.joints.push((id, joint));
        id
    }
    /// Borrows a constraint by its stable identity.
    pub fn joint(&self, id: JointId) -> Option<&Joint> {
        self.joints
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Iterates constraints in authored order.
    pub fn joints(&self) -> impl ExactSizeIterator<Item = (JointId, &Joint)> {
        self.joints.iter().map(|(id, value)| (*id, value))
    }
    /// Replaces a constraint without changing references to its identity.
    pub fn set_joint(&mut self, id: JointId, joint: Joint) -> Result<(), Error> {
        let row = self
            .joints
            .iter_mut()
            .find(|(key, _)| *key == id)
            .ok_or_else(|| {
                Error::new("wem.physics.joint", "constraint belongs to another domain")
            })?;
        row.1 = joint;
        Ok(())
    }
    /// Adds independently authored collision geometry.
    pub fn add_shape(&mut self, shape: Shape<'a>) -> ShapeId {
        let id = self.scope.allocate();
        self.shapes.push((id, shape));
        id
    }
    /// Adds a body to this simulation domain.
    pub fn add_body(&mut self, body: Body) -> BodyId {
        let id = self.scope.allocate();
        self.bodies.push((id, body));
        id
    }
    /// Replaces collision geometry while preserving all body relationships to its identity.
    pub fn set_shape(&mut self, id: ShapeId, shape: Shape<'a>) -> Result<(), Error> {
        let entry = self
            .shapes
            .iter_mut()
            .find(|(key, _)| *key == id)
            .ok_or_else(|| Error::new("wem.physics.shape", "collider belongs to another domain"))?;
        entry.1 = shape;
        Ok(())
    }
    /// Replaces body settings while preserving its constraint and following relationships.
    pub fn set_body(&mut self, id: BodyId, body: Body) -> Result<(), Error> {
        let entry = self
            .bodies
            .iter_mut()
            .find(|(key, _)| *key == id)
            .ok_or_else(|| Error::new("wem.physics.body", "body belongs to another domain"))?;
        entry.1 = body;
        Ok(())
    }
    /// Borrows collision geometry by its domain-owned identity.
    pub fn shape(&self, id: ShapeId) -> Option<&Shape<'a>> {
        self.shapes
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Borrows a body by its domain-owned identity.
    pub fn body(&self, id: BodyId) -> Option<&Body> {
        self.bodies
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Iterates collision geometry in authored order.
    pub fn shapes(&self) -> impl ExactSizeIterator<Item = (ShapeId, &Shape<'a>)> {
        self.shapes.iter().map(|(id, value)| (*id, value))
    }
    /// Iterates bodies in authored order.
    pub fn bodies(&self) -> impl ExactSizeIterator<Item = (BodyId, &Body)> {
        self.bodies.iter().map(|(id, value)| (*id, value))
    }
    /// Checks local body/shape relationships before graph publication.
    pub fn validate(&self) -> Result<(), Error> {
        if !self.reserved_soft_bodies.is_empty() {
            return Err(Error::new(
                "wem.physics.soft",
                "deformation reservations remain unfilled",
            ));
        }
        for (_, body) in &self.soft_bodies {
            body.validate()?;
            if body
                .colliders
                .iter()
                .any(|id| self.cloth_collider(*id).is_none())
            {
                return Err(Error::new(
                    "wem.physics.soft",
                    "contact collider belongs to another domain",
                ));
            }
        }
        if self.bodies.iter().any(|(_, body)| {
            body.shapes.iter().any(|id| self.shape(*id).is_none())
                || body
                    .wow
                    .as_ref()
                    .and_then(|value| value.parent)
                    .is_some_and(|id| self.body(id).is_none())
        }) || self
            .joints
            .iter()
            .any(|(_, joint)| joint.bodies.iter().any(|id| self.body(*id).is_none()))
        {
            return Err(Error::new(
                "wem.physics",
                "body, collider, or parent belongs to another simulation domain",
            ));
        }
        for (_, body) in &self.bodies {
            if body
                .damping
                .iter()
                .chain([&body.inertia_scale, &body.gravity_scale])
                .any(|value| !value.is_finite())
                || body
                    .wow
                    .as_ref()
                    .is_some_and(|wow| !wow.follow_factor.is_finite())
            {
                return Err(Error::new("wem.physics.body", "nonfinite body response"));
            }
        }
        for (_, joint) in &self.joints {
            if joint
                .frames
                .iter()
                .flatten()
                .flatten()
                .chain(&joint.limits)
                .chain(&joint.angular_spring)
                .chain(&joint.linear_spring)
                .chain(&joint.breaking)
                .chain([&joint.cone, &joint.friction_amount, &joint.rest_length])
                .chain(joint.max_motor_force.iter())
                .chain(joint.motor_speed.iter())
                .chain(joint.reference_translation.iter())
                .any(|value| !value.is_finite())
            {
                return Err(Error::new(
                    "wem.physics.joint",
                    "nonfinite constraint response",
                ));
            }
        }
        for (_, shape) in &self.shapes {
            if shape
                .transform
                .iter()
                .flatten()
                .chain(&shape.half_extents)
                .chain([
                    &shape.radius,
                    &shape.length,
                    &shape.material.density,
                    &shape.material.friction,
                    &shape.material.restitution,
                ])
                .any(|v| !v.is_finite())
                || shape
                    .points
                    .iter()
                    .chain(shape.vertices.iter())
                    .flatten()
                    .any(|v| !v.is_finite())
                || shape
                    .triangles
                    .iter()
                    .flatten()
                    .any(|index| index as usize >= shape.vertices.len())
            {
                return Err(Error::new(
                    "wem.physics.shape",
                    "invalid collision geometry",
                ));
            }
        }
        for (_, rig) in &self.rigs {
            if rig.bodies.iter().any(|id| self.body(*id).is_none()) {
                return Err(Error::new(
                    "wem.physics.rig",
                    "body belongs to another simulation domain",
                ));
            }
        }
        self.validate_cloth()?;
        Ok(())
    }
}
impl ValidateLinks for Physics<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for (_, body) in &self.bodies {
            body.node.visit_links(visitor);
            body.simulates.visit_links(visitor);
        }
        for (_, collider) in &self.cloth_colliders {
            collider.visit_links(visitor);
        }
        for (_, cloth) in &self.cloth {
            cloth.visit_links(visitor);
        }
        for (_, rig) in &self.rigs {
            rig.visit_links(visitor);
        }
        for (_, body) in &self.soft_bodies {
            body.visit_links(visitor);
        }
        if self.validate().is_err() {
            visitor(Link::InvalidValue)
        }
    }
}
