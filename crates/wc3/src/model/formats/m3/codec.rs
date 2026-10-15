//! Bounded little-endian M3 record codecs and borrowed fixed-record arrays.
use super::dialect::{Envelope, Md34};
use super::{ReadError, WriteError};
use crate::model::common::{
    binary::{Cursor, Output},
    DecodeRecord, EncodeRecord, EncodedArray, RecordCodec,
};
use crate::model::common::{record_primitives, try_array};
use std::marker::PhantomData;
use std::mem::{size_of, take};

/// A value decoded from one bounded section payload under an envelope layout.
pub trait Read<'data, E: Envelope = Md34>: Sized {
    /// Reads native fields and inline section references.
    fn read(input: &mut Decoder<'data, E>) -> Result<Self, ReadError>;
}
/// A value encoded under an envelope's reference layout.
pub trait Write<E: Envelope = Md34> {
    /// Appends native fields and inline section references.
    fn write(&self, output: &mut Encoder<E>) -> Result<(), WriteError>;
}
/// Fixed inline record width under the selected reference envelope.
pub trait RecordLayout<E: Envelope = Md34> {
    /// Returns the encoded byte width.
    fn size() -> usize;
}
/// Optional semantic checks, separate from bounded decoding.
pub trait Validate {
    /// Checks locally available relationships.
    fn validate(&self) -> Result<(), ReadError>;
}

/// A cursor within one bounded M3 chunk payload.
#[derive(Clone, Debug)]
pub struct Decoder<'data, E: Envelope = Md34> {
    cursor: Cursor<'data>,
    envelope: PhantomData<E>,
}
impl<'data, E: Envelope> Decoder<'data, E> {
    /// Creates a cursor at the start of a payload.
    pub fn new(data: &'data [u8]) -> Self {
        Self {
            cursor: Cursor::new(data),
            envelope: PhantomData,
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
    pub fn read<T: Read<'data, E>>(&mut self) -> Result<T, ReadError> {
        T::read(self)
    }
}
/// An append-only M3 payload encoder.
#[derive(Clone, Debug, Default)]
pub struct Encoder<E: Envelope = Md34> {
    data: Output,
    envelope: PhantomData<E>,
}
impl<E: Envelope> Encoder<E> {
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
    pub fn write<T: Write<E> + ?Sized>(&mut self, value: &T) -> Result<(), WriteError> {
        value.write(self)
    }
    /// Returns the complete payload.
    pub fn finish(self) -> Vec<u8> {
        self.data.finish()
    }
}
record_primitives!([E: Envelope], [E], ['_, E], Decoder<'_, E>, Encoder<E>; u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);
impl<E: Envelope> Read<'_, E> for () {
    fn read(_: &mut Decoder<'_, E>) -> Result<Self, ReadError> {
        Ok(())
    }
}
impl<E: Envelope> Write<E> for () {
    fn write(&self, _: &mut Encoder<E>) -> Result<(), WriteError> {
        Ok(())
    }
}
impl<E: Envelope> RecordLayout<E> for () {
    fn size() -> usize {
        0
    }
}
impl Validate for () {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}
impl<'data, E: Envelope, T: Read<'data, E>, const N: usize> Read<'data, E> for [T; N] {
    fn read(input: &mut Decoder<'data, E>) -> Result<Self, ReadError> {
        try_array(|| T::read(input))
    }
}
impl<E: Envelope, T: Write<E>, const N: usize> Write<E> for [T; N] {
    fn write(&self, output: &mut Encoder<E>) -> Result<(), WriteError> {
        for value in self {
            value.write(output)?;
        }
        Ok(())
    }
}
impl<E: Envelope, T: RecordLayout<E>, const N: usize> RecordLayout<E> for [T; N] {
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
pub type Array<'data, T, E = Md34> = EncodedArray<'data, T, Records<E>>;
impl<'data, T: RecordLayout<E>, E: Envelope> Array<'data, T, E> {
    /// Checks fixed-record framing and borrows a section payload.
    pub fn read(bytes: &'data [u8]) -> Result<Self, ReadError> {
        Self::decode(bytes)
    }
}

impl<T: RecordLayout<E>, E: Envelope> Write<E> for Array<'_, T, E> {
    fn write(&self, output: &mut Encoder<E>) -> Result<(), WriteError> {
        output.bytes(self.as_bytes())
    }
}
impl<T: RecordLayout<E> + Validate, E: Envelope> Validate for Array<'_, T, E>
where
    for<'view> T: Read<'view, E>,
{
    fn validate(&self) -> Result<(), ReadError> {
        for record in self.iter() {
            record?.validate()?;
        }
        Ok(())
    }
}

impl<'data, T: RecordLayout<E>, E: Envelope> Read<'data, E> for Array<'data, T, E> {
    fn read(input: &mut Decoder<'data, E>) -> Result<Self, ReadError> {
        let origin = input.absolute_position();
        let bytes = input.bytes(input.remaining())?;
        Self::decode(bytes).map_err(|mut error| {
            error.offset = origin.saturating_add(error.offset);
            error
        })
    }
}

impl<'data, T, E: Envelope> Read<'data, E> for PhantomData<T> {
    fn read(_: &mut Decoder<'data, E>) -> Result<Self, ReadError> {
        Ok(PhantomData)
    }
}
impl<T, E: Envelope> Write<E> for PhantomData<T> {
    fn write(&self, _: &mut Encoder<E>) -> Result<(), WriteError> {
        Ok(())
    }
}
impl<T, E: Envelope> RecordLayout<E> for PhantomData<T> {
    fn size() -> usize {
        0
    }
}
impl<T> Validate for PhantomData<T> {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}

/// Native record adapter for shared encoded storage.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Records<E: Envelope>(PhantomData<E>);
impl<E: Envelope, T: RecordLayout<E>> RecordCodec<T> for Records<E> {
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
impl<E: Envelope, T: RecordLayout<E>> DecodeRecord<T> for Records<E>
where
    for<'view> T: Read<'view, E>,
{
    fn decode(bytes: &[u8], origin: usize) -> Result<T, ReadError> {
        let mut input = Decoder::<E>::new(bytes).with_origin(origin);
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
impl<E: Envelope, T: RecordLayout<E> + Write<E>> EncodeRecord<T> for Records<E> {
    fn encode(value: &T, output: &mut Output) -> Result<(), WriteError> {
        let mut encoder = Encoder {
            data: take(output),
            envelope: PhantomData,
        };
        let result = value.write(&mut encoder);
        *output = encoder.data;
        result
    }
}
