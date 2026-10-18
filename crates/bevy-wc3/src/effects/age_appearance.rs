//! Age-driven twinkle and independent secondary texture motion.
//!
//! Random streams are deterministic renderer choices; the native RNG and its
//! random table are deliberately not reproduced.
use super::simulation::EmitterRng;
use bevy::prelude::Vec2;
use std::array::from_fn;
use std::sync::Arc;
use wc3::model::M2ParticleDetails;

#[derive(Clone)]
pub(super) struct AgeAppearance {
    twinkle: [f32; 4],
    table: Option<Arc<[f32; 128]>>,
    textures: Option<TextureMotion>,
}
#[derive(Clone)]
struct TextureMotion {
    scales: [f32; 2],
    middle: [Vec2; 2],
    range: [Vec2; 2],
}
/// Optional per-birth sidecar, separate from the common motion record.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct AgeParticleAppearance {
    phase: u8,
    origins: [Vec2; 2],
    velocities: [Vec2; 2],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AgeStyle {
    pub visible: bool,
    pub scale: f32,
    pub offsets: [Vec2; 2],
}
impl Default for AgeStyle {
    fn default() -> Self {
        Self {
            visible: true,
            scale: 1.,
            offsets: [Vec2::ZERO; 2],
        }
    }
}
impl AgeAppearance {
    pub fn new(details: &M2ParticleDetails<'_>, emitter_seed: u32) -> Self {
        let mut random = EmitterRng::new(emitter_seed ^ 0x9e3779b9);
        let scroll = details.rendering.scroll.unwrap_or([0.; 8]);
        Self {
            twinkle: [
                details.twinkle_speed,
                details.twinkle_percent,
                details.twinkle_scale[0],
                details.twinkle_scale[1],
            ],
            table: (details.twinkle_percent < 1.
                || details.twinkle_scale[0] != details.twinkle_scale[1])
                .then(|| Arc::new(from_fn(|_| random.random()))),
            textures: details.flags.multi_texture().then(|| TextureMotion {
                scales: details.rendering.texture_scale,
                middle: [
                    Vec2::new(scroll[0], scroll[1]),
                    Vec2::new(scroll[2], scroll[3]),
                ],
                range: [
                    Vec2::new(scroll[4], scroll[5]),
                    Vec2::new(scroll[6], scroll[7]),
                ],
            }),
        }
    }
    /// Retain twinkle while omitting unresolved additional texture programmes.
    pub fn primary_only(mut self) -> Self {
        self.textures = None;
        self
    }
    pub fn needs_particle_state(&self) -> bool {
        self.textures.is_some() || self.twinkle[1] < 1. || self.twinkle[2] != self.twinkle[3]
    }
    /// These scales belong to the additional textures, never to the atlas input.
    pub fn texture_scales(&self) -> [f32; 2] {
        self.textures
            .as_ref()
            .map_or([1.; 2], |textures| textures.scales)
    }
    pub fn birth(&self, seed: u32) -> AgeParticleAppearance {
        let mut random = EmitterRng::new(seed ^ 0x85ebca6b);
        let mut birth = AgeParticleAppearance {
            phase: ((random.random() * 128.) as u8).min(127),
            origins: [Vec2::ZERO; 2],
            velocities: [Vec2::ZERO; 2],
        };
        if let Some(textures) = &self.textures {
            for layer in 0..2 {
                birth.origins[layer] = Vec2::new(random.random(), random.random());
                // Each layer's X/Y velocity variation uses the same random
                // scalar, preserving the authored vector's correlation.
                birth.velocities[layer] =
                    textures.middle[layer] + textures.range[layer] * random.signed();
            }
        }
        birth
    }
    pub fn sample(&self, birth: &AgeParticleAppearance, age_seconds: f32) -> AgeStyle {
        let random = self.table.as_ref().map_or(0., |table| {
            let tick = (f64::from(age_seconds.max(0.)) * f64::from(self.twinkle[0].max(0.)))
                .floor()
                .rem_euclid(128.) as usize;
            table[(usize::from(birth.phase) + tick) & 127]
        });
        AgeStyle {
            visible: random <= self.twinkle[1],
            scale: self.twinkle[2] + (self.twinkle[3] - self.twinkle[2]) * random,
            offsets: if self.textures.is_none() {
                [Vec2::ZERO; 2]
            } else {
                from_fn(|layer| {
                    let origin = birth.origins[layer];
                    let velocity = birth.velocities[layer];
                    Vec2::new(
                        (f64::from(origin.x)
                            + f64::from(age_seconds.max(0.)) * f64::from(velocity.x))
                        .rem_euclid(1.) as f32,
                        (f64::from(origin.y)
                            + f64::from(age_seconds.max(0.)) * f64::from(velocity.y))
                        .rem_euclid(1.) as f32,
                    )
                })
            },
        }
    }
}

#[cfg(test)]
#[path = "age_appearance_tests.rs"]
mod tests;
