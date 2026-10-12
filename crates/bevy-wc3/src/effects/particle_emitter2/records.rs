//! Birth records, shader parameters, and immutable extraction snapshots.
use crate::effects::records::RecordRing;
use crate::effects::simulation::split_time;
use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::sync_component::SyncComponent;
use bytemuck::{Pod, Zeroable};
use std::array::from_fn;
use std::f32::consts::{FRAC_PI_8, PI};
use std::sync::Arc;
use wc3::model::mdlx::emitters::{Particle2FilterMode, ParticleEmitter2};

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
        }
    }

    pub(crate) fn as_tail(mut self) -> Self {
        self.lifetime_tail_birth[1] = 1.0;
        self
    }

    #[cfg(test)]
    pub(crate) fn spawn_scale(&self) -> Vec3 {
        Vec3::from_slice(&self.scale_facing[..3])
    }

    #[cfg(test)]
    pub(crate) fn is_tail(&self) -> bool {
        self.lifetime_tail_birth[1] > 0.5
    }

    pub(crate) fn center(&self, uniform: &ParticleEmitterUniform) -> Vec3 {
        let age = ((uniform.clock_tail[0] - self.position_birth[3])
            + (uniform.clock_tail[1] - self.lifetime_tail_birth[2]))
            .max(0.0);
        let position = Vec3::from_slice(&self.position_birth[..3]);
        let velocity = Vec3::from_slice(&self.velocity_gravity[..3]);
        let center =
            position + velocity * age - Vec3::Z * (0.5 * self.velocity_gravity[3] * age * age);
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
    // Unshaded, followed by reserved lanes.
    render_flags: [u32; 4],
}

impl ParticleEmitterUniform {
    pub(crate) fn new(
        definition: &ParticleEmitter2<'static>,
        transform: &GlobalTransform,
        time: f64,
    ) -> Self {
        let [high, low] = split_time(time);
        Self {
            world_from_local: transform.to_matrix().to_cols_array_2d(),
            colors: from_fn(|i| {
                let color = definition.segment_colors[i];
                [
                    color[0],
                    color[1],
                    color[2],
                    definition.alpha[i] as f32 / 255.0,
                ]
            }),
            scaling: [
                definition.particle_scaling[0],
                definition.particle_scaling[1],
                definition.particle_scaling[2],
                definition.time.clamp(0.001, 0.999),
            ],
            intervals: definition
                .uv_animations
                .map(|interval| [interval[0], interval[1], interval[2], 0]),
            atlas_flags: [
                definition.rows.max(1),
                definition.columns.max(1),
                u32::from(definition.node.flags.model_space()),
                u32::from(definition.node.flags.xy_quad()),
            ],
            clock_tail: [high, low, definition.tail_length, 0.0],
            render_flags: [u32::from(definition.node.flags.unshaded()), 0, 0, 0],
        }
    }
}

#[derive(Component, Clone)]
pub(crate) struct ParticleInstances {
    pub(crate) records: Arc<RecordRing<ParticleInstance>>,
    pub(crate) live_indices: Arc<Vec<u32>>,
    pub(crate) uniform: ParticleEmitterUniform,
    pub(crate) texture: Option<Handle<Image>>,
    pub(crate) filter: Particle2FilterMode,
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
