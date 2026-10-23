//! Exact transfer receipts; each prepare family owns its own accumulator.
use bevy::prelude::*;

/// Actual GPU buffer transfers, excluding allocated-but-unwritten capacity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EffectUploadTraffic {
    /// Bytes passed to executed RenderQueue::write_buffer calls.
    pub queue_bytes: u64,
    /// Executed RenderQueue::write_buffer calls, including zero-byte calls.
    pub queue_writes: u64,
    /// Bytes initialized by create_buffer_with_data, independently of queue writes.
    pub initial_bytes: u64,
    /// Buffers initialized through create_buffer_with_data.
    pub initializations: u64,
}
impl EffectUploadTraffic {
    pub(crate) fn queued(&mut self, bytes: usize) {
        self.queue_bytes = self.queue_bytes.saturating_add(bytes as u64);
        self.queue_writes = self.queue_writes.saturating_add(1);
    }
    pub(crate) fn initialized(&mut self, bytes: usize) {
        self.initial_bytes = self.initial_bytes.saturating_add(bytes as u64);
        self.initializations = self.initializations.saturating_add(1);
    }
    pub(crate) fn add(&mut self, other: Self) {
        self.queue_bytes = self.queue_bytes.saturating_add(other.queue_bytes);
        self.queue_writes = self.queue_writes.saturating_add(other.queue_writes);
        self.initial_bytes = self.initial_bytes.saturating_add(other.initial_bytes);
        self.initializations = self.initializations.saturating_add(other.initializations);
    }
}
/// Cumulative, saturating executed shared-effect transfers in one render world.
/// Read after render completion; subtract two snapshots to inspect an observation.
/// Meshes, textures and unrelated Bevy buffers are outside these counters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EffectUploadStats {
    /// Stable immutable-appearance birth blocks.
    pub birth_records: EffectUploadTraffic,
    /// CPU sampled M2, RPID and programme geometry records.
    pub sampled_records: EffectUploadTraffic,
    /// PRE2 and trail ring ranges.
    pub resident_records: EffectUploadTraffic,
    /// Unsorted source index buffers.
    pub natural_indices: EffectUploadTraffic,
    /// Actual per-view sorted order buffers.
    pub view_orders: EffectUploadTraffic,
    /// Shared effect-pass uniforms.
    pub uniforms: EffectUploadTraffic,
}
impl EffectUploadStats {
    pub(crate) fn add(&mut self, other: Self) {
        self.birth_records.add(other.birth_records);
        self.sampled_records.add(other.sampled_records);
        self.resident_records.add(other.resident_records);
        self.natural_indices.add(other.natural_indices);
        self.view_orders.add(other.view_orders);
        self.uniforms.add(other.uniforms);
    }
}
// Separate resources do not serialize otherwise independent prepare systems.
#[derive(Resource, Default)]
pub(crate) struct SpriteUploads(pub EffectUploadStats);
#[derive(Resource, Default)]
pub(crate) struct TrailUploads(pub EffectUploadStats);
#[derive(Resource, Default)]
pub(crate) struct SampledUploads(pub EffectUploadStats);

/// Snapshot the completed render world's shared effect-buffer traffic.
/// This does not synchronize rendering. A main world has no render receipts.
pub fn effect_upload_stats(render_world: &World) -> EffectUploadStats {
    let mut stats = EffectUploadStats::default();
    if let Some(value) = render_world.get_resource::<SpriteUploads>() {
        stats.add(value.0);
    }
    if let Some(value) = render_world.get_resource::<TrailUploads>() {
        stats.add(value.0);
    }
    if let Some(value) = render_world.get_resource::<SampledUploads>() {
        stats.add(value.0);
    }
    stats
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_family_receipts_sum_without_counting_reserved_capacity() {
        let mut world = World::new();
        let mut sprites = SpriteUploads::default();
        sprites.0.resident_records.queued(80);
        sprites.0.uniforms.initialized(64);
        world.insert_resource(sprites);
        let mut sampled = SampledUploads::default();
        sampled.0.birth_records.queued(4096);
        sampled.0.uniforms.queued(64);
        world.insert_resource(sampled);
        let stats = effect_upload_stats(&world);
        assert_eq!(stats.birth_records.queue_bytes, 4096);
        assert_eq!(stats.resident_records.queue_bytes, 80);
        assert_eq!(stats.uniforms.queue_writes, 1);
        assert_eq!(stats.uniforms.initializations, 1);
        assert_eq!(stats.uniforms.initial_bytes, 64);
        assert_eq!(stats.view_orders, EffectUploadTraffic::default());
        assert_eq!(
            effect_upload_stats(&World::new()),
            EffectUploadStats::default()
        );
    }
}
