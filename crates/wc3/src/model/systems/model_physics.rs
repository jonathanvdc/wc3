//! Physics bodies, collision shapes, and constraints use hierarchy identities.
use crate::model::common::IntoOwned;
use crate::model::error::{Location, PhysicsError};
use crate::model::values::model::{
    PhysicsBodyKind, PhysicsGameFlags, PhysicsKind, PhysicsMotorMode,
};
use crate::model::PhysicsResourceId;
use crate::model::ValueGroups;
use crate::model::{Error, Link, NodeId, PackedArray, ValidateLinks};
use glam::{Mat4, Vec2, Vec3, Vec4};
/// Physics simulation data, optionally supplied by an external resource.
#[derive(Clone, Debug, IntoOwned)]
pub struct Physics<'a> {
    /// Simulation role.
    pub kind: PhysicsKind,
    /// External physics resource identity.
    pub resource: Option<PhysicsResourceId>,
    /// Rigid bodies and their attached collision shapes.
    pub bodies: Vec<Body<'a>>,
    /// Constraints between body nodes.
    pub joints: Vec<Joint>,
    /// Collision-filter pairs: permitted contact between body nodes.
    pub collision_pairs: Vec<(NodeId, NodeId)>,
    /// Environmental response parameters.
    pub wind_coefficients: Option<Vec3>,
    /// Host-skeleton name hashes allowed to use authored pose-follow factors.
    pub follow_allow_list: PackedArray<'a, u32>,
    /// Environmental vegetation response systems.
    pub vegetation: Vec<Vegetation>,
}
/// A collision body anchored in the common hierarchy.
#[derive(Clone, Debug, IntoOwned)]
pub struct Body<'a> {
    /// Body placement node, including its local body offset.
    pub node: NodeId,
    /// Simulation motion choice.
    pub kind: PhysicsBodyKind,
    /// Gravity multiplier.
    pub gravity_scale: f32,
    /// Inertia multiplier.
    pub inertia_scale: f32,
    /// Linear velocity damping.
    pub linear_damping: f32,
    /// Angular velocity damping.
    pub angular_damping: f32,
    /// Strength of following the animation pose.
    pub follow_factor: f32,
    /// Body hosting an attachment, when present.
    pub parent_body: Option<NodeId>,
    /// Whether child attached bodies are supported.
    pub attachment_host: bool,
    /// Whether this body roots a ragdoll.
    pub ragdoll_root: bool,
    /// Collision geometry attached to this body.
    pub shapes: Vec<Shape<'a>>,
}
/// A local collision shape and contact response.
#[derive(Clone, Debug, IntoOwned)]
pub struct Shape<'a> {
    /// Local affine transform relative to the body.
    pub transform: Mat4,
    /// Shape geometry.
    pub geometry: ShapeGeometry<'a>,
    /// Surface friction coefficient.
    pub friction: f32,
    /// Surface restitution coefficient.
    pub restitution: f32,
    /// Fixture density, with mass units determined by the caller's simulation convention.
    pub density: f32,
    /// Explicit authored mass properties of convex geometry.
    pub mass_properties: Option<MassProperties>,
    /// Game collision flags and extension bits.
    pub game_flags: PhysicsGameFlags,
    /// Collision mask category.
    pub collision_mask: CollisionMask,
}
/// Geometry of a collision primitive or convex polytope.
#[derive(Clone, Debug, IntoOwned)]
pub enum ShapeGeometry<'a> {
    /// Box dimensions as half extents.
    Box(Vec3),
    /// Capsule radius and cylinder height along the local Z axis.
    Capsule {
        /// Radius of the spherical end caps and cylindrical body.
        radius: f32,
        /// Centers of the two end caps in shape coordinates.
        endpoints: [Vec3; 2],
    },
    /// Sphere radius.
    Sphere(f32),
    /// Convex-hull vertices and oriented faces.
    ConvexHull {
        /// Hull vertices in shape coordinates.
        vertices: PackedArray<'a, Vec3>,
        /// Plane equations of the convex boundary.
        planes: PackedArray<'a, Vec4>,
        /// Oriented polygon boundaries, expressed as vertex indices.
        faces: ValueGroups<'a, u32>,
    },
}
/// Constraint frame in a body's local coordinates.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    /// Local X axis.
    pub axis_x: Vec3,
    /// Local Y axis.
    pub axis_y: Vec3,
    /// Local Z axis.
    pub axis_z: Vec3,
    /// Local anchor position.
    pub origin: Vec3,
}
/// Frequency and damping ratio of a constraint spring.
#[derive(Clone, Copy, Debug, Default)]
pub struct Spring {
    /// Natural spring frequency in cycles per second.
    pub frequency_hz: f32,
    /// Damping relative to critical damping.
    pub damping_ratio: f32,
}
/// Constraint motor behavior.
#[derive(Clone, Copy, Debug, Default)]
pub struct Motor {
    /// Motor evaluation mode.
    pub mode: PhysicsMotorMode,
    /// Maximum force or torque supplied by the constraint motor.
    pub maximum_force: f32,
    /// Spring response controlling compliance.
    pub spring: Spring,
}
/// Constraint parameters independent of table-indexed physical payloads.
#[derive(Clone, Debug)]
pub enum Constraint {
    /// Ball-and-socket constraint with friction torque.
    Spherical {
        /// Torque opposing relative angular motion.
        friction_torque: f32,
    },
    /// Cone and twist limits, in radians.
    Shoulder {
        /// Minimum and maximum twist angles in radians.
        twist: [f64; 2],
        /// Maximum swing angle in radians.
        cone: f64,
        /// Motor response for this degree of freedom.
        motor: Motor,
    },
    /// Rigid relative transform with compliant springs.
    Weld {
        /// Translational compliance of the welded constraint.
        linear: Spring,
        /// Rotational compliance of the welded constraint.
        angular: Spring,
    },
    /// Hinge angular range in radians.
    Revolute {
        /// Minimum and maximum hinge angles in radians.
        angles: [f64; 2],
        /// Torque opposing relative angular motion.
        friction_torque: f32,
        /// Motor response for this degree of freedom.
        motor: Motor,
    },
    /// Sliding distance range in model units.
    Prismatic {
        /// Minimum and maximum sliding distances in model units.
        distances: Vec2,
        /// Authored reference offset along the sliding axis.
        reference_translation: f32,
        /// Target sliding speed in model units per second.
        motor_speed: f32,
        /// Force opposing sliding motion.
        friction_force: f32,
        /// Motor response for this degree of freedom.
        motor: Motor,
    },
    /// Anchor distance constraint in model units.
    Distance {
        /// Required anchor separation in model units.
        distance: f32,
        /// Spring response controlling compliance.
        spring: Spring,
    },
}
/// A physical constraint between two body placement nodes.
#[derive(Clone, Debug)]
pub struct Joint {
    /// First body node.
    pub body_a: NodeId,
    /// Second body node.
    pub body_b: NodeId,
    /// Anchor frame in the first body.
    pub frame_a: Frame,
    /// Anchor frame in the second body.
    pub frame_b: Frame,
    /// Actual constraint behavior.
    pub constraint: Constraint,
}
impl ValidateLinks for Body<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.node.visit_links(v);
        self.parent_body.visit_links(v)
    }
}
impl ValidateLinks for Joint {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.body_a.visit_links(v);
        self.body_b.visit_links(v)
    }
}
impl ValidateLinks for Physics<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.resource.visit_links(v);
        self.bodies.visit_links(v);
        self.joints.visit_links(v);
        for (a, b) in &self.collision_pairs {
            a.visit_links(v);
            b.visit_links(v)
        }
    }
}

/// Authored integral geometry quantities used by the physics solver.
#[derive(Clone, Copy, Debug)]
pub struct MassProperties {
    /// Center of the authored volume in shape coordinates.
    pub centroid: Vec3,
    /// Enclosed volume in cubed model units.
    pub volume: f32,
    /// Boundary area in squared model units.
    pub surface_area: f32,
}
/// Vegetation interaction and relaxation controls.
#[derive(Clone, Copy, Debug)]
pub struct Vegetation {
    /// Maximum displacement caused by positional interaction.
    pub position_max_push: f32,
    /// Strength of the positional push response.
    pub position_push_amount: f32,
    /// Rate of returning positional displacement toward rest.
    pub position_relax_speed: f32,
    /// Maximum displacement caused by velocity interaction.
    pub velocity_max_push: f32,
    /// Strength of the velocity-dependent push response.
    pub velocity_speed: f32,
    /// Squared distance below which interaction is suppressed.
    pub minimum_push_distance_squared: f32,
}

impl Physics<'_> {
    /// Checks the physical parameters and relationships belonging to this simulation system.
    pub fn validate(&self) -> Result<(), Error> {
        use std::collections::{BTreeMap, BTreeSet};
        let bodies: BTreeMap<_, _> = self.bodies.iter().map(|body| (body.node, body)).collect();
        if bodies.len() != self.bodies.len() {
            let mut seen = BTreeSet::new();
            let body = self
                .bodies
                .iter()
                .find(|body| !seen.insert(body.node))
                .unwrap()
                .node;
            return Err(Error::at(
                Location::for_owner(body),
                PhysicsError::DuplicateBody { body },
            ));
        }
        for body in &self.bodies {
            for (field, value) in [
                ("gravity_scale", body.gravity_scale),
                ("inertia_scale", body.inertia_scale),
                ("linear_damping", body.linear_damping),
                ("angular_damping", body.angular_damping),
                ("follow_factor", body.follow_factor),
            ] {
                finite_physics(f64::from(value), || {
                    Location::for_owner(body.node).field(field)
                })?;
            }
            let mut chain = BTreeSet::new();
            chain.insert(body.node);
            let mut parent = body.parent_body;
            while let Some(id) = parent {
                if !chain.insert(id) {
                    return Err(Error::at(
                        Location::for_owner(body.node).field("parent_body"),
                        PhysicsError::BodyCycle { body: id },
                    ));
                }
                parent = bodies
                    .get(&id)
                    .ok_or_else(|| {
                        Error::at(
                            Location::for_owner(body.node).field("parent_body"),
                            PhysicsError::MissingBody { body: id },
                        )
                    })?
                    .parent_body;
            }
            for (shape_index, shape) in body.shapes.iter().enumerate() {
                let at = || {
                    Location::for_owner(body.node)
                        .field("shapes")
                        .index(shape_index)
                };
                if !shape.transform.is_finite() {
                    return Err(Error::at(
                        at().field("transform"),
                        PhysicsError::NonFiniteMatrix {
                            matrix: shape.transform,
                        },
                    ));
                }
                for (field, value) in [
                    ("friction", shape.friction),
                    ("restitution", shape.restitution),
                    ("density", shape.density),
                ] {
                    nonnegative_physics(f64::from(value), || at().field(field))?;
                }
                let last_row = Vec4::new(
                    shape.transform.x_axis.w,
                    shape.transform.y_axis.w,
                    shape.transform.z_axis.w,
                    shape.transform.w_axis.w,
                );
                if last_row != Vec4::W {
                    return Err(Error::at(
                        at().field("transform"),
                        PhysicsError::NonAffineMatrix { last_row },
                    ));
                }
                if let Some(mass) = shape.mass_properties {
                    if !mass.centroid.is_finite() {
                        return Err(Error::at(
                            at().field("mass_properties").field("centroid"),
                            PhysicsError::NonFinitePosition {
                                position: mass.centroid,
                            },
                        ));
                    }
                    for (field, value) in
                        [("volume", mass.volume), ("surface_area", mass.surface_area)]
                    {
                        nonnegative_physics(f64::from(value), || {
                            at().field("mass_properties").field(field)
                        })?;
                    }
                }
                match &shape.geometry {
                    ShapeGeometry::Box(extents) => {
                        for (component, value) in extents.to_array().into_iter().enumerate() {
                            nonnegative_physics(f64::from(value), || {
                                at().field("geometry").index(0).index(component)
                            })?;
                        }
                    }
                    ShapeGeometry::Sphere(radius) => {
                        nonnegative_physics(f64::from(*radius), || {
                            at().field("geometry").index(0)
                        })?;
                    }
                    ShapeGeometry::Capsule { radius, endpoints } => {
                        nonnegative_physics(f64::from(*radius), || {
                            at().field("geometry").field("radius")
                        })?;
                        for (index, position) in endpoints.iter().copied().enumerate() {
                            if !position.is_finite() {
                                return Err(Error::at(
                                    at().field("geometry").field("endpoints").index(index),
                                    PhysicsError::NonFinitePosition { position },
                                ));
                            }
                        }
                    }
                    ShapeGeometry::ConvexHull {
                        vertices,
                        planes,
                        faces,
                    } => {
                        if let Some((index, position)) = vertices
                            .iter()
                            .enumerate()
                            .find(|(_, position)| !position.is_finite())
                        {
                            return Err(Error::at(
                                at().field("geometry").field("vertices").index(index),
                                PhysicsError::NonFinitePosition { position },
                            ));
                        }
                        for (index, plane) in planes.iter().enumerate() {
                            for (component, value) in plane.to_array().into_iter().enumerate() {
                                finite_physics(f64::from(value), || {
                                    at().field("geometry")
                                        .field("planes")
                                        .index(index)
                                        .index(component)
                                })?;
                            }
                        }
                        if faces.len() != planes.len() {
                            return Err(Error::at(
                                at().field("geometry"),
                                PhysicsError::HullPlaneCount {
                                    faces: faces.len(),
                                    planes: planes.len(),
                                },
                            ));
                        }
                        let mut edges = BTreeSet::new();
                        for (face_index, face) in faces.iter().enumerate() {
                            let face_at =
                                || at().field("geometry").field("faces").index(face_index);
                            if face.len() < 3 {
                                return Err(Error::at(
                                    face_at(),
                                    PhysicsError::HullFaceVertexCount { actual: face.len() },
                                ));
                            }
                            for (component, vertex) in face.iter().enumerate() {
                                if vertex as usize >= vertices.len() {
                                    return Err(Error::at(
                                        face_at().index(component),
                                        PhysicsError::HullVertexOutOfBounds {
                                            vertex,
                                            vertices: vertices.len(),
                                        },
                                    ));
                                }
                            }
                            for index in 0..face.len() {
                                let a = face.get(index).expect("checked face index");
                                let b = face
                                    .get((index + 1) % face.len())
                                    .expect("checked face index");
                                if a == b {
                                    return Err(Error::at(
                                        face_at().index(index),
                                        PhysicsError::HullDegenerateEdge { vertex: a },
                                    ));
                                }
                                if !edges.insert((a, b)) {
                                    return Err(Error::at(
                                        face_at().index(index),
                                        PhysicsError::HullDuplicateEdge { from: a, to: b },
                                    ));
                                }
                            }
                        }
                        if let Some(&(from, to)) =
                            edges.iter().find(|&&(a, b)| !edges.contains(&(b, a)))
                        {
                            return Err(Error::at(
                                at().field("geometry").field("faces"),
                                PhysicsError::HullOpenEdge { from, to },
                            ));
                        }
                    }
                }
            }
        }
        for (index, joint) in self.joints.iter().enumerate() {
            let at = || Location::new().field("joints").index(index);
            joint
                .constraint
                .validate()
                .map_err(|error| error.with_context(at().field("constraint")))?;
            for (field, body) in [("body_a", joint.body_a), ("body_b", joint.body_b)] {
                if !bodies.contains_key(&body) {
                    return Err(Error::at(
                        at().field(field),
                        PhysicsError::MissingBody { body },
                    ));
                }
            }
            for (field, frame) in [("frame_a", joint.frame_a), ("frame_b", joint.frame_b)] {
                for (component, vector) in [
                    ("axis_x", frame.axis_x),
                    ("axis_y", frame.axis_y),
                    ("axis_z", frame.axis_z),
                    ("origin", frame.origin),
                ] {
                    for (index, value) in vector.to_array().into_iter().enumerate() {
                        finite_physics(f64::from(value), || {
                            at().field(field).field(component).index(index)
                        })?;
                    }
                }
            }
        }
        for (pair, &(a, b)) in self.collision_pairs.iter().enumerate() {
            for (endpoint, body) in [a, b].into_iter().enumerate() {
                if !bodies.contains_key(&body) {
                    return Err(Error::at(
                        Location::new()
                            .field("collision_pairs")
                            .index(pair)
                            .index(endpoint),
                        PhysicsError::MissingBody { body },
                    ));
                }
            }
        }
        if let Some(wind) = self.wind_coefficients {
            for (component, value) in wind.to_array().into_iter().enumerate() {
                finite_physics(f64::from(value), || {
                    Location::new().field("wind_coefficients").index(component)
                })?;
            }
        }
        for (index, vegetation) in self.vegetation.iter().enumerate() {
            for (field, value) in [
                ("position_max_push", vegetation.position_max_push),
                ("position_push_amount", vegetation.position_push_amount),
                ("position_relax_speed", vegetation.position_relax_speed),
                ("velocity_max_push", vegetation.velocity_max_push),
                ("velocity_speed", vegetation.velocity_speed),
                (
                    "minimum_push_distance_squared",
                    vegetation.minimum_push_distance_squared,
                ),
            ] {
                finite_physics(f64::from(value), || {
                    Location::new()
                        .field("vegetation")
                        .index(index)
                        .field(field)
                })?;
            }
        }
        Ok(())
    }
}

fn finite_physics(value: f64, location: impl FnOnce() -> Location) -> Result<(), Error> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(Error::at(
            location(),
            PhysicsError::NonFiniteValue { value },
        ))
    }
}

fn nonnegative_physics(value: f64, location: impl FnOnce() -> Location) -> Result<(), Error> {
    if !value.is_finite() {
        Err(Error::at(
            location(),
            PhysicsError::NonFiniteValue { value },
        ))
    } else if value < 0. {
        Err(Error::at(location(), PhysicsError::NegativeValue { value }))
    } else {
        Ok(())
    }
}

fn validate_spring(value: &Spring) -> Result<(), Error> {
    finite_physics(f64::from(value.frequency_hz), || {
        Location::new().field("frequency_hz")
    })?;
    finite_physics(f64::from(value.damping_ratio), || {
        Location::new().field("damping_ratio")
    })
}

fn validate_motor(value: &Motor) -> Result<(), Error> {
    finite_physics(f64::from(value.maximum_force), || {
        Location::new().field("maximum_force")
    })?;
    validate_spring(&value.spring)
        .map_err(|error| error.with_context(Location::new().field("spring")))
}

impl Constraint {
    fn validate(&self) -> Result<(), Error> {
        match self {
            Self::Spherical { friction_torque } => {
                finite_physics(f64::from(*friction_torque), || {
                    Location::new().field("friction_torque")
                })
            }
            Self::Shoulder { twist, cone, motor } => {
                for (index, &value) in twist.iter().enumerate() {
                    finite_physics(value, || Location::new().field("twist").index(index))?;
                }
                finite_physics(*cone, || Location::new().field("cone"))?;
                validate_motor(motor)
                    .map_err(|error| error.with_context(Location::new().field("motor")))
            }
            Self::Weld { linear, angular } => {
                validate_spring(linear)
                    .map_err(|error| error.with_context(Location::new().field("linear")))?;
                validate_spring(angular)
                    .map_err(|error| error.with_context(Location::new().field("angular")))
            }
            Self::Revolute {
                angles,
                friction_torque,
                motor,
            } => {
                for (index, &value) in angles.iter().enumerate() {
                    finite_physics(value, || Location::new().field("angles").index(index))?;
                }
                finite_physics(f64::from(*friction_torque), || {
                    Location::new().field("friction_torque")
                })?;
                validate_motor(motor)
                    .map_err(|error| error.with_context(Location::new().field("motor")))
            }
            Self::Prismatic {
                distances,
                reference_translation,
                motor_speed,
                friction_force,
                motor,
            } => {
                for (index, value) in distances.to_array().into_iter().enumerate() {
                    finite_physics(f64::from(value), || {
                        Location::new().field("distances").index(index)
                    })?;
                }
                for (field, value) in [
                    ("reference_translation", reference_translation),
                    ("motor_speed", motor_speed),
                    ("friction_force", friction_force),
                ] {
                    finite_physics(f64::from(*value), || Location::new().field(field))?;
                }
                validate_motor(motor)
                    .map_err(|error| error.with_context(Location::new().field("motor")))
            }
            Self::Distance { distance, spring } => {
                finite_physics(f64::from(*distance), || Location::new().field("distance"))?;
                validate_spring(spring)
                    .map_err(|error| error.with_context(Location::new().field("spring")))
            }
        }
    }
}

use bitfield::bitfield;
bitfield! {
    /// Authored contact-category selection mask.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct CollisionMask(u32);
    /// Complete retained category mask.
    pub bits, _:31,0;
}
impl CollisionMask {
    /// Retains every authored category bit.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
