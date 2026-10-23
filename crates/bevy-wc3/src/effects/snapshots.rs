//! Reusable immutable extraction snapshots shared by sampled effect transports.
use std::sync::Arc;

pub(crate) struct SnapshotPool<T> {
    snapshots: Vec<Arc<Vec<T>>>,
}

impl<T> Default for SnapshotPool<T> {
    fn default() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }
}

impl<T> SnapshotPool<T> {
    /// Reuse a released snapshot without copying data still owned by extraction.
    pub fn begin(&mut self) -> &mut Arc<Vec<T>> {
        let index = self
            .snapshots
            .iter()
            .position(|snapshot| Arc::strong_count(snapshot) == 1)
            .unwrap_or_else(|| {
                let capacity = self
                    .snapshots
                    .iter()
                    .map(|snapshot| snapshot.capacity())
                    .max()
                    .unwrap_or(0);
                self.snapshots.push(Arc::new(Vec::with_capacity(capacity)));
                self.snapshots.len() - 1
            });
        Arc::get_mut(&mut self.snapshots[index])
            .expect("released effect snapshot")
            .clear();
        &mut self.snapshots[index]
    }
}
