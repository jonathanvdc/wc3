use crate::effects::render::SortPolicy;
use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::sync_component::SyncComponent;
use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wc3::model::values::model::ParticleBlend;

/// One exact CPU appearance/motion sample, shared by head and tail and all views.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct AgeSpriteRecord {
    pub position: [f32; 4],
    pub velocity_tail: [f32; 4],
    pub size_spin: [f32; 4],
    pub color: [f32; 4],
    pub offsets: [f32; 4],
    pub scales: [f32; 4],
    pub cells: [u32; 4],
}

impl AgeSpriteRecord {
    /// Sampled-only lanes: next atlas cell, interpolation fraction, stable LUT V.
    /// M2 multi-texture and refraction keep their original record interpretation.
    pub(crate) fn set_sampled_appearance(&mut self, next: u32, fraction: f32, lut_v: f32) {
        self.cells[2] = next;
        self.scales[2] = fraction;
        self.scales[3] = lut_v;
    }
}

#[derive(Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct AgeSpriteUniform {
    pub axes: [[f32; 4]; 2],
    /// Fixed XY plane, velocity facing, unlit, fog enabled.
    pub flags: [u32; 4],
    /// Atlas rows, columns, composition flags plus V flip, blend code.
    pub atlas: [u32; 4],
}

/// Local UV operations, after image orientation and before atlas placement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SpriteUv {
    pub rotate: bool,
    pub flip_u: bool,
    pub flip_v: bool,
}

/// Known native composition controls, independent of opaque TXAC selectors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum AgeSpriteComposition {
    #[default]
    Single,
    Refraction,
    Sampled {
        uv: SpriteUv,
        crossfade: bool,
        alpha_remap: bool,
    },
    Multi {
        three_colors: bool,
        alpha_modulate4x: bool,
    },
}
impl AgeSpriteComposition {
    /// GPU composition bits; sampled UV controls do not overlap native multi/refraction bits.
    pub(crate) fn flags(self) -> u32 {
        match self {
            Self::Single => 0,
            Self::Refraction => 16,
            Self::Sampled {
                uv,
                crossfade,
                alpha_remap,
            } => {
                32 | (u32::from(crossfade) << 6)
                    | (u32::from(alpha_remap) << 7)
                    | (u32::from(uv.rotate) << 8)
                    | (u32::from(uv.flip_u) << 9)
                    | (u32::from(uv.flip_v) << 10)
            }
            Self::Multi {
                three_colors,
                alpha_modulate4x,
            } => 1 | (u32::from(three_colors) << 1) | (u32::from(alpha_modulate4x) << 2),
        }
    }
}

#[derive(Clone)]
pub(crate) struct AgeSpriteStyle {
    pub textures: [Option<Handle<Image>>; 3],
    pub blend: ParticleBlend,
    pub priority: f32,
    pub sort: SortPolicy,
    pub composition: AgeSpriteComposition,
    /// Reverse authored image V while keeping geometric corners unchanged.
    pub uv_flip_v: bool,
    pub unlit: bool,
    pub head: bool,
    pub tail: bool,
}

#[derive(Component, Clone)]
pub(crate) struct AgeSpriteData {
    pub records: Arc<Vec<AgeSpriteRecord>>,
    pub live_indices: Arc<Vec<u32>>,
    pub uniform: AgeSpriteUniform,
    pub style: AgeSpriteStyle,
}

/// Recycle immutable extraction snapshots once the render world releases them.
/// Replacing a shared Vec before clearing would copy a whole previous frame.
#[derive(Default)]
pub(crate) struct SampleBuffers {
    records: Vec<Arc<Vec<AgeSpriteRecord>>>,
    indices: Vec<Arc<Vec<u32>>>,
}
impl SampleBuffers {
    pub(crate) fn begin(&mut self) -> (&mut Arc<Vec<AgeSpriteRecord>>, &mut Arc<Vec<u32>>) {
        (writable(&mut self.records), writable(&mut self.indices))
    }
}
fn writable<T>(pool: &mut Vec<Arc<Vec<T>>>) -> &mut Arc<Vec<T>> {
    let index = if let Some(index) = pool
        .iter()
        .position(|buffer| Arc::strong_count(buffer) == 1)
    {
        index
    } else {
        let capacity = pool
            .iter()
            .map(|buffer| buffer.capacity())
            .max()
            .unwrap_or(0);
        pool.push(Arc::new(Vec::with_capacity(capacity)));
        pool.len() - 1
    };
    Arc::get_mut(&mut pool[index]).unwrap().clear();
    &mut pool[index]
}
impl SyncComponent for AgeSpriteData {
    type Target = Self;
}
impl ExtractComponent for AgeSpriteData {
    type QueryData = (&'static AgeSpriteData, &'static InheritedVisibility);
    type QueryFilter = ();
    type Out = Self;
    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        item.1.get().then(|| item.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampled_lanes_preserve_motion_and_other_texture_parameters() {
        let mut record = AgeSpriteRecord {
            position: [2.; 4],
            velocity_tail: [3.; 4],
            size_spin: [4.; 4],
            color: [5.; 4],
            offsets: [6.; 4],
            scales: [7.; 4],
            cells: [8; 4],
        };
        record.set_sampled_appearance(19, 0.25, 0.75);
        assert_eq!(record.position, [2.; 4]);
        assert_eq!(record.velocity_tail, [3.; 4]);
        assert_eq!(record.size_spin, [4.; 4]);
        assert_eq!(record.color, [5.; 4]);
        assert_eq!(record.offsets, [6.; 4]);
        assert_eq!(record.scales, [7., 7., 0.25, 0.75]);
        assert_eq!(record.cells, [8, 8, 19, 8]);
        assert_eq!(size_of::<AgeSpriteRecord>(), 112);
    }

    #[test]
    fn sampled_flags_do_not_enable_multi_or_refraction() {
        let sampled = AgeSpriteComposition::Sampled {
            uv: SpriteUv {
                rotate: true,
                flip_u: true,
                flip_v: true,
            },
            crossfade: true,
            alpha_remap: true,
        }
        .flags();
        assert_eq!(sampled & 31, 0);
        assert_eq!(AgeSpriteComposition::Refraction.flags(), 16);
        assert_eq!(
            AgeSpriteComposition::Multi {
                three_colors: true,
                alpha_modulate4x: true
            }
            .flags(),
            7
        );
    }

    #[test]
    fn live_samples_preserve_extracted_frames_and_reuse_released_allocations() {
        let mut samples = SampleBuffers::default();
        let (records, indices) = samples.begin();
        Arc::get_mut(records)
            .unwrap()
            .push(AgeSpriteRecord::zeroed());
        Arc::get_mut(indices).unwrap().push(17);
        let first_pointer = records.as_ptr();
        let first_records = records.clone();
        let first_indices = indices.clone();

        let (records, indices) = samples.begin();
        Arc::get_mut(records).unwrap().push(AgeSpriteRecord {
            position: [1.; 4],
            ..AgeSpriteRecord::zeroed()
        });
        Arc::get_mut(indices).unwrap().push(24);
        let second_records = records.clone();
        let second_indices = indices.clone();
        assert_eq!(first_records[0].position, [0.; 4]);
        assert_eq!(first_indices.as_slice(), &[17]);
        drop(first_records);
        drop(first_indices);

        let (records, indices) = samples.begin();
        assert!(records.is_empty());
        assert!(indices.is_empty());
        assert_eq!(records.as_ptr(), first_pointer);
        assert_eq!(second_records[0].position, [1.; 4]);
        assert_eq!(second_indices.as_slice(), &[24]);
    }
}
