//! Packed storage independent of the mathematical value's memory layout.
use super::{AsULE, Borrowed, Detached, ZeroVec};
use glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};
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
    Borrowed(ZeroVec<'a, T>),
    Owned(Arc<ZeroVec<'static, T>>),
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
            storage: Storage::Borrowed(ZeroVec::new()),
        }
    }
    /// Adopts compatible packed storage without copying or decoding it.
    #[inline]
    pub fn from_storage(storage: ZeroVec<'a, T::Storage>) -> Self {
        if storage.is_empty() {
            Self::new()
        } else if storage.is_owned() {
            Self {
                storage: Storage::Owned(Arc::new(storage.into_owned())),
            }
        } else {
            Self {
                storage: Storage::Borrowed(storage),
            }
        }
    }
    /// Exposes the encoded representation at a native format boundary.
    #[inline]
    pub fn storage(&self) -> &ZeroVec<'a, T::Storage> {
        match &self.storage {
            Storage::Borrowed(storage) => storage,
            Storage::Owned(storage) => storage,
        }
    }
    /// Moves the encoded representation out at a native format boundary.
    /// A uniquely owned buffer moves unchanged; a shared buffer is copied.
    #[inline]
    pub fn into_storage(self) -> ZeroVec<'a, T::Storage> {
        match self.storage {
            Storage::Borrowed(storage) => storage,
            Storage::Owned(storage) => Arc::unwrap_or_clone(storage),
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
        self.mutable_storage().to_mut_slice()[index] = value.pack().to_unaligned();
        Some(previous)
    }
    /// Edits values in place after making the packed buffer uniquely owned.
    pub fn for_each_mut(&mut self, mut edit: impl FnMut(&mut T)) {
        if self.is_empty() {
            return;
        }
        self.mutable_storage().for_each_mut(|stored| {
            let mut value = T::unpack(*stored);
            edit(&mut value);
            *stored = value.pack();
        });
    }
    /// Copies borrowed storage and retains already owned buffers without copying.
    pub fn into_owned(self) -> PackedArray<'static, T> {
        match self.storage {
            Storage::Borrowed(storage) => PackedArray::from_storage(storage.into_owned()),
            Storage::Owned(storage) => PackedArray {
                storage: Storage::Owned(storage),
            },
        }
    }

    fn mutable_storage(&mut self) -> &mut ZeroVec<'static, T::Storage> {
        if let Storage::Borrowed(storage) = &self.storage {
            self.storage = Storage::Owned(Arc::new(storage.clone().into_owned()));
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
        PackedArray::from_storage(self.storage().as_borrowed())
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
mod tests {
    use super::*;
    use crate::model::common::IntoOwned;

    #[test]
    fn unaligned_vectors_borrow_and_detach_without_layout_assumptions() {
        let raw: Vec<_> = [1f32, -2., 3., 4., 5., -6.]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect();
        let mut unaligned = vec![0u8];
        unaligned.extend_from_slice(&raw);
        let values = PackedArray::<Vec3>::parse_bytes(&unaligned[1..]).unwrap();
        assert_eq!(values.as_bytes().as_ptr(), unaligned[1..].as_ptr());
        assert_eq!(values.get(1), Some(Vec3::new(4., 5., -6.)));
        let tail = values.get_subslice(1..2).unwrap();
        assert_eq!(tail.as_bytes().as_ptr(), unaligned[13..].as_ptr());
        assert!(values.get_subslice(1..3).is_none());
        assert!(values.get(2).is_none());
        let owned = values.clone().into_owned();
        assert_eq!(owned, values);
        assert_ne!(owned.as_bytes().as_ptr(), values.as_bytes().as_ptr());
        let pointer = owned.as_bytes().as_ptr();
        let moved = IntoOwned::into_owned(owned);
        assert_eq!(moved.as_bytes().as_ptr(), pointer);
        assert_eq!(moved.as_borrowed().as_bytes().as_ptr(), pointer);
        assert!(PackedArray::<Vec3>::parse_bytes(&raw[..23]).is_err());
    }

    #[test]
    fn quaternion_storage_preserves_authored_components_and_float_bits() {
        let bits = [0x80000000, 0x7fc01234, 0x3f800000, 0x40000000];
        let bytes: Vec<_> = bits.into_iter().flat_map(u32::to_le_bytes).collect();
        let values = PackedArray::<Quat>::parse_bytes(&bytes).unwrap();
        assert_eq!(values.get(0).unwrap().to_array().map(f32::to_bits), bits);
        let encoded: PackedArray<Quat> = values.iter().collect();
        assert_eq!(encoded.as_bytes(), bytes);
        let unnormalized = Quat::from_xyzw(0., 0., 0., 2.);
        assert_eq!(
            [unnormalized].into_packed_array().get(0),
            Some(unnormalized)
        );
    }

    #[test]
    fn edits_copy_borrowed_bytes_once_and_reuse_owned_storage() {
        let source = [Vec3::ONE, Vec3::ZERO].into_packed_array();
        let mut edited = source.as_borrowed();
        assert!(!edited.is_owned());
        assert_eq!(edited.set(2, Vec3::X), None);
        assert!(!edited.is_owned());
        assert_eq!(edited.set(0, Vec3::Y), Some(Vec3::ONE));
        assert_eq!(source.first(), Some(Vec3::ONE));
        let pointer = edited.as_bytes().as_ptr();
        edited.for_each_mut(|v| *v *= 2.);
        assert_eq!(edited.as_bytes().as_ptr(), pointer);
        assert_eq!(edited.first(), Some(Vec3::Y * 2.));
    }

    #[test]
    fn owned_adoption_cloning_and_detaching_retain_encoded_storage() {
        let native: ZeroVec<u32> = [7, 11, 13].into_iter().collect();
        let pointer = native.as_bytes().as_ptr();
        let values = PackedArray::<u32>::from_storage(native);
        assert!(values.is_owned());
        assert_eq!(values.as_bytes().as_ptr(), pointer);
        let cloned = values.clone();
        assert_eq!(cloned.as_bytes().as_ptr(), pointer);
        let detached = IntoOwned::into_owned(cloned);
        assert_eq!(detached.as_bytes().as_ptr(), pointer);
        drop(values);
        let native = detached.into_storage();
        assert_eq!(native.as_bytes().as_ptr(), pointer);
        assert_eq!(native.to_vec(), [7, 11, 13]);
    }

    #[test]
    fn exclusive_native_extraction_copies_only_shared_owned_storage() {
        let values = [3u16, 5, 8].into_packed_array();
        let pointer = values.as_bytes().as_ptr();
        let extracted = values.clone().into_storage();
        assert_ne!(extracted.as_bytes().as_ptr(), pointer);
        assert_eq!(extracted.as_bytes(), values.as_bytes());
        assert_eq!(values.into_storage().as_bytes().as_ptr(), pointer);

        let bytes = [3u16, 5, 8]
            .into_iter()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>();
        let borrowed = PackedArray::<u16>::parse_bytes(&bytes).unwrap();
        assert_eq!(borrowed.clone().as_bytes().as_ptr(), bytes.as_ptr());
        let native = borrowed.into_storage();
        assert!(!native.is_owned());
        assert_eq!(native.as_bytes().as_ptr(), bytes.as_ptr());
    }

    #[test]
    fn shared_edits_detach_once_and_unique_edits_reuse_storage() {
        let original = [2u32, 4, 6].into_packed_array();
        let pointer = original.as_bytes().as_ptr();
        let mut single_edit = original.clone();
        assert_eq!(single_edit.set(3, 9), None);
        assert_eq!(single_edit.as_bytes().as_ptr(), pointer);
        assert_eq!(single_edit.set(1, 8), Some(4));
        let edited_pointer = single_edit.as_bytes().as_ptr();
        assert_ne!(edited_pointer, pointer);
        assert_eq!(single_edit.set(2, 10), Some(6));
        assert_eq!(single_edit.as_bytes().as_ptr(), edited_pointer);
        assert_eq!(single_edit.to_vec(), [2, 8, 10]);
        assert_eq!(original.to_vec(), [2, 4, 6]);

        let mut bulk_edit = original.clone();
        bulk_edit.for_each_mut(|value| *value += 1);
        let edited_pointer = bulk_edit.as_bytes().as_ptr();
        assert_ne!(edited_pointer, pointer);
        assert_eq!(bulk_edit.to_vec(), [3, 5, 7]);
        bulk_edit.for_each_mut(|value| *value *= 2);
        assert_eq!(bulk_edit.as_bytes().as_ptr(), edited_pointer);
        assert_eq!(bulk_edit.to_vec(), [6, 10, 14]);
        drop(single_edit);
        drop(bulk_edit);
        let mut unique = original;
        unique.for_each_mut(|value| *value *= 3);
        assert_eq!(unique.as_bytes().as_ptr(), pointer);
        assert_eq!(unique.to_vec(), [6, 12, 18]);
    }

    #[test]
    fn shared_float_storage_preserves_bits_through_detachment_and_extraction() {
        let bits = [0x80000000, 0x7fc01234, 0x3f800000, 0x40000000];
        let bytes: Vec<_> = bits.into_iter().flat_map(u32::to_le_bytes).collect();
        let values = PackedArray::<Quat>::parse_bytes(&bytes)
            .unwrap()
            .into_owned();
        let pointer = values.as_bytes().as_ptr();
        let alias = values.clone().into_owned();
        assert_eq!(alias.as_bytes().as_ptr(), pointer);
        assert_eq!(alias.as_bytes(), bytes);
        let extracted = alias.into_storage();
        assert_eq!(extracted.as_bytes(), bytes);
        assert_ne!(extracted.as_bytes().as_ptr(), pointer);
        assert_eq!(values.as_bytes(), bytes);
    }

    #[test]
    fn panicking_bulk_edit_leaves_shared_aliases_unchanged() {
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let original = [f32::from_bits(0x7fc01234), -0., 1.].into_packed_array();
        let alias = original.clone();
        let bytes = original.as_bytes().to_vec();
        let mut edited = original.clone();
        let mut visits = 0;
        let result = catch_unwind(AssertUnwindSafe(|| {
            edited.for_each_mut(|value| {
                visits += 1;
                assert_ne!(visits, 2, "stop after the first committed edit");
                *value = 2.;
            });
        }));
        assert!(result.is_err());
        assert_eq!(original.as_bytes(), bytes);
        assert_eq!(alias.as_bytes(), bytes);
        assert_ne!(edited.as_bytes().as_ptr(), original.as_bytes().as_ptr());
        assert_eq!(edited.first(), Some(2.));
        assert_eq!(edited.get(1).unwrap().to_bits(), (-0f32).to_bits());
        assert_eq!(edited.last(), Some(1.));
    }

    #[test]
    fn empty_storage_stays_borrowed_through_clone_ownership_and_edits() {
        const EMPTY: PackedArray<'static, u32> = PackedArray::new();
        let mut empty = EMPTY.clone().into_owned();
        assert!(matches!(empty.storage, Storage::Borrowed(_)));
        assert_eq!(empty.set(0, 1), None);
        empty.for_each_mut(|_| panic!("empty arrays have no values"));
        assert!(matches!(empty.storage, Storage::Borrowed(_)));
        assert!(!empty.into_storage().is_owned());
        let native = ZeroVec::<u32>::new_owned(Vec::with_capacity(4));
        let empty = PackedArray::<u32>::from_storage(native);
        assert!(matches!(empty.storage, Storage::Borrowed(_)));
        assert!(empty.is_empty());
    }

    #[test]
    fn owning_storage_does_not_require_static_semantic_values() {
        use std::marker::PhantomData;

        #[derive(Clone, Copy)]
        struct Tagged<'a>(u8, PhantomData<&'a ()>);
        impl PackedValue for Tagged<'_> {
            type Storage = u8;
            fn pack(self) -> u8 {
                self.0
            }
            fn unpack(value: u8) -> Self {
                Self(value, PhantomData)
            }
        }
        fn detach<'a>(value: Tagged<'a>) -> PackedArray<'static, Tagged<'a>> {
            [value].into_packed_array().into_owned()
        }
        let values = detach(Tagged(7, PhantomData));
        assert_eq!(values.clone().first().unwrap().0, 7);
    }

    #[test]
    fn matrices_and_scalar_collections_round_trip() {
        let matrix = Mat4::from_scale_rotation_translation(
            Vec3::new(2., 3., 4.),
            Quat::IDENTITY,
            Vec3::new(5., 6., 7.),
        );
        let values = [matrix].into_packed_array();
        assert_eq!(values.as_bytes().len(), 64);
        assert_eq!(
            PackedArray::<Mat4>::parse_bytes(values.as_bytes())
                .unwrap()
                .first(),
            Some(matrix)
        );
        let scalar = [1u32, 2, 3].into_packed_array();
        assert_eq!(scalar.iter().rev().collect::<Vec<_>>(), [3, 2, 1]);
    }
}
