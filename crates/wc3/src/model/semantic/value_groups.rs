//! Variable-length rows use one packed element buffer and explicit row boundaries.
use crate::model::common::{AsULE, Borrowed, Detached, ZeroVec};

/// Packed variable-length rows. Boundaries include the final element count.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueGroups<'a, T: AsULE> {
    values: ZeroVec<'a, T>,
    offsets: ZeroVec<'a, u32>,
}
impl<T: AsULE> Default for ValueGroups<'_, T> {
    fn default() -> Self {
        Self {
            values: ZeroVec::new(),
            offsets: [0].into_iter().collect(),
        }
    }
}
impl<'a, T: AsULE> ValueGroups<'a, T> {
    /// Encodes rows into flat storage, preserving empty rows and authored order.
    pub fn from_rows(rows: impl IntoIterator<Item = impl IntoIterator<Item = T>>) -> Self {
        let mut values = Vec::new();
        let mut offsets = vec![0];
        for row in rows {
            values.extend(row);
            offsets.push(u32::try_from(values.len()).expect("group element count exceeds u32"));
        }
        Self {
            values: values.into_iter().collect(),
            offsets: offsets.into_iter().collect(),
        }
    }
    /// Checks canonical boundaries for an existing packed buffer.
    pub fn new(values: ZeroVec<'a, T>, offsets: ZeroVec<'a, u32>) -> Option<Self> {
        let valid = offsets.first() == Some(0)
            && offsets.last().map(|n| n as usize) == Some(values.len())
            && offsets
                .iter()
                .zip(offsets.iter().skip(1))
                .all(|(a, b)| a <= b);
        valid.then_some(Self { values, offsets })
    }
    /// Flat encoded elements in row order.
    pub fn values(&self) -> &ZeroVec<'a, T> {
        &self.values
    }
    /// Row boundaries, including the final element count.
    pub fn offsets(&self) -> &ZeroVec<'a, u32> {
        &self.offsets
    }
    /// Number of rows, including empty rows.
    pub fn len(&self) -> usize {
        self.offsets.len() - 1
    }
    /// Whether no rows exist.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Borrows one row's packed element buffer.
    pub fn get(&self, index: usize) -> Option<ZeroVec<'_, T>> {
        let start = self.offsets.get(index)? as usize;
        let end = self.offsets.get(index.checked_add(1)?)? as usize;
        Some(self.values.get_subslice(start..end)?.as_zerovec())
    }
    /// Iterates borrowed row buffers.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = ZeroVec<'_, T>> {
        (0..self.len()).map(|i| self.get(i).expect("validated row index"))
    }
    /// Materializes rows as native vectors.
    pub fn to_vec(&self) -> Vec<Vec<T>> {
        self.iter().map(|row| row.to_vec()).collect()
    }
    /// Encodes native rows.
    pub fn from_slice(rows: &[Vec<T>]) -> Self {
        Self::from_rows(rows.iter().map(|r| r.iter().copied()))
    }
    /// Encodes owned native rows.
    pub fn owned(rows: Vec<Vec<T>>) -> Self {
        Self::from_rows(rows)
    }
    /// Copies borrowed buffers into lifetime-independent storage.
    pub fn into_owned(self) -> ValueGroups<'static, T> {
        ValueGroups {
            values: self.values.into_owned(),
            offsets: self.offsets.into_owned(),
        }
    }
}
impl<'a, T: AsULE> From<Vec<Vec<T>>> for ValueGroups<'a, T> {
    fn from(rows: Vec<Vec<T>>) -> Self {
        Self::from_rows(rows)
    }
}

impl<'view, T: AsULE + 'view> Borrowed<'view> for ValueGroups<'_, T> {
    type View = ValueGroups<'view, T>;
    fn as_borrowed(&'view self) -> Self::View {
        ValueGroups {
            values: self.values.as_borrowed(),
            offsets: self.offsets.as_borrowed(),
        }
    }
}
impl<'owned, T: AsULE + 'owned> Detached<'owned> for ValueGroups<'_, T> {
    type Detached = ValueGroups<'owned, T>;
    fn into_detached(self) -> Self::Detached {
        ValueGroups {
            values: self.values.into_owned(),
            offsets: self.offsets.into_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ValueGroups;
    use crate::model::common::{Borrowed, IntoZeroVec};

    #[test]
    fn rows_preserve_empty_boundaries_and_borrow_flat_storage() {
        let rows = ValueGroups::from_rows([vec![3u32, 4], vec![], vec![5]]);
        assert_eq!(rows.offsets().to_vec(), [0, 2, 2, 3]);
        assert_eq!(rows.to_vec(), [vec![3, 4], vec![], vec![5]]);
        let view = rows.as_borrowed();
        assert_eq!(
            view.values().as_bytes().as_ptr(),
            rows.values().as_bytes().as_ptr()
        );
        let first = view.get(0).unwrap();
        assert_eq!(first.as_bytes().as_ptr(), rows.values().as_bytes().as_ptr());
        assert!(view.get(1).unwrap().is_empty());
        assert!(view.get(3).is_none());
        assert_eq!(view.into_owned().to_vec(), rows.to_vec());
    }
    #[test]
    fn boundaries_must_cover_flat_values_in_order() {
        for offsets in [vec![], vec![1, 2], vec![0, 3], vec![0, 2, 1, 2]] {
            assert!(ValueGroups::new([1u32, 2].into_zero_vec(), offsets.into_zero_vec()).is_none());
        }
        assert!(ValueGroups::<u32>::default().is_empty());
    }
}
