//! Growing record rings with cheap immutable extraction snapshots and birth-range uploads.
use bevy::render::render_resource::Buffer;
use bevy::render::renderer::RenderQueue;
use bytemuck::{cast_slice, Pod};
use std::ops::{Index, Range};
use std::sync::Arc;

// Copy only a small block when render extraction still owns an older snapshot.
const BLOCK_RECORDS: usize = 64;
const INITIAL_RECORDS: usize = 16;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RingCursor {
    generation: u64,
    writes: u64,
    capacity: usize,
}

/// Physical records remain stationary between growth events. Births append at
/// `writes % capacity`; retirement advances the logical head by reducing `len`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RecordRing<T> {
    blocks: Vec<Arc<Vec<T>>>,
    cursor: RingCursor,
    len: usize,
}

impl<T> Default for RecordRing<T> {
    fn default() -> Self {
        Self {
            blocks: Vec::new(),
            cursor: RingCursor::default(),
            len: 0,
        }
    }
}

impl<T> RecordRing<T> {
    pub(crate) fn len(&self) -> usize {
        self.len
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub(crate) fn capacity(&self) -> usize {
        self.cursor.capacity
    }
    pub(crate) fn cursor(&self) -> RingCursor {
        self.cursor
    }

    /// Convert a chronological live-record index into its physical GPU index.
    pub(crate) fn index(&self, logical: usize) -> u32 {
        assert!(logical < self.len);
        ((self
            .cursor
            .writes
            .wrapping_sub(self.len as u64)
            .wrapping_add(logical as u64))
            % self.capacity() as u64) as u32
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len).map(move |i| &self[self.index(i) as usize])
    }

    pub(crate) fn pop_front(&mut self) {
        assert!(self.len > 0);
        self.len -= 1;
    }

    /// Determine writes since the GPU's last preparation, including skipped
    /// extraction frames. Growth or a full lap requires a complete upload.
    pub(crate) fn upload_ranges(&self, previous: Option<RingCursor>) -> [Range<usize>; 2] {
        let Some(previous) = previous.filter(|previous| {
            previous.generation == self.cursor.generation && previous.capacity == self.capacity()
        }) else {
            return [0..self.capacity(), 0..0];
        };
        let Some(writes) = self
            .cursor
            .writes
            .checked_sub(previous.writes)
            .filter(|&writes| writes < self.capacity() as u64)
        else {
            return [0..self.capacity(), 0..0];
        };
        if writes == 0 {
            return [0..0, 0..0];
        }
        let start = (previous.writes % self.capacity() as u64) as usize;
        let end = start + writes as usize;
        [
            start..end.min(self.capacity()),
            0..end.saturating_sub(self.capacity()),
        ]
    }
}

impl<T: Pod> RecordRing<T> {
    pub(crate) fn push(&mut self, record: T, limit: usize) {
        assert!(self.len < limit);
        if self.len == self.capacity() {
            let capacity = (self.capacity() * 2).max(INITIAL_RECORDS).min(limit);
            let mut values: Vec<_> = self.iter().copied().collect();
            values.resize(capacity, T::zeroed());
            self.blocks = values
                .chunks(BLOCK_RECORDS)
                .map(|block| Arc::new(block.to_vec()))
                .collect();
            self.cursor = RingCursor {
                generation: self.cursor.generation.wrapping_add(1),
                writes: self.len as u64,
                capacity,
            };
        }
        let index = (self.cursor.writes % self.capacity() as u64) as usize;
        Arc::make_mut(&mut self.blocks[index / BLOCK_RECORDS])[index % BLOCK_RECORDS] = record;
        self.cursor.writes = self.cursor.writes.wrapping_add(1);
        if self.cursor.writes == 0 {
            self.cursor.generation = self.cursor.generation.wrapping_add(1);
        }
        self.len += 1;
    }

    pub(crate) fn copy_range(&self, range: Range<usize>, output: &mut Vec<T>) {
        output.clear();
        let mut start = range.start;
        while start < range.end {
            let block = &self.blocks[start / BLOCK_RECORDS];
            let offset = start % BLOCK_RECORDS;
            let count = (range.end - start).min(block.len() - offset);
            output.extend_from_slice(&block[offset..offset + count]);
            start += count;
        }
    }
}

impl<T> Index<usize> for RecordRing<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.blocks[index / BLOCK_RECORDS][index % BLOCK_RECORDS]
    }
}

#[cfg(test)]
impl<T: Pod> From<Vec<T>> for RecordRing<T> {
    fn from(values: Vec<T>) -> Self {
        let mut records = Self::default();
        let limit = values.len().max(1).next_power_of_two();
        for value in values {
            records.push(value, limit);
        }
        records
    }
}

/// Reuse staging memory and issue at most two contiguous buffer writes. Only
/// records born since the GPU's last cursor are copied, except when growing.
pub(crate) fn upload_records<T: Pod>(
    records: &RecordRing<T>,
    previous: Option<RingCursor>,
    buffer: &Buffer,
    queue: &RenderQueue,
    scratch: &mut Vec<T>,
) {
    for range in records.upload_ranges(previous) {
        if range.is_empty() {
            continue;
        }
        let offset = (range.start * size_of::<T>()) as u64;
        records.copy_range(range, scratch);
        queue.write_buffer(buffer, offset, cast_slice(scratch));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_preserves_live_order_and_uploads_only_birth_ranges() {
        let mut ring = RecordRing::default();
        for i in 0..4u32 {
            ring.push(i, 4);
        }
        ring.pop_front();
        ring.pop_front();
        let previous = ring.cursor();
        ring.push(4, 4);
        assert_eq!(ring.iter().copied().collect::<Vec<_>>(), vec![2, 3, 4]);
        assert_eq!(ring.index(0), 2);
        assert_eq!(ring.upload_ranges(Some(previous)), [0..1, 0..0]);
        ring.pop_front();
        ring.pop_front();
        let previous = ring.cursor();
        for i in 5..8 {
            ring.push(i, 4);
        }
        assert_eq!(ring.upload_ranges(Some(previous)), [1..4, 0..0]);
        ring.pop_front();
        let previous = ring.cursor();
        ring.push(8, 4);
        assert_eq!(ring.upload_ranges(Some(previous)), [0..1, 0..0]);
    }

    #[test]
    fn growth_rebases_wrapped_records_and_invalidates_the_gpu_cursor() {
        let mut ring = RecordRing::default();
        for i in 0..BLOCK_RECORDS as u32 {
            ring.push(i, 128);
        }
        for _ in 0..60 {
            ring.pop_front();
        }
        for i in 64..124 {
            ring.push(i, 128);
        }
        let previous = ring.cursor();
        ring.push(124, 128);
        assert_eq!(ring.capacity(), 128);
        assert_eq!(ring.index(0), 0);
        assert_eq!(
            ring.iter().copied().collect::<Vec<_>>(),
            (60..125).collect::<Vec<_>>()
        );
        assert_eq!(ring.upload_ranges(Some(previous)), [0..128, 0..0]);
    }

    #[test]
    fn extraction_snapshots_copy_only_written_blocks_and_keep_old_values() {
        let mut ring = RecordRing::default();
        for i in 0..128u32 {
            ring.push(i, 128);
        }
        for _ in 0..65 {
            ring.pop_front();
        }
        let previous = ring.clone();
        ring.push(128, 128);
        assert!(!Arc::ptr_eq(&ring.blocks[0], &previous.blocks[0]));
        assert!(Arc::ptr_eq(&ring.blocks[1], &previous.blocks[1]));
        assert_eq!(previous[0], 0);
        assert_eq!(ring[0], 128);
        assert_eq!(ring.upload_ranges(Some(previous.cursor())), [0..1, 0..0]);
    }

    #[test]
    fn upload_ranges_reconstruct_live_records_after_skipped_frames_and_growth() {
        use std::collections::VecDeque;
        let mut ring = RecordRing::default();
        let mut expected = VecDeque::new();
        let mut gpu = Vec::new();
        let mut cursor = None;
        let mut scratch = Vec::new();
        let mut random = 1u32;
        let mut next = 1u32;
        for frame in 0..1000 {
            random = random.wrapping_mul(1664525).wrapping_add(1013904223);
            let retire = (random as usize % 9).min(expected.len());
            for _ in 0..retire {
                ring.pop_front();
                expected.pop_front();
            }
            let births = ((random >> 8) as usize % 13).min(128 - expected.len());
            for _ in 0..births {
                ring.push(next, 128);
                expected.push_back(next);
                next += 1;
            }
            assert_eq!(
                ring.iter().copied().collect::<Vec<_>>(),
                expected.iter().copied().collect::<Vec<_>>()
            );
            if frame % 7 != 0 {
                continue;
            }
            gpu.resize(ring.capacity(), 0);
            for range in ring.upload_ranges(cursor) {
                ring.copy_range(range.clone(), &mut scratch);
                gpu[range].copy_from_slice(&scratch);
            }
            cursor = Some(ring.cursor());
            for (i, &record) in expected.iter().enumerate() {
                assert_eq!(gpu[ring.index(i) as usize], record);
            }
        }
    }

    #[test]
    fn skipped_frames_and_full_laps_upload_current_contents() {
        let mut ring = RecordRing::default();
        for i in 0..4u32 {
            ring.push(i, 4);
        }
        let previous = ring.cursor();
        for i in 4..14 {
            ring.pop_front();
            ring.push(i, 4);
        }
        assert_eq!(ring.upload_ranges(Some(previous)), [0..4, 0..0]);
        let previous = ring.cursor();
        ring.pop_front();
        ring.push(14, 4);
        ring.pop_front();
        ring.push(15, 4);
        ring.pop_front();
        ring.push(16, 4);
        assert_eq!(ring.upload_ranges(Some(previous)), [2..4, 0..1]);
        let mut scratch = Vec::new();
        ring.copy_range(0..4, &mut scratch);
        assert_eq!(scratch, vec![16, 13, 14, 15]);
        let previous = ring.cursor();
        ring.pop_front();
        assert_eq!(ring.upload_ranges(Some(previous)), [0..0, 0..0]);
    }
}
