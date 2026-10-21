//! Bounded previous-step spatial snapshots with independent keys and payloads.
use glam::Vec3;
use std::mem::{size_of, swap};
type Result<T> = std::result::Result<T, &'static str>;
#[derive(Clone, Copy)]
struct Row {
    cell: [i64; 3],
    coordinate: Vec3,
    payload: Vec3,
    ordinal: u64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Search {
    Found(Vec3),
    Missing,
    Omitted,
}
/// One already-validated shared declaration's private per-instance storage.
pub(crate) struct Store {
    read: Vec<Row>,
    write: Vec<Row>,
    capacity: usize,
    cell_size: f64,
    next: u64,
}
impl Store {
    pub fn new(capacity: usize, cell_size: f32) -> Result<Self> {
        if !cell_size.is_finite() || cell_size <= 0. {
            return Err("spatial cell size must be finite and positive");
        }
        Ok(Self {
            read: Vec::with_capacity(capacity),
            write: Vec::with_capacity(capacity),
            capacity,
            cell_size: f64::from(cell_size),
            next: 0,
        })
    }
    pub fn storage_bytes(capacity: usize) -> Option<usize> {
        capacity
            .checked_mul(2)?
            .checked_mul(size_of::<Row>())?
            .checked_add(size_of::<Self>())
    }
    /// Swap exactly once before executing any layer of an actual simulation step.
    pub fn begin_tick(&mut self) {
        swap(&mut self.read, &mut self.write);
        self.write.clear();
        self.read
            .sort_unstable_by(|a, b| a.cell.cmp(&b.cell).then(a.ordinal.cmp(&b.ordinal)));
    }
    /// Restart, or discard an unexamined time tail without retaining stale rows.
    pub fn clear(&mut self) {
        self.read.clear();
        self.write.clear();
        self.next = 0;
    }
    /// Return false for an omitted write. The caller reports the attempted insert
    /// once; rejected rows never overwrite an existing snapshot or queue a retry.
    pub fn insert(&mut self, coordinate: Vec3, payload: Vec3) -> bool {
        if self.write.len() >= self.capacity || !payload.is_finite() {
            return false;
        }
        let Some(cell) = self.cell(coordinate) else {
            return false;
        };
        let Some(next) = self.next.checked_add(1) else {
            return false;
        };
        self.write.push(Row {
            cell,
            coordinate,
            payload,
            ordinal: self.next,
        });
        self.next = next;
        true
    }
    /// Read the previous snapshot, using an inclusive radius and original
    /// insertion order for exact-distance ties. No self-exclusion is applied.
    pub fn closest(&self, query: Vec3, radius: f32, checks: &mut usize) -> Result<Search> {
        if !radius.is_finite() || radius < 0. {
            return Err("spatial radius must be finite and nonnegative");
        }
        if !query.is_finite() {
            return Ok(Search::Missing);
        }
        let query = query.to_array().map(f64::from);
        let radius = f64::from(radius);
        let lower = query.map(|value| self.bound(value - radius));
        let upper = query.map(|value| self.bound(value + radius));
        let begin = self.read.partition_point(|row| row.cell[0] < lower[0]);
        let end = self.read.partition_point(|row| row.cell[0] <= upper[0]);
        let mut best: Option<(f64, u64, Vec3)> = None;
        for row in &self.read[begin..end] {
            let Some(remaining) = checks.checked_sub(1) else {
                return Ok(Search::Omitted);
            };
            *checks = remaining;
            if row.cell[1] < lower[1]
                || row.cell[1] > upper[1]
                || row.cell[2] < lower[2]
                || row.cell[2] > upper[2]
            {
                continue;
            }
            let coordinate = row.coordinate.to_array().map(f64::from);
            let distance: f64 = coordinate
                .into_iter()
                .zip(query)
                .map(|(value, query)| {
                    let delta = value - query;
                    delta * delta
                })
                .sum();
            if distance == 0. {
                // All exact-coordinate ties occupy one cell; the read snapshot
                // orders that cell by original insertion ordinal.
                return Ok(Search::Found(row.payload));
            }
            if distance > radius * radius {
                continue;
            }
            if best.is_none_or(|(previous, ordinal, _)| {
                distance < previous || (distance == previous && row.ordinal < ordinal)
            }) {
                best = Some((distance, row.ordinal, row.payload));
            }
        }
        Ok(best.map_or(Search::Missing, |(_, _, payload)| Search::Found(payload)))
    }
    fn cell(&self, coordinate: Vec3) -> Option<[i64; 3]> {
        if !coordinate.is_finite() {
            return None;
        }
        let cell = coordinate
            .to_array()
            .map(|value| (f64::from(value) / self.cell_size).floor());
        if cell
            .iter()
            .any(|value| *value < i64::MIN as f64 || *value >= -(i64::MIN as f64))
        {
            return None;
        }
        Some(cell.map(|value| value as i64))
    }
    fn bound(&self, value: f64) -> i64 {
        // Saturating conversion bounds search rather than iterating a potentially
        // enormous cubic cell range. Exact distances still use original f64 data.
        (value / self.cell_size).floor() as i64
    }
}
#[cfg(test)]
#[path = "spatial_tests.rs"]
mod tests;
