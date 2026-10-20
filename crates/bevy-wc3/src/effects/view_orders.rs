//! Per-view order residency without retaining duplicate CPU particle arrays.
use bevy::math::Affine3A;
use std::collections::HashMap;
use std::hash::Hash;

struct CachedView<B> {
    generation: u64,
    affine: Affine3A,
    buffer: B,
}

pub(crate) struct ViewOrders<V, B> {
    generation: u64,
    policy: SortPolicy,
    views: HashMap<V, CachedView<B>>,
}
impl<V, B> Default for ViewOrders<V, B> {
    fn default() -> Self {
        Self {
            generation: 0,
            policy: SortPolicy::None,
            views: HashMap::new(),
        }
    }
}
impl<V: Copy + Eq + Hash, B> ViewOrders<V, B> {
    pub(crate) fn set_policy(&mut self, policy: SortPolicy) {
        if policy != self.policy {
            self.invalidate();
            self.policy = policy;
        }
    }
    pub(crate) fn invalidate(&mut self) {
        if let Some(next) = self.generation.checked_add(1) {
            self.generation = next;
        } else {
            self.views.clear();
            self.generation = 0;
        }
    }
    pub(crate) fn retain(&mut self, mut live: impl FnMut(V) -> bool) {
        self.views.retain(|view, _| live(*view));
    }
    pub(crate) fn clear(&mut self) {
        self.views.clear();
    }
    pub(crate) fn get(&self, view: &V) -> Option<&B> {
        self.views.get(view).map(|entry| &entry.buffer)
    }
    /// The callback owns sorting and upload; a cache hit does not call it.
    pub(crate) fn prepare(
        &mut self,
        view: V,
        affine: Affine3A,
        prepare: impl FnOnce(Option<&B>) -> B,
    ) -> bool {
        let previous = self.views.get(&view);
        if previous
            .is_some_and(|entry| entry.generation == self.generation && entry.affine == affine)
        {
            return false;
        }
        let buffer = prepare(previous.map(|entry| &entry.buffer));
        self.views.insert(
            view,
            CachedView {
                generation: self.generation,
                affine,
                buffer,
            },
        );
        true
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SortPolicy {
    #[default]
    None,
    Far,
    Near,
}

struct Depth<I> {
    index: I,
    distance: f32,
    ordinal: usize,
}
pub(crate) struct SortScratch<I> {
    depths: Vec<Depth<I>>,
    pub(crate) order: Vec<I>,
}
impl<I> Default for SortScratch<I> {
    fn default() -> Self {
        Self {
            depths: Vec::new(),
            order: Vec::new(),
        }
    }
}
impl<I: Copy> SortScratch<I> {
    pub(crate) fn clear(&mut self) {
        self.depths.clear();
        self.order.clear();
    }
    pub(crate) fn push(&mut self, index: I, distance: f32) {
        self.depths.push(Depth {
            index,
            distance,
            ordinal: self.depths.len(),
        });
    }
    /// Explicit input ordinals retain stable ties without stable-sort allocation.
    pub(crate) fn sort(&mut self, near: bool, mut emit: impl FnMut(I, &mut Vec<I>)) {
        self.depths.sort_unstable_by(|a, b| {
            let depth = if near {
                a.distance.total_cmp(&b.distance)
            } else {
                b.distance.total_cmp(&a.distance)
            };
            depth.then(a.ordinal.cmp(&b.ordinal))
        });
        for entry in &self.depths {
            emit(entry.index, &mut self.order);
        }
    }
}

#[cfg(test)]
#[path = "view_orders_tests.rs"]
mod tests;
