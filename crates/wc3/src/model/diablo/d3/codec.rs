//! Checked payload-relative descriptors and relocating record encoding.
use super::{ReadError, WriteError};
use crate::model::common::binary::{align_position, AddressSpace, Output, OutputError, ValueError};
use std::{mem::size_of, slice::from_ref};
use zerovec::{
    ule::{AsULE, ULE},
    ZeroVec,
};

/// Fixed byte width of a native record before referenced data.
pub trait RecordLayout {
    /// Number of bytes in the fixed record.
    const SIZE: usize;
}
/// Reads a record in the enclosing payload address space.
pub trait Read<'data>: RecordLayout + Sized {
    /// Decodes a record at a payload-relative byte offset.
    fn read(decoder: &Decoder<'data>, offset: usize) -> Result<Self, ReadError>;
}
/// Writes a record while rebuilding its referenced data.
pub trait Write: RecordLayout {
    /// Writes a record into reserved fixed storage.
    fn write(&self, encoder: &mut Encoder, offset: usize) -> Result<(), WriteError>;
}
/// Checked byte address space shared by every nested descriptor.
#[derive(Clone, Copy, Debug)]
pub struct Decoder<'data> {
    data: AddressSpace<'data>,
}
impl<'data> Decoder<'data> {
    /// Selects the payload beginning after the 16-byte SNO header.
    pub fn new(data: &'data [u8]) -> Self {
        Self {
            data: AddressSpace::new(data),
        }
    }
    /// Returns an exactly bounded range.
    pub fn range(&self, offset: usize, size: usize) -> Result<&'data [u8], ReadError> {
        self.data
            .range(offset, size)
            .map_err(|_| ReadError::Range { offset, size })
    }
    /// Reads an unaligned fixed-width value without native pointer casts.
    pub fn value<T: AsULE>(&self, offset: usize) -> Result<T, ReadError> {
        self.data.value(offset).map_err(|error| match error {
            ValueError::Bounds(_) => ReadError::Range {
                offset,
                size: size_of::<T::ULE>(),
            },
            ValueError::Invalid => ReadError::Value { offset },
        })
    }
    /// Checks a redundant native count against a resolved array.
    pub fn count(&self, at: usize, expected: usize) -> Result<(), ReadError> {
        let actual = self.value::<i32>(at)?;
        if actual < 0 || actual as usize != expected {
            return Err(ReadError::Count {
                offset: at,
                expected,
                actual,
            });
        }
        Ok(())
    }
    fn descriptor(
        &self,
        at: usize,
        stride: usize,
        prefix: usize,
    ) -> Result<(usize, usize), ReadError> {
        let offset: i32 = self.value(at)?;
        let size: i32 = self.value(at + 4)?;
        if size == 0 {
            if offset < 0 {
                return Err(ReadError::Descriptor { offset: at });
            }
            return Ok((0, 0));
        }
        if offset <= 0 || size < 0 {
            return Err(ReadError::Descriptor { offset: at });
        }
        let offset = offset as usize;
        let size = size as usize;
        self.range(offset, size)?;
        if size < prefix || stride == 0 || !(size - prefix).is_multiple_of(stride) {
            return Err(ReadError::Descriptor { offset: at });
        }
        let count = (size - prefix) / stride;
        if prefix == 4 {
            let encoded: i32 = self.value(offset)?;
            if encoded < 0 || encoded as usize != count {
                return Err(ReadError::Count {
                    offset,
                    expected: count,
                    actual: encoded,
                });
            }
        }
        Ok((offset + prefix, count))
    }
    /// Borrows a checked fixed-width leaf array, including optional tag-map count.
    pub fn array<T: AsULE>(
        &self,
        at: usize,
        prefix: usize,
    ) -> Result<ZeroVec<'data, T>, ReadError> {
        let (offset, count) = self.descriptor(at, size_of::<T::ULE>(), prefix)?;
        ZeroVec::parse_bytes(self.range(offset, count * size_of::<T::ULE>())?)
            .map_err(|_| ReadError::Value { offset })
    }
    /// Resolves metadata records whose nested leaves still borrow the source.
    pub fn records<T: Read<'data>>(&self, at: usize, prefix: usize) -> Result<Vec<T>, ReadError> {
        let (offset, count) = self.descriptor(at, T::SIZE, prefix)?;
        (0..count)
            .map(|i| T::read(self, offset + i * T::SIZE))
            .collect()
    }
}
/// Relocating payload builder. Fixed record ranges are reserved before their leaves.
#[derive(Debug)]
pub struct Encoder {
    data: Output,
}
impl Encoder {
    /// Reserves the root's fixed image at offset zero.
    pub fn new(root_size: usize) -> Self {
        Self {
            data: Output::from(vec![0; root_size]),
        }
    }
    /// Finishes the payload.
    pub fn finish(self) -> Vec<u8> {
        self.data.finish()
    }
    /// Stores a fixed-width value in previously reserved space.
    pub fn value<T: AsULE>(&mut self, offset: usize, value: T) -> Result<(), WriteError> {
        self.data
            .value(offset, &value)
            .map_err(|error| match error {
                OutputError::Bounds(_) => WriteError::Range { offset },
                _ => WriteError::Size,
            })
    }
    /// Regenerates a redundant native count from the edited array.
    pub fn count(&mut self, at: usize, count: usize) -> Result<(), WriteError> {
        self.value(at, i32::try_from(count).map_err(|_| WriteError::Size)?)
    }
    fn reserve(&mut self, size: usize) -> Result<usize, WriteError> {
        align_position(self.data.len(), 4)
            .map_err(|_| WriteError::Size)?
            .checked_add(size)
            .filter(|end| *end <= i32::MAX as usize)
            .ok_or(WriteError::Size)?;
        self.data.reserve(size, 4).map_err(|_| WriteError::Size)
    }
    fn descriptor(
        &mut self,
        at: usize,
        count: usize,
        stride: usize,
        prefix: usize,
    ) -> Result<usize, WriteError> {
        if count == 0 {
            self.value(at, 0i32)?;
            self.value(at + 4, 0i32)?;
            return Ok(0);
        }
        let size = count
            .checked_mul(stride)
            .and_then(|n| n.checked_add(prefix))
            .ok_or(WriteError::Size)?;
        let offset = self.reserve(size)?;
        self.value(at, i32::try_from(offset).map_err(|_| WriteError::Size)?)?;
        self.value(at + 4, i32::try_from(size).map_err(|_| WriteError::Size)?)?;
        if prefix == 4 {
            self.value(offset, i32::try_from(count).map_err(|_| WriteError::Size)?)?;
        }
        Ok(offset + prefix)
    }
    /// Copies encoded leaves and regenerates their descriptor.
    pub fn array<T: AsULE>(
        &mut self,
        at: usize,
        values: &ZeroVec<'_, T>,
        prefix: usize,
    ) -> Result<(), WriteError> {
        let offset = self.descriptor(at, values.len(), size_of::<T::ULE>(), prefix)?;
        let bytes = values.as_bytes();
        self.data
            .patch(offset, bytes)
            .map_err(|_| WriteError::Range { offset })?;
        Ok(())
    }
    /// Relocates a metadata array and all leaves reachable from it.
    pub fn records<T: Write>(
        &mut self,
        at: usize,
        values: &[T],
        prefix: usize,
    ) -> Result<(), WriteError> {
        let offset = self.descriptor(at, values.len(), T::SIZE, prefix)?;
        for (i, value) in values.iter().enumerate() {
            value.write(self, offset + i * T::SIZE)?;
        }
        Ok(())
    }
}

/// Copies a fixed unaligned value into an equally sized byte range.
pub(crate) fn copy_value<T: AsULE>(bytes: &mut [u8], value: T) {
    let raw = value.to_unaligned();
    bytes.copy_from_slice(T::ULE::slice_as_bytes(from_ref(&raw)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_rejects_unrepresentable_reservation_without_mutating_storage() {
        let mut encoder = Encoder::new(5);
        assert_eq!(encoder.reserve(i32::MAX as usize), Err(WriteError::Size));
        assert_eq!(encoder.finish(), vec![0; 5]);
    }

    #[test]
    fn checked_scalar_access_retains_native_errors() {
        let decoder = Decoder::new(&[0; 4]);
        assert_eq!(
            decoder.value::<u32>(1),
            Err(ReadError::Range { offset: 1, size: 4 })
        );
        let mut encoder = Encoder::new(4);
        assert_eq!(encoder.value(1, 7u32), Err(WriteError::Range { offset: 1 }));
        assert_eq!(encoder.finish(), vec![0; 4]);
    }
}
