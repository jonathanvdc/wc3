//! Sequential MDX writing into a caller-owned byte buffer.
use crate::{Error, Tag};

/// Writes little-endian MDX fields into one growing buffer.
pub struct Encoder<'a> {
    bytes: &'a mut Vec<u8>,
}

/// Position of a size word that will be filled after its record is written.
pub struct SizeMarker(usize);

/// A value with a little-endian MDX representation.
pub trait Writable {
    fn write_to(self, encoder: &mut Encoder<'_>);
}

macro_rules! writable_scalars {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Writable for $ty {
                fn write_to(self, encoder: &mut Encoder<'_>) {
                    encoder.write_bytes(&self.to_le_bytes());
                }
            }
        )*
    };
}
writable_scalars!(u16, u32, i32, f32);

impl<T: Writable + Copy> Writable for &T {
    fn write_to(self, encoder: &mut Encoder<'_>) {
        (*self).write_to(encoder);
    }
}

impl<T: Writable + Copy, const N: usize> Writable for [T; N] {
    fn write_to(self, encoder: &mut Encoder<'_>) {
        for value in self {
            encoder.write(value);
        }
    }
}

impl<T: Writable + Copy> Writable for &[T] {
    fn write_to(self, encoder: &mut Encoder<'_>) {
        for &value in self {
            encoder.write(value);
        }
    }
}

impl<'a> Encoder<'a> {
    /// Wraps a byte buffer and appends to its existing contents.
    pub fn new(bytes: &'a mut Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Returns the current byte offset in the buffer.
    pub fn position(&self) -> usize {
        self.bytes.len()
    }

    /// Appends bytes without interpreting them.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    /// Appends a value in its little-endian MDX representation.
    pub fn write<T: Writable>(&mut self, value: T) {
        value.write_to(self);
    }

    /// Writes a placeholder for a size that includes its own four bytes.
    pub fn begin_sized(&mut self) -> SizeMarker {
        let marker = SizeMarker(self.position());
        self.write(0u32);
        marker
    }

    /// Fills a record size word after its contents have been written.
    pub fn finish_sized(&mut self, marker: SizeMarker, tag: Tag) -> Result<(), Error> {
        self.finish_sized_with_flags(marker, tag, 0)
    }

    /// Fills a chunk size word with the payload length, excluding the word itself.
    pub fn finish_payload(&mut self, marker: SizeMarker, tag: Tag) -> Result<(), Error> {
        let size = self.position() - marker.0 - 4;
        let size = u32::try_from(size).map_err(|_| Error::ChunkTooLarge { tag, size })?;
        self.bytes[marker.0..marker.0 + 4].copy_from_slice(&size.to_le_bytes());
        Ok(())
    }

    /// Fills a record size word whose high bits carry format flags.
    pub fn finish_sized_with_flags(
        &mut self,
        marker: SizeMarker,
        tag: Tag,
        flags: u32,
    ) -> Result<(), Error> {
        let size = self.position() - marker.0;
        let size = u32::try_from(size).map_err(|_| Error::ChunkTooLarge { tag, size })?;
        self.bytes[marker.0..marker.0 + 4].copy_from_slice(&(size | flags).to_le_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Encoder;

    #[test]
    fn writes_arrays_and_slices_in_element_order() {
        let mut bytes = Vec::new();
        let mut encoder = Encoder::new(&mut bytes);
        encoder.write([1u32, 2]);
        encoder.write(&[[1.5f32, -2.5]]);
        encoder.write(&[3u16, 4][..]);
        assert_eq!(
            bytes,
            [1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0xc0, 0x3f, 0, 0, 0x20, 0xc0, 3, 0, 4, 0,]
        );
    }

    #[test]
    fn sized_fields_append_to_existing_bytes() {
        let mut bytes = vec![0xaa];
        let mut encoder = Encoder::new(&mut bytes);
        encoder.write_bytes(b"TEST");
        let chunk = encoder.begin_sized();
        encoder.write(0x1234u16);
        encoder.finish_payload(chunk, *b"TEST").unwrap();
        let record = encoder.begin_sized();
        encoder.write(1.5f32);
        encoder.finish_sized(record, *b"TEST").unwrap();
        assert_eq!(
            bytes,
            [0xaa, b'T', b'E', b'S', b'T', 2, 0, 0, 0, 0x34, 0x12, 8, 0, 0, 0, 0, 0, 0xc0, 0x3f]
        );
    }
}
