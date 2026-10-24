//! Packed storage independent of the mathematical value's memory layout.
use super::{AsULE, Borrowed, Detached, ZeroSlice, ZeroVec};
use glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};
use std::borrow::Cow;
use std::fmt::{self, Debug};
use std::ops::Range;
use std::sync::Arc;
use zerovec::ule::UleError;

/// A value with a lossless packed representation. Conversion preserves components;
/// validation and normalization belong to the value's semantic domain.
pub trait PackedValue: Copy {
    /// Aligned value understood by the underlying little-endian storage.
    type Storage: AsULE;
    /// Converts to storage without changing the value's meaning.
    fn pack(self) -> Self::Storage;
    /// Restores a value from storage.
    fn unpack(value: Self::Storage) -> Self;
}
macro_rules! identity {
    ($($ty:ty),* $(,)?) => {$ (
        impl $crate::model::common::PackedValue for $ty {
            type Storage = Self;
            #[inline]
            fn pack(self) -> Self { self }
            #[inline]
            fn unpack(value: Self) -> Self { value }
        }
    )*};
}
pub(crate) use identity as packed_identity;
identity!(bool, u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);
impl<T: PackedValue, const N: usize> PackedValue for [T; N]
where
    [T::Storage; N]: AsULE,
{
    type Storage = [T::Storage; N];
    #[inline]
    fn pack(self) -> Self::Storage {
        self.map(T::pack)
    }
    #[inline]
    fn unpack(value: Self::Storage) -> Self {
        value.map(T::unpack)
    }
}
macro_rules! math {
    ($($ty:ty => $storage:ty, $to:ident, $from:ident);* $(;)?) => {$ (
        impl PackedValue for $ty {
            type Storage = $storage;
            #[inline]
            fn pack(self) -> Self::Storage { self.$to() }
            #[inline]
            fn unpack(value: Self::Storage) -> Self { Self::$from(value) }
        }
    )*};
}
math!(Vec2 => [f32; 2], to_array, from_array;
    Vec3 => [f32; 3], to_array, from_array;
    Vec4 => [f32; 4], to_array, from_array;
    Quat => [f32; 4], to_array, from_array);
impl PackedValue for Mat3 {
    type Storage = [f32; 9];
    #[inline]
    fn pack(self) -> Self::Storage {
        self.to_cols_array()
    }
    #[inline]
    fn unpack(value: Self::Storage) -> Self {
        Self::from_cols_array(&value)
    }
}
impl PackedValue for Mat4 {
    type Storage = [f32; 16];
    #[inline]
    fn pack(self) -> Self::Storage {
        self.to_cols_array()
    }
    #[inline]
    fn unpack(value: Self::Storage) -> Self {
        Self::from_cols_array(&value)
    }
}

/// Borrowed or owned little-endian elements exposed as mathematical values.
/// Reads decode one element by value, without requiring aligned source bytes.
/// Clones share owned buffers; editing a shared or borrowed buffer first copies it.
#[derive(Clone)]
pub struct PackedArray<'a, T: PackedValue> {
    storage: Storage<'a, T::Storage>,
}

#[derive(Clone)]
enum Storage<'a, T: AsULE> {
    Borrowed(&'a [T::ULE]),
    Owned(Arc<Vec<T::ULE>>),
}
impl<T: PackedValue> Default for PackedArray<'_, T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: PackedValue + Debug> Debug for PackedArray<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
impl<T: PackedValue + PartialEq> PartialEq for PackedArray<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}
impl<'a, T: PackedValue> PackedArray<'a, T> {
    /// Empty borrowed storage.
    pub const fn new() -> Self {
        Self {
            storage: Storage::Borrowed(&[]),
        }
    }
    /// Adopts compatible packed storage without copying or decoding it.
    #[inline]
    pub fn from_storage(storage: ZeroVec<'a, T::Storage>) -> Self {
        if storage.is_empty() {
            Self::new()
        } else {
            match storage.into_cow() {
                Cow::Borrowed(values) => Self {
                    storage: Storage::Borrowed(values),
                },
                Cow::Owned(values) => Self {
                    storage: Storage::Owned(Arc::new(values)),
                },
            }
        }
    }
    /// Borrows the encoded elements at a native format boundary.
    #[inline]
    pub fn storage(&self) -> &ZeroSlice<T::Storage> {
        match &self.storage {
            Storage::Borrowed(storage) => ZeroSlice::from_ule_slice(storage),
            Storage::Owned(storage) => ZeroSlice::from_ule_slice(storage),
        }
    }
    /// Moves the encoded representation out at a native format boundary.
    /// A uniquely owned buffer moves unchanged; a shared buffer is copied.
    #[inline]
    pub fn into_storage(self) -> ZeroVec<'a, T::Storage> {
        match self.storage {
            Storage::Borrowed(storage) => ZeroVec::new_borrowed(storage),
            Storage::Owned(storage) => ZeroVec::new_owned(Arc::unwrap_or_clone(storage)),
        }
    }
    /// Borrows a validated little-endian byte buffer.
    pub fn parse_bytes(bytes: &'a [u8]) -> Result<Self, UleError> {
        ZeroVec::parse_bytes(bytes).map(Self::from_storage)
    }
    /// Encodes a slice into owned packed storage.
    pub fn alloc_from_slice(values: &[T]) -> Self {
        values.iter().copied().collect()
    }
    /// Number of elements.
    #[inline]
    pub fn len(&self) -> usize {
        self.storage().len()
    }
    /// Whether the encoded buffer is owned.
    pub fn is_owned(&self) -> bool {
        matches!(self.storage, Storage::Owned(_))
    }
    /// Whether no elements exist.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.storage().is_empty()
    }
    /// Decodes one element.
    #[inline]
    pub fn get(&self, index: usize) -> Option<T> {
        self.storage().get(index).map(T::unpack)
    }
    /// First element, when present.
    #[inline]
    pub fn first(&self) -> Option<T> {
        self.storage().first().map(T::unpack)
    }
    /// Last element, when present.
    #[inline]
    pub fn last(&self) -> Option<T> {
        self.storage().last().map(T::unpack)
    }
    /// Decodes elements lazily.
    #[inline]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = T> + ExactSizeIterator + '_ {
        self.storage().iter().map(T::unpack)
    }
    /// Materializes aligned values.
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().collect()
    }
    /// Encoded bytes, independent of the aligned value's memory layout.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.storage().as_bytes()
    }
    /// Borrows a range without decoding its elements.
    pub fn get_subslice(&self, range: Range<usize>) -> Option<PackedArray<'_, T>> {
        self.storage()
            .get_subslice(range)
            .map(|v| PackedArray::from_storage(v.as_zerovec()))
    }
    /// Replaces one element, first copying a borrowed or shared buffer.
    /// Returns the previous value, or `None` without allocating for an absent index.
    pub fn set(&mut self, index: usize, value: T) -> Option<T> {
        let previous = self.get(index)?;
        self.mutable_storage()[index] = value.pack().to_unaligned();
        Some(previous)
    }
    /// Edits values in place after making the packed buffer uniquely owned.
    pub fn for_each_mut(&mut self, mut edit: impl FnMut(&mut T)) {
        if self.is_empty() {
            return;
        }
        self.mutable_storage().iter_mut().for_each(|stored| {
            let mut value = T::unpack(T::Storage::from_unaligned(*stored));
            edit(&mut value);
            *stored = value.pack().to_unaligned();
        });
    }
    /// Copies borrowed storage and retains already owned buffers without copying.
    pub fn into_owned(self) -> PackedArray<'static, T> {
        match self.storage {
            Storage::Borrowed([]) => PackedArray::new(),
            Storage::Borrowed(storage) => PackedArray {
                storage: Storage::Owned(Arc::new(storage.to_vec())),
            },
            Storage::Owned(storage) => PackedArray {
                storage: Storage::Owned(storage),
            },
        }
    }

    fn mutable_storage(&mut self) -> &mut Vec<<T::Storage as AsULE>::ULE> {
        if let Storage::Borrowed(storage) = &self.storage {
            self.storage = Storage::Owned(Arc::new(storage.to_vec()));
        }
        match &mut self.storage {
            Storage::Owned(storage) => Arc::make_mut(storage),
            Storage::Borrowed(_) => unreachable!("borrowed storage was made owned"),
        }
    }
}
impl<'a, T: PackedValue> FromIterator<T> for PackedArray<'a, T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_storage(iter.into_iter().map(T::pack).collect())
    }
}
impl<'view, T: PackedValue + 'view> Borrowed<'view> for PackedArray<'_, T> {
    type View = PackedArray<'view, T>;
    fn as_borrowed(&'view self) -> Self::View {
        PackedArray {
            storage: Storage::Borrowed(self.storage().as_ule_slice()),
        }
    }
}
impl<'owned, T: PackedValue + 'owned> Detached<'owned> for PackedArray<'_, T> {
    type Detached = PackedArray<'owned, T>;
    fn into_detached(self) -> Self::Detached {
        self.into_owned()
    }
}
/// Supplies semantic values or existing packed storage to an array constructor.
pub trait IntoPackedArray<'a, T: PackedValue> {
    /// Moves existing storage or encodes supplied values.
    fn into_packed_array(self) -> PackedArray<'a, T>;
}
impl<'a, T: PackedValue> IntoPackedArray<'a, T> for PackedArray<'a, T> {
    fn into_packed_array(self) -> PackedArray<'a, T> {
        self
    }
}
impl<'a, T: PackedValue> IntoPackedArray<'a, T> for Vec<T> {
    fn into_packed_array(self) -> PackedArray<'a, T> {
        self.into_iter().collect()
    }
}
impl<'a, T: PackedValue, const N: usize> IntoPackedArray<'a, T> for [T; N] {
    fn into_packed_array(self) -> PackedArray<'a, T> {
        self.into_iter().collect()
    }
}
impl<'a, T: PackedValue> IntoPackedArray<'a, T> for &[T] {
    fn into_packed_array(self) -> PackedArray<'a, T> {
        PackedArray::alloc_from_slice(self)
    }
}
impl<'a, T: PackedValue, const N: usize> IntoPackedArray<'a, T> for &[T; N] {
    fn into_packed_array(self) -> PackedArray<'a, T> {
        PackedArray::alloc_from_slice(self)
    }
}

impl<'a, T: PackedValue> IntoPackedArray<'a, T> for ZeroVec<'a, T::Storage> {
    fn into_packed_array(self) -> PackedArray<'a, T> {
        PackedArray::from_storage(self)
    }
}

#[cfg(test)]
#[path = "packed_tests.rs"]
mod tests;
