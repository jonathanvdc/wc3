//! Borrowed MDX buffers and independently owned edits.
use super::{mdx, Borrowed, Cursor, Detached, Encoder};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::ops::{Deref, DerefMut, Range};
use zerovec::ule::AsULE;
use zerovec::ZeroVec;

/// Opaque bytes borrowed from a source or held in an owned allocation.
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct ModelBytes<'data>(Cow<'data, [u8]>);
impl<'data> ModelBytes<'data> {
    /// Moves an existing allocation into the buffer.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(Cow::Owned(bytes))
    }
    /// Borrows bytes without allocating.
    pub fn borrowed(bytes: &'data [u8]) -> Self {
        Self(Cow::Borrowed(bytes))
    }
    pub(crate) fn into_cow(self) -> Cow<'data, [u8]> {
        self.0
    }
    /// Borrows this buffer, including when it owns its allocation.
    pub fn as_borrowed(&self) -> ModelBytes<'_> {
        ModelBytes::borrowed(self)
    }
    /// Copies this buffer into independent owned storage.
    pub fn to_owned(&self) -> ModelBytes<'static> {
        self.as_borrowed().into_owned()
    }
    /// Moves owned bytes and copies borrowed bytes into independent storage.
    pub fn into_owned(self) -> ModelBytes<'static> {
        self.into_detached()
    }
    /// Returns mutable bytes, copying a borrowed buffer once.
    pub fn to_mut(&mut self) -> &mut Vec<u8> {
        self.0.to_mut()
    }
    /// Whether this buffer borrows its bytes.
    pub fn is_borrowed(&self) -> bool {
        matches!(self.0, Cow::Borrowed(_))
    }
}
impl From<Vec<u8>> for ModelBytes<'_> {
    fn from(bytes: Vec<u8>) -> Self {
        Self::new(bytes)
    }
}
impl Deref for ModelBytes<'_> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}
impl DerefMut for ModelBytes<'_> {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.to_mut()
    }
}
impl<'view> Borrowed<'view> for ModelBytes<'_> {
    type View = ModelBytes<'view>;
    fn as_borrowed(&'view self) -> Self::View {
        self.as_borrowed()
    }
}
impl<'owned> Detached<'owned> for ModelBytes<'_> {
    type Detached = ModelBytes<'owned>;
    fn into_detached(self) -> Self::Detached {
        ModelBytes::new(self.0.into_owned())
    }
}
impl<const N: usize> PartialEq<[u8; N]> for ModelBytes<'_> {
    fn eq(&self, other: &[u8; N]) -> bool {
        **self == *other
    }
}
impl<const N: usize> PartialEq<&[u8; N]> for ModelBytes<'_> {
    fn eq(&self, other: &&[u8; N]) -> bool {
        **self == **other
    }
}
impl PartialEq<Vec<u8>> for ModelBytes<'_> {
    fn eq(&self, other: &Vec<u8>) -> bool {
        &**self == other.as_slice()
    }
}

#[derive(Clone)]
enum Storage<'data, T: AsULE> {
    Encoded(ZeroVec<'data, T>),
    Values(Cow<'data, [T]>),
}
/// An array borrowing little-endian, possibly unaligned MDX bytes or native values.
///
/// `get` and `iter` return decoded values without allocating. `to_mut` owns only
/// this array and exposes ordinary Rust values for editing. `as_borrowed` lends
/// the current storage, while `into_owned` detaches it from its source.
#[derive(Clone)]
pub struct ModelArray<'data, T: AsULE> {
    storage: Storage<'data, T>,
}
impl<'data, T: AsULE> ModelArray<'data, T> {
    /// Moves a native array into the model without encoding or copying it.
    pub fn new(values: Vec<T>) -> Self {
        Self {
            storage: Storage::Values(Cow::Owned(values)),
        }
    }
    /// Borrows native values without encoding or copying them.
    pub fn borrowed(values: &'data [T]) -> Self {
        Self {
            storage: Storage::Values(Cow::Borrowed(values)),
        }
    }
    /// Number of elements.
    pub fn len(&self) -> usize {
        match &self.storage {
            Storage::Encoded(v) => v.len(),
            Storage::Values(v) => v.len(),
        }
    }
    /// Whether this array is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Reads an element by value without creating a typed buffer.
    pub fn get(&self, index: usize) -> Option<T> {
        match &self.storage {
            Storage::Encoded(v) => v.get(index),
            Storage::Values(v) => v.get(index).copied(),
        }
    }
    /// Reads an element by value; equivalent to `get`.
    pub fn value(&self, index: usize) -> Option<T> {
        self.get(index)
    }
    /// Iterates values without allocating.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = T> + DoubleEndedIterator + '_ {
        (0..self.len()).map(|i| self.get(i).expect("model array index"))
    }
    /// Iterates values without allocating; equivalent to `iter`.
    pub fn values(&self) -> impl ExactSizeIterator<Item = T> + DoubleEndedIterator + '_ {
        self.iter()
    }
    /// Borrows encoded wire bytes when this array retains an MDX representation.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        self.encoded_bytes().map(|(bytes, _)| bytes)
    }
    /// Copies decoded values into a native vector.
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().collect()
    }
    /// Borrows native storage when this array has native values.
    pub fn as_native_slice(&self) -> Option<&[T]> {
        match &self.storage {
            Storage::Values(v) => Some(v),
            _ => None,
        }
    }
    /// Owns this array and returns its native values for editing.
    pub fn to_mut(&mut self) -> &mut Vec<T> {
        if !matches!(self.storage, Storage::Values(_)) {
            self.storage = Storage::Values(Cow::Owned(self.to_vec()));
        }
        match &mut self.storage {
            Storage::Values(v) => v.to_mut(),
            _ => unreachable!(),
        }
    }
    /// Edits one element, owning this array only when the index exists.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len() {
            return None;
        }
        self.to_mut().get_mut(index)
    }
    /// Borrows this storage, including when it currently owns its buffer.
    pub fn as_borrowed(&self) -> ModelArray<'_, T> {
        ModelArray {
            storage: match &self.storage {
                Storage::Encoded(v) => Storage::Encoded(v.as_slice().as_zerovec()),
                Storage::Values(v) => Storage::Values(Cow::Borrowed(v)),
            },
        }
    }
    /// Copies this array into independent owned storage.
    pub fn to_owned(&self) -> ModelArray<'static, T>
    where
        T: 'static,
    {
        self.as_borrowed().into_owned()
    }
    /// Moves owned buffers and copies borrowed buffers into independent storage.
    pub fn into_owned(self) -> ModelArray<'static, T>
    where
        T: 'static,
    {
        self.into_detached()
    }
    /// Whether this array borrows its buffer.
    pub fn is_borrowed(&self) -> bool {
        match &self.storage {
            Storage::Encoded(v) => !v.is_owned(),
            Storage::Values(v) => matches!(v, Cow::Borrowed(_)),
        }
    }
    /// Returns the partition boundary without materializing native values.
    pub fn partition_point(&self, mut predicate: impl FnMut(T) -> bool) -> usize {
        let mut start = 0;
        let mut len = self.len();
        while len > 0 {
            let half = len / 2;
            let mid = start + half;
            if predicate(self.get(mid).expect("model array index")) {
                start = mid + 1;
                len -= half + 1
            } else {
                len = half
            }
        }
        start
    }
    pub(crate) fn encoded_bytes(&self) -> Option<(&[u8], usize)> {
        match &self.storage {
            Storage::Encoded(v) => Some((v.as_bytes(), size_of::<T::ULE>())),

            _ => None,
        }
    }
}
pub(crate) trait WireValue: for<'input> mdx::Read<'input> + AsULE {}
impl WireValue for u8 {}
impl WireValue for u16 {}
impl WireValue for u32 {}
impl WireValue for i32 {}
impl WireValue for f32 {}
impl<T: WireValue + Default, const N: usize> WireValue for [T; N] {}
impl<'data, T: AsULE + for<'input> mdx::Read<'input>> ModelArray<'data, T> {
    pub(crate) fn read_layout(cursor: Cursor<'data>, width: usize) -> Result<Self, mdx::ReadError> {
        let malformed = || {
            mdx::ReadError::new(
                cursor.absolute_position(),
                mdx::ReadErrorKind::InvalidValue {
                    field: "model array layout",
                },
            )
        };
        if width != size_of::<T::ULE>() {
            return Err(malformed());
        }
        let values = ZeroVec::parse_bytes(cursor.remaining()).map_err(|_| malformed())?;
        Ok(Self {
            storage: Storage::Encoded(values),
        })
    }
    pub(crate) fn read(mut cursor: Cursor<'data>, width: usize) -> Result<Self, mdx::ReadError> {
        let source = cursor;
        while !cursor.remaining().is_empty() {
            let mut element = cursor.subcursor(width)?;
            element.read::<T>()?;
            element.finish()?;
        }
        Self::read_layout(source, width)
    }
}
impl<T: AsULE + mdx::Write> ModelArray<'_, T> {
    pub(crate) fn write(&self, encoder: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        if let Some((bytes, _)) = self.encoded_bytes() {
            encoder.write_bytes(bytes);
            return Ok(());
        }
        for value in self.iter() {
            encoder.write(&value)?
        }
        Ok(())
    }
}
impl<T: AsULE> Default for ModelArray<'_, T> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
impl<T: AsULE> From<Vec<T>> for ModelArray<'_, T> {
    fn from(values: Vec<T>) -> Self {
        Self::new(values)
    }
}
impl<T: AsULE> FromIterator<T> for ModelArray<'_, T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}
impl<T: AsULE + Debug> Debug for ModelArray<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_list().entries(self.iter()).finish()
    }
}
impl<T: AsULE + PartialEq> PartialEq for ModelArray<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}
impl<'view, T: AsULE + 'view> Borrowed<'view> for ModelArray<'_, T> {
    type View = ModelArray<'view, T>;
    fn as_borrowed(&'view self) -> Self::View {
        self.as_borrowed()
    }
}
impl<'owned, T: AsULE + 'owned> Detached<'owned> for ModelArray<'_, T> {
    type Detached = ModelArray<'owned, T>;
    fn into_detached(self) -> Self::Detached {
        ModelArray {
            storage: match self.storage {
                Storage::Encoded(v) => Storage::Encoded(v.into_owned()),
                Storage::Values(v) => Storage::Values(Cow::Owned(v.into_owned())),
            },
        }
    }
}
impl<T: AsULE + PartialEq, const N: usize> PartialEq<[T; N]> for ModelArray<'_, T> {
    fn eq(&self, other: &[T; N]) -> bool {
        self.len() == N && self.iter().eq(other.iter().copied())
    }
}
impl<T: AsULE + PartialEq, const N: usize> PartialEq<[T; N]> for &ModelArray<'_, T> {
    fn eq(&self, other: &[T; N]) -> bool {
        self.len() == N && self.iter().eq(other.iter().copied())
    }
}
impl<T: AsULE + PartialEq> PartialEq<Vec<T>> for ModelArray<'_, T> {
    fn eq(&self, other: &Vec<T>) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter().copied())
    }
}
impl<T: AsULE + PartialEq> PartialEq<Vec<T>> for &ModelArray<'_, T> {
    fn eq(&self, other: &Vec<T>) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter().copied())
    }
}
impl<T: AsULE + PartialEq> PartialEq<&[T]> for ModelArray<'_, T> {
    fn eq(&self, other: &&[T]) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter().copied())
    }
}

/// Iterator over decoded model values, without a native slice allocation.
pub struct ModelValues<'view, 'data, T: AsULE> {
    array: &'view ModelArray<'data, T>,
    indices: Range<usize>,
}
impl<T: AsULE> Iterator for ModelValues<'_, '_, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.indices.next().and_then(|i| self.array.get(i))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}
impl<T: AsULE> DoubleEndedIterator for ModelValues<'_, '_, T> {
    fn next_back(&mut self) -> Option<T> {
        self.indices.next_back().and_then(|i| self.array.get(i))
    }
}
impl<T: AsULE> ExactSizeIterator for ModelValues<'_, '_, T> {}
impl<'view, 'data, T: AsULE> IntoIterator for &'view ModelArray<'data, T> {
    type Item = T;
    type IntoIter = ModelValues<'view, 'data, T>;
    fn into_iter(self) -> Self::IntoIter {
        ModelValues {
            array: self,
            indices: 0..self.len(),
        }
    }
}
