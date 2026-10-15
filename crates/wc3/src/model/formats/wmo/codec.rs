//! Bounded little-endian WMO record codecs and borrowed fixed-record arrays.
use super::{Read, ReadError, RecordLayout, Validate, Write, WriteError};
use crate::model::common::{
    binary::{Cursor, Output},
    DecodeRecord, EncodeRecord, EncodedArray, RecordCodec,
};
use crate::model::common::{record_primitives, try_array};
use std::mem::{size_of, take};
/// A cursor within one bounded WMO chunk payload.
#[derive(Clone, Debug)]
pub struct Decoder<'data> {
    cursor: Cursor<'data>,
}
impl<'data> Decoder<'data> {
    /// Creates a cursor at the start of a payload.
    pub fn new(data: &'data [u8]) -> Self {
        Self {
            cursor: Cursor::new(data),
        }
    }
    /// Associates payload-relative locations with an absolute source position.
    pub fn with_origin(mut self, origin: usize) -> Self {
        self.cursor = self.cursor.with_origin(origin);
        self
    }
    /// Relative cursor position in bytes.
    pub fn position(&self) -> usize {
        self.cursor.position()
    }
    /// Number of unread bytes.
    pub fn remaining(&self) -> usize {
        self.cursor.remaining()
    }
    /// Entire bounded payload.
    pub fn data(&self) -> &'data [u8] {
        self.cursor.data()
    }
    /// Absolute cursor position.
    pub fn absolute_position(&self) -> usize {
        self.cursor.absolute_position()
    }
    /// Reads a bounded byte slice without copying.
    pub fn bytes(&mut self, count: usize) -> Result<&'data [u8], ReadError> {
        self.cursor
            .bytes(count)
            .map_err(|e| ReadError::invalid(e.offset, "truncated record"))
    }
    /// Reads a composable native value.
    pub fn read<T: Read<'data>>(&mut self) -> Result<T, ReadError> {
        T::read(self)
    }
}
/// An append-only WMO payload encoder.
#[derive(Clone, Debug, Default)]
pub struct Encoder {
    data: Output,
}
impl Encoder {
    /// Creates an empty payload.
    pub fn new() -> Self {
        Self::default()
    }
    /// Encoded byte count.
    pub fn position(&self) -> usize {
        self.data.position()
    }
    /// Appends bytes, reporting allocation failure as a codec error.
    pub fn bytes(&mut self, bytes: &[u8]) -> Result<(), WriteError> {
        self.data
            .bytes(bytes)
            .map_err(|_| WriteError::invalid("payload allocation"))
    }
    /// Writes a composable native value.
    pub fn write<T: Write + ?Sized>(&mut self, value: &T) -> Result<(), WriteError> {
        value.write(self)
    }
    /// Returns the complete payload.
    pub fn finish(self) -> Vec<u8> {
        self.data.finish()
    }
}
record_primitives!([], [], ['_], Decoder<'_>, Encoder; u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);
impl Read<'_> for () {
    fn read(_: &mut Decoder<'_>) -> Result<Self, ReadError> {
        Ok(())
    }
}
impl Write for () {
    fn write(&self, _: &mut Encoder) -> Result<(), WriteError> {
        Ok(())
    }
}
impl RecordLayout for () {
    fn size() -> usize {
        0
    }
}
impl Validate for () {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}
impl<'data, T: Read<'data>, const N: usize> Read<'data> for [T; N] {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        try_array(|| T::read(input))
    }
}
impl<T: Write, const N: usize> Write for [T; N] {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        for value in self {
            value.write(output)?;
        }
        Ok(())
    }
}
impl<T: RecordLayout, const N: usize> RecordLayout for [T; N] {
    fn size() -> usize {
        T::size() * N
    }
}
impl<T: Validate, const N: usize> Validate for [T; N] {
    fn validate(&self) -> Result<(), ReadError> {
        for value in self {
            value.validate()?;
        }
        Ok(())
    }
}
/// Fixed records backed by borrowed encoded bytes until an edit requires ownership.
pub type Array<'data, T> = EncodedArray<'data, T, Records>;
impl<T: RecordLayout> Write for Array<'_, T> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(self.as_bytes())
    }
}
impl<T: RecordLayout + Validate> Validate for Array<'_, T>
where
    for<'view> T: Read<'view>,
{
    fn validate(&self) -> Result<(), ReadError> {
        for record in self.iter() {
            record?.validate()?;
        }
        Ok(())
    }
}

impl<'data, T: RecordLayout> Read<'data> for Array<'data, T> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let origin = input.absolute_position();
        let bytes = input.bytes(input.remaining())?;
        Self::decode(bytes).map_err(|mut error| {
            error.offset = origin.saturating_add(error.offset);
            error
        })
    }
}

/// Native record adapter for shared encoded storage.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Records;
impl<T: RecordLayout> RecordCodec<T> for Records {
    type ReadError = ReadError;
    type WriteError = WriteError;
    fn size() -> usize {
        T::size()
    }
    fn read_error(offset: usize, field: &'static str) -> ReadError {
        ReadError::invalid(offset, field)
    }
    fn write_error(field: &'static str) -> WriteError {
        WriteError::invalid(field)
    }
}
impl<T: RecordLayout> DecodeRecord<T> for Records
where
    for<'view> T: Read<'view>,
{
    fn decode(bytes: &[u8], origin: usize) -> Result<T, ReadError> {
        let mut input = Decoder::new(bytes).with_origin(origin);
        let record = T::read(&mut input)?;
        if input.remaining() != 0 {
            return Err(ReadError::invalid(
                input.absolute_position(),
                "fixed-record width",
            ));
        }
        Ok(record)
    }
}
impl<T: RecordLayout + Write> EncodeRecord<T> for Records {
    fn encode(value: &T, output: &mut Output) -> Result<(), WriteError> {
        let mut encoder = Encoder { data: take(output) };
        let result = value.write(&mut encoder);
        *output = encoder.data;
        result
    }
}
