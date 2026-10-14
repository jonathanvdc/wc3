//! Portable authored scene features, simulation constraints, and deformation systems.
mod features;
mod physics;
use crate::model::semantic::{Error, Link, Scope, ValidateLinks};
pub use features::*;
pub use physics::*;
use std::mem::replace;
/// Complete portable feature graph accompanying common geometry and animation.
#[derive(Clone, Debug, Default)]
pub struct PhysicsSystem<'a> {
    /// Collision geometry shared by simulation bodies.
    shapes: Vec<(ShapeId, Shape<'a>)>,
    /// Dynamic, kinematic, and static simulation bodies.
    bodies: Vec<(BodyId, Body)>,
    /// Body constraints.
    joints: Vec<(JointId, Joint)>,
    scope: Scope,
}
impl ValidateLinks for PhysicsSystem<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for (_, body) in &self.bodies {
            body.node.visit_links(visitor);
            body.simulates.visit_links(visitor);
        }
    }
}
impl<'a> PhysicsSystem<'a> {
    /// Validates local collision geometry and constraint relationships.
    pub fn validate(&self) -> Result<(), Error> {
        for (_, body) in &self.bodies {
            if body.shapes.iter().any(|shape| self.shape(*shape).is_none())
                || body.follow.as_ref().is_some_and(|follow| {
                    follow
                        .parent
                        .is_some_and(|parent| self.body(parent).is_none())
                })
            {
                return Err(Error::new(
                    "physics.bodies",
                    "shape or following-body identity does not exist",
                ));
            }
            if body
                .damping
                .iter()
                .chain([&body.inertia_scale, &body.gravity_scale])
                .any(|v| !v.is_finite())
            {
                return Err(Error::new(
                    "physics.bodies",
                    "simulation parameters must be finite",
                ));
            }
        }
        for (_, joint) in &self.joints {
            if joint.bodies.iter().any(|body| self.body(*body).is_none()) {
                return Err(Error::new(
                    "physics.joints",
                    "constrained body does not exist",
                ));
            }
        }
        for (_, shape) in &self.shapes {
            if shape.transform.iter().flatten().any(|v| !v.is_finite()) {
                return Err(Error::new(
                    "physics.shapes",
                    "shape transform must be finite",
                ));
            }
            if let Geometry::Triangles {
                positions,
                triangles,
            } = &shape.geometry
            {
                if positions.iter().flatten().any(|v| !v.is_finite())
                    || triangles
                        .iter()
                        .flatten()
                        .any(|index| index as usize >= positions.len())
                {
                    return Err(Error::new(
                        "physics.shapes",
                        "collision triangle geometry is invalid",
                    ));
                }
            }
        }
        Ok(())
    }
}

impl<'a> PhysicsSystem<'a> {
    /// Iterates authored shapes with their scoped identities.
    pub fn shapes(&self) -> impl ExactSizeIterator<Item = (ShapeId, &Shape<'a>)> {
        self.shapes.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves an identity owned by this simulation system.
    pub fn shape(&self, id: ShapeId) -> Option<&Shape<'a>> {
        self.shapes
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Adds a checked authored object to the simulation system.
    pub fn add_shape(&mut self, value: Shape<'a>) -> Result<ShapeId, Error> {
        let id = self.scope.allocate();
        self.shapes.push((id, value));
        if let Err(error) = self.validate() {
            self.shapes.pop();
            return Err(error);
        }
        Ok(id)
    }
    /// Replaces an existing object after validating its relationships and values.
    pub fn set_shape(&mut self, id: ShapeId, value: Shape<'a>) -> Result<(), Error> {
        let index = self
            .shapes
            .iter()
            .position(|(key, _)| *key == id)
            .ok_or_else(|| {
                Error::new(
                    "physics.shapes",
                    "identity belongs to a different simulation system or does not exist",
                )
            })?;
        let old = replace(&mut self.shapes[index].1, value);
        if let Err(error) = self.validate() {
            self.shapes[index].1 = old;
            return Err(error);
        }
        Ok(())
    }
}

impl<'a> PhysicsSystem<'a> {
    /// Iterates authored bodies with their scoped identities.
    pub fn bodies(&self) -> impl ExactSizeIterator<Item = (BodyId, &Body)> {
        self.bodies.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves an identity owned by this simulation system.
    pub fn body(&self, id: BodyId) -> Option<&Body> {
        self.bodies
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Adds a checked authored object to the simulation system.
    pub fn add_body(&mut self, value: Body) -> Result<BodyId, Error> {
        let id = self.scope.allocate();
        self.bodies.push((id, value));
        if let Err(error) = self.validate() {
            self.bodies.pop();
            return Err(error);
        }
        Ok(id)
    }
    /// Replaces an existing object after validating its relationships and values.
    pub fn set_body(&mut self, id: BodyId, value: Body) -> Result<(), Error> {
        let index = self
            .bodies
            .iter()
            .position(|(key, _)| *key == id)
            .ok_or_else(|| {
                Error::new(
                    "physics.bodies",
                    "identity belongs to a different simulation system or does not exist",
                )
            })?;
        let old = replace(&mut self.bodies[index].1, value);
        if let Err(error) = self.validate() {
            self.bodies[index].1 = old;
            return Err(error);
        }
        Ok(())
    }
}

impl<'a> PhysicsSystem<'a> {
    /// Iterates authored joints with their scoped identities.
    pub fn joints(&self) -> impl ExactSizeIterator<Item = (JointId, &Joint)> {
        self.joints.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves an identity owned by this simulation system.
    pub fn joint(&self, id: JointId) -> Option<&Joint> {
        self.joints
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Adds a checked authored object to the simulation system.
    pub fn add_joint(&mut self, value: Joint) -> Result<JointId, Error> {
        let id = self.scope.allocate();
        self.joints.push((id, value));
        if let Err(error) = self.validate() {
            self.joints.pop();
            return Err(error);
        }
        Ok(id)
    }
    /// Replaces an existing object after validating its relationships and values.
    pub fn set_joint(&mut self, id: JointId, value: Joint) -> Result<(), Error> {
        let index = self
            .joints
            .iter()
            .position(|(key, _)| *key == id)
            .ok_or_else(|| {
                Error::new(
                    "physics.joints",
                    "identity belongs to a different simulation system or does not exist",
                )
            })?;
        let old = replace(&mut self.joints[index].1, value);
        if let Err(error) = self.validate() {
            self.joints[index].1 = old;
            return Err(error);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn shape() -> Shape<'static> {
        Shape {
            geometry: Geometry::Sphere(1.),
            transform: [[0.; 4]; 4],
            material: ContactMaterial::default(),
            game_controls: 0,
        }
    }
    #[test]
    fn unrelated_system_identities_are_not_interchangeable() {
        let mut first = PhysicsSystem::default();
        let mut second = PhysicsSystem::default();
        let first_id = first.add_shape(shape()).unwrap();
        let second_id = second.add_shape(shape()).unwrap();
        assert_ne!(first_id, second_id);
        assert!(first.shape(second_id).is_none());
        assert!(first.set_shape(second_id, shape()).is_err());
        assert_eq!(first.shapes().len(), 1);
    }
    #[test]
    fn invalid_replacements_leave_the_original_object_intact() {
        let mut system = PhysicsSystem::default();
        let id = system.add_shape(shape()).unwrap();
        let mut invalid = shape();
        invalid.transform[0][0] = f32::NAN;
        assert!(system.set_shape(id, invalid).is_err());
        assert_eq!(system.shape(id).unwrap().transform[0][0], 0.);
    }
}
