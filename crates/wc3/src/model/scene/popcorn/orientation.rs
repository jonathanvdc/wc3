//! Original quaternion and emitter-axis kernels with explicit exceptional-input policies.
use glam::{Mat4, Quat, Vec3};
type Result<T> = std::result::Result<T, &'static str>;
fn finite(vector: Vec3) -> Result<Vec3> {
    if vector.is_finite() {
        Ok(vector)
    } else {
        Err("orientation operation requires finite vector components")
    }
}
pub(crate) fn from_forward(forward: Vec3) -> Result<Quat> {
    finite(forward)?;
    // Standard shortest-arc construction from +Z; do not change the caller's vector length.
    let cross = Vec3::Z.cross(forward);
    let quaternion = Quat::from_xyzw(cross.x, cross.y, cross.z, 1. + Vec3::Z.dot(forward));
    let square = quaternion.length_squared();
    if !square.is_finite() {
        return Err("orientation operation exceeds finite quaternion range");
    }
    Ok(if square < 1e-12 {
        Quat::from_xyzw(1., 0., 0., 0.)
    } else {
        quaternion / square.sqrt()
    })
}
pub(crate) fn rotate(vector: Vec3, rotation: Quat) -> Result<Vec3> {
    finite(vector)?;
    if !rotation.is_finite() {
        return Err("orientation operation requires finite quaternion components");
    }
    let imaginary = Vec3::new(rotation.x, rotation.y, rotation.z);
    // Unit-quaternion vector kernel applied as authored, without implicit normalization.
    let cross = imaginary.cross(vector);
    finite(vector + (cross * rotation.w + imaginary.cross(cross)) * 2.)
}
pub(crate) fn rotate_about(vector: Vec3, axis: Vec3, angle: f32) -> Result<Vec3> {
    finite(vector)?;
    finite(axis)?;
    if !angle.is_finite() {
        return Err("axis rotation requires a finite angle");
    }
    let axis = axis
        .try_normalize()
        .ok_or("axis rotation requires a nonzero finite axis")?;
    rotate(vector, Quat::from_axis_angle(axis, angle))
}
pub(crate) fn emitter_axis(world: Mat4, up: bool) -> Result<Vec3> {
    let fallback = if up { Vec3::Z } else { Vec3::X };
    let vector = if up {
        world.z_axis.truncate()
    } else {
        world.x_axis.truncate()
    };
    finite(vector)?;
    let square = vector.length_squared();
    if !square.is_finite() {
        return Err("emitter axis exceeds finite normalization range");
    }
    Ok(if square <= 1e-12 {
        fallback
    } else {
        vector / square.sqrt()
    })
}
#[cfg(test)]
#[path = "orientation_tests.rs"]
mod tests;
