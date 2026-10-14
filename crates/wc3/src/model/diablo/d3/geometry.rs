//! Native packed geometry values. Coordinate-system conversion belongs to callers.

use super::records::{FatVertex, Quaternion16};

fn direction(word: u32) -> [f32; 3] {
    let bytes = word.to_le_bytes();
    [bytes[0], bytes[1], bytes[2]].map(|component| f32::from(component) / 127.5 - 1.0)
}

fn texture_coordinates(word: u32) -> [f32; 2] {
    let bytes = word.to_le_bytes();
    [
        u16::from_le_bytes([bytes[0], bytes[1]]),
        u16::from_le_bytes([bytes[2], bytes[3]]),
    ]
    .map(|component| f32::from(component) / 512.0 - 64.0)
}

impl FatVertex {
    /// Decodes the stored normal without changing coordinate systems.
    pub fn normal_vector(&self) -> [f32; 3] {
        direction(self.normal)
    }

    /// Decodes the stored tangent without changing coordinate systems.
    pub fn tangent_vector(&self) -> [f32; 3] {
        direction(self.tangent)
    }

    /// Decodes the stored binormal without changing coordinate systems.
    pub fn binormal_vector(&self) -> [f32; 3] {
        direction(self.binormal)
    }

    /// Returns the four stored color channels in red, green, blue, alpha order.
    pub fn rgba(&self) -> [u8; 4] {
        self.color.to_le_bytes()
    }

    /// Decodes the first stored texture coordinate pair.
    pub fn uv0(&self) -> [f32; 2] {
        texture_coordinates(self.tex_coord0)
    }

    /// Decodes the second stored texture coordinate pair.
    pub fn uv1(&self) -> [f32; 2] {
        texture_coordinates(self.tex_coord1)
    }
}

impl Quaternion16 {
    /// Interprets the four signed fixed-point components in X, Y, Z, W order.
    pub fn components(&self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
            .map(|bits| f32::from(i16::from_le_bytes(bits.to_le_bytes())) / 32767.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{direction, texture_coordinates, Quaternion16};

    #[test]
    fn direction_extremes_and_midpoint_straddle_zero() {
        let actual = direction(u32::from_le_bytes([0, 127, 255, 93]));
        assert_eq!(actual[0], -1.0);
        assert_eq!(actual[2], 1.0);
        assert!((actual[1] + 1.0 / 255.0).abs() < 1e-7);
        assert!(direction(128)[0] > 0.0);
    }

    #[test]
    fn texture_axes_have_identical_scale_and_bias() {
        assert_eq!(texture_coordinates(0x8000_8000), [0.0, 0.0]);
        assert_eq!(
            texture_coordinates(0xffff_0000),
            [-64.0, 64.0 - 1.0 / 512.0]
        );
        assert_eq!(texture_coordinates(0x8200_8200), [1.0, 1.0]);
    }

    #[test]
    fn quaternion_preserves_signed_components_and_raw_scale() {
        let q = Quaternion16 {
            x: 0,
            y: 32767,
            z: (-32767_i16) as u16,
            w: 32768,
        };
        let values = q.components();
        assert_eq!(&values[..3], &[0.0, 1.0, -1.0]);
        assert!(values[3] < -1.0);
    }
}
