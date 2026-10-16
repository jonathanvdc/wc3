//! Explicit coordinate-space policy for externally instantiated child models.
use super::coordinates::basis;
use bevy::prelude::*;
use wc3::model::Model;

pub use wc3::model::CoordinatePolicy;

/// Mount policy retained on a request so reloading always recomputes from source.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct ChildCoordinates {
    pub coordinates: CoordinatePolicy,
    pub parent_basis: Transform,
}
impl ChildCoordinates {
    pub(crate) fn from_model(model: &Model<'_>, coordinates: CoordinatePolicy) -> Self {
        // Compilation validates this same frame before exposing the definitions.
        Self {
            coordinates,
            parent_basis: basis(model).unwrap_or_default(),
        }
    }
    pub(crate) fn basis(self, child_basis: Transform) -> Transform {
        match self.coordinates {
            // inverse(child_basis) * child_basis is exactly identity. Avoid
            // unnecessary inversion error when retaining authored coordinates.
            CoordinatePolicy::Host => Transform::IDENTITY,
            CoordinatePolicy::Child => Transform::from_matrix(
                self.parent_basis.to_matrix().inverse() * child_basis.to_matrix(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn close(actual: Mat4, expected: Mat4) {
        for (actual, expected) in actual
            .to_cols_array()
            .into_iter()
            .zip(expected.to_cols_array())
        {
            assert!((actual - expected).abs() < 1e-5, "{actual} != {expected}");
        }
    }
    #[test]
    fn authored_and_physical_mounts_preserve_distinct_frame_and_unit_policies() {
        let parent =
            Transform::from_rotation(Quat::from_rotation_x(-1.2)).with_scale(Vec3::splat(0.01));
        let child =
            Transform::from_rotation(Quat::from_rotation_z(0.7)).with_scale(Vec3::splat(2.));
        let node = Transform::from_xyz(3., 4., 5.)
            .with_rotation(Quat::from_rotation_y(0.3))
            .to_matrix();
        let authored = ChildCoordinates {
            coordinates: CoordinatePolicy::Host,
            parent_basis: parent,
        };
        let physical = ChildCoordinates {
            coordinates: CoordinatePolicy::Child,
            parent_basis: parent,
        };
        close(
            parent.to_matrix() * node * authored.basis(child).to_matrix(),
            parent.to_matrix() * node,
        );
        close(
            parent.to_matrix() * node * physical.basis(child).to_matrix(),
            parent.to_matrix() * node * parent.to_matrix().inverse() * child.to_matrix(),
        );
        assert!((authored.basis(child).scale - physical.basis(child).scale).length() > 1.);
    }
    #[test]
    fn reload_recomputes_from_original_frame_without_accumulating_correction() {
        let placement = ChildCoordinates {
            coordinates: CoordinatePolicy::Child,
            parent_basis: Transform::from_rotation(Quat::from_rotation_x(0.9))
                .with_scale(Vec3::splat(0.02)),
        };
        let original =
            Transform::from_rotation(Quat::from_rotation_z(0.2)).with_scale(Vec3::splat(0.5));
        let first = placement.basis(original);
        for _ in 0..100 {
            close(placement.basis(original).to_matrix(), first.to_matrix());
        }
    }
}
