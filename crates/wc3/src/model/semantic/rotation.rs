//! Rotation values are distinct from colors and arbitrary four-component vectors.
/// Quaternion components in x, y, z, w order.
/// Draft components are checked when inserted into a curve or model. Semantic spline
/// derivatives and control points use separate finite component-vector domains.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternion(pub [f32; 4]);
impl Quaternion {
    /// Rotation that leaves vectors unchanged.
    pub const IDENTITY: Self = Self([0., 0., 0., 1.]);
    /// Components in x, y, z, w order.
    pub const fn components(self) -> [f32; 4] {
        self.0
    }
    /// Whether all components are finite and the quaternion has nonzero norm.
    pub fn is_rotation(self) -> bool {
        self.0.iter().all(|value| value.is_finite())
            && self
                .0
                .iter()
                .map(|value| f64::from(*value).powi(2))
                .sum::<f64>()
                > 0.
    }
}
impl Default for Quaternion {
    fn default() -> Self {
        Self::IDENTITY
    }
}
