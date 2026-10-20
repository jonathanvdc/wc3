//! Selection of ordinary-emitter and immutable routed-payload transform spaces.
use crate::model::scene::popcorn::orientation::rotate;
use glam::{Mat4, Quat, Vec3};
type Result<T> = std::result::Result<T, &'static str>;
pub(crate) fn snapshot(position: Option<Vec3>, orientation: Option<Quat>) -> Result<Option<Mat4>> {
    if position.is_none() && orientation.is_none() {
        return Ok(None);
    }
    let position = position.unwrap_or(Vec3::ZERO);
    if !position.is_finite() {
        return Err("payload position requires finite components");
    }
    let rotation = orientation.unwrap_or(Quat::IDENTITY);
    // The authored quaternion kernel does not normalize, even for basis construction.
    Ok(Some(Mat4::from_cols(
        rotate(Vec3::X, rotation)?.extend(0.),
        rotate(Vec3::Y, rotation)?.extend(0.),
        rotate(Vec3::Z, rotation)?.extend(0.),
        position.extend(1.),
    )))
}
pub(crate) fn select(ordinary: Mat4, payload: Option<Mat4>, mask: u32) -> Result<Mat4> {
    let enter = (mask >> 3) & 3;
    let leave = (mask >> 5) & 3;
    if leave != 0 {
        return Err("payload transform destination space is unsupported");
    }
    match enter {
        1 => Ok(ordinary),
        2 => Ok(payload.unwrap_or(ordinary)),
        _ => Err("payload transform source space is unsupported"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;
    #[test]
    fn payload_presence_replaces_the_complete_frame_while_ordinary_space_ignores_payload() {
        let ordinary = Mat4::from_scale_rotation_translation(
            Vec3::splat(3.),
            Quat::from_rotation_z(FRAC_PI_2),
            Vec3::splat(10.),
        );
        assert_eq!(select(ordinary, None, 19).unwrap(), ordinary);
        let position_only = snapshot(Some(Vec3::new(20., 30., 40.)), None).unwrap();
        assert_eq!(
            select(ordinary, position_only, 19)
                .unwrap()
                .transform_point3(Vec3::X),
            Vec3::new(21., 30., 40.)
        );
        assert_eq!(select(ordinary, position_only, 11).unwrap(), ordinary);
        let orientation_only = snapshot(None, Some(Quat::from_rotation_z(FRAC_PI_2))).unwrap();
        let transformed = select(ordinary, orientation_only, 19)
            .unwrap()
            .transform_point3(Vec3::X);
        assert!((transformed - Vec3::Y).length() < 1e-6);
        assert_eq!(orientation_only.unwrap().w_axis.truncate(), Vec3::ZERO);
        assert!(select(ordinary, None, 67).is_err());
    }
}
