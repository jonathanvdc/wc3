//! Original isotropic vector sampling with explicit finite shell policies.
use glam::Vec3;
use std::f32::consts::TAU;
/// Angles consume the first two draws; the optional shell radius consumes the third.
pub(crate) fn vector(draws: [f32; 3], radii: Option<(f32, f32)>) -> Result<Vec3, &'static str> {
    let count = if radii.is_some() { 3 } else { 2 };
    if draws[..count]
        .iter()
        .any(|draw| !draw.is_finite() || !(0.0..=1.0).contains(draw))
    {
        return Err("random-vector draws must be finite unit fractions");
    }
    let radius = if let Some((first, second)) = radii {
        if !first.is_finite() || !second.is_finite() {
            return Err("random-vector shell radii must be finite");
        }
        let first = first.max(0.);
        let second = second.max(0.);
        let minimum = first.min(second);
        let maximum = first.max(second);
        if maximum == 0. {
            0.
        } else {
            // Normalize the cubic volume before interpolation to avoid overflow
            // for finite large radii while keeping the shell distribution intact.
            let ratio = minimum / maximum;
            maximum * (ratio.powi(3) * (1. - draws[2]) + draws[2]).cbrt()
        }
    } else {
        1.
    };
    let azimuth = TAU * draws[0];
    let polar = 1. - 2. * draws[1];
    let radial = (1. - polar * polar).max(0.).sqrt();
    let (sin, cos) = azimuth.sin_cos();
    let direction = Vec3::new(radial * cos, polar, radial * sin);
    let value = direction * radius;
    if value.is_finite() {
        Ok(value)
    } else {
        Err("random-vector result exceeds finite range")
    }
}
#[cfg(test)]
#[path = "random_tests.rs"]
mod tests;
