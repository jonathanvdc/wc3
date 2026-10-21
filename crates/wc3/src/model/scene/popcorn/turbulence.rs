//! Original coherent scalar-potential curl field; native stochastic fidelity is unverified.
use super::runtime::mix;
use glam::{DVec3, Vec3};
use std::array::from_fn;
use std::f64::consts::TAU;

type Result<T> = std::result::Result<T, &'static str>;
const MAX_OCTAVES: u32 = 24;
const BASIS_SIZE: usize = 256;
const FREQUENCY_BANDS: usize = 8;
#[cfg(test)]
use std::cell::Cell;
#[cfg(test)]
use std::thread_local;
#[cfg(test)]
thread_local! {
    static CORNER_VISITS: Cell<usize> = const { Cell::new(0) };
    static OSCILLATOR_PAIRS: Cell<usize> = const { Cell::new(0) };
}
// Keep f64 spatial fractions and angular phases resolved rather than saturating casts.
const DOMAIN_LIMIT: f64 = 1_099_511_627_776.; // 2^40
#[derive(Clone, Copy, Debug)]
pub(crate) struct Parameters {
    pub scale: f32,
    pub strength: f32,
    pub wavelength: f32,
    pub octaves: u32,
    pub lacunarity: f32,
    pub gain: f32,
    pub gain_multiplier: f32,
    pub interpolator: i32,
    pub time_scale: f32,
    pub time_base: f32,
    pub time_variation: f32,
    pub seed: u32,
}
impl Default for Parameters {
    fn default() -> Self {
        Self {
            scale: 1.,
            strength: 0.1,
            wavelength: 0.5,
            octaves: 2,
            lacunarity: 0.5,
            gain: 0.5,
            gain_multiplier: 1.,
            interpolator: 1,
            time_scale: 0.,
            time_base: 0.,
            time_variation: 0.5,
            seed: 1_114_229_502,
        }
    }
}
#[derive(Clone, Copy, Debug)]
struct Octave {
    wavelength: f64,
    amplitude: f64,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Phase {
    oscillators: [(f64, f64); FREQUENCY_BANDS],
}
#[derive(Clone, Copy, Debug)]
struct BasisValue {
    sine: f64,
    cosine: f64,
    band: usize,
}
#[derive(Clone, Debug)]
struct Frozen {
    values: Box<[f64; BASIS_SIZE]>,
    phase: Phase,
}
#[derive(Clone, Debug)]
pub(crate) struct Turbulence {
    parameters: Parameters,
    octaves: Box<[Octave]>,
    basis: Box<[BasisValue; BASIS_SIZE]>,
    frequencies: [f64; FREQUENCY_BANDS],
    frozen: Option<Frozen>,
}
impl Turbulence {
    pub(crate) fn new(parameters: Parameters) -> Result<Self> {
        let p = parameters;
        if [
            p.scale,
            p.strength,
            p.wavelength,
            p.lacunarity,
            p.gain,
            p.gain_multiplier,
            p.time_scale,
            p.time_base,
            p.time_variation,
        ]
        .iter()
        .any(|v| !v.is_finite())
        {
            return Err("turbulence controls must be finite");
        }
        if p.scale <= 0.
            || p.wavelength <= 0.
            || p.lacunarity <= 0.
            || p.time_variation < 0.
            || p.octaves > MAX_OCTAVES
            || !(0..=2).contains(&p.interpolator)
        {
            return Err("unsupported turbulence scale, wavelength, lacunarity, variation, octave count or interpolator");
        }
        let mut wavelength = f64::from(p.scale) * f64::from(p.wavelength);
        let mut amplitude = f64::from(p.scale) * f64::from(p.strength);
        let mut octaves = Vec::with_capacity(p.octaves.max(1) as usize);
        for _ in 0..p.octaves.max(1) {
            if !wavelength.is_finite()
                || wavelength <= 0.
                || !wavelength.recip().is_finite()
                || !amplitude.is_finite()
            {
                return Err("turbulence octave frequency or amplitude exceeds finite range");
            }
            octaves.push(Octave {
                wavelength,
                amplitude,
            });
            wavelength *= f64::from(p.lacunarity);
            amplitude *= f64::from(p.gain) * f64::from(p.gain_multiplier);
        }
        let mut result = Self {
            parameters,
            octaves: octaves.into_boxed_slice(),
            basis: Box::new(from_fn(|index| {
                let hash = mix(u64::from(p.seed) ^ index as u64);
                let (sine, cosine) = (unit(hash) * TAU).sin_cos();
                BasisValue {
                    sine,
                    cosine,
                    band: (mix(hash) as usize) & (FREQUENCY_BANDS - 1),
                }
            })),
            frequencies: from_fn(|band| {
                let deviation = (band as f64 + 0.5) * (2. / FREQUENCY_BANDS as f64) - 1.;
                TAU * (1. + f64::from(p.time_variation) * deviation)
            }),
            frozen: None,
        };
        if p.time_scale == 0. {
            let phase = result.phase(0.)?;
            let values = Box::new(from_fn(|index| result.basis[index].value(&phase)));
            result.frozen = Some(Frozen { values, phase });
        }
        Ok(result)
    }
    fn phase(&self, time: f64) -> Result<Phase> {
        if !time.is_finite() {
            return Err("turbulence effect time must be finite");
        }
        let tau =
            time * f64::from(self.parameters.time_scale) + f64::from(self.parameters.time_base);
        let angles = self.frequencies.map(|frequency| tau * frequency);
        if angles
            .iter()
            .any(|angle| !angle.is_finite() || angle.abs() > DOMAIN_LIMIT)
        {
            return Err("turbulence temporal phase exceeds the checked precision range");
        }
        Ok(Phase {
            oscillators: angles.map(|angle| {
                #[cfg(test)]
                OSCILLATOR_PAIRS.with(|count| count.set(count.get() + 1));
                angle.rem_euclid(TAU).sin_cos()
            }),
        })
    }
    pub(crate) fn sample(&self, point: Vec3, phase: &Phase) -> Result<Vec3> {
        // The exact unavailable sentinel propagates without inventing a zero field.
        if point == Vec3::splat(f32::INFINITY) {
            return Ok(point);
        }
        if !point.is_finite() {
            return Err("turbulence query must be finite or exactly unavailable");
        }
        let mut velocity = DVec3::ZERO;
        for octave in &self.octaves {
            let query = point.as_dvec3() / octave.wavelength;
            if !query.is_finite() || query.abs().max_element() > DOMAIN_LIMIT {
                return Err("turbulence query exceeds the checked lattice precision range");
            }
            if octave.amplitude == 0. {
                continue;
            }
            let gradients: [DVec3; 3] =
                from_fn(|channel| self.potential(query, phase, channel as u64).1);
            // Gradient is in lattice coordinates: amplitude must NOT gain 1/wavelength.
            velocity += DVec3::new(
                gradients[2].y - gradients[1].z,
                gradients[0].z - gradients[2].x,
                gradients[1].x - gradients[0].y,
            ) * octave.amplitude;
        }
        let result = velocity.as_vec3();
        if result.is_finite() {
            Ok(result)
        } else {
            Err("turbulence velocity exceeds finite Float3 range")
        }
    }
    fn potential(&self, query: DVec3, phase: &Phase, channel: u64) -> (f64, DVec3) {
        let floor = query.floor();
        let base = [floor.x as i64, floor.y as i64, floor.z as i64];
        let fraction = query - floor;
        let smooth = |t: f64| match self.parameters.interpolator {
            0 => (t, 1.),
            1 => (t * t * (3. - 2. * t), 6. * t * (1. - t)),
            _ => (
                t * t * t * (t * (6. * t - 15.) + 10.),
                30. * t * t * (t - 1.) * (t - 1.),
            ),
        };
        let weights = [smooth(fraction.x), smooth(fraction.y), smooth(fraction.z)];
        let mut value = 0.;
        let mut gradient = DVec3::ZERO;
        for corner in 0..8 {
            let bits: [usize; 3] = from_fn(|axis| (corner >> axis) & 1);
            let mut hash = mix(u64::from(self.parameters.seed) ^ channel);
            for axis in 0..3 {
                hash = mix(hash ^ (base[axis] + bits[axis] as i64) as u64);
            }
            #[cfg(test)]
            CORNER_VISITS.with(|count| count.set(count.get() + 1));
            let index = (hash as usize) & (BASIS_SIZE - 1);
            let amplitude = self.frozen.as_ref().map_or_else(
                || self.basis[index].value(phase),
                |frozen| frozen.values[index],
            );
            let w: [f64; 3] = from_fn(|axis| {
                if bits[axis] == 0 {
                    1. - weights[axis].0
                } else {
                    weights[axis].0
                }
            });
            let d: [f64; 3] = from_fn(|axis| {
                if bits[axis] == 0 {
                    -weights[axis].1
                } else {
                    weights[axis].1
                }
            });
            value += amplitude * w[0] * w[1] * w[2];
            gradient +=
                DVec3::new(d[0] * w[1] * w[2], w[0] * d[1] * w[2], w[0] * w[1] * d[2]) * amplitude;
        }
        (value, gradient)
    }
}
impl BasisValue {
    fn value(self, phase: &Phase) -> f64 {
        let (sine, cosine) = phase.oscillators[self.band];
        self.sine * cosine + self.cosine * sine
    }
}
fn unit(word: u64) -> f64 {
    (word >> 11) as f64 / 9_007_199_254_740_992.
}
#[derive(Clone, Copy)]
struct TemporalEntry {
    sampler: usize,
    time: u64,
    phase: Phase,
}
/// Small per-instance effect-time cache; no particle RNG, allocation or shared mutation.
#[derive(Default)]
pub(crate) struct TemporalCache {
    entries: [Option<TemporalEntry>; 4],
    next: usize,
}
impl TemporalCache {
    pub(crate) fn phase<'a>(
        &'a mut self,
        sampler: usize,
        resource: &'a Turbulence,
        time: f64,
    ) -> Result<&'a Phase> {
        if !time.is_finite() {
            return Err("turbulence effect time must be finite");
        }
        if let Some(frozen) = &resource.frozen {
            return Ok(&frozen.phase);
        }
        if let Some(index) = self.entries.iter().position(|entry| {
            entry
                .as_ref()
                .is_some_and(|entry| entry.sampler == sampler && entry.time == time.to_bits())
        }) {
            return Ok(&self.entries[index].as_ref().unwrap().phase);
        }
        let phase = resource.phase(time)?;
        let index = self.next;
        self.entries[index] = Some(TemporalEntry {
            sampler,
            time: time.to_bits(),
            phase,
        });
        self.next = (self.next + 1) % self.entries.len();
        Ok(&self.entries[index].as_ref().unwrap().phase)
    }
}
#[cfg(test)]
#[path = "turbulence_tests.rs"]
mod tests;
