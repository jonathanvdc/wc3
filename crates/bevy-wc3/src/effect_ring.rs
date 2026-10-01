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
#[path = "effect_ring_tests.rs"]
mod tests;
