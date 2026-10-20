//! Birth records, shader parameters, and immutable extraction snapshots.
use crate::effects::blend::EffectBlend;
use crate::effects::records::RecordRing;
use crate::effects::simulation::split_time;
use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::sync_component::SyncComponent;
use bytemuck::{Pod, Zeroable};
use std::f32::consts::{FRAC_PI_8, PI};
use std::sync::Arc;

/// Immutable spawn data indexed by the growing record ring.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct ParticleInstance {
    position_birth: [f32; 4],
    velocity_gravity: [f32; 4],
    // Lifetime, tail marker, low birth time, padding.
    lifetime_tail_birth: [f32; 4],
    // XYZ size multipliers sampled at birth, then XYQuad facing in radians.
    scale_facing: [f32; 4],
    acceleration_drag: [f32; 4],
}

impl ParticleInstance {
    pub(crate) fn new(
        position: Vec3,
        velocity: Vec3,
        gravity: f32,
        birth_time: f64,
        lifetime: f32,
        size_scale: Vec3,
        tail: bool,
    ) -> Self {
        let [high, low] = split_time(birth_time);
        Self {
            position_birth: [position.x, position.y, position.z, high],
            velocity_gravity: [velocity.x, velocity.y, velocity.z, gravity],
            lifetime_tail_birth: [lifetime, f32::from(tail), low, 0.0],
            // XYQuad rotates its texture axes by the initial XY velocity angle
            // minus pi plus pi/8. Store it independently of later velocity changes.
            // atan2 also yields a finite facing for vertical/zero speed.
            scale_facing: [
                size_scale.x,
                size_scale.y,
                size_scale.z,
                velocity.y.atan2(velocity.x) - PI + FRAC_PI_8,
            ],
            acceleration_drag: [0., 0., -gravity, 0.],
        }
    }

    /// Supplies vector acceleration and exponential velocity damping.
    pub(crate) fn with_acceleration(mut self, acceleration: Vec3, drag: f32) -> Self {
        self.acceleration_drag = [acceleration.x, acceleration.y, acceleration.z, drag.max(0.)];
        self
    }
    pub(crate) fn with_facing(mut self, facing: f32) -> Self {
        self.scale_facing[3] = facing;
        self
    }
    pub(crate) fn as_tail(mut self) -> Self {
        self.lifetime_tail_birth[1] = 1.0;
        self
    }

    pub(crate) fn center(&self, uniform: &ParticleEmitterUniform) -> Vec3 {
        let age = ((uniform.clock_tail[0] - self.position_birth[3])
            + (uniform.clock_tail[1] - self.lifetime_tail_birth[2]))
            .max(0.0);
        let position = Vec3::from_slice(&self.position_birth[..3]);
        let velocity = Vec3::from_slice(&self.velocity_gravity[..3]);
        let acceleration = Vec3::from_slice(&self.acceleration_drag[..3]);
        let drag = self.acceleration_drag[3];
        let center = if drag > 0.00001 {
            let velocity_time = (1. - (-drag * age).exp()) / drag;
            position + velocity * velocity_time + acceleration * ((age - velocity_time) / drag)
        } else {
            position + velocity * age + acceleration * (0.5 * age * age)
        };
        if uniform.atlas_flags[2] != 0 {
            Mat4::from_cols_array_2d(&uniform.world_from_local).transform_point3(center)
        } else {
            center
        }
    }
}

/// All fields occupy whole 16-byte lanes, matching the WGSL uniform layout.
#[derive(Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct ParticleEmitterUniform {
    world_from_local: [[f32; 4]; 4],
    colors: [[f32; 4]; 3],
    scaling: [f32; 4],
    intervals: [[u32; 4]; 4],
    atlas_flags: [u32; 4],
    clock_tail: [f32; 4],
    // Unshaded, fog enabled, followed by reserved lanes.
    render_flags: [u32; 4],
    plane_axes: [[f32; 4]; 2],
}

/// Staged sprite appearance supplied independently of any native model records.
#[derive(Clone, Debug)]
pub(crate) struct StagedSpriteParameters {
    pub colors: [[f32; 4]; 3],
    pub scales: [f32; 3],
    pub middle: f32,
    pub intervals: [[u32; 3]; 4],
    pub atlas: [u32; 2],
    pub model_space: bool,
    pub xy_quad: bool,
    pub tail_length: f32,
    pub unlit: bool,
    pub fog_enabled: bool,
    pub fog_blend: u32,
    pub plane_axes: [Vec3; 2],
}
impl ParticleEmitterUniform {
    pub(crate) fn from_staged(
        parameters: &StagedSpriteParameters,
        transform: Mat4,
        time: f64,
    ) -> Self {
        let [high, low] = split_time(time);
        Self {
            world_from_local: transform.to_cols_array_2d(),
            colors: parameters.colors,
            scaling: [
                parameters.scales[0],
                parameters.scales[1],
                parameters.scales[2],
                parameters.middle.clamp(0.001, 0.999),
            ],
            intervals: parameters.intervals.map(|v| [v[0], v[1], v[2], 0]),
            atlas_flags: [
                parameters.atlas[0].max(1),
                parameters.atlas[1].max(1),
                u32::from(parameters.model_space),
                u32::from(parameters.xy_quad),
            ],
            clock_tail: [high, low, parameters.tail_length, 0.],
            render_flags: [
                u32::from(parameters.unlit),
                u32::from(parameters.fog_enabled),
                parameters.fog_blend,
                0,
            ],
            plane_axes: parameters.plane_axes.map(|v| v.extend(0.).to_array()),
        }
    }
}

#[derive(Component, Clone)]
pub(crate) struct ParticleInstances {
    pub(crate) records: Arc<RecordRing<ParticleInstance>>,
    pub(crate) live_indices: Arc<Vec<u32>>,
    pub(crate) uniform: ParticleEmitterUniform,
    pub(crate) texture: Option<Handle<Image>>,
    pub(crate) filter: EffectBlend,
    pub(crate) priority_plane: u32,
    pub(crate) sort_far: bool,
}

impl SyncComponent for ParticleInstances {
    type Target = Self;
}
impl ExtractComponent for ParticleInstances {
    type QueryData = (&'static ParticleInstances, &'static InheritedVisibility);
    type QueryFilter = ();
    type Out = Self;
    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        item.1.get().then(|| item.0.clone())
    }
}

#[cfg(test)]
#[path = "records_tests.rs"]
mod tests;
