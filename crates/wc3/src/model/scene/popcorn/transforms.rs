//! Checked original forward/inverse transform kernels with shared payload-frame selection.
use super::payload::select;
use glam::{Affine3A, Mat4, Vec3, Vec4};
type Result<T> = std::result::Result<T, &'static str>;
fn finite(value: Vec3) -> Result<Vec3> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err("effect transform produced nonfinite components")
    }
}
pub(crate) fn forward(
    ordinary: Mat4,
    payload: Option<Mat4>,
    value: Vec3,
    mask: u32,
    direction: bool,
) -> Result<Vec3> {
    let world = select(ordinary, payload, mask)?;
    if value == Vec3::splat(f32::INFINITY) {
        if !matches!((direction, mask), (false, 17 | 11 | 19) | (true, 18)) {
            return Err("unsupported native local-to-world transform mask");
        }
        if !world.is_finite()
            || world.row(3) != Vec4::W
            || !world.determinant().is_finite()
            || world.determinant() == 0.
        {
            return Err("unavailable position requires a finite affine frame");
        }
        return Ok(value);
    }
    finite(match (direction, mask) {
        (false, 17) => value + world.w_axis.truncate(),
        (false, 11 | 19) => world.transform_point3(value),
        (true, 18) => world.transform_vector3(value),
        _ => return Err("unsupported native local-to-world transform mask"),
    })
}
pub(crate) fn inverse(
    ordinary: Mat4,
    payload: Option<Mat4>,
    value: Vec3,
    mask: u32,
    direction: bool,
) -> Result<Vec3> {
    let forward_mask = match (direction, mask) {
        (false, 67) => 19,
        (true, 66) => 18,
        _ => return Err("unsupported native world-to-local transform mask"),
    };
    let unavailable = value == Vec3::splat(f32::INFINITY);
    if !unavailable {
        finite(value)?;
    }
    let frame = select(ordinary, payload, forward_mask)?;
    if !frame.is_finite() || frame.row(3) != Vec4::W {
        return Err("inverse effect transform requires a finite affine frame");
    }
    let frame = Affine3A::from_mat4(frame);
    let determinant = frame.matrix3.determinant();
    if !determinant.is_finite() || determinant == 0. {
        return Err("inverse effect transform frame is singular or exceeds finite range");
    }
    let inverse = frame.inverse();
    if !inverse.is_finite() {
        return Err("inverse effect transform exceeds finite range");
    }
    if unavailable {
        return Ok(value);
    }
    finite(if direction {
        inverse.transform_vector3(value)
    } else {
        inverse.transform_point3(value)
    })
}
#[cfg(test)]
#[path = "transforms_tests.rs"]
mod tests;
