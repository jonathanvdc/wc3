//! Immutable arrays borrow native values or decode individual packed elements on access.
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;
use zerovec::{ule::AsULE, ZeroVec};

trait Source<T>: Send + Sync {
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<T>;
    fn slice(&self) -> Option<&[T]> {
        None
    }
    fn owned(&self) -> Option<Arc<[T]>> {
        None
    }
    fn packed_bytes(&self) -> Option<&[u8]> {
        None
    }
}
struct Borrowed<'a, T>(&'a [T]);
impl<T: Clone + Send + Sync> Source<T> for Borrowed<'_, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn get(&self, i: usize) -> Option<T> {
        self.0.get(i).cloned()
    }
    fn slice(&self) -> Option<&[T]> {
        Some(self.0)
    }
}
struct Owned<T>(Arc<[T]>);
impl<T: Clone + Send + Sync> Source<T> for Owned<T> {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn get(&self, i: usize) -> Option<T> {
        self.0.get(i).cloned()
    }
    fn slice(&self) -> Option<&[T]> {
        Some(&self.0)
    }
    fn owned(&self) -> Option<Arc<[T]>> {
        Some(self.0.clone())
    }
}
struct Packed<'a, T: AsULE>(ZeroVec<'a, T>);
impl<T: AsULE + Send + Sync> Source<T> for Packed<'_, T>
where
    T::ULE: Send + Sync,
{
    fn len(&self) -> usize {
        self.0.len()
    }
    fn get(&self, index: usize) -> Option<T> {
        self.0.get(index)
    }
    fn packed_bytes(&self) -> Option<&[u8]> {
        Some(self.0.as_bytes())
    }
}
struct View<F> {
    len: usize,
    get: F,
}
impl<T, F: Fn(usize) -> T + Send + Sync> Source<T> for View<F> {
    fn len(&self) -> usize {
        self.len
    }
    fn get(&self, i: usize) -> Option<T> {
        (i < self.len).then(|| (self.get)(i))
    }
}

/// A logical value array independent of native packing, alignment, and byte order.
/// Cloning shares storage. A view can borrow an encoded buffer without materializing its elements.
pub struct Values<'a, T> {
    source: Arc<dyn Source<T> + 'a>,
    borrowed: bool,
    borrowed_packed: Option<&'a [u8]>,
}
impl<T> Clone for Values<'_, T> {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            borrowed: self.borrowed,
            borrowed_packed: self.borrowed_packed,
        }
    }
}
impl<T: Debug> Debug for Values<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Values")
            .field("len", &self.len())
            .field("borrowed", &self.borrowed)
            .finish()
    }
}
impl<'a, T: Clone + Send + Sync + 'a> Default for Values<'a, T> {
    fn default() -> Self {
        Self::owned(Vec::new())
    }
}
impl<'a, T: Clone + Send + Sync + 'a> Values<'a, T> {
    /// Borrows an aligned slice directly.
    pub fn from_slice(values: &'a [T]) -> Self {
        Self {
            source: Arc::new(Borrowed(values)),
            borrowed: true,
            borrowed_packed: None,
        }
    }
    /// Moves values into shared owned storage.
    pub fn owned(values: impl Into<Arc<[T]>>) -> Self {
        Self {
            source: Arc::new(Owned(values.into())),
            borrowed: false,
            borrowed_packed: None,
        }
    }
    /// Creates a checked-length view, decoding an element only when it is requested.
    pub(crate) fn view(len: usize, get: impl Fn(usize) -> T + Send + Sync + 'a) -> Self {
        Self {
            source: Arc::new(View { len, get }),
            borrowed: true,
            borrowed_packed: None,
        }
    }
    /// Maps values lazily without materializing a second array.
    pub fn map<U: Clone + Send + Sync + 'a>(
        self,
        map: impl Fn(T) -> U + Send + Sync + 'a,
    ) -> Values<'a, U> {
        let len = self.len();
        Values::view(len, move |i| map(self.get(i).expect("checked view index")))
    }
    /// Detaches borrowed data and retains already owned storage without copying values.
    pub fn into_owned(self) -> Values<'static, T>
    where
        T: 'static,
    {
        Values::owned(
            self.source
                .owned()
                .unwrap_or_else(|| self.iter().collect::<Vec<_>>().into()),
        )
    }
    /// Creates a view borrowing this array's storage.
    pub fn as_borrowed(&self) -> Values<'_, T> {
        Values {
            source: self.source.clone(),
            borrowed: self.borrowed,
            borrowed_packed: self.borrowed_packed,
        }
    }
    /// Materializes values only when a consumer explicitly needs a native vector.
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().collect()
    }
}
impl<'a, T: AsULE + Send + Sync + 'a> Values<'a, T>
where
    T::ULE: Send + Sync,
{
    /// Stores a canonical packed array without materializing its values.
    pub(crate) fn from_zero_vec(values: ZeroVec<'a, T>) -> Self {
        let borrowed_packed = values.as_maybe_borrowed().map(|value| value.as_bytes());
        let borrowed = !values.is_owned();
        Self {
            source: Arc::new(Packed(values)),
            borrowed,
            borrowed_packed,
        }
    }
    /// Returns only packed storage whose borrow comes from the original input lifetime.
    pub(crate) fn borrowed_canonical_bytes(&self) -> Option<&'a [u8]> {
        self.borrowed_packed
    }
    /// Borrows canonical packed storage when this logical array already has that representation.
    pub(crate) fn as_zero_vec(&self) -> Option<ZeroVec<'_, T>> {
        ZeroVec::parse_bytes(self.source.packed_bytes()?).ok()
    }
}
impl<T> Values<'_, T> {
    /// Number of logical elements.
    pub fn len(&self) -> usize {
        self.source.len()
    }
    /// Whether the array has no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Decodes one logical element, returning none outside the array.
    pub fn get(&self, index: usize) -> Option<T> {
        self.source.get(index)
    }
    /// Returns directly available native storage without allocating.
    pub fn as_slice(&self) -> Option<&[T]> {
        self.source.slice()
    }
    /// Whether storage may borrow its source.
    pub fn is_borrowed(&self) -> bool {
        self.borrowed
    }
    /// Iterates logical values, including values in packed source views.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = T> + '_ {
        (0..self.len()).map(|i| self.get(i).expect("checked view index"))
    }
}
impl<T: PartialEq> PartialEq for Values<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}
impl<T: Eq> Eq for Values<'_, T> {}
impl<'a, T: Clone + Send + Sync + 'a> From<Vec<T>> for Values<'a, T> {
    fn from(v: Vec<T>) -> Self {
        Self::owned(v)
    }
}
impl<'a, T: Clone + Send + Sync + 'a> From<&'a [T]> for Values<'a, T> {
    fn from(v: &'a [T]) -> Self {
        Self::from_slice(v)
    }
}
impl<'a, T: Clone + Send + Sync + 'a, const N: usize> From<[T; N]> for Values<'a, T> {
    fn from(v: [T; N]) -> Self {
        Self::owned(Vec::from(v))
    }
}

#[cfg(test)]
mod tests {
    use super::Values;
    use zerovec::ZeroVec;

    fn original_bytes(bytes: &[u8]) -> &[u8] {
        let values = Values::<u32>::from_zero_vec(ZeroVec::parse_bytes(bytes).unwrap());
        values.borrowed_canonical_bytes().unwrap()
    }
    #[test]
    fn packed_original_borrows_survive_the_array_wrapper_without_exposing_owned_storage() {
        let bytes = 42u32.to_le_bytes();
        assert_eq!(original_bytes(&bytes).as_ptr(), bytes.as_ptr());
        let owned = Values::from_zero_vec(ZeroVec::alloc_from_slice(&[42u32]));
        assert!(owned.borrowed_canonical_bytes().is_none());
        assert_eq!(owned.as_zero_vec().unwrap().as_bytes(), bytes.as_slice());
        let mapped = Values::from_zero_vec(ZeroVec::<u32>::parse_bytes(&bytes).unwrap())
            .map(|value| value + 1);
        assert!(mapped.borrowed_canonical_bytes().is_none());
        assert_eq!(mapped.get(0), Some(43));
    }
}
