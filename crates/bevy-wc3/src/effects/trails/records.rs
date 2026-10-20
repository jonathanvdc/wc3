//! Birth records, shader parameters, and immutable extraction snapshots.
use crate::effects::blend::EffectBlend;
use crate::effects::records::RecordRing;
use crate::effects::simulation::split_time;
use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::sync_component::SyncComponent;
use bytemuck::{Pod, Zeroable};
use std::sync::Arc;

/// Birth-time endpoints. The split clock retains precision in long-running scenes.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct RibbonSection {
    pub(crate) above_birth: [f32; 4],
    pub(crate) below_birth: [f32; 4],
}

impl RibbonSection {
    pub(crate) fn new(above: Vec3, below: Vec3, birth: f64) -> Self {
        let [high, low] = split_time(birth);
        Self {
            above_birth: [above.x, above.y, above.z, high],
            below_birth: [below.x, below.y, below.z, low],
        }
    }

    pub(super) fn center(&self, uniform: &RibbonUniform) -> Vec3 {
        let age = ((uniform.clock_gravity[0] - self.above_birth[3])
            + (uniform.clock_gravity[1] - self.below_birth[3]))
            .max(0.0);
        let axis = Vec3::from_slice(&uniform.gravity_axis[..3]);
        let axis = if axis == Vec3::ZERO { Vec3::Z } else { axis };
        (Vec3::from_slice(&self.above_birth[..3]) + Vec3::from_slice(&self.below_birth[..3])) * 0.5
            - axis * (0.5 * uniform.clock_gravity[2] * age * age)
    }
}

/// Endpoint ring indices, position in the chain, and number of segments in that chain.
/// Draw sorting moves this entire record without changing connectivity or UVs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct RibbonSegment(pub(crate) [u32; 4]);

pub(super) fn segment_center(
    records: &RecordRing<RibbonSection>,
    segment: RibbonSegment,
    uniform: &RibbonUniform,
) -> Vec3 {
    (records[segment.0[0] as usize].center(uniform)
        + records[segment.0[1] as usize].center(uniform))
        * 0.5
}

#[derive(Clone, Copy, Default, Debug, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct RibbonUniform {
    pub(crate) color: [f32; 4],
    pub(crate) clock_gravity: [f32; 4],
    pub(crate) atlas_flags: [u32; 4],
    pub(crate) uv_transform: [[f32; 4]; 4],
    pub(crate) gravity_axis: [f32; 4],
    pub(crate) render_flags: [u32; 4],
}

#[derive(Component, Clone)]
pub(crate) struct RibbonInstances {
    pub(crate) records: Arc<RecordRing<RibbonSection>>,
    pub(crate) live_indices: Arc<Vec<RibbonSegment>>,
    pub(crate) uniform: RibbonUniform,
    pub(crate) texture: Option<Handle<Image>>,
    pub(crate) filter: EffectBlend,
    pub(crate) priority_plane: i32,
    pub(crate) emitter: Entity,
    pub(crate) layer_index: usize,
    pub(crate) sort_far: bool,
    pub(crate) sort_near: bool,
    pub(crate) no_depth_test: bool,
    pub(crate) no_depth_set: bool,
    pub(crate) two_sided: bool,
}

impl SyncComponent for RibbonInstances {
    type Target = Self;
}
impl ExtractComponent for RibbonInstances {
    type QueryData = (&'static RibbonInstances, &'static InheritedVisibility);
    type QueryFilter = ();
    type Out = Self;
    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        item.1.get().then(|| item.0.clone())
    }
}
