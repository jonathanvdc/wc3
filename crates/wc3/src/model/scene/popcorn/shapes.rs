//! Original bounded primitive sampling in the caller's native simulation frame.
use glam::{EulerRot, Quat, Vec3};
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Primitive {
    Box,
    Sphere,
    Ellipsoid,
    Cylinder,
    Capsule,
    Cone,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Domain {
    Surface,
    Volume,
}
#[derive(Clone, Debug)]
pub(crate) struct Shape {
    pub primitive: Primitive,
    pub domain: Domain,
    pub dimensions: Vec3,
    pub radius: f32,
    pub inner_radius: f32,
    pub height: f32,
    pub hemisphere: bool,
    pub scale: Vec3,
    pub translation: Vec3,
    pub rotation: Quat,
}
impl Shape {
    pub fn rotation(euler_degrees: Vec3) -> Quat {
        // Column-vector application order is X, then Y, then Z.
        Quat::from_euler(
            EulerRot::ZYX,
            euler_degrees.z.to_radians(),
            euler_degrees.y.to_radians(),
            euler_degrees.x.to_radians(),
        )
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.dimensions.is_finite()
            || self.dimensions.min_element() < 0.
            || !self.radius.is_finite()
            || self.radius < 0.
            || !self.inner_radius.is_finite()
            || self.inner_radius < 0.
            || self.inner_radius > self.radius
            || !self.height.is_finite()
            || self.height < 0.
            || !self.scale.is_finite()
            || !self.translation.is_finite()
            || !self.rotation.is_finite()
        {
            return Err(
                "shape parameters must be finite with ordered nonnegative dimensions and radii",
            );
        }
        Ok(())
    }
    pub fn sample(&self, draws: [f32; 6]) -> Result<Vec3, &'static str> {
        let local = match self.primitive {
            Primitive::Box => self.sample_box(draws),
            Primitive::Sphere | Primitive::Ellipsoid => {
                let mut point = self.sample_sphere(draws);
                if self.primitive == Primitive::Ellipsoid {
                    if self.hemisphere {
                        point.z = point.z.abs();
                    }
                    point *= self.scale;
                }
                point
            }
            Primitive::Cylinder => self.sample_cylinder(draws),
            Primitive::Capsule => {
                let (sphere, cylinder) = match self.domain {
                    Domain::Volume => (
                        4. / 3. * PI * (self.radius.powi(3) - self.inner_radius.powi(3)),
                        PI * self.height * (self.radius.powi(2) - self.inner_radius.powi(2)),
                    ),
                    Domain::Surface => (
                        4. * PI * (self.radius.powi(2) + self.inner_radius.powi(2)),
                        TAU * self.height * (self.radius + self.inner_radius),
                    ),
                };
                if sphere + cylinder > 0. && draws[5] * (sphere + cylinder) < sphere {
                    let mut point = self.sample_sphere(draws);
                    point.z += if point.z >= 0. {
                        self.height * 0.5
                    } else {
                        -self.height * 0.5
                    };
                    point
                } else {
                    self.sample_cylinder(draws)
                }
            }
            Primitive::Cone => {
                let axial = match self.domain {
                    Domain::Volume => draws[0].cbrt(),
                    Domain::Surface => draws[0].sqrt(),
                };
                let radial = self.radius
                    * axial
                    * if self.domain == Domain::Volume {
                        draws[2].sqrt()
                    } else {
                        1.
                    };
                let (sin, cos) = (draws[1] * TAU).sin_cos();
                Vec3::new(radial * cos, radial * sin, self.height * (1. - axial))
            }
        };
        let point = self.rotation * local + self.translation;
        if !point.is_finite() {
            return Err("shape sampling produced nonfinite coordinates");
        }
        Ok(point)
    }
    fn sample_sphere(&self, draws: [f32; 6]) -> Vec3 {
        let axial = 2. * draws[0] - 1.;
        let ring = (1. - axial * axial).max(0.).sqrt();
        let (sin, cos) = (draws[1] * TAU).sin_cos();
        let radius = match self.domain {
            Domain::Volume => (self.inner_radius.powi(3)
                + draws[2] * (self.radius.powi(3) - self.inner_radius.powi(3)))
            .cbrt(),
            Domain::Surface => {
                let inner = self.inner_radius.powi(2);
                let outer = self.radius.powi(2);
                if draws[2] * (inner + outer) < inner {
                    self.inner_radius
                } else {
                    self.radius
                }
            }
        };
        Vec3::new(ring * cos, ring * sin, axial) * radius
    }
    fn sample_cylinder(&self, draws: [f32; 6]) -> Vec3 {
        let (sin, cos) = (draws[0] * TAU).sin_cos();
        let radius = match self.domain {
            Domain::Surface => {
                if draws[2] * (self.inner_radius + self.radius) < self.inner_radius {
                    self.inner_radius
                } else {
                    self.radius
                }
            }
            Domain::Volume => {
                let exponent = if self.radius > 0. {
                    (1. + self.inner_radius / self.radius) * 0.5
                } else {
                    0.5
                };
                self.inner_radius + (self.radius - self.inner_radius) * draws[2].powf(exponent)
            }
        };
        Vec3::new(radius * cos, radius * sin, (draws[1] - 0.5) * self.height)
    }
    fn sample_box(&self, draws: [f32; 6]) -> Vec3 {
        let volume = || self.dimensions * Vec3::new(draws[0] - 0.5, draws[1] - 0.5, draws[2] - 0.5);
        if self.domain == Domain::Volume {
            return volume();
        }
        let areas = Vec3::new(
            self.dimensions.y * self.dimensions.z,
            self.dimensions.x * self.dimensions.z,
            self.dimensions.x * self.dimensions.y,
        );
        let total = areas.element_sum();
        if total <= 0. {
            return volume();
        }
        let selected = draws[0] * total;
        let axis = if selected < areas.x {
            0
        } else if selected < areas.x + areas.y {
            1
        } else {
            2
        };
        let mut point = Vec3::ZERO;
        point[axis] = if draws[1] < 0.5 { -0.5 } else { 0.5 };
        point[(axis + 1) % 3] = draws[2] - 0.5;
        point[(axis + 2) % 3] = draws[3] - 0.5;
        point * self.dimensions
    }
}

#[cfg(test)]
#[path = "shape_tests.rs"]
mod tests;
