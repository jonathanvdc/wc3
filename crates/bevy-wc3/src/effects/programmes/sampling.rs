//! Compact sampled appearance independent of extraction order.
pub(super) fn atlas_sample(cursor: f32, count: u32) -> (u32, u32, f32) {
    let cursor = if cursor.is_finite() {
        cursor.abs().min((count - 1) as f32)
    } else {
        0.
    };
    let cell = (cursor.floor() as u32).min(count - 1);
    (cell, cell.saturating_add(1).min(count - 1), cursor.fract())
}
pub(super) fn lut_cursor(slot: u32, generation: u32, layer: u32) -> f32 {
    let mut seed = slot ^ generation.wrapping_mul(0x9e3779b9) ^ layer.wrapping_mul(0x85ebca6b);
    seed ^= seed >> 16;
    seed = seed.wrapping_mul(0x7feb352d);
    seed ^= seed >> 15;
    (seed >> 8) as f32 * (1. / 16777216.)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn atlas_clamps_before_adjacent_frame_interpolation() {
        assert_eq!(atlas_sample(-2.25, 8), (2, 3, 0.25));
        assert_eq!(atlas_sample(7.75, 8), (7, 7, 0.));
        assert_eq!(atlas_sample(f32::NAN, 8), (0, 1, 0.));
        assert_eq!(atlas_sample(f32::INFINITY, 8), (0, 1, 0.));
        assert_eq!(atlas_sample(9., 1), (0, 0, 0.));
        assert!(atlas_sample(f32::MAX, u32::MAX).1 < u32::MAX);
    }
    #[test]
    fn lut_coordinates_follow_slot_incarnations_and_not_extraction_order() {
        let before = [(1, 4), (8, 2), (3, 1)].map(|(s, g)| lut_cursor(s, g, 5));
        let after = [(3, 1), (1, 4)].map(|(s, g)| lut_cursor(s, g, 5));
        assert_eq!(after, [before[2], before[0]]);
        assert_ne!(lut_cursor(1, 4, 5), lut_cursor(1, 5, 5));
        assert_ne!(lut_cursor(1, 4, 5), lut_cursor(1, 4, 6));
        for s in 0..10000 {
            assert!((0.0..1.0).contains(&lut_cursor(s, 7, 5)));
        }
    }
}
