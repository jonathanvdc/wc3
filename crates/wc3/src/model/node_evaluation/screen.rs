//! Camera-screen reconstruction independent of native node declarations.
use crate::model::BillboardAxis;
use glam::{Mat3, Mat4, Quat, Vec3};

/// Remove affine camera distortion in model-to-view order. The returned camera
/// frame is right handed; screen reconstruction handles view handedness once.
pub(super) fn camera_rotation(camera: Mat3) -> Option<Quat> {
    if !camera.is_finite() || camera.determinant() == 0. {
        return None;
    }
    let view = camera.inverse();
    let x = normalized(view.x_axis)?;
    let y = normalized(view.y_axis - x * view.y_axis.dot(x))?;
    let z = normalized(view.z_axis - x * view.z_axis.dot(x) - y * view.z_axis.dot(y))?;
    let mut view = Mat3::from_cols(x, y, z);
    if view.determinant() < 0. {
        view.x_axis.z = -view.x_axis.z;
        view.y_axis.z = -view.y_axis.z;
        view.z_axis.z = -view.z_axis.z;
    }
    Some(Quat::from_mat3(&view.transpose()).normalize())
}

/// Preserve one composed axis and reconstruct its free axes against the image
/// plane. Parallel/collapsed frames leave the caller's original affine intact.
pub(super) fn screen_axis(
    before: Mat4,
    camera: Quat,
    pivot: Vec3,
    axis: BillboardAxis,
) -> Option<(Mat4, Quat, Vec3)> {
    let view = Mat3::from_quat(camera).transpose();
    let flip = |v: Vec3| Vec3::new(v.x, v.y, -v.z);
    let input = Mat3::from_mat4(before);
    let lengths = Vec3::new(
        input.x_axis.length(),
        input.y_axis.length(),
        input.z_axis.length(),
    );
    if !lengths.is_finite() {
        return None;
    }
    let (x, y, z) = match axis {
        BillboardAxis::X => {
            let x = normalized(flip(view * input.x_axis))?;
            let y = normalized(Vec3::new(x.y, -x.x, 0.))?;
            (x, y, y.cross(x))
        }
        BillboardAxis::Y => {
            let y = normalized(flip(view * input.y_axis))?;
            let x = normalized(Vec3::new(-y.y, y.x, 0.))?;
            (x, y, y.cross(x))
        }
        BillboardAxis::Z => {
            let z = normalized(flip(view * input.z_axis))?;
            let y = normalized(Vec3::new(z.y, -z.x, 0.))?;
            (z.cross(y), y, z)
        }
    };
    let rotation = Mat3::from_cols(camera * flip(x), camera * flip(y), camera * flip(z));
    let anchor = before.transform_point3(pivot);
    let orientation = Quat::from_mat3(&rotation).normalize();
    let matrix = Mat4::from_scale_rotation_translation(
        lengths,
        orientation,
        anchor - rotation * (lengths * pivot),
    );
    matrix.is_finite().then_some((matrix, orientation, lengths))
}

fn normalized(value: Vec3) -> Option<Vec3> {
    let length = value.length();
    (length.is_finite() && length > 1e-8).then(|| value / length)
}
