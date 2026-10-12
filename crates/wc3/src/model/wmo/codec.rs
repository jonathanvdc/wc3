//! Bounded little-endian WMO record codecs and borrowed fixed-record arrays.
use super::{Read, ReadError, RecordLayout, Validate, Write, WriteError};
use crate::model::common::{binary::read_bytes, Ownership};
use std::array::from_fn;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::mem::size_of;
/// A cursor within one bounded WMO chunk payload.
#[derive(Clone, Debug)]
pub struct Decoder<'data> {
    data: &'data [u8],
    position: usize,
    origin: usize,
}
impl<'data> Decoder<'data> {
    /// Creates a cursor at the start of a payload.
    pub fn new(data: &'data [u8]) -> Self {
        Self {
            data,
            position: 0,
            origin: 0,
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
    pub fn read<T: Read<'data>>(&mut self) -> Result<T, ReadError> {
        T::read(self)
    }
}
/// An append-only WMO payload encoder.
#[derive(Clone, Debug, Default)]
pub struct Encoder {
    data: Vec<u8>,
}
impl Encoder {
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
    pub fn write<T: Write + ?Sized>(&mut self, value: &T) -> Result<(), WriteError> {
        value.write(self)
    }
    /// Returns the complete payload.
    pub fn finish(self) -> Vec<u8> {
        self.data
    }
}
macro_rules! scalars{($($ty:ty),*)=>{$(
 impl Read<'_> for $ty{fn read(input:&mut Decoder<'_>)->Result<Self,ReadError>{Ok(Self::from_le_bytes(input.bytes(size_of::<Self>())?.try_into().unwrap()))}}
 impl Write for $ty{fn write(&self,output:&mut Encoder)->Result<(),WriteError>{output.bytes(&self.to_le_bytes())}}
 impl RecordLayout for $ty{fn size()->usize{size_of::<Self>()}}
 impl Validate for $ty{fn validate(&self)->Result<(),ReadError>{Ok(())}}
)*};}
scalars!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);
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
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct Array<'data, T> {
    bytes: Cow<'data, [u8]>,
    record: PhantomData<T>,
}
impl<T> Default for Array<'_, T> {
    fn default() -> Self {
        Self {
            bytes: Cow::Borrowed(&[]),
            record: PhantomData,
        }
    }
}
impl<'data, T: RecordLayout> Array<'data, T> {
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
        T: Write,
    {
        let mut output = Encoder::new();
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
        for<'view> T: Read<'view>,
    {
        if index >= self.len() {
            return None;
        }
        let start = index.checked_mul(T::size())?;
        let end = start.checked_add(T::size())?;
        let bytes = self.bytes.get(start..end)?;
        let mut input = Decoder::new(bytes).with_origin(start);
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
        for<'view> T: Read<'view>,
    {
        (0..self.len()).map(|index| self.get(index).unwrap())
    }
    /// Replaces the complete record sequence with independently owned bytes.
    pub fn replace(&mut self, records: &[T]) -> Result<(), WriteError>
    where
        T: Write,
    {
        *self = Self::from_records(records)?;
        Ok(())
    }
    /// Replaces a record, copying borrowed encoded storage only on the first edit.
    pub fn set(&mut self, index: usize, value: &T) -> Result<(), WriteError>
    where
        T: Write,
    {
        if index >= self.len() {
            return Err(WriteError::invalid("array record index"));
        }
        let mut output = Encoder::new();
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
impl<T: RecordLayout> Write for Array<'_, T> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.bytes)
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
