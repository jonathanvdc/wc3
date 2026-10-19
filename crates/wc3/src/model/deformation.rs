//! CPU deformation shared with renderer snapshot adapters.
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
        if transforms.len() != count
            || (!self.normals.is_empty() && self.normals.len() != count)
            || (!self.tangents.is_empty() && self.tangents.len() != count)
        {
            return Err(Error::new(
                "geometry.deform",
                "inconsistent vertex stream count",
            ));
        }
        if self
            .positions
            .iter()
            .zip(transforms)
            .any(|(p, m)| !m.transform_point3(*p).is_finite())
        {
            return Err(Error::new("geometry.deform", "nonfinite posed positions"));
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
    for row in influences {
        let mut matrix = Mat4::ZERO;
        let mut positive = false;
        for (index, weight) in row {
            if !weight.is_finite() || weight < 0. {
                return Err(Error::new("geometry.skin", "invalid skin weight"));
            }
            if weight > 0. {
                matrix += *joints
                    .get(index)
                    .ok_or_else(|| Error::new("geometry.skin", "invalid pose joint index"))?
                    * weight;
                positive = true;
            }
        }
        if !positive {
            return Err(Error::new(
                "geometry.skin",
                "vertex has no positive joint contribution",
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
            return Err(Error::new(
                "geometry.skin",
                "inconsistent compact joint palette",
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
        for position in positions {
            let point = *position;
            if !point.is_finite() {
                return Err(Error::new("geometry.bounds", "nonfinite position"));
            }
            min = min.min(point);
            max = max.max(point);
        }
        let radius = ((max.as_dvec3() - min.as_dvec3()) * 0.5).length() as f32;
        if !radius.is_finite() {
            return Err(Error::new("geometry.bounds", "bounding radius exceeds f32"));
        }
        Ok(Some(Self {
            minimum: min,
            maximum: max,
            sphere_radius: radius,
        }))
    }
}
