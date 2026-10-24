//! Bevy frame selection for the shared model coordinate conversion.
use crate::ModelError;
use bevy::prelude::*;
use wc3::model::{Axis, Basis, Handedness, LengthUnit, Model};

pub(super) fn axis(axis: Axis) -> Vec3 {
    axis.vector()
}
pub(crate) fn render_basis() -> Basis {
    Basis::new(Axis::PositiveY, Axis::NegativeZ, Handedness::Right)
        .expect("independent Bevy frame axes")
}
pub(super) fn basis(model: &Model<'_>) -> Result<Transform, ModelError> {
    model
        .coordinate_transform(render_basis(), LengthUnit::MetersPerUnit(1.))
        .map(Transform::from_matrix)
        .map_err(ModelError::from_source)
}
