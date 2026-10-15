use super::{read_bytes, BoundsError};
use std::array;
use std::error::Error;
use std::fmt;
use std::mem::size_of;
use zerovec::ule::{AsULE, ULE};

/// Failure to decode a fixed-width native value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueError {
    /// The value extends beyond its address space.
    Bounds(BoundsError),
    /// The bytes are invalid for the value, or its encoding has zero width.
    Invalid,
}
impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bounds(error) => error.fmt(f),
            Self::Invalid => f.write_str("invalid scalar encoding"),
        }
    }
}
impl Error for ValueError {}

/// A bounded borrowed byte address space with an absolute diagnostic origin.
#[derive(Clone, Copy, Debug)]
pub struct AddressSpace<'data> {
    data: &'data [u8],
    origin: usize,
}
impl<'data> AddressSpace<'data> {
    /// Creates an address space whose offsets start at zero.
    pub fn new(data: &'data [u8]) -> Self {
        Self { data, origin: 0 }
    }
    /// Associates the first byte with an absolute source offset.
    pub fn with_origin(mut self, origin: usize) -> Self {
        self.origin = origin;
        self
    }
    /// Returns the entire bounded source.
    pub fn data(&self) -> &'data [u8] {
        self.data
    }
    /// Returns the absolute source offset of the first byte.
    pub fn origin(&self) -> usize {
        self.origin
    }
    /// Borrows a checked range addressed relative to the first byte.
    pub fn range(&self, at: usize, size: usize) -> Result<&'data [u8], BoundsError> {
        let mut position = at;
        read_bytes(self.data, &mut position, size, self.origin)
    }
    /// Decodes a little-endian scalar or fixed scalar array.
    pub fn value<T: AsULE>(&self, at: usize) -> Result<T, ValueError> {
        let size = size_of::<T::ULE>();
        if size == 0 {
            return Err(ValueError::Invalid);
        }
        let bytes = self.range(at, size).map_err(ValueError::Bounds)?;
        let values = T::ULE::parse_bytes_to_slice(bytes).map_err(|_| ValueError::Invalid)?;
        Ok(T::from_unaligned(values[0]))
    }
}

/// A sequential cursor over a bounded byte address space.
#[derive(Clone, Copy, Debug)]
pub struct Cursor<'data> {
    space: AddressSpace<'data>,
    position: usize,
}
impl<'data> Cursor<'data> {
    /// Creates a cursor at the start of the supplied source.
    pub fn new(data: &'data [u8]) -> Self {
        Self {
            space: AddressSpace::new(data),
            position: 0,
        }
    }
    /// Associates the first byte with an absolute source offset.
    pub fn with_origin(mut self, origin: usize) -> Self {
        self.space = self.space.with_origin(origin);
        self
    }
    /// Returns the relative cursor position.
    pub fn position(&self) -> usize {
        self.position
    }
    /// Returns the number of unread bytes.
    pub fn remaining(&self) -> usize {
        self.space.data.len() - self.position
    }
    /// Returns the entire bounded source.
    pub fn data(&self) -> &'data [u8] {
        self.space.data
    }
    /// Returns the absolute position, saturating on diagnostic overflow.
    pub fn absolute_position(&self) -> usize {
        self.space.origin.saturating_add(self.position)
    }
    /// Reads bytes, advancing only when the full range is available.
    pub fn bytes(&mut self, count: usize) -> Result<&'data [u8], BoundsError> {
        read_bytes(
            self.space.data,
            &mut self.position,
            count,
            self.space.origin,
        )
    }
    /// Sets the relative position, allowing the end of the source.
    pub fn seek(&mut self, position: usize) -> Result<(), BoundsError> {
        self.space.range(position, 0)?;
        self.position = position;
        Ok(())
    }
    /// Creates another cursor at a checked relative position.
    pub fn at(&self, position: usize) -> Result<Self, BoundsError> {
        let mut result = *self;
        result.seek(position)?;
        Ok(result)
    }
    /// Reads a native value, advancing only after successful decoding.
    pub fn value<T: AsULE>(&mut self) -> Result<T, ValueError> {
        let result = self.space.value(self.position)?;
        self.position += size_of::<T::ULE>();
        Ok(result)
    }
}

/// A failed output allocation, alignment, or bounded write.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputError {
    /// Size arithmetic overflowed.
    Overflow,
    /// Alignment was zero.
    Alignment,
    /// Memory could not be reserved.
    Allocation,
    /// A patch addressed bytes outside the existing output.
    Bounds(BoundsError),
    /// A native scalar encoding has zero width.
    InvalidValue,
}
impl fmt::Display for OutputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow => f.write_str("output size overflow"),
            Self::Alignment => f.write_str("zero output alignment"),
            Self::Allocation => f.write_str("output allocation failed"),
            Self::Bounds(error) => error.fmt(f),
            Self::InvalidValue => f.write_str("invalid scalar encoding"),
        }
    }
}
impl Error for OutputError {}

/// Rounds a position up to any nonzero byte alignment with checked arithmetic.
pub fn align_position(position: usize, alignment: usize) -> Result<usize, OutputError> {
    if alignment == 0 {
        return Err(OutputError::Alignment);
    }
    let padding = (alignment - position % alignment) % alignment;
    position.checked_add(padding).ok_or(OutputError::Overflow)
}

/// Appends bytes to owned or borrowed output storage with checked allocation.
pub fn append_bytes(data: &mut Vec<u8>, bytes: &[u8]) -> Result<(), OutputError> {
    data.len()
        .checked_add(bytes.len())
        .ok_or(OutputError::Overflow)?;
    data.try_reserve(bytes.len())
        .map_err(|_| OutputError::Allocation)?;
    data.extend_from_slice(bytes);
    Ok(())
}

/// Reserves a zero-filled aligned block and returns its byte offset.
pub fn reserve_bytes(
    data: &mut Vec<u8>,
    count: usize,
    alignment: usize,
) -> Result<usize, OutputError> {
    let at = align_position(data.len(), alignment)?;
    let end = at.checked_add(count).ok_or(OutputError::Overflow)?;
    data.try_reserve(end - data.len())
        .map_err(|_| OutputError::Allocation)?;
    data.resize(end, 0);
    Ok(at)
}

/// Patches an existing byte range without changing its length.
pub fn patch_bytes(data: &mut [u8], at: usize, bytes: &[u8]) -> Result<(), OutputError> {
    AddressSpace::new(data)
        .range(at, bytes.len())
        .map_err(OutputError::Bounds)?;
    data[at..at + bytes.len()].copy_from_slice(bytes);
    Ok(())
}

/// Growable output storage supporting aligned reservations and bounded patches.
#[derive(Clone, Debug, Default)]
pub struct Output {
    data: Vec<u8>,
}
impl From<Vec<u8>> for Output {
    fn from(data: Vec<u8>) -> Self {
        Self { data }
    }
}
impl Output {
    /// Creates empty output storage.
    pub fn new() -> Self {
        Self::default()
    }
    /// Uses existing owned bytes as the initial output.
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self::from(data)
    }
    /// Returns the encoded byte count.
    pub fn position(&self) -> usize {
        self.data.len()
    }
    /// Returns the encoded byte count.
    pub fn len(&self) -> usize {
        self.data.len()
    }
    /// Reports whether the output has no bytes.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    /// Borrows the encoded bytes.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    /// Appends bytes, preserving existing output on allocation failure.
    pub fn append(&mut self, bytes: &[u8]) -> Result<(), OutputError> {
        append_bytes(&mut self.data, bytes)
    }
    /// Appends bytes as a sequential codec operation.
    pub fn bytes(&mut self, bytes: &[u8]) -> Result<(), OutputError> {
        self.append(bytes)
    }
    /// Reserves zero-filled bytes at an aligned offset, including zero padding.
    /// Any nonzero alignment is accepted; it need not be a power of two.
    pub fn reserve(&mut self, count: usize, alignment: usize) -> Result<usize, OutputError> {
        reserve_bytes(&mut self.data, count, alignment)
    }
    /// Borrows a checked range of existing output bytes.
    pub fn range(&self, at: usize, count: usize) -> Result<&[u8], BoundsError> {
        AddressSpace::new(&self.data).range(at, count)
    }
    /// Replaces an existing range without changing the output length.
    pub fn patch(&mut self, at: usize, bytes: &[u8]) -> Result<(), OutputError> {
        patch_bytes(&mut self.data, at, bytes)
    }
    /// Writes a native scalar or fixed scalar array into an existing range.
    pub fn value<T: AsULE>(&mut self, at: usize, value: &T) -> Result<(), OutputError> {
        if size_of::<T::ULE>() == 0 {
            return Err(OutputError::InvalidValue);
        }
        let value = value.to_unaligned();
        self.patch(at, T::ULE::slice_as_bytes(array::from_ref(&value)))
    }
    /// Returns the complete owned output.
    pub fn finish(self) -> Vec<u8> {
        self.data
    }
}

#[cfg(test)]
#[path = "storage_tests.rs"]
mod tests;
