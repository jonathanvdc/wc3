//! Bounded little-endian M3 record codecs and borrowed fixed-record arrays.
use super::dialect::{Envelope, Md34};
use super::{ReadError, WriteError};
use crate::model::common::{binary::read_bytes, Ownership};
use std::array::from_fn;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::mem::size_of;

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
    data: &'data [u8],
    position: usize,
    origin: usize,
    envelope: PhantomData<E>,
}
impl<'data, E: Envelope> Decoder<'data, E> {
    /// Creates a cursor at the start of a payload.
    pub fn new(data: &'data [u8]) -> Self {
        Self {
            data,
            position: 0,
            origin: 0,
            envelope: PhantomData,
        }
    }
    /// Associates payload-relative locations with an absolute source position.
    pub fn with_origin(mut self, origin: usize) -> Self {
        self.origin = origin;
        self
    }
    /// Relative cursor position in bytes.
    pub fn position(&self) -> usize {
        self.position
    }
    /// Number of unread bytes.
    pub fn remaining(&self) -> usize {
        self.data.len() - self.position
    }
    /// Entire bounded payload.
    pub fn data(&self) -> &'data [u8] {
        self.data
    }
    /// Absolute cursor position.
    pub fn absolute_position(&self) -> usize {
        self.origin.saturating_add(self.position)
    }
    /// Reads a bounded byte slice without copying.
    pub fn bytes(&mut self, count: usize) -> Result<&'data [u8], ReadError> {
        read_bytes(self.data, &mut self.position, count, self.origin)
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
    data: Vec<u8>,
    envelope: PhantomData<E>,
}
impl<E: Envelope> Encoder<E> {
    /// Creates an empty payload.
    pub fn new() -> Self {
        Self::default()
    }
    /// Encoded byte count.
    pub fn position(&self) -> usize {
        self.data.len()
    }
    /// Appends bytes, reporting allocation failure as a codec error.
    pub fn bytes(&mut self, bytes: &[u8]) -> Result<(), WriteError> {
        self.data
            .try_reserve(bytes.len())
            .map_err(|_| WriteError::invalid("payload allocation"))?;
        self.data.extend_from_slice(bytes);
        Ok(())
    }
    /// Writes a composable native value.
    pub fn write<T: Write<E> + ?Sized>(&mut self, value: &T) -> Result<(), WriteError> {
        value.write(self)
    }
    /// Returns the complete payload.
    pub fn finish(self) -> Vec<u8> {
        self.data
    }
}
macro_rules! scalars{($($ty:ty),*)=>{$(
 impl<E:Envelope> Read<'_,E> for $ty{fn read(input:&mut Decoder<'_,E>)->Result<Self,ReadError>{Ok(Self::from_le_bytes(input.bytes(size_of::<Self>())?.try_into().unwrap()))}}
 impl<E:Envelope> Write<E> for $ty{fn write(&self,output:&mut Encoder<E>)->Result<(),WriteError>{output.bytes(&self.to_le_bytes())}}
 impl<E:Envelope> RecordLayout<E> for $ty{fn size()->usize{size_of::<Self>()}}
 impl Validate for $ty{fn validate(&self)->Result<(),ReadError>{Ok(())}}
)*};}
scalars!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);
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
        let mut first_error: Option<ReadError> = None;
        let values: [Result<T, ReadError>; N] = from_fn(|_| match &first_error {
            Some(error) => Err(error.clone()),
            None => {
                let result = T::read(input);
                if let Err(error) = &result {
                    first_error = Some(error.clone());
                }
                result
            }
        });
        if let Some(error) = first_error {
            return Err(error);
        }
        Ok(values.map(|value| value.ok().unwrap()))
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
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct Array<'data, T, E: Envelope = Md34> {
    bytes: Cow<'data, [u8]>,
    record: PhantomData<(T, E)>,
}
impl<T, E: Envelope> Default for Array<'_, T, E> {
    fn default() -> Self {
        Self {
            bytes: Cow::Borrowed(&[]),
            record: PhantomData,
        }
    }
}
impl<'data, T: RecordLayout<E>, E: Envelope> Array<'data, T, E> {
    /// Checks fixed-record framing and borrows a section payload.
    pub fn read(bytes: &'data [u8]) -> Result<Self, ReadError> {
        Self::decode(bytes)
    }
    /// Checks complete record framing and borrows the encoded bytes.
    pub fn decode(bytes: &'data [u8]) -> Result<Self, ReadError> {
        let stride = T::size();
        if stride == 0 || !bytes.len().is_multiple_of(stride) {
            return Err(ReadError::invalid(0, "fixed-record array width"));
        }
        Ok(Self {
            bytes: Cow::Borrowed(bytes),
            record: PhantomData,
        })
    }
    /// Number of complete records.
    pub fn len(&self) -> usize {
        if T::size() == 0 {
            0
        } else {
            self.bytes.len() / T::size()
        }
    }
    /// Whether the record array is empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
    /// Encoded array bytes in native order.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Encodes records into independently owned storage.
    pub fn from_records(records: &[T]) -> Result<Self, WriteError>
    where
        T: Write<E>,
    {
        let mut output = Encoder::<E>::new();
        for record in records {
            let start = output.position();
            record.write(&mut output)?;
            if output.position() - start != T::size() {
                return Err(WriteError::invalid("fixed-record width"));
            }
        }
        if T::size() == 0 {
            return Err(WriteError::invalid("zero-width array record"));
        }
        Ok(Self {
            bytes: Cow::Owned(output.finish()),
            record: PhantomData,
        })
    }
    /// Decodes one record on demand without allocating its encoded storage.
    pub fn get(&self, index: usize) -> Option<Result<T, ReadError>>
    where
        for<'view> T: Read<'view, E>,
    {
        if index >= self.len() {
            return None;
        }
        let start = index.checked_mul(T::size())?;
        let end = start.checked_add(T::size())?;
        let bytes = self.bytes.get(start..end)?;
        let mut input = Decoder::<E>::new(bytes).with_origin(start);
        Some(T::read(&mut input).and_then(|record| {
            if input.remaining() != 0 {
                return Err(ReadError::invalid(
                    input.absolute_position(),
                    "fixed-record width",
                ));
            }
            Ok(record)
        }))
    }
    /// Iterates decoded records in native order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = Result<T, ReadError>> + '_
    where
        for<'view> T: Read<'view, E>,
    {
        (0..self.len()).map(|index| self.get(index).unwrap())
    }
    /// Replaces the complete record sequence with independently owned bytes.
    pub fn replace(&mut self, records: &[T]) -> Result<(), WriteError>
    where
        T: Write<E>,
    {
        *self = Self::from_records(records)?;
        Ok(())
    }
    /// Replaces a record, copying borrowed encoded storage only on the first edit.
    pub fn set(&mut self, index: usize, value: &T) -> Result<(), WriteError>
    where
        T: Write<E>,
    {
        if index >= self.len() {
            return Err(WriteError::invalid("array record index"));
        }
        let mut output = Encoder::<E>::new();
        value.write(&mut output)?;
        let bytes = output.finish();
        if bytes.len() != T::size() {
            return Err(WriteError::invalid("fixed-record width"));
        }
        let start = index * T::size();
        self.bytes.to_mut()[start..start + T::size()].copy_from_slice(&bytes);
        Ok(())
    }
}
impl<T: RecordLayout<E>, E: Envelope> Write<E> for Array<'_, T, E> {
    fn write(&self, output: &mut Encoder<E>) -> Result<(), WriteError> {
        output.bytes(&self.bytes)
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
