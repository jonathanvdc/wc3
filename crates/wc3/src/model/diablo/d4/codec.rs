//! Checked access to main-file and external-payload address spaces.
use super::records::Polymorphic;
use super::{Read, ReadError, RecordLayout, Write, WriteError};
use crate::model::common::binary::{align_position, AddressSpace, Output, OutputError, ValueError};
use crate::model::common::try_array;
use zerovec::{ule::AsULE, ZeroVec};

/// Native output address space. Offsets exclude the 16-byte SNO envelope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Space {
    /// Main SNO body following its envelope.
    Main,
    /// Separately stored external payload body.
    Payload,
}
/// Checked reader over one SNO body, with an optional external payload body.
#[derive(Clone, Copy, Debug)]
pub struct Decoder<'a> {
    data: AddressSpace<'a>,
    payload: Option<&'a [u8]>,
    depth: usize,
}
impl<'a> Decoder<'a> {
    /// Constructs a codec context for its selected address spaces.
    pub fn new(data: &'a [u8], payload: Option<&'a [u8]>) -> Self {
        Self {
            data: AddressSpace::new(data),
            payload,
            depth: 0,
        }
    }
    /// Returns a checked borrowed byte range.
    pub fn range(&self, at: usize, size: usize) -> Result<&'a [u8], ReadError> {
        at.checked_add(size).ok_or(ReadError::Overflow)?;
        self.data
            .range(at, size)
            .map_err(|_| ReadError::Truncated { offset: at, size })
    }
    /// Reads or writes one little-endian scalar or fixed scalar array.
    pub fn value<T: AsULE>(&self, at: usize) -> Result<T, ReadError> {
        at.checked_add(size_of::<T::ULE>())
            .ok_or(ReadError::Overflow)?;
        self.data.value(at).map_err(|error| match error {
            ValueError::Bounds(_) => ReadError::Truncated {
                offset: at,
                size: size_of::<T::ULE>(),
            },
            ValueError::Invalid => ReadError::Invalid("scalar encoding"),
        })
    }
    fn descriptor(&self, at: usize, external: bool) -> Result<(Self, usize, usize), ReadError> {
        if self.value::<u64>(at)? != 0 {
            return Err(ReadError::Invalid("nonzero array runtime pointer"));
        }
        let off: i32 = self.value(at + 8)?;
        let size: i32 = self.value(at + 12)?;
        if off < 0 || size < 0 {
            return Err(ReadError::Invalid("negative array descriptor"));
        }
        let mut child = *self;
        child.depth += 1;
        if child.depth > 128 {
            return Err(ReadError::Invalid("array nesting limit"));
        }
        if external && size != 0 {
            child.data = AddressSpace::new(self.payload.ok_or(ReadError::MissingPayload)?);
        }
        let off = off as usize;
        let size = size as usize;
        child.range(off, size)?;
        Ok((child, off, size))
    }
    /// Resolves or relocates a descriptor for an encoded leaf array.
    pub fn array<T: AsULE>(&self, at: usize, external: bool) -> Result<ZeroVec<'a, T>, ReadError> {
        if size_of::<T::ULE>() == 0 {
            return Err(ReadError::Invalid("array element encoding"));
        }
        let (child, off, size) = self.descriptor(at, external)?;
        ZeroVec::parse_bytes(child.range(off, size)?)
            .map_err(|_| ReadError::Invalid("array element encoding"))
    }
    /// Resolves or relocates a descriptor for complete fixed-width records.
    pub fn records<T: Read<'a>>(&self, at: usize, external: bool) -> Result<Vec<T>, ReadError> {
        let (child, off, size) = self.descriptor(at, external)?;
        if T::SIZE == 0 || size % T::SIZE != 0 {
            return Err(ReadError::Invalid("record array size"));
        }
        (0..size / T::SIZE)
            .map(|i| T::read(&child, off + i * T::SIZE))
            .collect()
    }
    /// Reads or writes an inline fixed-size record array.
    pub fn fixed<T: Read<'a>, const N: usize>(&self, at: usize) -> Result<[T; N], ReadError> {
        let size = T::SIZE.checked_mul(N).ok_or(ReadError::Overflow)?;
        self.range(at, size)?;
        let mut index = 0;
        try_array(|| {
            let value = T::read(self, at + index * T::SIZE)?;
            index += 1;
            Ok(value)
        })
    }
    /// Resolves or relocates a native byte-string descriptor.
    pub fn bytes(&self, at: usize) -> Result<&'a [u8], ReadError> {
        let (c, o, n) = self.descriptor(at, false)?;
        c.range(o, n)
    }
    /// Resolves or relocates supported concrete polymorphic record bodies.
    pub fn polymorphic(
        &self,
        at: usize,
        external: bool,
        base: u32,
    ) -> Result<Vec<Polymorphic<'a>>, ReadError> {
        let (c, o, n) = self.descriptor(at, external)?;
        let count: i32 = self.value(at + 16)?;
        if count < 0 {
            return Err(ReadError::Invalid("negative polymorphic count"));
        }
        let table = (count as usize).checked_mul(8).ok_or(ReadError::Overflow)?;
        if table > n {
            return Err(ReadError::Invalid("polymorphic table size"));
        }
        if c.range(o, table)?.iter().any(|&byte| byte != 0) {
            return Err(ReadError::Invalid(
                "unsupported nonzero polymorphic prefix table",
            ));
        }
        let mut cursor = o + table;
        let end = o + n;
        let mut result = Vec::new();
        for _ in 0..count {
            let v = Polymorphic::read(&c, cursor, base)?;
            cursor = cursor.checked_add(v.size()).ok_or(ReadError::Overflow)?;
            if cursor > end {
                return Err(ReadError::Invalid("polymorphic record size"));
            }
            result.push(v);
        }
        if cursor != end {
            return Err(ReadError::Invalid("polymorphic trailing bytes"));
        }
        Ok(result)
    }
}
/// Relocating writer with independent main-file and external-payload buffers.
#[derive(Clone, Debug, Default)]
pub struct Encoder {
    main: Output,
    payload: Output,
}
impl Encoder {
    /// Constructs a codec context for its selected address spaces.
    pub fn new(size: usize) -> Self {
        Self {
            main: Output::from(vec![0; size]),
            payload: Output::new(),
        }
    }
    /// Returns the encoded main body and external payload body, without SNO envelopes.
    pub fn finish(self) -> (Vec<u8>, Vec<u8>) {
        (self.main.finish(), self.payload.finish())
    }
    /// Checks that the complete fixed record fits in previously reserved output storage.
    pub fn range(&self, at: usize, size: usize, space: Space) -> Result<(), WriteError> {
        let end = at.checked_add(size).ok_or(WriteError::Overflow)?;
        let buffer = match space {
            Space::Main => &self.main,
            Space::Payload => &self.payload,
        };
        buffer
            .range(at, end - at)
            .map_err(|_| WriteError::Invalid("record output range"))?;
        Ok(())
    }
    fn buffer(&mut self, space: Space) -> &mut Output {
        match space {
            Space::Main => &mut self.main,
            Space::Payload => &mut self.payload,
        }
    }
    /// Reserves aligned fixed storage in the selected output address space.
    pub fn reserve(&mut self, size: usize, space: Space) -> Result<usize, WriteError> {
        let b = self.buffer(space);
        let end = align_position(b.len(), 8)
            .map_err(|_| WriteError::Overflow)?
            .checked_add(size)
            .ok_or(WriteError::Overflow)?;
        i32::try_from(end).map_err(|_| WriteError::Overflow)?;
        b.reserve(size, 8).map_err(|_| WriteError::Overflow)
    }
    /// Reads or writes one little-endian scalar or fixed scalar array.
    pub fn value<T: AsULE>(
        &mut self,
        at: usize,
        value: &T,
        space: Space,
    ) -> Result<(), WriteError> {
        if size_of::<T::ULE>() == 0 {
            return Err(WriteError::Invalid("scalar encoding"));
        }
        self.buffer(space)
            .value(at, value)
            .map_err(|error| match error {
                OutputError::Bounds(_) => WriteError::Invalid("scalar output range"),
                _ => WriteError::Overflow,
            })
    }
    fn descriptor(
        &mut self,
        at: usize,
        off: usize,
        size: usize,
        space: Space,
    ) -> Result<(), WriteError> {
        self.value(at, &0u64, space)?;
        self.value(
            at + 8,
            &i32::try_from(off).map_err(|_| WriteError::Overflow)?,
            space,
        )?;
        self.value(
            at + 12,
            &i32::try_from(size).map_err(|_| WriteError::Overflow)?,
            space,
        )
    }
    /// Resolves or relocates a descriptor for an encoded leaf array.
    pub fn array<T: AsULE>(
        &mut self,
        at: usize,
        values: &ZeroVec<'_, T>,
        space: Space,
        external: bool,
    ) -> Result<(), WriteError> {
        if size_of::<T::ULE>() == 0 {
            return Err(WriteError::Invalid("array element encoding"));
        }
        let target = if external { Space::Payload } else { space };
        let bytes = values.as_bytes();
        let off = if bytes.is_empty() {
            0
        } else {
            self.reserve(bytes.len(), target)?
        };
        if !bytes.is_empty() {
            self.buffer(target)
                .patch(off, bytes)
                .map_err(|_| WriteError::Invalid("array output range"))?;
        }
        self.descriptor(at, off, bytes.len(), space)
    }
    /// Resolves or relocates a descriptor for complete fixed-width records.
    pub fn records<T: Write + RecordLayout>(
        &mut self,
        at: usize,
        values: &[T],
        space: Space,
        external: bool,
    ) -> Result<(), WriteError> {
        if T::SIZE == 0 {
            return Err(WriteError::Invalid("record array size"));
        }
        let target = if external { Space::Payload } else { space };
        let size = values
            .len()
            .checked_mul(T::SIZE)
            .ok_or(WriteError::Overflow)?;
        let off = if size == 0 {
            0
        } else {
            self.reserve(size, target)?
        };
        self.descriptor(at, off, size, space)?;
        for (i, value) in values.iter().enumerate() {
            value.write(self, off + i * T::SIZE, target)?;
        }
        Ok(())
    }
    /// Reads or writes an inline fixed-size record array.
    pub fn fixed<T: Write + RecordLayout, const N: usize>(
        &mut self,
        at: usize,
        values: &[T; N],
        space: Space,
    ) -> Result<(), WriteError> {
        let size = T::SIZE.checked_mul(N).ok_or(WriteError::Overflow)?;
        self.range(at, size, space)?;
        for (i, value) in values.iter().enumerate() {
            value.write(self, at + i * T::SIZE, space)?;
        }
        Ok(())
    }
    /// Resolves or relocates a native byte-string descriptor.
    pub fn bytes(&mut self, at: usize, bytes: &[u8], space: Space) -> Result<(), WriteError> {
        let off = if bytes.is_empty() {
            0
        } else {
            self.reserve(bytes.len(), space)?
        };
        if !bytes.is_empty() {
            self.buffer(space)
                .patch(off, bytes)
                .map_err(|_| WriteError::Invalid("byte output range"))?;
        }
        self.descriptor(at, off, bytes.len(), space)
    }
    /// Resolves or relocates supported concrete polymorphic record bodies.
    pub fn polymorphic(
        &mut self,
        at: usize,
        values: &[Polymorphic<'_>],
        space: Space,
        external: bool,
        base: u32,
    ) -> Result<(), WriteError> {
        if values
            .iter()
            .any(|v| !Polymorphic::supports_hash(base, v.type_hash()))
        {
            return Err(WriteError::Invalid("polymorphic base family"));
        }
        let target = if external { Space::Payload } else { space };
        let table = values.len().checked_mul(8).ok_or(WriteError::Overflow)?;
        let size = values.iter().try_fold(table, |n, v| {
            n.checked_add(v.size()).ok_or(WriteError::Overflow)
        })?;
        let off = if size == 0 {
            0
        } else {
            self.reserve(size, target)?
        };
        self.descriptor(at, off, size, space)?;
        self.value(
            at + 16,
            &i32::try_from(values.len()).map_err(|_| WriteError::Overflow)?,
            space,
        )?;
        self.value(at + 20, &0u32, space)?;
        let mut cursor = off + table;
        for v in values {
            v.write(self, cursor, target)?;
            cursor += v.size();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_rejects_unrepresentable_reservation_without_padding_storage() {
        let mut encoder = Encoder::new(5);
        assert_eq!(
            encoder.reserve(i32::MAX as usize, Space::Main),
            Err(WriteError::Overflow)
        );
        assert_eq!(encoder.finish(), (vec![0; 5], vec![]));
    }

    #[test]
    fn address_space_errors_retain_native_distinctions() {
        let decoder = Decoder::new(&[0; 4], None);
        assert_eq!(decoder.range(usize::MAX, 4), Err(ReadError::Overflow));
        assert_eq!(
            decoder.value::<u32>(1),
            Err(ReadError::Truncated { offset: 1, size: 4 })
        );
        let mut encoder = Encoder::new(4);
        assert_eq!(
            encoder.value(1, &7u32, Space::Main),
            Err(WriteError::Invalid("scalar output range"))
        );
        assert_eq!(encoder.finish(), (vec![0; 4], vec![]));
    }
}
