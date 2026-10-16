//! Bevy frame selection for the shared model coordinate conversion.
use crate::ModelError;
use bevy::prelude::*;
use wc3::model::{Axis, Basis, Handedness, LengthUnit, Model};

pub(super) fn axis(axis: Axis) -> Vec3 {
    axis.vector()
}
pub(super) fn basis(model: &Model<'_>) -> Result<Transform, ModelError> {
    let target = Basis::new(Axis::PositiveY, Axis::NegativeZ, Handedness::Right)
        .expect("independent Bevy frame axes");
    model
        .coordinate_transform(target, LengthUnit::MetersPerUnit(1.))
        .map(Transform::from_matrix)
        .map_err(|error| ModelError(error.to_string()))
}
