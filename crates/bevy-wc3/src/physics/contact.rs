//! Primitive contact queries with explicit unsupported pairs.
use bevy::math::{Mat3, Vec3};

#[derive(Clone, Copy, Debug)]
pub enum Primitive {
    Sphere {
        center: Vec3,
        radius: f32,
    },
    Capsule {
        endpoints: [Vec3; 2],
        radius: f32,
    },
    /// Orthonormal world axes; half extents already include authored scale.
    Box {
        center: Vec3,
        axes: Mat3,
        half_extents: Vec3,
    },
}
#[derive(Clone, Copy, Debug)]
pub struct Contact {
    /// Unit normal from A toward B; positional repair moves A against it.
    pub normal: Vec3,
    pub points: [Vec3; 2],
    pub depth: f32,
}
impl Contact {
    fn reversed(self) -> Self {
        Self {
            normal: -self.normal,
            points: [self.points[1], self.points[0]],
            depth: self.depth,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryError {
    UnsupportedPair,
    InvalidPrimitive,
}

#[cfg(test)]
pub fn query(a: Primitive, b: Primitive) -> Result<Option<Contact>, QueryError> {
    query_margin(a, b, 0.)
}
/// Numerical persistence tolerance only; witnesses remain on the original surfaces.
pub fn query_margin(
    a: Primitive,
    b: Primitive,
    margin: f32,
) -> Result<Option<Contact>, QueryError> {
    if !margin.is_finite() || margin < 0. {
        return Err(QueryError::InvalidPrimitive);
    }
    validate(a)?;
    validate(b)?;
    let result = match (a, b) {
        (
            Primitive::Sphere {
                center: a,
                radius: ra,
            },
            Primitive::Sphere {
                center: b,
                radius: rb,
            },
        ) => Ok(rounded([a, a], ra, [b, b], rb, margin)),
        (
            Primitive::Sphere {
                center: a,
                radius: ra,
            },
            Primitive::Capsule {
                endpoints: b,
                radius: rb,
            },
        ) => Ok(rounded([a, a], ra, b, rb, margin)),
        (
            Primitive::Capsule {
                endpoints: a,
                radius: ra,
            },
            Primitive::Sphere {
                center: b,
                radius: rb,
            },
        ) => Ok(rounded(a, ra, [b, b], rb, margin)),
        (
            Primitive::Capsule {
                endpoints: a,
                radius: ra,
            },
            Primitive::Capsule {
                endpoints: b,
                radius: rb,
            },
        ) => Ok(rounded(a, ra, b, rb, margin)),
        (
            Primitive::Sphere { center, radius },
            Primitive::Box {
                center: origin,
                axes,
                half_extents,
            },
        ) => Ok(sphere_box(
            center,
            radius,
            origin,
            axes,
            half_extents,
            margin,
        )),
        (a @ Primitive::Box { .. }, b @ Primitive::Sphere { .. }) => {
            query_margin(b, a, margin).map(|contact| contact.map(Contact::reversed))
        }
        _ => Err(QueryError::UnsupportedPair),
    }?;
    if result.is_some_and(|contact| {
        !contact.normal.is_finite()
            || contact.points.iter().any(|point| !point.is_finite())
            || !contact.depth.is_finite()
            || contact.depth < 0.
    }) {
        return Err(QueryError::InvalidPrimitive);
    }
    Ok(result)
}
fn validate(primitive: Primitive) -> Result<(), QueryError> {
    let valid = match primitive {
        Primitive::Sphere { center, radius } => {
            center.is_finite() && radius.is_finite() && radius >= 0.
        }
        Primitive::Capsule { endpoints, radius } => {
            endpoints.iter().all(|point| point.is_finite()) && radius.is_finite() && radius >= 0.
        }
        Primitive::Box {
            center,
            axes,
            half_extents,
        } => {
            center.is_finite()
                && axes.is_finite()
                && half_extents.is_finite()
                && half_extents.min_element() >= 0.
                && (axes.transpose() * axes - Mat3::IDENTITY)
                    .to_cols_array()
                    .iter()
                    .all(|value| value.abs() < 1e-4)
        }
    };
    valid.then_some(()).ok_or(QueryError::InvalidPrimitive)
}
/// Closest pair on finite segments, including zero-length and parallel inputs.
fn segment_pair(a: [Vec3; 2], b: [Vec3; 2]) -> [Vec3; 2] {
    let u = a[1] - a[0];
    let v = b[1] - b[0];
    let w = a[0] - b[0];
    let uu = u.length_squared();
    let vv = v.length_squared();
    let uw = u.dot(w);
    let vw = v.dot(w);
    let (s, t) = if uu <= 1e-12 && vv <= 1e-12 {
        (0., 0.)
    } else if uu <= 1e-12 {
        (0., (vw / vv).clamp(0., 1.))
    } else if vv <= 1e-12 {
        ((-uw / uu).clamp(0., 1.), 0.)
    } else {
        let uv = u.dot(v);
        let determinant = uu * vv - uv * uv;
        let mut s = if determinant > 1e-6 * uu * vv {
            ((uv * vw - vv * uw) / determinant).clamp(0., 1.)
        } else {
            // Parallel overlap has a continuum of closest points. Its midpoint
            // avoids injecting an arbitrary endpoint lever arm into response.
            let start = -uw / uu;
            let end = (uv - uw) / uu;
            let lower = start.min(end).max(0.);
            let upper = start.max(end).min(1.);
            if lower <= upper {
                (lower + upper) * 0.5
            } else {
                0.
            }
        };
        let mut t = (uv * s + vw) / vv;
        if t < 0. {
            t = 0.;
            s = (-uw / uu).clamp(0., 1.);
        } else if t > 1. {
            t = 1.;
            s = ((uv - uw) / uu).clamp(0., 1.);
        }
        (s, t)
    };
    [a[0] + u * s, b[0] + v * t]
}
fn rounded(a: [Vec3; 2], ra: f32, b: [Vec3; 2], rb: f32, margin: f32) -> Option<Contact> {
    let [pa, pb] = segment_pair(a, b);
    let delta = pb - pa;
    let distance = delta.length();
    let radius = ra + rb;
    if distance > radius + margin {
        return None;
    }
    let normal = delta.try_normalize().unwrap_or_else(|| {
        let ua = a[1] - a[0];
        let ub = b[1] - b[0];
        ua.cross(ub)
            .try_normalize()
            .or_else(|| {
                let axis = ua.try_normalize().or_else(|| ub.try_normalize())?;
                let reference = if axis.x.abs() < 0.8 { Vec3::X } else { Vec3::Y };
                axis.cross(reference).try_normalize()
            })
            .unwrap_or(Vec3::X)
    });
    Some(Contact {
        normal,
        points: [pa + normal * ra, pb - normal * rb],
        depth: (radius - distance).max(0.),
    })
}
fn sphere_box(
    center: Vec3,
    radius: f32,
    origin: Vec3,
    axes: Mat3,
    half: Vec3,
    margin: f32,
) -> Option<Contact> {
    let local = axes.transpose() * (center - origin);
    let nearest = local.clamp(-half, half);
    let toward = nearest - local;
    let distance = toward.length();
    if distance > radius + margin {
        return None;
    }
    let (normal, point, depth) = if distance > 1e-8 {
        let normal = axes * (toward / distance);
        (normal, origin + axes * nearest, radius - distance)
    } else {
        let margins = half - local.abs();
        let index = if margins.x <= margins.y && margins.x <= margins.z {
            0
        } else if margins.y <= margins.z {
            1
        } else {
            2
        };
        let mut outward = Vec3::ZERO;
        outward[index] = if local[index] < 0. { -1. } else { 1. };
        let world_outward = axes * outward;
        (
            -world_outward,
            center + world_outward * margins[index],
            radius + margins[index],
        )
    };
    Some(Contact {
        normal,
        points: [center + normal * radius, point],
        depth: depth.max(0.),
    })
}
#[cfg(test)]
#[path = "contact_tests.rs"]
mod tests;
