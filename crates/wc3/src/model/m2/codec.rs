//! Checked offset-based record decoding and relocating encoding.
use super::records::ModelFlags;
use super::{Read, ReadError, RecordLayout, Write, WriteError};
use crate::model::common::binary::read_bytes;
use crate::model::common::Ownership;
use crate::model::m2;
use std::array::from_fn;
use std::fmt::Debug;
use std::sync::Arc;
use zerovec::{ule::AsULE, ZeroVec};

/// An on-disk count and offset pair, relative to its containing address space.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, m2::Read, m2::Write, m2::RecordLayout,
)]
pub struct ArrayRef {
    /// Number of elements.
    pub count: u32,
    /// Byte offset of the first element.
    pub offset: u32,
}
/// An unaligned little-endian fixed-width value.
pub trait Value: AsULE + Debug + PartialEq + 'static {}
impl<T: AsULE + Debug + PartialEq + 'static> Value for T {}

/// A leaf array stored in this document or in an external animation stream.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum Array<'data, T: Value> {
    /// Resolved values borrowing their source or holding edited values.
    Inline(ZeroVec<'data, T>),
    /// An unresolved descriptor into another document.
    External(ArrayRef),
}
impl<T: Value> Default for Array<'_, T> {
    fn default() -> Self {
        Self::Inline(ZeroVec::new())
    }
}
impl<'data, T: Value> Array<'data, T> {
    /// Resolves an external descriptor using an explicitly selected stream.
    pub fn resolve<'view>(
        &'view self,
        stream: &'view [u8],
    ) -> Result<ZeroVec<'view, T>, ReadError> {
        match self {
            Self::Inline(values) => Ok(values.as_slice().as_zerovec()),
            Self::External(reference) => {
                Decoder::new(stream, 0, ModelFlags::default()).array(*reference)
            }
        }
    }
}

/// A bounded address space and a sequential metadata cursor.
#[derive(Clone, Debug)]
pub struct Decoder<'data> {
    data: &'data [u8],
    position: usize,
    origin: usize,
    /// Numeric core layout version.
    pub version: u32,
    /// Core model flags selecting conditional layouts.
    pub flags: ModelFlags,
    /// Whether each sequence stores its track leaves externally.
    pub external_sequences: Arc<[bool]>,
    /// Preserve ordinary leaf descriptors when their sequence storage context is unavailable.
    pub unresolved_sequences: bool,
}
impl<'data> Decoder<'data> {
    /// Creates an address space whose offsets are relative to its first byte.
    pub fn new(data: &'data [u8], version: u32, flags: ModelFlags) -> Self {
        Self {
            data,
            position: 0,
            origin: 0,
            version,
            flags,
            external_sequences: Arc::from([]),
            unresolved_sequences: false,
        }
    }
    /// Associates this address space with an absolute source offset.
    pub fn with_origin(mut self, origin: usize) -> Self {
        self.origin = origin;
        self
    }
    /// Current relative cursor position.
    pub fn position(&self) -> usize {
        self.position
    }
    /// Absolute cursor position within the enclosing encoded source.
    pub(crate) fn absolute_position(&self) -> usize {
        self.origin.saturating_add(self.position)
    }
    /// Entire address space.
    pub fn data(&self) -> &'data [u8] {
        self.data
    }
    /// Creates another metadata cursor sharing this address space and context.
    pub fn at(&self, offset: u32) -> Result<Self, ReadError> {
        let position = offset as usize;
        if position > self.data.len() {
            return Err(self.error("offset"));
        }
        let mut result = self.clone();
        result.position = position;
        Ok(result)
    }
    /// Reads a composable value in this address space.
    pub fn read<T: Read<'data>>(&mut self) -> Result<T, ReadError> {
        T::read(self)
    }
    /// Reads a bounded byte sequence.
    pub fn bytes(&mut self, count: usize) -> Result<&'data [u8], ReadError> {
        read_bytes(self.data, &mut self.position, count, self.origin)
            .map_err(|error| ReadError::invalid(error.offset, "truncated record"))
    }
    fn error(&self, field: &'static str) -> ReadError {
        ReadError::invalid(self.origin.saturating_add(self.position), field)
    }
    /// Reads an unsigned byte.
    pub fn u8(&mut self) -> Result<u8, ReadError> {
        Ok(self.bytes(1)?[0])
    }
    /// Reads a signed byte.
    pub fn i8(&mut self) -> Result<i8, ReadError> {
        Ok(self.u8()? as i8)
    }
    /// Reads an unsigned little-endian word.
    pub fn u16(&mut self) -> Result<u16, ReadError> {
        Ok(u16::from_le_bytes(self.bytes(2)?.try_into().unwrap()))
    }
    /// Reads a signed little-endian word.
    pub fn i16(&mut self) -> Result<i16, ReadError> {
        Ok(self.u16()? as i16)
    }
    /// Reads an unsigned little-endian doubleword.
    pub fn u32(&mut self) -> Result<u32, ReadError> {
        Ok(u32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }
    /// Reads a signed little-endian doubleword.
    pub fn i32(&mut self) -> Result<i32, ReadError> {
        Ok(self.u32()? as i32)
    }
    /// Reads an IEEE little-endian single-precision float.
    pub fn f32(&mut self) -> Result<f32, ReadError> {
        Ok(f32::from_bits(self.u32()?))
    }
    /// Reads a count/offset descriptor.
    pub fn descriptor(&mut self) -> Result<ArrayRef, ReadError> {
        Ok(ArrayRef {
            count: self.u32()?,
            offset: self.u32()?,
        })
    }
    /// Borrows an unaligned array without decoding into native storage.
    pub fn array<T: Value>(&self, reference: ArrayRef) -> Result<ZeroVec<'data, T>, ReadError> {
        if reference.count == 0 {
            return Ok(ZeroVec::new());
        }
        let count = (reference.count as usize)
            .checked_mul(size_of::<T::ULE>())
            .ok_or_else(|| self.error("array size overflow"))?;
        let mut cursor = self.at(reference.offset)?;
        let bytes = cursor.bytes(count)?;
        ZeroVec::parse_bytes(bytes).map_err(|_| cursor.error("array value"))
    }
    /// Decodes an array of offset-bearing records.
    pub fn records<T: Read<'data> + RecordLayout>(
        &self,
        reference: ArrayRef,
    ) -> Result<Vec<T>, ReadError> {
        if reference.count == 0 {
            return Ok(Vec::new());
        }
        let width = T::size(self.version, self.flags.bits());
        if width == 0 {
            return Err(self.error("zero-width record"));
        }
        let length = (reference.count as usize)
            .checked_mul(width)
            .ok_or_else(|| self.error("record array overflow"))?;
        let mut cursor = self.at(reference.offset)?;
        cursor.bytes(length)?;
        cursor.position = reference.offset as usize;
        let mut records = Vec::with_capacity(reference.count as usize);
        for _ in 0..reference.count {
            let start = cursor.position;
            let value = T::read(&mut cursor)?;
            if cursor.position != start + width {
                return Err(cursor.error("record width"));
            }
            records.push(value);
        }
        Ok(records)
    }
}

/// Deterministic allocator and metadata writer for one offset address space.
#[derive(Clone, Debug)]
pub struct Encoder {
    data: Vec<u8>,
    position: usize,
    /// Numeric core layout version.
    pub version: u32,
    /// Core flags selecting conditional record layouts.
    pub flags: ModelFlags,
}
impl Encoder {
    /// Starts a fresh address space.
    pub fn new(version: u32, flags: ModelFlags) -> Self {
        Self {
            data: Vec::new(),
            position: 0,
            version,
            flags,
        }
    }
    /// Seeds an address space whose opaque offsets must remain stable.
    pub fn from_bytes(version: u32, flags: ModelFlags, data: Vec<u8>) -> Result<Self, WriteError> {
        if data.len() > u32::MAX as usize {
            return Err(WriteError::invalid("32-bit address space"));
        }
        Ok(Self {
            data,
            position: 0,
            version,
            flags,
        })
    }
    /// Writes a composable value in this address space.
    pub fn write<T: Write>(&mut self, value: &T) -> Result<(), WriteError> {
        value.write(self)
    }
    /// Returns completed encoded bytes.
    pub fn finish(self) -> Vec<u8> {
        self.data
    }
    /// Current metadata write position.
    pub fn position(&self) -> usize {
        self.position
    }
    /// Length of all allocated blocks.
    pub fn len(&self) -> usize {
        self.data.len()
    }
    /// Whether no bytes have been allocated.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    /// Moves the metadata cursor within allocated storage.
    pub fn set_position(&mut self, position: usize) -> Result<(), WriteError> {
        if position > self.data.len() {
            return Err(WriteError::invalid("write position"));
        }
        self.position = position;
        Ok(())
    }
    /// Runs a metadata writer at a chosen allocated offset, restoring the cursor.
    pub fn with_position<T>(
        &mut self,
        position: usize,
        write: impl FnOnce(&mut Self) -> Result<T, WriteError>,
    ) -> Result<T, WriteError> {
        let saved = self.position;
        self.set_position(position)?;
        let result = write(self);
        self.position = saved;
        result
    }
    /// Ensures a complete inline header exists before allocating nested payloads.
    pub fn reserve_metadata(&mut self, width: usize) -> Result<(), WriteError> {
        let end = self
            .position
            .checked_add(width)
            .ok_or_else(|| WriteError::invalid("metadata size overflow"))?;
        if end > self.data.len() {
            self.reserve(end - self.data.len())?;
        }
        Ok(())
    }
    /// Appends zero-filled storage without moving the metadata cursor.
    pub fn reserve(&mut self, count: usize) -> Result<usize, WriteError> {
        let start = self.data.len();
        let end = start
            .checked_add(count)
            .ok_or_else(|| WriteError::invalid("size overflow"))?;
        if end > u32::MAX as usize {
            return Err(WriteError::invalid("32-bit offset overflow"));
        }
        self.data
            .try_reserve(count)
            .map_err(|_| WriteError::invalid("allocation"))?;
        self.data.resize(end, 0);
        Ok(start)
    }
    /// Aligns subsequent allocated blocks without moving the metadata cursor.
    pub fn align(&mut self, alignment: usize) -> Result<(), WriteError> {
        if !alignment.is_power_of_two() {
            return Err(WriteError::invalid("alignment"));
        }
        let padding = self.data.len().wrapping_neg() & (alignment - 1);
        self.reserve(padding)?;
        Ok(())
    }
    /// Writes bytes at the metadata cursor, extending storage when needed.
    pub fn bytes(&mut self, bytes: &[u8]) -> Result<(), WriteError> {
        let end = self
            .position
            .checked_add(bytes.len())
            .ok_or_else(|| WriteError::invalid("size overflow"))?;
        if end > self.data.len() {
            self.reserve(end - self.data.len())?;
        }
        self.data[self.position..end].copy_from_slice(bytes);
        self.position = end;
        Ok(())
    }
    /// Writes an unsigned byte.
    pub fn u8(&mut self, value: u8) -> Result<(), WriteError> {
        self.bytes(&[value])
    }
    /// Writes a signed byte.
    pub fn i8(&mut self, value: i8) -> Result<(), WriteError> {
        self.u8(value as u8)
    }
    /// Writes an unsigned little-endian word.
    pub fn u16(&mut self, value: u16) -> Result<(), WriteError> {
        self.bytes(&value.to_le_bytes())
    }
    /// Writes a signed little-endian word.
    pub fn i16(&mut self, value: i16) -> Result<(), WriteError> {
        self.u16(value as u16)
    }
    /// Writes an unsigned little-endian doubleword.
    pub fn u32(&mut self, value: u32) -> Result<(), WriteError> {
        self.bytes(&value.to_le_bytes())
    }
    /// Writes a signed little-endian doubleword.
    pub fn i32(&mut self, value: i32) -> Result<(), WriteError> {
        self.u32(value as u32)
    }
    /// Writes an IEEE single-precision float.
    pub fn f32(&mut self, value: f32) -> Result<(), WriteError> {
        self.u32(value.to_bits())
    }
    /// Patches a previously allocated doubleword.
    pub fn patch_u32(&mut self, position: usize, value: u32) -> Result<(), WriteError> {
        self.with_position(position, |output| output.u32(value))
    }
    /// Writes a count/offset descriptor at the metadata cursor.
    pub fn write_descriptor(&mut self, value: ArrayRef) -> Result<(), WriteError> {
        self.u32(value.count)?;
        self.u32(value.offset)
    }
    /// Allocates an encoded leaf array and returns its relocated descriptor.
    pub fn array<T: Value>(&mut self, values: &ZeroVec<'_, T>) -> Result<ArrayRef, WriteError> {
        if values.is_empty() {
            return Ok(ArrayRef::default());
        }
        self.align(16)?;
        let offset = self.reserve(values.as_bytes().len())?;
        self.data[offset..offset + values.as_bytes().len()].copy_from_slice(values.as_bytes());
        Ok(ArrayRef {
            count: u32::try_from(values.len()).map_err(|_| WriteError::invalid("array count"))?,
            offset: offset as u32,
        })
    }
    /// Allocates an array of record headers and their nested payloads.
    pub fn write_records<T: Write + RecordLayout>(
        &mut self,
        records: &[T],
    ) -> Result<ArrayRef, WriteError> {
        if records.is_empty() {
            return Ok(ArrayRef::default());
        }
        let width = T::size(self.version, self.flags.bits());
        if width == 0 {
            return Err(WriteError::invalid("zero-width record"));
        }
        let count =
            u32::try_from(records.len()).map_err(|_| WriteError::invalid("record count"))?;
        let size = records
            .len()
            .checked_mul(width)
            .ok_or_else(|| WriteError::invalid("record array overflow"))?;
        self.align(16)?;
        let offset = self.reserve(size)?;
        for (index, record) in records.iter().enumerate() {
            self.with_position(offset + index * width, |output| {
                let start = output.position;
                record.write(output)?;
                if output.position != start + width {
                    return Err(WriteError::invalid("record width"));
                }
                Ok(())
            })?;
        }
        Ok(ArrayRef {
            count,
            offset: offset as u32,
        })
    }
}

macro_rules! scalar_codecs {
    ($($ty:ty => $method:ident, $size:literal);* $(;)?) => { $(
        impl<'data> Read<'data> for $ty {
            fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> { input.$method() }
        }
        impl Write for $ty {
            fn write(&self, output: &mut Encoder) -> Result<(), WriteError> { output.$method(*self) }
        }
        impl RecordLayout for $ty {
            fn size(_: u32, _: u32) -> usize { $size }
        }
    )* };
}
scalar_codecs! {
    u8 => u8, 1; i8 => i8, 1; u16 => u16, 2; i16 => i16, 2;
    u32 => u32, 4; i32 => i32, 4; f32 => f32, 4;
}
impl<'data, T: Read<'data>, const N: usize> Read<'data> for [T; N] {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let mut result = Ok(());
        let values: [Option<T>; N] = from_fn(|_| {
            if result.is_err() {
                return None;
            }
            match T::read(input) {
                Ok(value) => Some(value),
                Err(error) => {
                    result = Err(error);
                    None
                }
            }
        });
        result?;
        Ok(values.map(|value| value.expect("every fixed array element was read")))
    }
}
impl<T: Write + RecordLayout, const N: usize> Write for [T; N] {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.reserve_metadata(Self::size(output.version, output.flags.bits()))?;
        for value in self {
            value.write(output)?;
        }
        Ok(())
    }
}
impl<T: RecordLayout, const N: usize> RecordLayout for [T; N] {
    fn size(version: u32, flags: u32) -> usize {
        N * T::size(version, flags)
    }
}
impl<'data, T: Value> Read<'data> for ZeroVec<'data, T> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let reference = input.descriptor()?;
        input.array(reference)
    }
}
impl<T: Value> Write for ZeroVec<'_, T> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.reserve_metadata(8)?;
        let reference = output.array(self)?;
        output.write_descriptor(reference)
    }
}
impl<T: Value> RecordLayout for ZeroVec<'_, T> {
    fn size(_: u32, _: u32) -> usize {
        8
    }
}
impl<'data, T: Read<'data> + RecordLayout> Read<'data> for Vec<T> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let reference = input.descriptor()?;
        input.records(reference)
    }
}
impl<T: Write + RecordLayout> Write for Vec<T> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.reserve_metadata(8)?;
        let reference = output.write_records(self)?;
        output.write_descriptor(reference)
    }
}
impl<T> RecordLayout for Vec<T> {
    fn size(_: u32, _: u32) -> usize {
        8
    }
}
impl<'data> Read<'data> for () {
    fn read(_: &mut Decoder<'data>) -> Result<Self, ReadError> {
        Ok(())
    }
}
impl Write for () {
    fn write(&self, _: &mut Encoder) -> Result<(), WriteError> {
        Ok(())
    }
}
impl RecordLayout for () {
    fn size(_: u32, _: u32) -> usize {
        0
    }
}
