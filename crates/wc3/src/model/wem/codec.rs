//! Bounded little-endian WEM records and borrowed variable-length array storage.
use super::{ReadError, WriteError};
use crate::model::common::{binary::read_bytes, Borrowed, Detached, Ownership};
use std::array::from_fn;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::mem::size_of;

/// Explicit native machine-word width for layouts that serialize `size_t`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ownership)]
pub enum NativeAbi {
    /// Four-byte native machine words.
    Abi32,
    /// Eight-byte native machine words.
    Abi64,
}

/// Composable decoding within one addressed section.
pub trait Read<'data>: Sized {
    /// Decodes a native value.
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError>;
    /// Advances over a value; derives skip fields without decoding owned lists.
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        Self::read(input).map(|_| ())
    }
}
/// Composable native field encoding.
pub trait Write {
    /// Appends a native value.
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError>;
}
/// Lower bound on bytes occupied by one inline record.
pub trait RecordLayout {
    /// Includes list and presence headers even when their payloads are empty.
    fn minimum_size() -> usize;
    /// Exact width when every field has a constant encoded length.
    fn fixed_size() -> Option<usize> {
        None
    }
}
/// Optional local checks independent of file resolution.
pub trait Validate {
    /// Checks locally available relationships.
    fn validate(&self) -> Result<(), ReadError>;
}
/// A bounded cursor carrying the addressed section revision.
#[derive(Clone, Debug)]
pub struct Decoder<'data> {
    data: &'data [u8],
    position: usize,
    origin: usize,
    /// Native revision for explicitly versioned inline children.
    pub version: u32,
    /// Explicit machine-word interpretation; absent when the source supplies no evidence.
    pub native_abi: Option<NativeAbi>,
}
impl<'data> Decoder<'data> {
    /// Starts a cursor over a bounded payload.
    pub fn new(data: &'data [u8], version: u32) -> Self {
        Self {
            data,
            position: 0,
            origin: 0,
            version,
            native_abi: None,
        }
    }
    /// Associates relative diagnostics with their position in a larger source.
    pub fn with_origin(mut self, origin: usize) -> Self {
        self.origin = origin;
        self
    }
    /// Selects an explicit ABI for native machine-word payloads.
    pub fn with_native_abi(mut self, native_abi: Option<NativeAbi>) -> Self {
        self.native_abi = native_abi;
        self
    }
    /// Relative byte position.
    pub fn position(&self) -> usize {
        self.position
    }
    /// Absolute byte position.
    pub fn absolute_position(&self) -> usize {
        self.origin.saturating_add(self.position)
    }
    /// Unread byte count.
    pub fn remaining(&self) -> usize {
        self.data.len() - self.position
    }
    /// Entire addressed payload.
    pub fn data(&self) -> &'data [u8] {
        self.data
    }
    /// Borrows a checked range and advances on success.
    pub fn bytes(&mut self, count: usize) -> Result<&'data [u8], ReadError> {
        read_bytes(self.data, &mut self.position, count, self.origin)
            .map_err(|e| ReadError::invalid(e.offset, "truncated record"))
    }
    /// Decodes a composable native value.
    pub fn read<T: Read<'data>>(&mut self) -> Result<T, ReadError> {
        T::read(self)
    }
}
/// An append-only native payload encoder.
#[derive(Clone, Debug)]
pub struct Encoder {
    data: Vec<u8>,
    /// Native revision for explicitly versioned inline children.
    pub version: u32,
    /// Explicit machine-word interpretation; absent when the source supplies no evidence.
    pub native_abi: Option<NativeAbi>,
}
impl Encoder {
    /// Creates an empty output payload.
    pub fn new(version: u32) -> Self {
        Self {
            data: Vec::new(),
            version,
            native_abi: None,
        }
    }
    /// Selects an explicit ABI for native machine-word payloads.
    pub fn with_native_abi(mut self, native_abi: Option<NativeAbi>) -> Self {
        self.native_abi = native_abi;
        self
    }
    /// Current encoded byte count.
    pub fn position(&self) -> usize {
        self.data.len()
    }
    /// Appends bytes after checking allocation capacity.
    pub fn bytes(&mut self, bytes: &[u8]) -> Result<(), WriteError> {
        self.data
            .try_reserve(bytes.len())
            .map_err(|_| WriteError::invalid("payload allocation"))?;
        self.data.extend_from_slice(bytes);
        Ok(())
    }
    /// Encodes one composable native value.
    pub fn write<T: Write>(&mut self, value: &T) -> Result<(), WriteError> {
        value.write(self)
    }
    /// Returns the encoded payload.
    pub fn finish(self) -> Vec<u8> {
        self.data
    }
}
macro_rules! scalars {
    ($($ty:ty),*) => {$(
        impl Read<'_> for $ty {
            fn read(input: &mut Decoder<'_>) -> Result<Self, ReadError> {
                Ok(Self::from_le_bytes(input.bytes(size_of::<Self>())?.try_into().unwrap()))
            }
            fn skip(input: &mut Decoder<'_>) -> Result<(), ReadError> { input.bytes(size_of::<Self>())?; Ok(()) }
        }
        impl Write for $ty {
            fn write(&self, output: &mut Encoder) -> Result<(), WriteError> { output.bytes(&self.to_le_bytes()) }
        }
        impl RecordLayout for $ty { fn minimum_size() -> usize { size_of::<Self>() } fn fixed_size() -> Option<usize> { Some(size_of::<Self>()) } }
        impl Validate for $ty { fn validate(&self) -> Result<(), ReadError> { Ok(()) } }
    )*};
}
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
    fn minimum_size() -> usize {
        0
    }
    fn fixed_size() -> Option<usize> {
        Some(0)
    }
}
impl Validate for () {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}
impl<'data, T: Read<'data>, const N: usize> Read<'data> for [T; N] {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let mut failure: Option<ReadError> = None;
        let values: [Result<T, ReadError>; N] = from_fn(|_| {
            if let Some(error) = &failure {
                return Err(error.clone());
            }
            let value = T::read(input);
            if let Err(error) = &value {
                failure = Some(error.clone());
            }
            value
        });
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(values.map(|value| value.ok().unwrap()))
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        for _ in 0..N {
            T::skip(input)?;
        }
        Ok(())
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
    fn minimum_size() -> usize {
        T::minimum_size().saturating_mul(N)
    }
    fn fixed_size() -> Option<usize> {
        T::fixed_size()?.checked_mul(N)
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

/// An encoded boolean that preserves every source byte.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership)]
pub struct Bool(
    /// Zero is false; all nonzero bytes indicate true.
    pub u8,
);
impl Bool {
    /// Interprets the stored byte without changing it.
    pub const fn get(self) -> bool {
        self.0 != 0
    }
    /// Changes the value; keeping true preserves an existing nonzero encoding.
    pub fn set(&mut self, value: bool) {
        if !value {
            self.0 = 0;
        } else if self.0 == 0 {
            self.0 = 1;
        }
    }
}
impl From<bool> for Bool {
    fn from(value: bool) -> Self {
        Self(u8::from(value))
    }
}
impl Read<'_> for Bool {
    fn read(input: &mut Decoder<'_>) -> Result<Self, ReadError> {
        Ok(Self(input.read()?))
    }
    fn skip(input: &mut Decoder<'_>) -> Result<(), ReadError> {
        u8::skip(input)
    }
}
impl Write for Bool {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        self.0.write(output)
    }
}
impl RecordLayout for Bool {
    fn minimum_size() -> usize {
        1
    }
    fn fixed_size() -> Option<usize> {
        Some(1)
    }
}
impl Validate for Bool {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}
impl Read<'_> for bool {
    fn read(input: &mut Decoder<'_>) -> Result<Self, ReadError> {
        let origin = input.absolute_position();
        match input.read::<u8>()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(ReadError::invalid(origin, "canonical boolean")),
        }
    }
}
impl Write for bool {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        u8::from(*self).write(output)
    }
}
impl RecordLayout for bool {
    fn minimum_size() -> usize {
        1
    }
    fn fixed_size() -> Option<usize> {
        Some(1)
    }
}
impl Validate for bool {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}

/// Inline optional payload retaining its original presence discriminator.
#[derive(Clone, Debug, Default, PartialEq, Ownership)]
pub struct InlineOption<T> {
    /// Zero omits the payload; a nonzero byte requires a value.
    pub presence: u8,
    /// Payload interpreted by the selected native record type.
    pub value: Option<T>,
}
impl<T> From<Option<T>> for InlineOption<T> {
    fn from(value: Option<T>) -> Self {
        Self {
            presence: u8::from(value.is_some()),
            value,
        }
    }
}
impl<'data, T: Read<'data>> Read<'data> for InlineOption<T> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let presence = input.read::<u8>()?;
        let value = if presence == 0 {
            None
        } else {
            Some(input.read()?)
        };
        Ok(Self { presence, value })
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        if input.read::<u8>()? != 0 {
            T::skip(input)?;
        }
        Ok(())
    }
}
impl<T: Write> Write for InlineOption<T> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        if (self.presence != 0) != self.value.is_some() {
            return Err(WriteError::invalid("optional presence/value mismatch"));
        }
        self.presence.write(output)?;
        if let Some(value) = &self.value {
            value.write(output)?;
        }
        Ok(())
    }
}
impl<T> RecordLayout for InlineOption<T> {
    fn minimum_size() -> usize {
        1
    }
}
impl<T: Validate> Validate for InlineOption<T> {
    fn validate(&self) -> Result<(), ReadError> {
        if (self.presence != 0) != self.value.is_some() {
            return Err(ReadError::invalid(0, "optional presence/value mismatch"));
        }
        if let Some(value) = &self.value {
            value.validate()?;
        }
        Ok(())
    }
}
impl<'data, T: Read<'data>> Read<'data> for Option<T> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        if input.read::<bool>()? {
            Ok(Some(input.read()?))
        } else {
            Ok(None)
        }
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        if input.read::<bool>()? {
            T::skip(input)?;
        }
        Ok(())
    }
}
impl<T: Write> Write for Option<T> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        self.is_some().write(output)?;
        if let Some(value) = self {
            value.write(output)?;
        }
        Ok(())
    }
}
impl<T> RecordLayout for Option<T> {
    fn minimum_size() -> usize {
        1
    }
}
impl<T: Validate> Validate for Option<T> {
    fn validate(&self) -> Result<(), ReadError> {
        if let Some(value) = self {
            value.validate()?;
        }
        Ok(())
    }
}

fn bounded_count<T: RecordLayout>(input: &Decoder<'_>, count: u32) -> Result<usize, ReadError> {
    let count = count as usize;
    let minimum = T::minimum_size();
    if count != 0 && (minimum == 0 || count > input.remaining() / minimum) {
        return Err(ReadError::invalid(
            input.absolute_position(),
            "inline record count",
        ));
    }
    Ok(count)
}
/// Explicit u32-counted inline rows, decoded only when their containing row is accessed.
///
/// Ordinary section-reference vectors use `Ref`, rather than this inline wrapper.
#[derive(Clone, Debug, Default, PartialEq, Ownership)]
pub struct InlineList<T>(
    /// Decoded rows in their encoded order.
    pub Vec<T>,
);
impl<'data, T: Read<'data> + RecordLayout> Read<'data> for InlineList<T> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let count = input.read::<u32>()?;
        let count = bounded_count::<T>(input, count)?;
        let mut values = Vec::new();
        values
            .try_reserve(count)
            .map_err(|_| ReadError::invalid(input.absolute_position(), "inline list allocation"))?;
        for _ in 0..count {
            values.push(input.read()?);
        }
        Ok(Self(values))
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        let count = input.read::<u32>()?;
        let count = bounded_count::<T>(input, count)?;
        if let Some(stride) = T::fixed_size() {
            let length = count
                .checked_mul(stride)
                .ok_or_else(|| ReadError::invalid(input.absolute_position(), "inline list span"))?;
            input.bytes(length)?;
        } else {
            for _ in 0..count {
                T::skip(input)?;
            }
        }
        Ok(())
    }
}
impl<T: Write> Write for InlineList<T> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        let count =
            u32::try_from(self.0.len()).map_err(|_| WriteError::invalid("inline list count"))?;
        count.write(output)?;
        for value in &self.0 {
            value.write(output)?;
        }
        Ok(())
    }
}
impl<T> RecordLayout for InlineList<T> {
    fn minimum_size() -> usize {
        4
    }
}
impl<T: Validate> Validate for InlineList<T> {
    fn validate(&self) -> Result<(), ReadError> {
        for value in &self.0 {
            value.validate()?;
        }
        Ok(())
    }
}

/// Borrowed variable-width record bytes with a checked row-position index.
#[derive(Clone, Debug, PartialEq)]
pub struct Array<'data, T> {
    bytes: Cow<'data, [u8]>,
    offsets: Cow<'data, [usize]>,
    stride: Option<usize>,
    count: usize,
    version: u32,
    native_abi: Option<NativeAbi>,
    record: PhantomData<T>,
}
impl<T> Default for Array<'_, T> {
    fn default() -> Self {
        Self {
            bytes: Cow::Borrowed(&[]),
            offsets: Cow::Borrowed(&[0]),
            stride: None,
            count: 0,
            version: 0,
            native_abi: None,
            record: PhantomData,
        }
    }
}
impl<'data, T: RecordLayout> Array<'data, T> {
    /// Scans row boundaries without decoding owned inline lists.
    pub fn read(bytes: &'data [u8], count: u32, version: u32) -> Result<Self, ReadError>
    where
        T: Read<'data>,
    {
        Self::read_with_abi(bytes, count, version, None)
    }
    /// Scans rows with an explicitly supplied native ABI.
    pub fn read_with_abi(
        bytes: &'data [u8],
        count: u32,
        version: u32,
        native_abi: Option<NativeAbi>,
    ) -> Result<Self, ReadError>
    where
        T: Read<'data>,
    {
        let mut input = Decoder::new(bytes, version).with_native_abi(native_abi);
        let count = bounded_count::<T>(&input, count)?;
        if let Some(stride) = T::fixed_size() {
            let span = count
                .checked_mul(stride)
                .ok_or_else(|| ReadError::invalid(0, "array byte span"))?;
            if span != bytes.len() {
                return Err(ReadError::invalid(0, "fixed array byte span"));
            }
            return Ok(Self {
                bytes: Cow::Borrowed(bytes),
                offsets: Cow::Borrowed(&[]),
                stride: Some(stride),
                count,
                version,
                native_abi,
                record: PhantomData,
            });
        }
        let mut offsets = Vec::new();
        offsets
            .try_reserve(
                count
                    .checked_add(1)
                    .ok_or_else(|| ReadError::invalid(0, "array count"))?,
            )
            .map_err(|_| ReadError::invalid(0, "array index allocation"))?;
        offsets.push(0);
        for _ in 0..count {
            let start = input.position();
            T::skip(&mut input)?;
            if input.position() <= start {
                return Err(ReadError::invalid(start, "zero-width array record"));
            }
            offsets.push(input.position());
        }
        if input.remaining() != 0 {
            return Err(ReadError::invalid(input.position(), "array trailing bytes"));
        }
        Ok(Self {
            bytes: Cow::Borrowed(bytes),
            offsets: Cow::Owned(offsets),
            stride: None,
            count,
            version,
            native_abi,
            record: PhantomData,
        })
    }
    /// Number of indexed rows.
    pub fn len(&self) -> usize {
        self.count
    }
    /// Whether no rows are present.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Original or edited encoded rows.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Native section revision used to decode nested inline payloads.
    pub fn version(&self) -> u32 {
        self.version
    }
    fn row_span(&self, index: usize) -> Option<(usize, usize)> {
        if index >= self.count {
            return None;
        }
        match self.stride {
            Some(stride) => {
                let start = index.checked_mul(stride)?;
                Some((start, start.checked_add(stride)?))
            }
            None => Some((*self.offsets.get(index)?, *self.offsets.get(index + 1)?)),
        }
    }
    /// Decodes one row, allocating only its explicitly inline lists.
    pub fn get(&self, index: usize) -> Option<Result<T, ReadError>>
    where
        for<'view> T: Read<'view>,
    {
        let (start, end) = self.row_span(index)?;
        let mut input = Decoder::new(&self.bytes[start..end], self.version)
            .with_origin(start)
            .with_native_abi(self.native_abi);
        Some(T::read(&mut input).and_then(|value| {
            if input.remaining() != 0 {
                return Err(ReadError::invalid(
                    input.absolute_position(),
                    "record trailing bytes",
                ));
            }
            Ok(value)
        }))
    }
    /// Decodes rows in native order on demand.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = Result<T, ReadError>> + '_
    where
        for<'view> T: Read<'view>,
    {
        (0..self.len()).map(|index| self.get(index).unwrap())
    }
    /// Encodes an independent array while indexing actual row lengths.
    pub fn from_records(values: &[T], version: u32) -> Result<Self, WriteError>
    where
        T: Write,
    {
        Self::from_records_with_abi(values, version, None)
    }
    /// Encodes rows with an explicitly supplied native ABI.
    pub fn from_records_with_abi(
        values: &[T],
        version: u32,
        native_abi: Option<NativeAbi>,
    ) -> Result<Self, WriteError>
    where
        T: Write,
    {
        let mut output = Encoder::new(version).with_native_abi(native_abi);
        let stride = T::fixed_size();
        let mut offsets = Vec::new();
        if stride.is_none() {
            offsets
                .try_reserve(
                    values
                        .len()
                        .checked_add(1)
                        .ok_or_else(|| WriteError::invalid("array count"))?,
                )
                .map_err(|_| WriteError::invalid("array index allocation"))?;
            offsets.push(0);
        }
        for value in values {
            let start = output.position();
            value.write(&mut output)?;
            if output.position() - start < T::minimum_size() || output.position() == start {
                return Err(WriteError::invalid("array record minimum size"));
            }
            if let Some(stride) = stride {
                if output.position() - start != stride {
                    return Err(WriteError::invalid("fixed array record width"));
                }
            } else {
                offsets.push(output.position());
            }
        }
        Ok(Self {
            bytes: Cow::Owned(output.finish()),
            offsets: Cow::Owned(offsets),
            stride,
            count: values.len(),
            version,
            native_abi,
            record: PhantomData,
        })
    }
    /// Replaces all rows, updating the position index for resized inline lists.
    pub fn replace(&mut self, values: &[T]) -> Result<(), WriteError>
    where
        T: Write,
    {
        *self = Self::from_records_with_abi(values, self.version, self.native_abi)?;
        Ok(())
    }
    /// Replaces a row without decoding its neighbors, allowing its byte width to change.
    pub fn set(&mut self, index: usize, value: &T) -> Result<(), WriteError>
    where
        T: Write,
    {
        let (start, end) = self
            .row_span(index)
            .ok_or_else(|| WriteError::invalid("array record index"))?;
        let mut output = Encoder::new(self.version).with_native_abi(self.native_abi);
        value.write(&mut output)?;
        let replacement = output.finish();
        if replacement.len() < T::minimum_size() || replacement.is_empty() {
            return Err(WriteError::invalid("array record minimum size"));
        }
        if let Some(stride) = self.stride {
            if replacement.len() != stride {
                return Err(WriteError::invalid("fixed array record width"));
            }
            self.bytes.to_mut()[start..end].copy_from_slice(&replacement);
            return Ok(());
        }
        let new_size = self
            .bytes
            .len()
            .checked_sub(end - start)
            .and_then(|size| size.checked_add(replacement.len()))
            .ok_or_else(|| WriteError::invalid("array byte size"))?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve(new_size)
            .map_err(|_| WriteError::invalid("array allocation"))?;
        bytes.extend_from_slice(&self.bytes[..start]);
        bytes.extend_from_slice(&replacement);
        bytes.extend_from_slice(&self.bytes[end..]);
        let mut offsets = self.offsets.to_vec();
        for offset in &mut offsets[index + 1..] {
            *offset = *offset - (end - start) + replacement.len();
        }
        self.bytes = Cow::Owned(bytes);
        self.offsets = Cow::Owned(offsets);
        Ok(())
    }
}
impl<T: RecordLayout + Validate> Validate for Array<'_, T>
where
    for<'view> T: Read<'view>,
{
    fn validate(&self) -> Result<(), ReadError> {
        for row in self.iter() {
            row?.validate()?;
        }
        Ok(())
    }
}

impl<'view, T> Borrowed<'view> for Array<'_, T> {
    type View = Array<'view, T>;
    fn as_borrowed(&'view self) -> Self::View {
        Array {
            bytes: Cow::Borrowed(&self.bytes),
            offsets: Cow::Borrowed(&self.offsets),
            stride: self.stride,
            count: self.count,
            version: self.version,
            native_abi: self.native_abi,
            record: PhantomData,
        }
    }
}
impl<'owned, T> Detached<'owned> for Array<'_, T> {
    type Detached = Array<'owned, T>;
    fn into_detached(self) -> Self::Detached {
        Array {
            bytes: Cow::Owned(self.bytes.into_owned()),
            offsets: Cow::Owned(self.offsets.into_owned()),
            stride: self.stride,
            count: self.count,
            version: self.version,
            native_abi: self.native_abi,
            record: PhantomData,
        }
    }
}
impl<'data, T: Read<'data>> Read<'data> for Box<T> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        Ok(Box::new(input.read()?))
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), ReadError> {
        T::skip(input)
    }
}
impl<T: Write> Write for Box<T> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        (**self).write(output)
    }
}
impl<T: RecordLayout> RecordLayout for Box<T> {
    fn minimum_size() -> usize {
        T::minimum_size()
    }
    fn fixed_size() -> Option<usize> {
        T::fixed_size()
    }
}
impl<T: Validate> Validate for Box<T> {
    fn validate(&self) -> Result<(), ReadError> {
        (**self).validate()
    }
}

#[cfg(test)]
mod tests {
    use super::{Array, InlineOption};
    use crate::model::common::Borrowed as _;

    #[test]
    fn fixed_arrays_need_no_per_row_boundaries() {
        let bytes = [7, 8, 9];
        let array = Array::<u8>::read(&bytes, 3, 0).unwrap();
        assert!(array.offsets.is_empty());
        assert_eq!(array.stride, Some(1));
        assert_eq!(array.get(2).unwrap().unwrap(), 9);
        assert_eq!(array.as_bytes().as_ptr(), bytes.as_ptr());
    }

    #[test]
    fn variable_array_views_borrow_the_boundary_index() {
        let bytes = [0, 7, 1, 0, 0, 0];
        let array = Array::<InlineOption<u32>>::read(&bytes, 2, 0).unwrap();
        let view = array.as_borrowed();
        assert_eq!(view.offsets.as_ptr(), array.offsets.as_ptr());
        assert_eq!(view.as_bytes().as_ptr(), array.as_bytes().as_ptr());
        assert_eq!(view.get(1).unwrap().unwrap().presence, 7);
    }
}
