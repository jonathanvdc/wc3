//! Stable pooled birth slots, independent of FIFO retirement and extraction timing.
use bevy::prelude::*;
use std::sync::{Arc, OnceLock};

const SLOTS_PER_BLOCK: usize = 64;

pub(crate) struct ImmutableAppearance {
    pub source: String,
    pub words_per_slot: usize,
    /// Any explicit Time node, including otherwise unused guarded expressions.
    pub requires_time_horizon: bool,
    pub shader: OnceLock<Handle<Shader>>,
}
#[derive(Clone)]
pub(crate) struct BirthSnapshot {
    pub lineage: Arc<()>,
    pub words_per_slot: usize,
    pub capacity: usize,
    pub blocks: Vec<Arc<Vec<u32>>>,
}
impl BirthSnapshot {
    pub fn position(&self, slot: u32) -> Vec3 {
        let slot = slot as usize;
        let block = &self.blocks[slot / SLOTS_PER_BLOCK];
        let start = (slot % SLOTS_PER_BLOCK) * self.words_per_slot;
        Vec3::new(
            f32::from_bits(block[start]),
            f32::from_bits(block[start + 1]),
            f32::from_bits(block[start + 2]),
        )
    }
    pub fn word_count(&self) -> usize {
        self.capacity * self.words_per_slot
    }
    pub fn block_word_offset(&self, index: usize) -> usize {
        index * SLOTS_PER_BLOCK * self.words_per_slot
    }
}
#[derive(Clone)]
pub(crate) struct ImmutableSpriteData {
    pub appearance: Arc<ImmutableAppearance>,
    pub births: Arc<BirthSnapshot>,
}
/// IDs are renderer cache metadata; live membership still belongs to the core instance.
pub(crate) struct BirthSlots {
    pub snapshot: Arc<BirthSnapshot>,
    identities: Vec<Option<(u32, u64)>>,
    pub live: Arc<Vec<u32>>,
}
impl BirthSlots {
    pub fn new(capacity: usize, words_per_slot: usize) -> Option<Self> {
        capacity
            .checked_mul(words_per_slot)?
            .checked_mul(size_of::<u32>())
            .filter(|bytes| *bytes <= isize::MAX as usize)?;
        let blocks = (0..capacity.div_ceil(SLOTS_PER_BLOCK))
            .map(|index| {
                Arc::new(vec![
                    0;
                    (capacity - index * SLOTS_PER_BLOCK)
                        .min(SLOTS_PER_BLOCK)
                        * words_per_slot
                ])
            })
            .collect();
        Some(Self {
            snapshot: Arc::new(BirthSnapshot {
                lineage: Arc::new(()),
                words_per_slot,
                capacity,
                blocks,
            }),
            identities: vec![None; capacity],
            live: Arc::new(Vec::with_capacity(capacity)),
        })
    }
    pub fn matches(&self, slot: u32, generation: u32, serial: u64) -> bool {
        self.identities.get(slot as usize) == Some(&Some((generation, serial)))
    }
    pub fn set(&mut self, slot: u32, generation: u32, serial: u64, words: &[u32]) -> Option<()> {
        let slot = slot as usize;
        if slot >= self.snapshot.capacity || words.len() != self.snapshot.words_per_slot {
            return None;
        }
        let snapshot = Arc::make_mut(&mut self.snapshot);
        let start = (slot % SLOTS_PER_BLOCK) * snapshot.words_per_slot;
        let block = Arc::make_mut(&mut snapshot.blocks[slot / SLOTS_PER_BLOCK]);
        block[start..start + words.len()].copy_from_slice(words);
        self.identities[slot] = Some((generation, serial));
        Some(())
    }
    pub fn set_live(&mut self, order: &[u32]) {
        if self.live.as_slice() != order {
            let live = Arc::make_mut(&mut self.live);
            live.clear();
            live.extend_from_slice(order);
        }
    }
    pub fn reset(&mut self) {
        // A new lineage invalidates any resident cursor even when IDs repeat.
        Arc::make_mut(&mut self.snapshot).lineage = Arc::new(());
        self.identities.fill(None);
        Arc::make_mut(&mut self.live).clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arbitrary_reuse_preserves_old_extract_and_neighbor_slots() {
        let mut slots = BirthSlots::new(130, 4).unwrap();
        slots.set(65, 1, 7, &[1, 2, 3, 4]).unwrap();
        slots.set(2, 1, 8, &[5, 6, 7, 8]).unwrap();
        let before = slots.snapshot.clone();
        slots.set(65, 2, 9, &[9, 10, 11, 12]).unwrap();
        assert!(slots.matches(65, 2, 9));
        assert!(!slots.matches(65, 1, 7));
        assert_eq!(&before.blocks[1][4..8], &[1, 2, 3, 4]);
        assert_eq!(&slots.snapshot.blocks[1][4..8], &[9, 10, 11, 12]);
        assert!(Arc::ptr_eq(&before.blocks[0], &slots.snapshot.blocks[0]));
        assert!(Arc::ptr_eq(&before.blocks[2], &slots.snapshot.blocks[2]));
        slots.set_live(&[65, 2]);
        let live = slots.live.clone();
        slots.set_live(&[65, 2]);
        assert!(Arc::ptr_eq(&live, &slots.live));
        slots.set_live(&[2]);
        assert_eq!(&*live, &[65, 2]);
        let lineage = slots.snapshot.lineage.clone();
        slots.reset();
        assert!(!Arc::ptr_eq(&lineage, &slots.snapshot.lineage));
        assert!(!slots.matches(65, 2, 9));
        assert!(slots.live.is_empty());
        assert!(slots.set(130, 1, 1, &[0; 4]).is_none());
        assert!(slots.set(1, 1, 1, &[0; 3]).is_none());
        assert!(BirthSlots::new(isize::MAX as usize, 4).is_none());
        assert!(BirthSlots::new(2, usize::MAX).is_none());
    }
}
