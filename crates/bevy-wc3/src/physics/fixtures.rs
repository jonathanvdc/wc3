//! Prepared compound fixtures retain independent contact geometry and material response.
use super::contact::{self, Contact, Primitive};
use super::contact_work::{ContactWork, PhysicsContactIssue};
use super::convex::{self, Hull, Scratch, Shape as SupportShape, ShapeRef};
use super::mass::Mass;
use bevy::math::{Mat3, Mat4, Vec3};
use std::sync::Arc;
use wc3::model::systems::model_physics::{Shape, ShapeGeometry};

#[derive(Clone)]
pub(super) struct Fixture {
    pub geometry: Primitive,
    source: Primitive,
    source_transform: Mat4,
    hull: Option<Arc<Hull>>,
    pub center: Vec3,
    pub radius: f32,
    pub friction: f32,
    pub restitution: f32,
    pub approximate: bool,
}
fn linear(matrix: Mat4) -> Mat3 {
    Mat3::from_cols(
        matrix.x_axis.truncate(),
        matrix.y_axis.truncate(),
        matrix.z_axis.truncate(),
    )
}
fn orthogonal(matrix: Mat3) -> Option<(Mat3, Vec3)> {
    let lengths = Vec3::new(
        matrix.x_axis.length(),
        matrix.y_axis.length(),
        matrix.z_axis.length(),
    );
    if !lengths.is_finite() || lengths.min_element() < 1e-8 {
        return None;
    }
    let axes = matrix * Mat3::from_diagonal(lengths.recip());
    (axes.x_axis.dot(axes.y_axis).abs() < 1e-5
        && axes.x_axis.dot(axes.z_axis).abs() < 1e-5
        && axes.y_axis.dot(axes.z_axis).abs() < 1e-5)
        .then_some((axes, lengths))
}
fn similarity(matrix: Mat3) -> Option<f32> {
    let (_, scale) = orthogonal(matrix)?;
    ((scale.max_element() - scale.min_element()) <= 1e-5 * scale.max_element())
        .then_some(scale.max_element())
}
fn bound_scale(matrix: Mat3) -> f32 {
    if let Some(scale) = similarity(matrix) {
        return scale;
    }
    // Frobenius norm bounds the spectral norm even under shear.
    (matrix.x_axis.length_squared()
        + matrix.y_axis.length_squared()
        + matrix.z_axis.length_squared())
    .sqrt()
}
impl Fixture {
    #[cfg(test)]
    pub(super) fn test_primitive(
        geometry: Primitive,
        center: Vec3,
        radius: f32,
        friction: f32,
    ) -> Self {
        Self {
            geometry,
            source: geometry,
            source_transform: Mat4::IDENTITY,
            hull: None,
            center,
            radius,
            friction,
            restitution: 0.,
            approximate: false,
        }
    }
    pub fn prepare(shape: &Shape<'_>) -> (Self, Mass) {
        let (geometry, mass, center, radius, approximate) = match &shape.geometry {
            ShapeGeometry::Sphere(radius) => (
                Primitive::Sphere {
                    center: Vec3::ZERO,
                    radius: *radius,
                },
                Mass::sphere(*radius, shape.density),
                Vec3::ZERO,
                *radius,
                false,
            ),
            ShapeGeometry::Capsule { radius, endpoints } => {
                let center = (endpoints[0] + endpoints[1]) * 0.5;
                (
                    Primitive::Capsule {
                        endpoints: *endpoints,
                        radius: *radius,
                    },
                    Mass::capsule(*endpoints, *radius, shape.density),
                    center,
                    endpoints[0].distance(endpoints[1]) * 0.5 + radius,
                    false,
                )
            }
            ShapeGeometry::Box(half) => (
                Primitive::Box {
                    center: Vec3::ZERO,
                    axes: Mat3::IDENTITY,
                    half_extents: *half,
                },
                Mass::cuboid(*half, shape.density),
                Vec3::ZERO,
                half.length(),
                false,
            ),
            ShapeGeometry::ConvexHull { vertices, .. } => {
                let center = shape.mass_properties.map_or_else(
                    || {
                        if vertices.is_empty() {
                            Vec3::ZERO
                        } else {
                            vertices.iter().sum::<Vec3>() / vertices.len() as f32
                        }
                    },
                    |p| p.centroid,
                );
                let radius = vertices
                    .iter()
                    .map(|p| p.distance(center))
                    .fold(0., f32::max);
                let mass = shape.mass_properties.map_or(0., |p| p.volume) * shape.density;
                (
                    Primitive::Sphere { center, radius },
                    Mass {
                        mass,
                        center,
                        second_moment: Mat3::from_diagonal(Vec3::splat(
                            mass * radius * radius / 5.,
                        )),
                    },
                    center,
                    radius,
                    true,
                )
            }
        };
        let hull = match &shape.geometry {
            ShapeGeometry::ConvexHull { vertices, .. } => {
                Some(Arc::new(Hull::prepare(vertices.iter())))
            }
            _ => None,
        };
        let mut result = Self {
            geometry,
            source: geometry,
            source_transform: shape.transform,
            hull,
            center,
            radius,
            friction: shape.friction.max(0.),
            restitution: shape.restitution.clamp(0., 1.),
            approximate,
        };
        let (geometry, approximate) = result.transformed_geometry(shape.transform);
        result.geometry = geometry;
        result.approximate = approximate;
        result.center = shape.transform.transform_point3(center);
        result.radius *= bound_scale(linear(shape.transform));
        let mass = mass.transformed(linear(shape.transform), shape.transform.w_axis.truncate());
        (result, mass)
    }
    fn transformed_geometry(&self, matrix: Mat4) -> (Primitive, bool) {
        let mut approximate = self.approximate;
        let transform = linear(matrix);
        let center = matrix.transform_point3(self.center);
        let radius = self.radius * bound_scale(transform);
        let geometry = match self.geometry {
            Primitive::Sphere { center, radius } => {
                if let Some(scale) = similarity(transform) {
                    Primitive::Sphere {
                        center: matrix.transform_point3(center),
                        radius: radius * scale,
                    }
                } else {
                    approximate = true;
                    Primitive::Sphere {
                        center: matrix.transform_point3(center),
                        radius: radius * bound_scale(transform),
                    }
                }
            }
            Primitive::Capsule {
                endpoints,
                radius: cap_radius,
            } => {
                if let Some(scale) = similarity(transform) {
                    Primitive::Capsule {
                        endpoints: endpoints.map(|p| matrix.transform_point3(p)),
                        radius: cap_radius * scale,
                    }
                } else {
                    approximate = true;
                    Primitive::Sphere { center, radius }
                }
            }
            Primitive::Box {
                center: origin,
                axes,
                half_extents,
            } => {
                if let Some((axes, scale)) = orthogonal(transform * axes) {
                    Primitive::Box {
                        center: matrix.transform_point3(origin),
                        axes,
                        half_extents: half_extents * scale,
                    }
                } else {
                    approximate = true;
                    Primitive::Sphere { center, radius }
                }
            }
        };
        (geometry, approximate)
    }
    pub fn is_hull(&self) -> bool {
        self.hull.is_some()
    }
    pub fn hull_issue(&self) -> Option<PhysicsContactIssue> {
        self.hull.as_ref().and_then(|hull| hull.issue())
    }
    fn primitive_supports(left: Primitive, right: Primitive) -> bool {
        matches!(
            (left, right),
            (Primitive::Sphere { .. }, _)
                | (_, Primitive::Sphere { .. })
                | (Primitive::Capsule { .. }, Primitive::Capsule { .. })
        )
    }
    fn support_ref(&self, matrix: Mat4) -> Result<ShapeRef<'_>, PhysicsContactIssue> {
        let shape = self
            .hull
            .as_deref()
            .map_or(SupportShape::Primitive(self.source), SupportShape::Hull);
        ShapeRef::new(shape, matrix * self.source_transform)
    }
    fn support_contact(
        &self,
        a: Mat4,
        other: &Self,
        b: Mat4,
        margin: f32,
        work: &mut ContactWork,
        scratch: Option<&mut Scratch>,
    ) -> Result<Option<Contact>, PhysicsContactIssue> {
        let scratch = scratch.ok_or(PhysicsContactIssue::ScratchUnavailable)?;
        let left = self.support_ref(a)?;
        let right = other.support_ref(b)?;
        let scale = f64::from(self.radius * bound_scale(linear(a)))
            + f64::from(other.radius * bound_scale(linear(b)));
        convex::query(left, right, f64::from(margin), scale, scratch, work)
    }
    pub fn contact(
        &self,
        a: Mat4,
        other: &Self,
        b: Mat4,
        margin: f32,
        work: &mut ContactWork,
        scratch: Option<&mut Scratch>,
    ) -> Option<Contact> {
        if !work.begin_query() {
            return None;
        }
        let result = if self.is_hull() || other.is_hull() {
            self.support_contact(a, other, b, margin, work, scratch)
        } else {
            let (left, left_approximate) = self.transformed_geometry(a);
            let (right, right_approximate) = other.transformed_geometry(b);
            if left_approximate || right_approximate || !Self::primitive_supports(left, right) {
                self.support_contact(a, other, b, margin, work, scratch)
            } else {
                // Keep established exact analytical combinations unchanged.
                contact::query_margin(left, right, margin).map_err(|error| match error {
                    contact::QueryError::InvalidPrimitive => PhysicsContactIssue::InvalidInput,
                    contact::QueryError::UnsupportedPair => PhysicsContactIssue::UnsupportedPair,
                })
            }
        };
        match result {
            Ok(contact) => contact,
            Err(issue) => {
                work.issue(issue);
                None
            }
        }
    }
}

#[cfg(test)]
#[path = "fixtures_tests.rs"]
mod tests;
