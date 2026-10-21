//! Prepared compound fixtures retain independent contact geometry and material response.
use super::contact::{self, Contact, Primitive};
use super::mass::Mass;
use bevy::math::{Mat3, Mat4, Vec3};
use wc3::model::systems::model_physics::{Shape, ShapeGeometry};

#[derive(Clone, Copy)]
pub(super) struct Fixture {
    pub geometry: Primitive,
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
        let mut result = Self {
            geometry,
            center,
            radius,
            friction: shape.friction.max(0.),
            restitution: shape.restitution.clamp(0., 1.),
            approximate,
        };
        result = result.transformed(shape.transform);
        let mass = mass.transformed(linear(shape.transform), shape.transform.w_axis.truncate());
        (result, mass)
    }
    fn sphere(&self, matrix: Mat4) -> Primitive {
        Primitive::Sphere {
            center: matrix.transform_point3(self.center),
            radius: self.radius * bound_scale(linear(matrix)),
        }
    }
    fn transformed(mut self, matrix: Mat4) -> Self {
        let transform = linear(matrix);
        let center = matrix.transform_point3(self.center);
        let radius = self.radius * bound_scale(transform);
        self.geometry = match self.geometry {
            Primitive::Sphere { center, radius } => {
                if let Some(scale) = similarity(transform) {
                    Primitive::Sphere {
                        center: matrix.transform_point3(center),
                        radius: radius * scale,
                    }
                } else {
                    self.approximate = true;
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
                    self.approximate = true;
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
                    self.approximate = true;
                    Primitive::Sphere { center, radius }
                }
            }
        };
        self.center = center;
        self.radius = radius;
        self
    }
    pub fn supports(&self, other: &Self) -> bool {
        matches!(
            (self.geometry, other.geometry),
            (Primitive::Sphere { .. }, _)
                | (_, Primitive::Sphere { .. })
                | (Primitive::Capsule { .. }, Primitive::Capsule { .. })
        )
    }
    pub fn contact(&self, a: Mat4, other: &Self, b: Mat4, margin: f32) -> Option<Contact> {
        // Runtime scale/shear can change shape eligibility. Reuse the same checked policy.
        let left = (*self).transformed(a);
        let right = (*other).transformed(b);
        let (ga, gb) = if left.supports(&right) {
            (left.geometry, right.geometry)
        } else {
            (self.sphere(a), other.sphere(b))
        };
        if margin == 0. {
            contact::query(ga, gb)
        } else {
            contact::query_margin(ga, gb, margin)
        }
        .ok()
        .flatten()
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
}
