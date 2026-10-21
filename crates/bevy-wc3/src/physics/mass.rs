//! Centered second moments make affine fixture transforms and compound inertia explicit.
use bevy::math::{Mat3, Vec3};
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct Mass {
    pub mass: f32,
    pub center: Vec3,
    /// Integral of (x-center)*(x-center)^T over mass; not the inertia tensor.
    pub second_moment: Mat3,
}
fn outer(v: Vec3) -> Mat3 {
    Mat3::from_cols(v * v.x, v * v.y, v * v.z)
}
impl Mass {
    pub fn sphere(radius: f32, density: f32) -> Self {
        let mass = density * 4. * PI * radius.powi(3) / 3.;
        Self {
            mass,
            center: Vec3::ZERO,
            second_moment: Mat3::from_diagonal(Vec3::splat(mass * radius * radius / 5.)),
        }
    }
    pub fn cuboid(half: Vec3, density: f32) -> Self {
        let mass = density * 8. * half.element_product();
        Self {
            mass,
            center: Vec3::ZERO,
            second_moment: Mat3::from_diagonal(half * half * (mass / 3.)),
        }
    }
    pub fn capsule(endpoints: [Vec3; 2], radius: f32, density: f32) -> Self {
        let delta = endpoints[1] - endpoints[0];
        let length = delta.length();
        let axis = delta.try_normalize().unwrap_or(Vec3::Z);
        let cylinder = density * PI * radius * radius * length;
        let sphere = density * 4. * PI * radius.powi(3) / 3.;
        let perpendicular = cylinder * radius * radius / 4. + sphere * radius * radius / 5.;
        // Each hemisphere's first moment contributes the cross term3LR/8.
        let parallel = cylinder * length * length / 12.
            + sphere * (radius * radius / 5. + 3. * length * radius / 8. + length * length / 4.);
        Self {
            mass: cylinder + sphere,
            center: (endpoints[0] + endpoints[1]) * 0.5,
            second_moment: Mat3::from_diagonal(Vec3::splat(perpendicular))
                + outer(axis) * (parallel - perpendicular),
        }
    }
    pub fn transformed(self, linear: Mat3, translation: Vec3) -> Self {
        let volume_scale = linear.determinant().abs();
        Self {
            mass: self.mass * volume_scale,
            center: linear * self.center + translation,
            second_moment: (linear * self.second_moment * linear.transpose()) * volume_scale,
        }
    }
    pub fn compound(fixtures: impl Clone + Iterator<Item = Self>) -> Option<Self> {
        let (mass, weighted) = fixtures.clone().fold((0., Vec3::ZERO), |(m, c), shape| {
            (m + shape.mass, c + shape.center * shape.mass)
        });
        if mass <= 0. || !mass.is_finite() {
            return None;
        }
        let center = weighted / mass;
        let second = fixtures.fold(Mat3::ZERO, |sum, shape| {
            sum + shape.second_moment + outer(shape.center - center) * shape.mass
        });
        let result = Self {
            mass,
            center,
            second_moment: second,
        };
        (center.is_finite() && second.is_finite()).then_some(result)
    }
    pub fn inertia(self) -> Mat3 {
        let c = self.second_moment;
        let trace = c.x_axis.x + c.y_axis.y + c.z_axis.z;
        Mat3::from_diagonal(Vec3::splat(trace)) - c
    }
    pub fn inverse_inertia(self, scale: f32) -> Option<Mat3> {
        let inertia = self.inertia() * scale;
        if !inertia.is_finite() || inertia.determinant() <= 0. {
            return None;
        }
        let inverse = inertia.inverse();
        inverse.is_finite().then_some(inverse)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn close(a: Mat3, b: Mat3) {
        for (a, b) in a.to_cols_array().into_iter().zip(b.to_cols_array()) {
            assert!((a - b).abs() < 1e-4, "{a} != {b}");
        }
    }
    #[test]
    fn spheres_and_boxes_have_known_tensor_inertia() {
        let s = Mass::sphere(2., 3.);
        close(
            s.inertia(),
            Mat3::from_diagonal(Vec3::splat(0.4 * s.mass * 4.)),
        );
        let b = Mass::cuboid(Vec3::new(1., 2., 3.), 1.);
        close(
            b.inertia(),
            Mat3::from_diagonal(Vec3::new(13., 10., 5.) * (b.mass / 3.)),
        );
    }
    #[test]
    fn affine_scaling_and_rotation_transform_volume_and_second_moments_once() {
        let initial = Mass::cuboid(Vec3::ONE, 1.);
        let scaled = initial.transformed(Mat3::from_diagonal(Vec3::new(-1., 2., 3.)), Vec3::ZERO);
        close(
            scaled.inertia(),
            Mass::cuboid(Vec3::new(1., 2., 3.), 1.).inertia(),
        );
        let q = Mat3::from_rotation_z(0.3);
        let rotated = scaled.transformed(q, Vec3::new(2., 3., 4.));
        close(rotated.inertia(), q * scaled.inertia() * q.transpose());
        assert!((rotated.mass - scaled.mass).abs() < 1e-4);
    }
    #[test]
    fn compound_parallel_axis_retains_off_diagonal_products() {
        let a = Mass::sphere(0.5, 1.).transformed(Mat3::IDENTITY, Vec3::new(-2., -1., 0.));
        let b = Mass::sphere(0.5, 1.).transformed(Mat3::IDENTITY, Vec3::new(2., 1., 0.));
        let c = Mass::compound([a, b].into_iter()).unwrap();
        assert_eq!(c.center, Vec3::ZERO);
        assert!(c.inertia().x_axis.y < 0.);
        assert!(c.inverse_inertia(1.).unwrap().is_finite());
    }
    #[test]
    fn zero_length_capsule_matches_sphere_and_rotation_preserves_axis_moments() {
        let capsule = Mass::capsule([Vec3::ZERO; 2], 0.5, 2.);
        let sphere = Mass::sphere(0.5, 2.);
        close(capsule.inertia(), sphere.inertia());
        let along_z = Mass::capsule([-Vec3::Z, Vec3::Z], 0.5, 2.);
        let along_x = Mass::capsule([-Vec3::X, Vec3::X], 0.5, 2.);
        assert!((along_z.inertia().z_axis.z - along_x.inertia().x_axis.x).abs() < 1e-4);
    }
}
