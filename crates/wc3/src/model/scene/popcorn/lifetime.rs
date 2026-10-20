//! Checked finite and authored instantaneous particle lifetimes.
#[derive(Clone, Copy, Debug)]
enum Rate {
    Finite(f32),
    Instant,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Lifetime {
    ratio: f32,
    rate: Rate,
}
impl Lifetime {
    pub fn read(ratio: f32, inverse: f32) -> Result<Self, &'static str> {
        if !ratio.is_finite() || inverse.is_nan() || inverse < 0. {
            return Err("particle lifetime requires a finite ratio and a nonnegative finite or instantaneous rate");
        }
        let rate = if inverse == f32::INFINITY {
            Rate::Instant
        } else {
            Rate::Finite(inverse)
        };
        Ok(Self { ratio, rate })
    }
    pub fn advance(self, seconds: f64) -> f32 {
        match self.rate {
            Rate::Instant if seconds > 0. => 1.,
            Rate::Instant => self.ratio,
            Rate::Finite(inverse) => (self.ratio + seconds as f32 * inverse).min(1.),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn instant_waits_for_positive_elapsed_time_and_invalid_nonfinite_controls_fail() {
        let instant = Lifetime::read(0., f32::INFINITY).unwrap();
        assert_eq!(instant.advance(0.), 0.);
        assert_eq!(instant.advance(f64::MIN_POSITIVE), 1.);
        assert_eq!(Lifetime::read(0.25, 0.).unwrap().advance(100.), 0.25);
        for invalid in [f32::NAN, f32::NEG_INFINITY, -1.] {
            assert!(Lifetime::read(0., invalid).is_err());
        }
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(Lifetime::read(invalid, 1.).is_err());
        }
    }
}
