//! Typed occurrence controls and deterministic M2 birth construction.
//! Ordinary emitters sample these controls at each occurrence; adopted templates
//! retain one immutable time-zero sample and use exactly the same construction.
use super::super::bones::BoneCandidate;
use super::super::generators::spline;
use super::{random, signed, Birth, BirthAppearance, Emitter};
use bevy::prelude::{Mat4, Vec2, Vec3};
use wc3::model::values::model::ParticleGenerator;
use wc3::model::ValueSample;

/// Shared bounded occurrence count, independent of continuous/burst policy.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct BirthPlan {
    pub requested: u64,
    pub accepted: usize,
    pub omitted: u64,
}
impl BirthPlan {
    pub(super) fn new(requested: u64, limit: usize) -> Self {
        let accepted = requested.min(limit as u64) as usize;
        Self {
            requested,
            accepted,
            omitted: requested - accepted as u64,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct BirthInputs {
    pub enabled: bool,
    pub lifespan: f32,
    pub lifespan_variation: f32,
    pub speed: f32,
    pub speed_variation: f32,
    pub vertical: f32,
    pub horizontal: f32,
    pub width: f32,
    pub length: f32,
    pub z_source: f32,
    pub gravity: Vec3,
}
impl BirthInputs {
    pub(super) fn sample(emitter: &Emitter, sample: &impl ValueSample) -> Self {
        let controls = emitter
            .controls
            .as_ref()
            .expect("age-driven emitter controls");
        Self {
            enabled: sample.animated(&controls.activation),
            lifespan: sample.animated(&emitter.lifespan),
            lifespan_variation: controls.lifespan_variation,
            speed: sample.animated(&emitter.speed),
            speed_variation: sample.animated(&controls.speed_variation),
            vertical: sample.animated(&controls.vertical),
            horizontal: sample.animated(&controls.horizontal),
            width: sample.animated(&controls.width),
            length: sample.animated(&controls.length),
            z_source: sample.animated(&controls.z_source),
            gravity: sample.animated(&controls.gravity),
        }
    }
}
#[derive(Clone, Copy)]
pub(super) struct BirthFrame {
    pub time: f64,
    pub node: Mat4,
    pub basis: Mat4,
    pub inherited: Vec3,
    pub ordinal: u64,
}
impl Emitter {
    pub(super) fn birth_sampled(
        &self,
        inputs: BirthInputs,
        frame: BirthFrame,
        bone_pose: impl FnMut(BoneCandidate) -> Option<(Vec3, Option<Vec3>)>,
    ) -> Option<(Birth, Option<BirthAppearance>)> {
        let BirthFrame {
            time,
            node,
            basis,
            inherited,
            ordinal,
        } = frame;
        if !node.is_finite() || node.determinant() == 0. {
            return None;
        }
        let mut seed = self.seed ^ ordinal.wrapping_mul(0xd6e8feb86659fd93);

        if !inputs.enabled {
            return None;
        }
        let lifetime = inputs.lifespan + signed(&mut seed) * inputs.lifespan_variation;
        if !lifetime.is_finite() || lifetime <= 0. {
            return None;
        }
        let speed = inputs.speed * (1. + signed(&mut seed) * inputs.speed_variation);
        let vertical = inputs.vertical;
        let horizontal = inputs.horizontal;
        let theta = signed(&mut seed) * vertical;
        let phi = signed(&mut seed) * horizontal;
        let mut direction = if self.generator == ParticleGenerator::Sphere {
            Vec3::new(
                theta.cos() * phi.cos(),
                theta.cos() * phi.sin(),
                theta.sin(),
            )
        } else {
            Vec3::new(
                theta.sin() * phi.cos(),
                theta.sin() * phi.sin(),
                theta.cos(),
            )
        };
        let width = inputs.width;
        let length = inputs.length;
        let origin = match self.generator {
            ParticleGenerator::Plane => Vec3::new(
                signed(&mut seed) * length * 0.5,
                signed(&mut seed) * width * 0.5,
                0.,
            ),
            ParticleGenerator::Sphere => {
                let radius = length + (width - length) * random(&mut seed);
                direction * radius
            }
            ParticleGenerator::Spline => {
                let (position, tangent) = spline(&self.spline, length, width, random(&mut seed));
                direction = tangent;
                position
            }
            ParticleGenerator::Bone => {
                let draws = [
                    random(&mut seed),
                    random(&mut seed),
                    random(&mut seed),
                    random(&mut seed),
                ];
                let model_origin = self
                    .bones
                    .as_ref()?
                    .sample(draws, width, length, bone_pose)?;
                direction = Vec3::Z;
                (node.inverse() * basis).transform_point3(model_origin)
            }
            _ => return None,
        };
        let z_source = inputs.z_source;
        let aimed = if self.generator == ParticleGenerator::Plane {
            z_source > 0.001
        } else {
            z_source != 0.
        };
        if aimed {
            direction = (origin - Vec3::Z * z_source)
                .try_normalize()
                .unwrap_or(direction);
        }
        if !aimed && self.generator == ParticleGenerator::Sphere && self.flags.sphere_up() {
            direction = Vec3::Z;
        }
        let gravity = inputs.gravity;
        let local = self.flags.attached();
        let (origin, mut velocity, acceleration) = if local {
            (origin, direction * speed, gravity + self.wind)
        } else {
            (
                node.transform_point3(origin),
                node.transform_vector3(direction * speed),
                basis.transform_vector3(gravity + self.wind),
            )
        };
        velocity += if local {
            node.inverse().transform_vector3(inherited)
        } else {
            inherited
        };
        let x_variation = (1. + signed(&mut seed) * self.appearance.scale_variation[0]).max(0.);
        let y_variation = if self.flags.independent_scale() {
            (1. + signed(&mut seed) * self.appearance.scale_variation[1]).max(0.)
        } else {
            x_variation
        };
        let scale_variation = Vec2::new(x_variation, y_variation);
        Some((
            Birth {
                appearance_index: u32::MAX,
                time,
                lifetime,
                origin,
                velocity,
                acceleration,
                spin: self.spin[0] + signed(&mut seed) * self.spin[1],
                spin_rate: self.spin[2] + signed(&mut seed) * self.spin[3],
                scale_variation,
                size_scale: self.particle_scale(node, basis),
                random_cell: if self.flags.random_texture() {
                    (random(&mut seed)
                        * f32::from(self.atlas[0].max(1))
                        * f32::from(self.atlas[1].max(1))) as u32
                } else {
                    0
                },
            },
            self.age_appearance
                .as_ref()
                .filter(|appearance| appearance.needs_particle_state())
                .map(|appearance| BirthAppearance::Age(appearance.birth(seed as u32))),
        ))
    }
}
