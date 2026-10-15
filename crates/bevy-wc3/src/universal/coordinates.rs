//! Model frame conversion shared by geometry, rig, and effect compilation.
use crate::ModelError;
use bevy::prelude::*;
use wc3::model::{Axis, Handedness, LengthUnit, Model};

pub(super) fn axis(axis: Axis) -> Vec3 {
    match axis {
        Axis::PositiveX => Vec3::X,
        Axis::NegativeX => Vec3::NEG_X,
        Axis::PositiveY => Vec3::Y,
        Axis::NegativeY => Vec3::NEG_Y,
        Axis::PositiveZ => Vec3::Z,
        Axis::NegativeZ => Vec3::NEG_Z,
    }
}

pub(super) fn basis(model: &Model<'_>) -> Result<Transform, ModelError> {
    let authored = model.basis();
    let up = axis(authored.up());
    let forward = axis(authored.forward());
    let right = match authored.handedness() {
        Handedness::Right => forward.cross(up),
        Handedness::Left => up.cross(forward),
    };
    // Bevy's canonical frame is +Y up, -Z forward, +X right.
    let orientation = Mat3::from_cols(right, up, -forward).transpose();
    let scale = match model.units() {
        LengthUnit::Unspecified => 1.,
        LengthUnit::MetersPerUnit(value) => value as f32,
    };
    if !scale.is_finite() || scale <= 0. {
        return Err(ModelError(
            "physical model scale cannot be represented by Bevy f32 transforms".into(),
        ));
    }
    Ok(Transform::from_matrix(
        Mat4::from_mat3(orientation) * Mat4::from_scale(Vec3::splat(scale)),
    ))
}
