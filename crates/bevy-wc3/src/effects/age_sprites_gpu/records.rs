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

#[derive(Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct AgeSpriteUniform {
    pub axes: [[f32; 4]; 2],
    /// Fixed XY plane, velocity facing, unlit, fog enabled.
    pub flags: [u32; 4],
    /// Atlas rows, columns, composition programme, blend code.
    pub atlas: [u32; 4],
}

#[derive(Clone)]
pub(crate) struct AgeSpriteStyle {
    pub textures: [Option<Handle<Image>>; 3],
    pub blend: ParticleBlend,
    pub priority: f32,
    pub composition: u32,
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
