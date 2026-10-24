//! CPU deformation shared with renderer snapshot adapters.
use crate::model::error::{GeometryError, Location};
use crate::model::{Bounds, Error, ProjectedGeometry, SkinWeights};
use glam::Vec4;
use glam::{Mat4, Vec3};

/// Borrowed editable geometry streams. Other application attributes remain untouched.
pub struct DeformableGeometry<'a> {
    /// Positions to deform.
    pub positions: &'a mut [Vec3],
    /// Optional normals, empty when absent.
    pub normals: &'a mut [Vec3],
    /// Optional tangents and handedness, empty when absent.
    pub tangents: &'a mut [Vec4],
}
impl DeformableGeometry<'_> {
    /// Applies one affine matrix per vertex. Normals use inverse transpose;
    /// singular normal transforms become zero, and reflections flip tangent handedness.
    /// Validation precedes mutation. Other streams and vertex ordering are unchanged.
    pub fn deform(&mut self, transforms: &[Mat4]) -> Result<(), Error> {
        let count = self.positions.len();
        for (field, actual, optional) in [
            ("transforms", transforms.len(), false),
            ("normals", self.normals.len(), true),
            ("tangents", self.tangents.len(), true),
        ] {
            if actual != count && (!optional || actual != 0) {
                return Err(Error::at(
                    Location::new().field(field),
                    GeometryError::CountMismatch {
                        expected: count,
                        actual,
                    },
                ));
            }
        }
        for (index, (position, matrix)) in self.positions.iter().zip(transforms).enumerate() {
            let point = matrix.transform_point3(*position);
            if let Some((component, value)) = point
                .to_array()
                .into_iter()
                .enumerate()
                .find(|(_, value)| !value.is_finite())
            {
                return Err(Error::at(
                    Location::new()
                        .field("positions")
                        .index(index)
                        .index(component),
                    GeometryError::NonFinite { value },
                ));
            }
        }
        for (position, matrix) in self.positions.iter_mut().zip(transforms) {
            *position = matrix.transform_point3(*position);
        }
        for (normal, matrix) in self.normals.iter_mut().zip(transforms) {
            *normal = if matrix.determinant().abs() <= 1e-12 {
                Vec3::ZERO
            } else {
                matrix
                    .inverse()
                    .transpose()
                    .transform_vector3(*normal)
                    .normalize_or_zero()
            };
        }
        for (tangent, matrix) in self.tangents.iter_mut().zip(transforms) {
            let sign = if matrix.determinant() < 0. { -1. } else { 1. };
            *tangent = matrix
                .transform_vector3(tangent.truncate())
                .normalize_or_zero()
                .extend(tangent[3] * sign);
        }
        Ok(())
    }
}
/// Constructs weighted matrices using the same sum as GPU linear blend skinning.
/// `influences` yields compact joint indices and normalized weights for each vertex.
/// Empty rows are rejected. Rigid geometry should use its placement matrix directly.
pub fn skin_matrices<I, R>(
    influences: I,
    joints: &[Mat4],
    output: &mut Vec<Mat4>,
) -> Result<(), Error>
where
    I: IntoIterator<Item = R>,
    R: IntoIterator<Item = (usize, f32)>,
{
    output.clear();
    for (vertex, row) in influences.into_iter().enumerate() {
        let mut matrix = Mat4::ZERO;
        let mut positive = false;
        let mut sum = 0.;
        for (influence, (index, weight)) in row.into_iter().enumerate() {
            if !weight.is_finite() || weight < 0. {
                return Err(Error::at(
                    Location::new()
                        .field("influences")
                        .index(vertex)
                        .index(influence)
                        .field("weight"),
                    GeometryError::InvalidWeight { weight },
                ));
            }
            if weight > 0. {
                matrix += *joints.get(index).ok_or_else(|| {
                    Error::at(
                        Location::new()
                            .field("influences")
                            .index(vertex)
                            .index(influence)
                            .field("joint"),
                        GeometryError::IndexOutOfBounds {
                            index,
                            len: joints.len(),
                        },
                    )
                })? * weight;
                positive = true;
                sum += weight;
            }
        }
        if !positive {
            return Err(Error::at(
                Location::new().field("influences").index(vertex),
                GeometryError::NoPositiveWeights { sum },
            ));
        }
        output.push(matrix);
    }
    Ok(())
}
impl SkinWeights {
    /// Fills reusable deformation matrices. Joint transforms use compact palette order.
    pub fn matrices_into(&self, joints: &[Mat4], output: &mut Vec<Mat4>) -> Result<(), Error> {
        if joints.len() != self.palette.len() {
            return Err(Error::at(
                Location::new().field("palette"),
                GeometryError::CountMismatch {
                    expected: self.palette.len(),
                    actual: joints.len(),
                },
            ));
        }
        skin_matrices(
            self.rows.iter().map(|row| row.iter().copied()),
            joints,
            output,
        )
    }
}
impl ProjectedGeometry {
    /// Deforms editable geometry while retaining source mappings, colors, and UV streams.
    pub fn deform(&mut self, transforms: &[Mat4]) -> Result<(), Error> {
        DeformableGeometry {
            positions: &mut self.positions,
            normals: &mut self.normals,
            tangents: &mut self.tangents,
        }
        .deform(transforms)
    }
    /// Computes finite bounds; an empty batch has no bounds.
    pub fn bounds(&self) -> Result<Option<Bounds>, Error> {
        Bounds::from_positions(&self.positions)
    }
}
impl Bounds {
    /// Computes an axis-aligned box and a conservative sphere around its center.
    pub fn from_positions(positions: &[Vec3]) -> Result<Option<Self>, Error> {
        if positions.is_empty() {
            return Ok(None);
        }
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for (index, position) in positions.iter().enumerate() {
            let point = *position;
            if !point.is_finite() {
                let (component, value) = point
                    .to_array()
                    .into_iter()
                    .enumerate()
                    .find(|(_, value)| !value.is_finite())
                    .expect("nonfinite point");
                return Err(Error::at(
                    Location::new()
                        .field("positions")
                        .index(index)
                        .index(component),
                    GeometryError::NonFinite { value },
                ));
            }
            min = min.min(point);
            max = max.max(point);
        }
        let wide_radius = ((max.as_dvec3() - min.as_dvec3()) * 0.5).length();
        let radius = wide_radius as f32;
        if !radius.is_finite() {
            return Err(Error::at(
                Location::new().field("sphere_radius"),
                GeometryError::RadiusOverflow {
                    radius: wide_radius,
                },
            ));
        }
        Ok(Some(Self {
            minimum: min,
            maximum: max,
            sphere_radius: radius,
        }))
    }
}
