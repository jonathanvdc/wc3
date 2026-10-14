//! Checked access to main-file and external-payload address spaces.
use super::records::Polymorphic;
use super::{Read, ReadError, RecordLayout, Write, WriteError};
use std::array;
use zerovec::{
    ule::{AsULE, ULE},
    ZeroVec,
};

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
    data: &'a [u8],
    payload: Option<&'a [u8]>,
    depth: usize,
}
impl<'a> Decoder<'a> {
    /// Constructs a codec context for its selected address spaces.
    pub fn new(data: &'a [u8], payload: Option<&'a [u8]>) -> Self {
        Self {
            data,
            payload,
            depth: 0,
        }
    }
    /// Returns a checked borrowed byte range.
    pub fn range(&self, at: usize, size: usize) -> Result<&'a [u8], ReadError> {
        let end = at.checked_add(size).ok_or(ReadError::Overflow)?;
        self.data
            .get(at..end)
            .ok_or(ReadError::Truncated { offset: at, size })
    }
    /// Reads or writes one little-endian scalar or fixed scalar array.
    pub fn value<T: AsULE>(&self, at: usize) -> Result<T, ReadError> {
        let size = size_of::<T::ULE>();
        if size == 0 {
            return Err(ReadError::Invalid("scalar encoding"));
        }
        let bytes = self.range(at, size)?;
        let v = T::ULE::parse_bytes_to_slice(bytes)
            .map_err(|_| ReadError::Invalid("scalar encoding"))?;
        Ok(T::from_unaligned(v[0]))
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
            child.data = self.payload.ok_or(ReadError::MissingPayload)?;
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
        let values = (0..N)
            .map(|i| T::read(self, at + i * T::SIZE))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(values.try_into().ok().unwrap())
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
    main: Vec<u8>,
    payload: Vec<u8>,
}
impl Encoder {
    /// Constructs a codec context for its selected address spaces.
    pub fn new(size: usize) -> Self {
        Self {
            main: vec![0; size],
            payload: Vec::new(),
        }
    }
    /// Returns the encoded main body and external payload body, without SNO envelopes.
    pub fn finish(self) -> (Vec<u8>, Vec<u8>) {
        (self.main, self.payload)
    }
    /// Checks that the complete fixed record fits in previously reserved output storage.
    pub fn range(&self, at: usize, size: usize, space: Space) -> Result<(), WriteError> {
        let end = at.checked_add(size).ok_or(WriteError::Overflow)?;
        let buffer = match space {
            Space::Main => &self.main,
            Space::Payload => &self.payload,
        };
        buffer
            .get(at..end)
            .ok_or(WriteError::Invalid("record output range"))?;
        Ok(())
    }
    fn buffer(&mut self, space: Space) -> &mut Vec<u8> {
        match space {
            Space::Main => &mut self.main,
            Space::Payload => &mut self.payload,
        }
    }
    /// Reserves aligned fixed storage in the selected output address space.
    pub fn reserve(&mut self, size: usize, space: Space) -> Result<usize, WriteError> {
        let b = self.buffer(space);
        while !b.len().is_multiple_of(8) {
            b.push(0);
        }
        let at = b.len();
        let end = at.checked_add(size).ok_or(WriteError::Overflow)?;
        i32::try_from(end).map_err(|_| WriteError::Overflow)?;
        b.resize(end, 0);
        Ok(at)
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
        let value = value.to_unaligned();
        let bytes = T::ULE::slice_as_bytes(array::from_ref(&value));
        let end = at.checked_add(bytes.len()).ok_or(WriteError::Overflow)?;
        self.buffer(space)
            .get_mut(at..end)
            .ok_or(WriteError::Invalid("scalar output range"))?
            .copy_from_slice(bytes);
        Ok(())
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
            self.buffer(target)[off..off + bytes.len()].copy_from_slice(bytes);
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
            self.buffer(space)[off..off + bytes.len()].copy_from_slice(bytes);
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
