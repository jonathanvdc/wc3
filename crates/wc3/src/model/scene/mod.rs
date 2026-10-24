//! Portable authored scene features, simulation constraints, and deformation systems.
use crate::model::common::IntoOwned;
use crate::model::error::{Location, PhysicsError, ReferenceError};
/// Compiled particle programmes and their typed host boundary.
pub mod popcorn;
/// Region-local illumination and cookie programs.
pub mod region_lights;
/// World regions, placements, environment, and illumination.
pub mod world;

mod features;
mod physics;
use crate::model::{Error, Link, Scope, ValidateLinks};
pub use features::*;
pub use physics::*;
use std::mem::replace;
/// Complete portable feature graph accompanying common geometry and animation.
#[derive(Clone, Debug, Default, IntoOwned)]
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
        for (id, body) in &self.bodies {
            for (index, &shape) in body.shapes.iter().enumerate() {
                if self.shape(shape).is_none() {
                    return Err(Error::at(
                        Location::for_owner(*id).field("shapes").index(index),
                        if self.scope.contains(shape) {
                            ReferenceError::Unknown {
                                target: shape.into(),
                            }
                        } else {
                            ReferenceError::Foreign {
                                target: shape.into(),
                            }
                        },
                    ));
                }
            }
            if let Some(parent) = body.follow.as_ref().and_then(|follow| follow.parent) {
                if self.body(parent).is_none() {
                    return Err(Error::at(
                        Location::for_owner(*id).field("follow").field("parent"),
                        if self.scope.contains(parent) {
                            ReferenceError::Unknown {
                                target: parent.into(),
                            }
                        } else {
                            ReferenceError::Foreign {
                                target: parent.into(),
                            }
                        },
                    ));
                }
            }
            for (index, value) in body.damping.to_array().into_iter().enumerate() {
                if !value.is_finite() {
                    return Err(Error::at(
                        Location::for_owner(*id).field("damping").index(index),
                        PhysicsError::NonFiniteValue {
                            value: f64::from(value),
                        },
                    ));
                }
            }
            for (field, value) in [
                ("inertia_scale", body.inertia_scale),
                ("gravity_scale", body.gravity_scale),
            ] {
                if !value.is_finite() {
                    return Err(Error::at(
                        Location::for_owner(*id).field(field),
                        PhysicsError::NonFiniteValue {
                            value: f64::from(value),
                        },
                    ));
                }
            }
        }
        for (id, joint) in &self.joints {
            for (index, &body) in joint.bodies.iter().enumerate() {
                if self.body(body).is_none() {
                    return Err(Error::at(
                        Location::for_owner(*id).field("bodies").index(index),
                        if self.scope.contains(body) {
                            ReferenceError::Unknown {
                                target: body.into(),
                            }
                        } else {
                            ReferenceError::Foreign {
                                target: body.into(),
                            }
                        },
                    ));
                }
            }
        }
        for (id, shape) in &self.shapes {
            if !shape.transform.is_finite() {
                return Err(Error::at(
                    Location::for_owner(*id).field("transform"),
                    PhysicsError::NonFiniteMatrix {
                        matrix: shape.transform,
                    },
                ));
            }
            if let Geometry::Triangles {
                positions,
                triangles,
            } = &shape.geometry
            {
                for (index, position) in positions.iter().enumerate() {
                    if !position.is_finite() {
                        return Err(Error::at(
                            Location::for_owner(*id)
                                .field("geometry")
                                .field("positions")
                                .index(index),
                            PhysicsError::NonFinitePosition { position },
                        ));
                    }
                }
                for (index, triangle) in triangles.iter().enumerate() {
                    for (corner, vertex) in triangle.into_iter().enumerate() {
                        if vertex as usize >= positions.len() {
                            return Err(Error::at(
                                Location::for_owner(*id)
                                    .field("geometry")
                                    .field("triangles")
                                    .index(index)
                                    .index(corner),
                                PhysicsError::TriangleVertexOutOfBounds {
                                    vertex,
                                    positions: positions.len(),
                                },
                            ));
                        }
                    }
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
                Error::at(
                    Location::for_owner(id),
                    if self.scope.contains(id) {
                        ReferenceError::Unknown { target: id.into() }
                    } else {
                        ReferenceError::Foreign { target: id.into() }
                    },
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
                Error::at(
                    Location::for_owner(id),
                    if self.scope.contains(id) {
                        ReferenceError::Unknown { target: id.into() }
                    } else {
                        ReferenceError::Foreign { target: id.into() }
                    },
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
                Error::at(
                    Location::for_owner(id),
                    if self.scope.contains(id) {
                        ReferenceError::Unknown { target: id.into() }
                    } else {
                        ReferenceError::Foreign { target: id.into() }
                    },
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
    use glam::Mat4;
    fn shape() -> Shape<'static> {
        Shape {
            geometry: Geometry::Sphere(1.),
            transform: Mat4::ZERO,
            material: ContactMaterial::default(),
            callback_categories: 0,
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
        invalid.transform.x_axis.x = f32::NAN;
        assert!(system.set_shape(id, invalid).is_err());
        assert_eq!(system.shape(id).unwrap().transform.x_axis.x, 0.);
    }
}
