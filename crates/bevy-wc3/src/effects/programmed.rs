//! Narrow semantic M3 billboard programme; unsupported simulation modes are explicit.
//!
//! The three size and rotation components are particle-age start/middle/end
//! controls. Model-clock properties are sampled at birth; their age controls
//! then evolve independently. Primitive emission and free-flight conventions
//! remain visually unverified against the game, separately from missing modes.
use super::frame::EffectFrame;
use bevy::prelude::*;
#[cfg(test)]
use std::array::from_fn;
use std::collections::HashMap;
use std::f32::consts::TAU;
use wc3::model::animation::programmed::Property;
use wc3::model::values::programmed::{
    EmitterShape, ParticleInstance, Smoothing, Variation, VelocityKind,
};
use wc3::model::{
    Animated, ClipId, Clock, ParticleAppearance, ParticleDefinition, ParticleDetails,
    ParticleGravity, ParticleScalar, ParticleTrigger, ProgrammedAge, RetimedAnimation,
    VariationControl,
};

#[derive(Clone)]
pub(super) struct ProgrammedSprite {
    pub rate: Animated<f32>,
    pub squirt: Animated<u16>,
    pub lifetime: Animated<f32>,
    pub lifetime_other: Animated<f32>,
    pub random_lifetime: bool,
    pub speed: Animated<f32>,
    pub speed_other: Animated<f32>,
    pub random_speed: bool,
    pub shape: EmitterShape,
    pub dimensions: Animated<[f32; 3]>,
    pub radius: Animated<f32>,
    pub velocity: VelocityKind,
    pub angle_x: Animated<f32>,
    pub angle_y: Animated<f32>,
    pub spread_x: Animated<f32>,
    pub spread_y: Animated<f32>,
    pub gravity: f32,
    pub model_space: bool,
    pub max_particles: usize,
    pub lit: bool,
    sizes: Animated<[f32; 3]>,
    rotations: Animated<[f32; 3]>,
    colors: [Animated<[f32; 4]>; 3],
    middle: [f32; 4],
    size_other: Animated<[f32; 3]>,
    rotation_other: Animated<[f32; 3]>,
    color_other: [Animated<[f32; 4]>; 3],
    random_size: bool,
    random_rotation: bool,
    random_color: bool,
    random_alpha: bool,
    smoothing: [Smoothing; 3],
    pub atlas: [u16; 2],
    cells: [u8; 4],
    cell_middle: f32,
}
#[derive(Clone, PartialEq)]
pub(super) struct ProgrammedAppearance {
    size: [f32; 3],
    rotation: [f32; 3],
    color: [[f32; 4]; 3],
    middle: [f32; 4],
    smoothing: [Smoothing; 3],
    cells: [u8; 4],
    cell_middle: f32,
}
pub(super) struct ProgrammedBirth {
    pub lifetime: f32,
    pub origin: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub appearance: ProgrammedAppearance,
}
#[derive(Clone, Copy)]
pub(super) struct SpriteStyle {
    pub color: [f32; 4],
    pub size: [f32; 2],
    pub rotation: f32,
    pub cell: u32,
}
pub(super) fn animated<T: Clone>(property: &Property<T>) -> Animated<T> {
    Animated {
        default: property.initial.clone(),
        property: property.binding.clone(),
    }
}
fn scalar(value: &ParticleScalar) -> Animated<f32> {
    match value {
        ParticleScalar::Animated(value) => value.clone(),
        ParticleScalar::Programmed(property) => animated(property),
    }
}
impl ProgrammedSprite {
    #[cfg(test)]
    pub(super) fn fixture() -> Self {
        Self {
            rate: Animated::constant(10.),
            squirt: Animated::constant(0),
            lifetime: Animated::constant(2.),
            lifetime_other: Animated::constant(4.),
            random_lifetime: false,
            speed: Animated::constant(2.),
            speed_other: Animated::constant(6.),
            random_speed: false,
            shape: EmitterShape::Point,
            dimensions: Animated::constant([2.; 3]),
            radius: Animated::constant(1.),
            velocity: VelocityKind::ZAxis,
            angle_x: Animated::constant(0.),
            angle_y: Animated::constant(0.),
            spread_x: Animated::constant(0.),
            spread_y: Animated::constant(0.),
            gravity: -4.,
            model_space: false,
            max_particles: 20,
            lit: false,
            sizes: Animated::constant([1., 2., 0.]),
            rotations: Animated::constant([0., 1., 2.]),
            colors: [
                Animated::constant([1., 0., 0., 1.]),
                Animated::constant([0., 1., 0., 1.]),
                Animated::constant([0., 0., 1., 0.]),
            ],
            middle: [0.5; 4],
            size_other: Animated::constant([1.; 3]),
            rotation_other: Animated::constant([0.; 3]),
            color_other: from_fn(|_| Animated::constant([1.; 4])),
            random_size: false,
            random_rotation: false,
            random_color: false,
            random_alpha: false,
            smoothing: [Smoothing::Linear; 3],
            atlas: [1, 1],
            cells: [0; 4],
            cell_middle: 0.5,
        }
    }
    pub fn compile(definition: &ParticleDefinition<'_>) -> Result<(Self, Vec<String>), String> {
        let ParticleDetails::Programmed(details) = &definition.details else {
            return Err("Expected programmed sprite controls".into());
        };
        let ParticleAppearance::Programmed(age) = &definition.appearance else {
            return Err("Programmed particle has no age controls".into());
        };
        if details.instance_type != ParticleInstance::Billboard
            || details.flags.model_particles()
            || !details.models.is_empty()
        {
            return Err("Programmed particle instance mode requires mesh, model, trail or orientation geometry; only standard billboards are implemented".into());
        }
        if !matches!(
            details.emitter_shape,
            EmitterShape::Point | EmitterShape::Plane | EmitterShape::Box | EmitterShape::Sphere
        ) {
            return Err("Programmed emitter shape has no implemented sampling distribution".into());
        }
        if details.flags.emit_shape_cutout() {
            return Err("Programmed emitter cutout distribution is not implemented".into());
        }
        if !matches!(
            details.velocity_type,
            VelocityKind::Constant
                | VelocityKind::ZAxis
                | VelocityKind::Random
                | VelocityKind::Radial
        ) {
            return Err("Programmed particle velocity selector is unresolved".into());
        }
        if details.velocity_type == VelocityKind::Radial
            && details.emitter_shape == EmitterShape::Point
        {
            return Err("Point-shaped radial particle direction is undefined".into());
        }
        let gravity=match definition.motion.gravity {
            ParticleGravity::Programmed{x:0,y:0,magnitude}=>magnitude,
            _=>return Err("Programmed particle horizontal gravity words require a resolved vector interpretation".into()),
        };
        if ![
            details.size_smoothing,
            details.rotation_smoothing,
            details.color_smoothing,
        ]
        .iter()
        .all(|mode| matches!(mode, Smoothing::Linear | Smoothing::LinearSmooth))
        {
            return Err(
                "Programmed Bezier/hold/unknown particle-age smoothing is not implemented".into(),
            );
        }
        let mut diagnostics=vec!["M3 primitive billboard motion and linear/smooth age controls are implemented; game fidelity remains unverified".into()];
        if details.direction.drag != 0.
            || details.forces.local_forces.bits() != 0
            || details.forces.world_forces.bits() != 0
            || details.wind_multiplier != 0.
        {
            diagnostics.push("Programmed drag, mass-coupled force channels and world wind require an application force interpreter; only authored Z acceleration is simulated".into());
        }
        if details.noise.noise_amplitude != 0.
            || details.flags.collide_terrain()
            || details.flags.collide_objects()
            || details.collision_spawn.is_some()
            || details.trail.is_some()
            || details.ribbon.is_some()
            || details.projection.is_some()
        {
            diagnostics.push("Programmed noise, collision and secondary-emitter/projection simulation are not implemented".into());
        }
        if details.kill_radius != 0.
            || details.flags.inherit_parent_velocity()
            || details.flags.scale_time_parent()
            || details.flags.local_time()
            || details.flags.simulate_init()
        {
            diagnostics.push("Programmed kill radius, inherited velocity, parent/local time and prewarming controls are not implemented".into());
        }
        if nondefault_scalar(&details.particle_velocity) || nondefault_scalar(&details.phase_shift)
        {
            diagnostics.push(
                "Programmed particle_velocity and phase_shift controls are not implemented".into(),
            );
        }
        if details.flags.bits() & ((1 << 0) | (3 << 7)) != 0 {
            diagnostics.push(
                "Programmed distance/height/reverse particle sorting is not implemented".into(),
            );
        }
        if details.flags.bits() & (63 << 9) != 0 {
            diagnostics.push("Programmed deprecated size/color/rotation smoothing flags are not interpreted; explicit smoothing selectors are used".into());
        }
        if details.flipbook_rows == 0 || details.flipbook_columns == 0 {
            diagnostics.push("Programmed zero-dimensional flipbook is rendered as one cell".into());
        }
        diagnostics.push("Programmed effect variation functions are not implemented; keyed squirt counts use bounded authored clock crossings independently of continuous emission".into());
        let squirt = match &definition.emission.trigger {
            ParticleTrigger::Programmed(value) => animated(value),
            _ => Animated::constant(0),
        };
        Ok((
            Self {
                rate: scalar(&definition.emission.rate),
                squirt,
                lifetime: scalar(&definition.lifetime.duration),
                lifetime_other: scalar(&definition.lifetime.variation),
                random_lifetime: details.additional_flags.lifespan_randomize(),
                speed: scalar(&definition.motion.speed),
                speed_other: scalar(&definition.motion.variation),
                random_speed: details.additional_flags.emit_speed_randomize(),
                shape: details.emitter_shape,
                dimensions: animated(&details.shape_outer),
                radius: animated(&details.outer_radius),
                velocity: details.velocity_type,
                angle_x: animated(&details.direction.initial_yaw),
                angle_y: animated(&details.direction.initial_pitch),
                spread_x: animated(&details.direction.initial_horizontal),
                spread_y: animated(&details.direction.initial_vertical),
                gravity,
                model_space: !details.additional_flags.world_space(),
                max_particles: (details.max_particles as usize).min(8192),
                lit: details.flags.lit_parts(),
                sizes: animated(&age.size_animation),
                rotations: animated(&age.rotation_animation),
                colors: [
                    animated(&age.color_start),
                    animated(&age.color_mid),
                    animated(&age.color_end),
                ],
                middle: [
                    age.size_mid_time,
                    age.rotation_mid_time,
                    age.color_mid_time,
                    age.alpha_mid_time,
                ],
                size_other: animated(&details.size_random_animation),
                rotation_other: animated(&details.rotation_random_animation),
                color_other: [
                    animated(&details.color_start_random),
                    animated(&details.color_mid_random),
                    animated(&details.color_end_random),
                ],
                random_size: details.size_random_enable != 0,
                random_rotation: details.rotation_random_enable != 0,
                random_color: details.color_random_enable != 0,
                random_alpha: details.alpha_random_enable != 0,
                smoothing: [
                    details.size_smoothing,
                    details.rotation_smoothing,
                    details.color_smoothing,
                ],
                atlas: [
                    details.flipbook_rows.max(1),
                    details.flipbook_columns.max(1),
                ],
                cells: [
                    details.flipbook_start_init_index,
                    details.flipbook_start_stop_index,
                    details.flipbook_end_init_index,
                    details.flipbook_end_stop_index,
                ],
                cell_middle: details.flipbook_mid_time,
            },
            diagnostics,
        ))
    }
    pub fn birth(
        &self,
        sample: &RetimedAnimation<'_, '_, '_>,
        node: Mat4,
        basis: Mat4,
        random: [f32; 12],
    ) -> Option<ProgrammedBirth> {
        let choose = |first: f32, second: f32, enabled: bool, value: f32| {
            if enabled {
                first + (second - first) * value
            } else {
                first
            }
        };
        let lifetime = choose(
            sample.animated(&self.lifetime),
            sample.animated(&self.lifetime_other),
            self.random_lifetime,
            random[0],
        );
        let speed = choose(
            sample.animated(&self.speed),
            sample.animated(&self.speed_other),
            self.random_speed,
            random[1],
        );
        if !lifetime.is_finite() || lifetime <= 0. || !speed.is_finite() {
            return None;
        }
        let dimensions = Vec3::from(sample.animated(&self.dimensions)).abs();
        let signed = Vec3::new(
            2. * random[2] - 1.,
            2. * random[3] - 1.,
            2. * random[4] - 1.,
        );
        let origin = match self.shape {
            EmitterShape::Point => Vec3::ZERO,
            EmitterShape::Plane => Vec3::new(
                signed.x * dimensions.x * 0.5,
                signed.y * dimensions.y * 0.5,
                0.,
            ),
            EmitterShape::Box => signed * dimensions * 0.5,
            EmitterShape::Sphere => {
                let z = signed.z;
                let radius = (1. - z * z).max(0.).sqrt();
                let angle = random[2] * TAU;
                Vec3::new(radius * angle.cos(), radius * angle.sin(), z)
                    * sample.animated(&self.radius).abs()
                    * random[3].cbrt()
            }
            _ => return None,
        };
        let direction = match self.velocity {
            VelocityKind::Constant => {
                let x = sample.animated(&self.angle_x)
                    + (random[5] - 0.5) * sample.animated(&self.spread_x);
                let y = sample.animated(&self.angle_y)
                    + (random[6] - 0.5) * sample.animated(&self.spread_y);
                Quat::from_rotation_x(x) * Quat::from_rotation_y(y) * Vec3::Z
            }
            VelocityKind::ZAxis => Vec3::Z,
            VelocityKind::Radial => origin.try_normalize().unwrap_or(Vec3::Z),
            VelocityKind::Random => {
                let z = 2. * random[5] - 1.;
                let radius = (1. - z * z).max(0.).sqrt();
                let angle = random[6] * TAU;
                Vec3::new(radius * angle.cos(), radius * angle.sin(), z)
            }
            _ => return None,
        };
        let acceleration = Vec3::Z * self.gravity;
        let (origin, velocity, acceleration) = if self.model_space {
            (origin, direction * speed, acceleration)
        } else {
            (
                node.transform_point3(origin),
                node.transform_vector3(direction * speed),
                basis.transform_vector3(acceleration),
            )
        };
        Some(ProgrammedBirth {
            lifetime,
            origin,
            velocity,
            acceleration,
            appearance: self.appearance(sample, [random[8], random[9], random[10], random[11]]),
        })
    }
    pub fn appearance(
        &self,
        sample: &RetimedAnimation<'_, '_, '_>,
        random: [f32; 4],
    ) -> ProgrammedAppearance {
        let mut size = sample.animated(&self.sizes);
        if self.random_size {
            let other = sample.animated(&self.size_other);
            for (index, value) in size.iter_mut().enumerate() {
                *value += (other[index] - *value) * random[0];
            }
        }
        let mut rotation = sample.animated(&self.rotations);
        if self.random_rotation {
            let other = sample.animated(&self.rotation_other);
            for (index, value) in rotation.iter_mut().enumerate() {
                *value += (other[index] - *value) * random[1];
            }
        }
        let mut color = [
            sample.animated(&self.colors[0]),
            sample.animated(&self.colors[1]),
            sample.animated(&self.colors[2]),
        ];
        for (index, color) in color.iter_mut().enumerate() {
            let other = sample.animated(&self.color_other[index]);
            for (channel, value) in color.iter_mut().enumerate() {
                if (channel < 3 && self.random_color) || (channel == 3 && self.random_alpha) {
                    *value += (other[channel] - *value) * random[if channel == 3 { 3 } else { 2 }];
                }
            }
        }
        ProgrammedAppearance {
            size,
            rotation,
            color,
            middle: self.middle,
            smoothing: self.smoothing,
            cells: self.cells,
            cell_middle: self.cell_middle,
        }
    }
}
pub(super) fn nondefault_scalar(value: &Property<f32>) -> bool {
    value.initial != 0. || value.binding.is_some()
}
pub(super) fn nondefault_variation(value: &VariationControl) -> bool {
    value.kind != Variation::None
        || nondefault_scalar(&value.amplitude)
        || nondefault_scalar(&value.frequency)
}
/// Bounded occurrence scratch; counts are independent of continuous emission.
#[derive(Default)]
pub(super) struct SquirtSchedule {
    pub occurrences: Vec<(f64, u16)>,
    pub initialized: bool,
}
impl SquirtSchedule {
    pub fn collect(
        &mut self,
        amount: &Animated<u16>,
        frame: &EffectFrame<'_, '_, '_>,
        clips: &HashMap<ClipId, (f64, bool)>,
        delta: f64,
        include_origin: bool,
    ) {
        self.occurrences.clear();
        let track = amount
            .property
            .as_ref()
            .and_then(|target| frame.sample.track(target));
        let clock = track
            .as_ref()
            .map_or(Clock::Clip, |track| track.track.clock);
        let origin = if matches!(clock, Clock::Timeline) {
            frame
                .selection
                .timeline_interval
                .map_or(0., |(start, _)| start.as_seconds())
        } else {
            0.
        };
        let raw = match clock {
            Clock::Clip => frame.selection.clip_time.as_seconds(),
            Clock::Shared(_) => frame.selection.shared_time.as_seconds(),
            Clock::Timeline if frame.selection.timeline_interval.is_some() => {
                origin + frame.selection.clip_time.as_seconds()
            }
            Clock::Timeline => frame.selection.timeline_time.as_seconds(),
        };
        let previous = raw - delta;
        let period =
            track
                .as_ref()
                .and_then(|track| track.period)
                .map(|period| period.as_seconds())
                .or_else(|| {
                    if track.is_some() {
                        return None;
                    }
                    frame.selection.clip.and_then(|id| clips.get(&id)).and_then(
                        |&(duration, looping)| (looping && duration > 0.).then_some(duration),
                    )
                });
        let interval = SquirtInterval {
            time: frame.time,
            raw,
            previous,
            period,
            include_origin,
        };
        let mut add = |key, count| interval.add(&mut self.occurrences, key, count);

        if let Some(track) = track {
            let times = track.track.curve.times();
            for (index, key) in times.iter().enumerate() {
                if times.get(index + 1) == Some(key) {
                    continue;
                }
                if matches!(clock, Clock::Timeline)
                    && frame
                        .selection
                        .timeline_interval
                        .is_some_and(|(start, end)| key < start || key > end)
                {
                    continue;
                }
                // A loop-end key aliases the next loop's origin.
                if period.is_some_and(|period| (key.as_seconds() - origin - period).abs() < 1e-9) {
                    continue;
                }
                add(
                    key.as_seconds(),
                    track.track.curve.values().get(index).unwrap(),
                );
            }
        } else {
            // An inactive bound programme must not fall back to an authored
            // default burst; defaults apply only to genuinely unbound values.
            if amount.property.is_none() {
                add(origin, amount.default);
            }
        }
        self.occurrences.sort_by(|a, b| a.0.total_cmp(&b.0));
        self.initialized = true;
    }
}
struct SquirtInterval {
    time: f64,
    raw: f64,
    previous: f64,
    period: Option<f64>,
    include_origin: bool,
}
impl SquirtInterval {
    fn add(&self, occurrences: &mut Vec<(f64, u16)>, key: f64, count: u16) {
        const MAX_OCCURRENCES: usize = 4096;
        if count == 0 {
            return;
        }
        let mut cycle = self
            .period
            .map_or(0., |period| ((self.previous - key) / period).ceil());
        while occurrences.len() < MAX_OCCURRENCES {
            let occurrence = key + self.period.unwrap_or(0.) * cycle;
            if occurrence > self.raw + 1e-9 {
                break;
            }
            if occurrence > self.previous + 1e-9
                || (self.include_origin && (occurrence - self.previous).abs() < 1e-9)
            {
                occurrences.push((self.time - (self.raw - occurrence), count));
            }
            if self.period.is_none() {
                break;
            }
            cycle += 1.;
        }
    }
}
fn stage(values: [f32; 3], middle: f32, age: f32, mode: Smoothing) -> f32 {
    if age >= 1. {
        return values[2];
    }
    if age <= 0. {
        return values[0];
    }
    let middle = middle.clamp(0., 1.);
    let (a, b, mut t) = if age < middle {
        (
            values[0],
            values[1],
            if middle > 0. { age / middle } else { 1. },
        )
    } else {
        (
            values[1],
            values[2],
            if middle < 1. {
                (age - middle) / (1. - middle)
            } else {
                0.
            },
        )
    };
    t = t.clamp(0., 1.);
    if mode == Smoothing::LinearSmooth {
        t = t * t * (3. - 2. * t);
    }
    a + (b - a) * t
}
impl ProgrammedAppearance {
    #[cfg(test)]
    pub(super) fn constant(value: f32) -> Self {
        Self {
            size: [value; 3],
            rotation: [0.; 3],
            color: [[1.; 4]; 3],
            middle: [0.5; 4],
            smoothing: [Smoothing::Linear; 3],
            cells: [0; 4],
            cell_middle: 0.5,
        }
    }
    pub fn from_age(
        age: &ProgrammedAge,
        sample: &RetimedAnimation<'_, '_, '_>,
        size_smoothing: Smoothing,
        color_smoothing: Smoothing,
    ) -> Self {
        let value3 = |property: &Property<[f32; 3]>| {
            property
                .binding
                .as_ref()
                .and_then(|binding| sample.property(binding))
                .unwrap_or(property.initial)
        };
        let value4 = |property: &Property<[f32; 4]>| {
            property
                .binding
                .as_ref()
                .and_then(|binding| sample.property(binding))
                .unwrap_or(property.initial)
        };
        Self {
            size: value3(&age.size_animation),
            rotation: value3(&age.rotation_animation),
            color: [
                value4(&age.color_start),
                value4(&age.color_mid),
                value4(&age.color_end),
            ],
            middle: [
                age.size_mid_time,
                age.rotation_mid_time,
                age.color_mid_time,
                age.alpha_mid_time,
            ],
            smoothing: [size_smoothing, Smoothing::Linear, color_smoothing],
            cells: [0; 4],
            cell_middle: 0.5,
        }
    }
    pub fn sample(&self, age: f32) -> SpriteStyle {
        let size = stage(self.size, self.middle[0], age, self.smoothing[0]);
        let mut color = [0.; 4];
        for (index, value) in color.iter_mut().enumerate() {
            *value = stage(
                [
                    self.color[0][index],
                    self.color[1][index],
                    self.color[2][index],
                ],
                self.middle[if index == 3 { 3 } else { 2 }],
                age,
                self.smoothing[2],
            );
        }
        color[3] = color[3].clamp(0., 1.);
        let middle = self.cell_middle.clamp(0., 1.);
        let (start, end, t) = if age >= 1. {
            (self.cells[3], self.cells[3], 1.)
        } else if age <= 0. {
            (self.cells[0], self.cells[0], 0.)
        } else if age < middle {
            (
                self.cells[0],
                self.cells[1],
                if middle > 0. { age / middle } else { 1. },
            )
        } else {
            (
                self.cells[2],
                self.cells[3],
                if middle < 1. {
                    (age - middle) / (1. - middle)
                } else {
                    0.
                },
            )
        };
        let cell = (f32::from(start) + (f32::from(end) - f32::from(start)) * t.clamp(0., 1.))
            .floor()
            .max(0.) as u32;
        SpriteStyle {
            color,
            size: [size; 2],
            rotation: stage(self.rotation, self.middle[1], age, self.smoothing[1]),
            cell,
        }
    }
}
#[cfg(test)]
#[path = "programmed_tests.rs"]
mod tests;
