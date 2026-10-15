//! Physics bodies, collision shapes, and constraints use hierarchy identities.
use super::values::{PhysicsBodyKind, PhysicsGameFlags, PhysicsKind, PhysicsMotorMode};
use crate::model::semantic::ValueGroups;
use crate::model::semantic::{Error, Link, NodeId, Resource, ValidateLinks, ZeroVec};
/// Physics simulation data, optionally supplied by an external resource.
#[derive(Clone, Debug)]
pub struct Physics<'a> {
    /// Simulation role.
    pub kind: PhysicsKind,
    /// External physics resource identity.
    pub resource: Option<Resource<'a>>,
    /// Rigid bodies and their attached collision shapes.
    pub bodies: Vec<Body<'a>>,
    /// Constraints between body nodes.
    pub joints: Vec<Joint>,
    /// Collision-filter pairs: permitted contact between body nodes.
    pub collision_pairs: Vec<(NodeId, NodeId)>,
    /// Environmental response parameters.
    pub wind_coefficients: Option<[f32; 3]>,
    /// Host-skeleton name hashes allowed to use authored pose-follow factors.
    pub follow_allow_list: ZeroVec<'a, u32>,
    /// Environmental vegetation response systems.
    pub vegetation: Vec<Vegetation>,
}
/// A collision body anchored in the common hierarchy.
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct Shape<'a> {
    /// Local affine transform relative to the body.
    pub transform: [[f32; 4]; 4],
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
#[derive(Clone, Debug)]
pub enum ShapeGeometry<'a> {
    /// Box dimensions as half extents.
    Box([f32; 3]),
    /// Capsule radius and cylinder height along the local Z axis.
    Capsule {
        /// Radius of the spherical end caps and cylindrical body.
        radius: f32,
        /// Centers of the two end caps in shape coordinates.
        endpoints: [[f32; 3]; 2],
    },
    /// Sphere radius.
    Sphere(f32),
    /// Convex-hull vertices and oriented faces.
    ConvexHull {
        /// Hull vertices in shape coordinates.
        vertices: ZeroVec<'a, [f32; 3]>,
        /// Plane equations of the convex boundary.
        planes: ZeroVec<'a, [f32; 4]>,
        /// Oriented polygon boundaries, expressed as vertex indices.
        faces: ValueGroups<'a, u32>,
    },
}
/// Constraint frame in a body's local coordinates.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    /// Local X axis.
    pub axis_x: [f32; 3],
    /// Local Y axis.
    pub axis_y: [f32; 3],
    /// Local Z axis.
    pub axis_z: [f32; 3],
    /// Local anchor position.
    pub origin: [f32; 3],
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
        distances: [f32; 2],
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
    pub centroid: [f32; 3],
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
        use crate::model::semantic::Error;
        use std::collections::{BTreeMap, BTreeSet};
        let bodies: BTreeMap<_, _> = self.bodies.iter().map(|body| (body.node, body)).collect();
        if bodies.len() != self.bodies.len() {
            return Err(Error::new(
                "physics.bodies",
                "body placements must have distinct identities",
            ));
        }
        for body in &self.bodies {
            if [
                body.gravity_scale,
                body.inertia_scale,
                body.linear_damping,
                body.angular_damping,
                body.follow_factor,
            ]
            .into_iter()
            .any(|v| !v.is_finite())
            {
                return Err(Error::new(
                    "physics.body",
                    "body coefficients must be finite",
                ));
            }
            let mut chain = BTreeSet::new();
            chain.insert(body.node);
            let mut parent = body.parent_body;
            while let Some(id) = parent {
                if !chain.insert(id) {
                    return Err(Error::new(
                        "physics.body.parent",
                        "body following contains a cycle",
                    ));
                }
                parent = bodies
                    .get(&id)
                    .ok_or_else(|| Error::new("physics.body.parent", "parent body is absent"))?
                    .parent_body;
            }
            for shape in &body.shapes {
                if shape.transform.iter().flatten().any(|v| !v.is_finite())
                    || [shape.friction, shape.restitution, shape.density]
                        .into_iter()
                        .any(|v| !v.is_finite() || v < 0.)
                {
                    return Err(Error::new(
                        "physics.shape",
                        "fixture parameters must be finite and contact coefficients nonnegative",
                    ));
                }
                if shape.transform[0][3] != 0.
                    || shape.transform[1][3] != 0.
                    || shape.transform[2][3] != 0.
                    || shape.transform[3][3] != 1.
                {
                    return Err(Error::new(
                        "physics.shape.transform",
                        "fixture transform must be affine",
                    ));
                }
                if let Some(mass) = shape.mass_properties {
                    if mass.centroid.iter().any(|v| !v.is_finite())
                        || !mass.volume.is_finite()
                        || mass.volume < 0.
                        || !mass.surface_area.is_finite()
                        || mass.surface_area < 0.
                    {
                        return Err(Error::new(
                            "physics.shape.mass",
                            "invalid authored mass properties",
                        ));
                    }
                }
                match &shape.geometry {
                    ShapeGeometry::Box(v) => {
                        if v.iter().any(|v| !v.is_finite() || *v < 0.) {
                            return Err(Error::new(
                                "physics.box",
                                "half extents must be finite and nonnegative",
                            ));
                        }
                    }
                    ShapeGeometry::Sphere(radius) => {
                        if !radius.is_finite() || *radius < 0. {
                            return Err(Error::new(
                                "physics.sphere",
                                "radius must be finite and nonnegative",
                            ));
                        }
                    }
                    ShapeGeometry::Capsule { radius, endpoints } => {
                        if !radius.is_finite()
                            || *radius < 0.
                            || endpoints.iter().flatten().any(|v| !v.is_finite())
                        {
                            return Err(Error::new(
                                "physics.capsule",
                                "endpoints and radius must be finite, radius nonnegative",
                            ));
                        }
                    }
                    ShapeGeometry::ConvexHull {
                        vertices,
                        planes,
                        faces,
                    } => {
                        if vertices
                            .iter()
                            .flatten()
                            .chain(planes.iter().flatten())
                            .any(|v| !v.is_finite())
                            || faces.len() != planes.len()
                        {
                            return Err(Error::new(
                                "physics.hull",
                                "boundary data must be finite and planes match faces",
                            ));
                        }
                        let mut edges = BTreeSet::new();
                        for face in faces.iter() {
                            if face.len() < 3 || face.iter().any(|i| i as usize >= vertices.len()) {
                                return Err(Error::new(
                                    "physics.hull.faces",
                                    "face vertex indices are invalid",
                                ));
                            }
                            for index in 0..face.len() {
                                let a = face.get(index).expect("checked face index");
                                let b = face
                                    .get((index + 1) % face.len())
                                    .expect("checked face index");
                                if a == b || !edges.insert((a, b)) {
                                    return Err(Error::new(
                                        "physics.hull.faces",
                                        "boundary contains duplicate or degenerate directed edges",
                                    ));
                                }
                            }
                        }
                        if edges.iter().any(|&(a, b)| !edges.contains(&(b, a))) {
                            return Err(Error::new("physics.hull.faces", "boundary is not closed"));
                        }
                    }
                }
            }
        }
        for joint in &self.joints {
            joint.constraint.validate()?;
            if !bodies.contains_key(&joint.body_a) || !bodies.contains_key(&joint.body_b) {
                return Err(Error::new(
                    "physics.joint",
                    "constraint endpoints must belong to this system",
                ));
            }
            for frame in [joint.frame_a, joint.frame_b] {
                if frame
                    .axis_x
                    .iter()
                    .chain(&frame.axis_y)
                    .chain(&frame.axis_z)
                    .chain(&frame.origin)
                    .any(|v| !v.is_finite())
                {
                    return Err(Error::new(
                        "physics.joint.frame",
                        "joint frames must be finite",
                    ));
                }
            }
        }
        for &(a, b) in &self.collision_pairs {
            if !bodies.contains_key(&a) || !bodies.contains_key(&b) {
                return Err(Error::new(
                    "physics.collision_pairs",
                    "collision endpoints must belong to this system",
                ));
            }
        }
        if self
            .wind_coefficients
            .is_some_and(|v| v.iter().any(|v| !v.is_finite()))
        {
            return Err(Error::new(
                "physics.wind",
                "wind coefficients must be finite",
            ));
        }
        for v in &self.vegetation {
            if [
                v.position_max_push,
                v.position_push_amount,
                v.position_relax_speed,
                v.velocity_max_push,
                v.velocity_speed,
                v.minimum_push_distance_squared,
            ]
            .into_iter()
            .any(|v| !v.is_finite())
            {
                return Err(Error::new(
                    "physics.vegetation",
                    "vegetation coefficients must be finite",
                ));
            }
        }
        Ok(())
    }
}

impl Constraint {
    fn validate(&self) -> Result<(), Error> {
        let spring =
            |value: &Spring| value.frequency_hz.is_finite() && value.damping_ratio.is_finite();
        let motor = |value: &Motor| value.maximum_force.is_finite() && spring(&value.spring);
        let valid = match self {
            Self::Spherical { friction_torque } => friction_torque.is_finite(),
            Self::Shoulder {
                twist,
                cone,
                motor: value,
            } => twist.iter().all(|value| value.is_finite()) && cone.is_finite() && motor(value),
            Self::Weld { linear, angular } => spring(linear) && spring(angular),
            Self::Revolute {
                angles,
                friction_torque,
                motor: value,
            } => {
                angles.iter().all(|value| value.is_finite())
                    && friction_torque.is_finite()
                    && motor(value)
            }
            Self::Prismatic {
                distances,
                reference_translation,
                motor_speed,
                friction_force,
                motor: value,
            } => {
                distances
                    .iter()
                    .chain([reference_translation, motor_speed, friction_force])
                    .all(|value| value.is_finite())
                    && motor(value)
            }
            Self::Distance {
                distance,
                spring: value,
            } => distance.is_finite() && spring(value),
        };
        if valid {
            Ok(())
        } else {
            Err(Error::new(
                "physics.constraint",
                "constraint coefficients must be finite",
            ))
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
