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
    fn sphere(&self, matrix: Mat4) -> Primitive {
        Primitive::Sphere {
            center: matrix.transform_point3(self.center),
            radius: self.radius * bound_scale(linear(matrix)),
        }
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
    pub fn supports(&self, other: &Self) -> bool {
        self.is_hull() || other.is_hull() || Self::primitive_supports(self.geometry, other.geometry)
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
            (|| {
                let scratch = scratch.ok_or(PhysicsContactIssue::ScratchUnavailable)?;
                let left = self.support_ref(a)?;
                let right = other.support_ref(b)?;
                let scale = f64::from(self.radius * bound_scale(linear(a)))
                    + f64::from(other.radius * bound_scale(linear(b)));
                convex::query(left, right, f64::from(margin), scale, scratch, work)
            })()
        } else {
            // Preserve the established analytical path and its explicit primitive-only bounds.
            let (left, _) = self.transformed_geometry(a);
            let (right, _) = other.transformed_geometry(b);
            let (ga, gb) = if Self::primitive_supports(left, right) {
                (left, right)
            } else {
                (self.sphere(a), other.sphere(b))
            };
            contact::query_margin(ga, gb, margin).map_err(|error| match error {
                contact::QueryError::InvalidPrimitive => PhysicsContactIssue::InvalidInput,
                contact::QueryError::UnsupportedPair => PhysicsContactIssue::UnsupportedPair,
            })
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
mod tests {
    use super::*;
    use bevy::math::Quat;
    fn capsule() -> Shape<'static> {
        Shape {
            geometry: ShapeGeometry::Capsule {
                radius: 0.5,
                endpoints: [-Vec3::Z, Vec3::Z],
            },
            density: 2.,
            transform: Mat4::IDENTITY,
            friction: 0.,
            restitution: 0.,
            mass_properties: None,
            game_flags: Default::default(),
            collision_mask: Default::default(),
        }
    }
    #[test]
    fn transformed_compound_capsules_retain_center_and_anisotropic_inertia() {
        let mut a = capsule();
        a.transform =
            Mat4::from_rotation_translation(Quat::from_rotation_y(0.5), Vec3::new(2., 1., 0.));
        let mut b = capsule();
        b.transform = Mat4::from_translation(Vec3::new(-2., -1., 0.));
        let (fa, ma) = Fixture::prepare(&a);
        let (fb, mb) = Fixture::prepare(&b);
        assert!(!fa.approximate && !fb.approximate);
        assert!(matches!(fa.geometry, Primitive::Capsule { .. }));
        let compound = Mass::compound([ma, mb].into_iter()).unwrap();
        assert_eq!(compound.center, Vec3::ZERO);
        assert!(compound.inertia().x_axis.y.abs() > 0.1);
        assert!((compound.mass - ma.mass - mb.mass).abs() < 1e-5);
    }
    #[test]
    fn nonuniform_capsule_fallback_is_explicit_and_conservative() {
        let mut shape = capsule();
        shape.transform = Mat4::from_cols(
            Vec3::X.extend(0.),
            Vec3::new(2., 1., 0.).extend(0.),
            (Vec3::Z * 3.).extend(0.),
            Vec3::new(4., 5., 6.).extend(1.),
        );
        let (fixture, mass) = Fixture::prepare(&shape);
        assert!(fixture.approximate);
        let Primitive::Sphere { center, radius } = fixture.geometry else {
            panic!("expected conservative bound");
        };
        for z in [-1., 1.] {
            for direction in [Vec3::X, -Vec3::X, Vec3::Y, -Vec3::Y, Vec3::Z, -Vec3::Z] {
                let point = shape
                    .transform
                    .transform_point3(Vec3::Z * z + direction * 0.5);
                assert!(point.distance(center) <= radius);
            }
        }
        assert!(mass.inverse_inertia(1.).is_some());
    }
    #[test]
    fn hull_queries_borrow_prepared_vertices_and_never_use_their_bound_as_contact() {
        use super::super::contact_work::PhysicsContactLimits;
        let vertices = (0..8)
            .map(|bits| {
                Vec3::new(
                    if bits & 1 == 0 { -1. } else { 1. },
                    if bits & 2 == 0 { -0.03 } else { 0.03 },
                    if bits & 4 == 0 { -0.4 } else { 0.4 },
                )
            })
            .collect();
        let hull = Arc::new(Hull::new(vertices));
        let mut left = Fixture::test_primitive(
            Primitive::Sphere {
                center: Vec3::ZERO,
                radius: 1.1,
            },
            Vec3::ZERO,
            1.1,
            0.,
        );
        left.hull = Some(hull.clone());
        let right = Fixture::test_primitive(
            Primitive::Sphere {
                center: Vec3::ZERO,
                radius: 0.1,
            },
            Vec3::ZERO,
            0.1,
            0.,
        );
        let mut scratch = Scratch::default();
        let mut work = ContactWork::new(PhysicsContactLimits::default());
        assert!(left
            .contact(
                Mat4::IDENTITY,
                &right,
                Mat4::from_translation(Vec3::Y * 0.3),
                0.,
                &mut work,
                Some(&mut scratch)
            )
            .is_none());
        assert_eq!(work.stats.inconclusive_queries, 0);
        assert_eq!(Arc::strong_count(&hull), 2);
        let mut tiny = ContactWork::new(PhysicsContactLimits {
            queries_per_update: 4,
            support_vertices_per_update: 1,
            ..Default::default()
        });
        assert!(left
            .contact(
                Mat4::IDENTITY,
                &right,
                Mat4::from_translation(Vec3::Y * 0.05),
                0.,
                &mut tiny,
                Some(&mut scratch)
            )
            .is_none());
        assert_eq!(tiny.stats.support_budget_exhaustions, 1);
        assert_eq!(tiny.stats.support_vertices, 1);
        // The same per-update quota still permits an analytical query afterwards.
        assert!(right
            .contact(
                Mat4::IDENTITY,
                &right,
                Mat4::from_translation(Vec3::X * 0.1),
                0.,
                &mut tiny,
                None
            )
            .is_some());
        assert_eq!(tiny.stats.queries, 2);
        assert_eq!(tiny.stats.support_vertices, 1);
        assert_eq!(Arc::strong_count(&hull), 2);
    }
    #[test]
    fn analytical_errors_are_observable_without_changing_valid_contact_math() {
        use super::super::contact_work::PhysicsContactLimits;
        let left = Fixture::test_primitive(
            Primitive::Sphere {
                center: Vec3::ZERO,
                radius: 0.5,
            },
            Vec3::ZERO,
            0.5,
            0.,
        );
        let mut work = ContactWork::new(PhysicsContactLimits::default());
        let actual = left
            .contact(
                Mat4::IDENTITY,
                &left,
                Mat4::from_translation(Vec3::X * 0.7),
                0.,
                &mut work,
                None,
            )
            .unwrap();
        let expected = contact::query(
            Primitive::Sphere {
                center: Vec3::ZERO,
                radius: 0.5,
            },
            Primitive::Sphere {
                center: Vec3::X * 0.7,
                radius: 0.5,
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(actual.normal, expected.normal);
        assert_eq!(actual.points, expected.points);
        assert_eq!(actual.depth, expected.depth);
        assert!(left
            .contact(
                Mat4::from_translation(Vec3::NAN),
                &left,
                Mat4::IDENTITY,
                0.,
                &mut work,
                None
            )
            .is_none());
        assert_eq!(
            work.stats.last_issue,
            Some(PhysicsContactIssue::InvalidInput)
        );
        assert_eq!(work.stats.inconclusive_queries, 1);
    }
}
