//! Lazy inspection of a vertex's joint-slot or direct-node contributions.
use super::{Influence, NodeInfluence, PackedArray};
use std::iter::FusedIterator;
use std::ops::Range;

/// One vertex's influences preserve whether they name binding slots or hierarchy nodes.
#[derive(Clone, Debug)]
pub enum VertexInfluences<'view> {
    /// Ordered slots interpreted through a skin's joint binding.
    Joints(JointWeights<'view>),
    /// Explicit node identities interpreted through the hierarchy's rig convention.
    Nodes(PackedArray<'view, NodeInfluence>),
}
impl<'view> VertexInfluences<'view> {
    /// Inspects contributions without discarding their reference domain.
    #[inline]
    pub fn weights(
        &self,
    ) -> impl DoubleEndedIterator<Item = f32> + ExactSizeIterator + use<'_, 'view> {
        let len = match self {
            Self::Joints(values) => values.len(),
            Self::Nodes(values) => values.len(),
        };
        (0..len).map(|index| match self {
            Self::Joints(values) => values.get(index).expect("checked influence index").weight,
            Self::Nodes(values) => values.get(index).expect("checked influence index").weight,
        })
    }
    /// Consumes a joint-slot view; direct-node distributions remain distinct.
    #[inline]
    pub fn into_joint_weights(self) -> Option<JointWeights<'view>> {
        match self {
            Self::Joints(values) => Some(values),
            Self::Nodes(_) => None,
        }
    }
}

/// Allocation-free access to one vertex's ordered joint contributions.
///
/// Packed rows borrow their encoded storage; fixed-slot rows copy their small
/// values inline. Matrix-group members receive equal weights on demand. Reads
/// preserve authored order, repeated joints, and zero-weight slots.
#[derive(Clone, Debug)]
pub struct JointWeights<'view> {
    storage: JointWeightStorage<'view>,
}
#[derive(Clone, Debug)]
enum JointWeightStorage<'view> {
    Packed(PackedArray<'view, Influence>),
    Three([Influence; 3]),
    Four([Influence; 4]),
    MatrixGroups {
        joints: PackedArray<'view, u32>,
        weight: f32,
    },
}
impl Default for JointWeights<'_> {
    fn default() -> Self {
        Self::packed(PackedArray::new())
    }
}
impl PartialEq for JointWeights<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}
impl<'view> JointWeights<'view> {
    pub(crate) fn packed(values: PackedArray<'view, Influence>) -> Self {
        Self {
            storage: JointWeightStorage::Packed(values),
        }
    }
    pub(crate) fn three(values: [Influence; 3]) -> Self {
        Self {
            storage: JointWeightStorage::Three(values),
        }
    }
    pub(crate) fn four(values: [Influence; 4]) -> Self {
        Self {
            storage: JointWeightStorage::Four(values),
        }
    }
    pub(crate) fn matrix_groups(values: PackedArray<'view, u32>) -> Self {
        let weight = if values.is_empty() {
            0.
        } else {
            1. / values.len() as f32
        };
        Self {
            storage: JointWeightStorage::MatrixGroups {
                joints: values,
                weight,
            },
        }
    }
    /// Number of contributions, including zero-weight slots.
    #[inline]
    pub fn len(&self) -> usize {
        match &self.storage {
            JointWeightStorage::Packed(values) => values.len(),
            JointWeightStorage::Three(_) => 3,
            JointWeightStorage::Four(_) => 4,
            JointWeightStorage::MatrixGroups { joints, .. } => joints.len(),
        }
    }
    /// Whether the authored row is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Decodes a contribution in authored order.
    #[inline]
    pub fn get(&self, index: usize) -> Option<Influence> {
        match &self.storage {
            JointWeightStorage::Packed(values) => values.get(index),
            JointWeightStorage::Three(values) => values.get(index).copied(),
            JointWeightStorage::Four(values) => values.get(index).copied(),
            JointWeightStorage::MatrixGroups { joints, weight } => {
                joints.get(index).map(|joint| Influence {
                    joint,
                    weight: *weight,
                })
            }
        }
    }
    /// Decodes contributions lazily without allocating.
    #[inline]
    pub fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = Influence> + ExactSizeIterator + FusedIterator + '_ {
        (0..self.len()).map(|index| self.get(index).expect("checked influence index"))
    }
    /// Materializes the contributions as an independently owned vector.
    pub fn to_vec(&self) -> Vec<Influence> {
        self.iter().collect()
    }
}

/// Consuming traversal of a joint contribution view without materializing its values.
#[derive(Clone, Debug)]
pub struct JointWeightIter<'view> {
    values: JointWeights<'view>,
    remaining: Range<usize>,
}
impl Iterator for JointWeightIter<'_> {
    type Item = Influence;
    fn next(&mut self) -> Option<Self::Item> {
        self.remaining
            .next()
            .and_then(|index| self.values.get(index))
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.remaining
            .nth(n)
            .and_then(|index| self.values.get(index))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.remaining.size_hint()
    }
}
impl DoubleEndedIterator for JointWeightIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.remaining
            .next_back()
            .and_then(|index| self.values.get(index))
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.remaining
            .nth_back(n)
            .and_then(|index| self.values.get(index))
    }
}
impl ExactSizeIterator for JointWeightIter<'_> {}
impl FusedIterator for JointWeightIter<'_> {}
impl<'view> IntoIterator for JointWeights<'view> {
    type Item = Influence;
    type IntoIter = JointWeightIter<'view>;
    fn into_iter(self) -> Self::IntoIter {
        JointWeightIter {
            remaining: 0..self.len(),
            values: self,
        }
    }
}

#[cfg(test)]
#[path = "influence_rows_tests.rs"]
mod tests;
