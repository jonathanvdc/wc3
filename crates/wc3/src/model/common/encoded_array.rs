//! Borrowed storage and editing mechanics for fixed-width encoded records.
use super::{binary::Output, Ownership};
use std::borrow::Cow;
use std::marker::PhantomData;

/// Format adapter supplying layout and format-qualified diagnostics.
pub trait RecordCodec<T> {
    /// Decoding failure.
    type ReadError;
    /// Encoding failure.
    type WriteError;
    /// Encoded record width.
    fn size() -> usize;
    /// Constructs a decoding diagnostic.
    fn read_error(offset: usize, field: &'static str) -> Self::ReadError;
    /// Constructs an encoding diagnostic.
    fn write_error(field: &'static str) -> Self::WriteError;
}
/// Decodes one complete record, checking that it consumes the supplied bytes.
pub trait DecodeRecord<T>: RecordCodec<T> {
    /// Decodes with an array-relative diagnostic origin.
    fn decode(bytes: &[u8], origin: usize) -> Result<T, Self::ReadError>;
}
/// Encodes one record. Storage checks its width before accepting an edit.
pub trait EncodeRecord<T>: RecordCodec<T> {
    /// Appends the encoded record to shared output storage.
    fn encode(value: &T, output: &mut Output) -> Result<(), Self::WriteError>;
}
/// Fixed records backed by borrowed encoded bytes until an edit requires ownership.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct EncodedArray<'data, T, C> {
    bytes: Cow<'data, [u8]>,
    record: PhantomData<(T, C)>,
}
impl<T, C> Default for EncodedArray<'_, T, C> {
    fn default() -> Self {
        Self {
            bytes: Cow::Borrowed(&[]),
            record: PhantomData,
        }
    }
}
impl<'data, T, C: RecordCodec<T>> EncodedArray<'data, T, C> {
    /// Checks complete record framing and borrows the encoded bytes.
    pub fn decode(bytes: &'data [u8]) -> Result<Self, C::ReadError> {
        let stride = C::size();
        if stride == 0 || !bytes.len().is_multiple_of(stride) {
            return Err(C::read_error(0, "fixed-record array width"));
        }
        Ok(Self {
            bytes: Cow::Borrowed(bytes),
            record: PhantomData,
        })
    }
    /// Number of complete records.
    pub fn len(&self) -> usize {
        if C::size() == 0 {
            0
        } else {
            self.bytes.len() / C::size()
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
    pub fn from_records(records: &[T]) -> Result<Self, C::WriteError>
    where
        C: EncodeRecord<T>,
    {
        let mut output = Output::new();
        for record in records {
            let start = output.position();
            C::encode(record, &mut output)?;
            if output.position().checked_sub(start) != Some(C::size()) {
                return Err(C::write_error("fixed-record width"));
            }
        }
        if C::size() == 0 {
            return Err(C::write_error("zero-width array record"));
        }
        Ok(Self {
            bytes: Cow::Owned(output.finish()),
            record: PhantomData,
        })
    }
    /// Decodes one record on demand without allocating its encoded storage.
    pub fn get(&self, index: usize) -> Option<Result<T, C::ReadError>>
    where
        C: DecodeRecord<T>,
    {
        if index >= self.len() {
            return None;
        }
        let start = index.checked_mul(C::size())?;
        let end = start.checked_add(C::size())?;
        let bytes = self.bytes.get(start..end)?;
        Some(C::decode(bytes, start))
    }
    /// Iterates decoded records in native order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = Result<T, C::ReadError>> + '_
    where
        C: DecodeRecord<T>,
    {
        (0..self.len()).map(|index| self.get(index).unwrap())
    }
    /// Replaces the complete record sequence with independently owned bytes.
    pub fn replace(&mut self, records: &[T]) -> Result<(), C::WriteError>
    where
        C: EncodeRecord<T>,
    {
        *self = Self::from_records(records)?;
        Ok(())
    }
    /// Replaces a record, copying borrowed encoded storage only on the first edit.
    pub fn set(&mut self, index: usize, value: &T) -> Result<(), C::WriteError>
    where
        C: EncodeRecord<T>,
    {
        if index >= self.len() {
            return Err(C::write_error("array record index"));
        }
        let mut output = Output::new();
        C::encode(value, &mut output)?;
        let bytes = output.finish();
        if bytes.len() != C::size() {
            return Err(C::write_error("fixed-record width"));
        }
        let start = index * C::size();
        self.bytes.to_mut()[start..start + C::size()].copy_from_slice(&bytes);
        Ok(())
    }
}

#[cfg(test)]
#[path = "encoded_array_tests.rs"]
mod tests;
