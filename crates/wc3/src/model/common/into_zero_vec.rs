//! Conversion of native values and encoded arrays into model storage.
use zerovec::{ule::AsULE, ZeroVec};

/// Supplies an encoded array to a model constructor.
///
/// An existing [`ZeroVec`] is moved unchanged, retaining its buffer and borrowing.
/// Native arrays, vectors, and slices are encoded into owned storage. A native
/// slice's lifetime does not constrain the resulting array's lifetime.
///
/// Collect generated values into `ZeroVec` before passing them to a constructor.
/// To retain an existing array, explicitly choose `clone()` for a storage clone
/// or [`Borrowed::as_borrowed`](super::Borrowed::as_borrowed) for a view of its buffer.
///
/// ```
/// use wc3::model::common::{IntoZeroVec, ZeroVec};
///
/// let values: ZeroVec<'static, u32> = [1, 2, 3].into_zero_vec();
/// let pointer = values.as_bytes().as_ptr();
/// let moved = values.into_zero_vec();
/// assert_eq!(moved.as_bytes().as_ptr(), pointer);
/// ```
pub trait IntoZeroVec<'data, T: AsULE> {
    /// Moves an encoded array or encodes native values into owned storage.
    fn into_zero_vec(self) -> ZeroVec<'data, T>;
}

impl<'data, T: AsULE> IntoZeroVec<'data, T> for ZeroVec<'data, T> {
    fn into_zero_vec(self) -> ZeroVec<'data, T> {
        self
    }
}

impl<'data, T: AsULE, const N: usize> IntoZeroVec<'data, T> for [T; N] {
    fn into_zero_vec(self) -> ZeroVec<'data, T> {
        self.into_iter().collect()
    }
}

impl<'data, T: AsULE> IntoZeroVec<'data, T> for Vec<T> {
    fn into_zero_vec(self) -> ZeroVec<'data, T> {
        self.into_iter().collect()
    }
}

impl<'data, T: AsULE> IntoZeroVec<'data, T> for &[T] {
    fn into_zero_vec(self) -> ZeroVec<'data, T> {
        ZeroVec::alloc_from_slice(self)
    }
}

impl<'data, T: AsULE, const N: usize> IntoZeroVec<'data, T> for &[T; N] {
    fn into_zero_vec(self) -> ZeroVec<'data, T> {
        ZeroVec::alloc_from_slice(self)
    }
}
