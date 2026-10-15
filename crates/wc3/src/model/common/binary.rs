//! Checked byte access, output storage, and tagged binary framing.
//!
//! [`AddressSpace`] reads explicit offsets; [`Cursor`] adds a sequential position
//! without adding format context. Both borrow the source and report bounds with
//! an absolute diagnostic origin. [`Output`] appends bytes, reserves aligned
//! blocks, and patches existing ranges. Format codecs translate these mechanical
//! failures into their own diagnostics and enforce their native offset limits.
//!
//! The free output helpers perform the same operations on caller-owned buffers.
//! [`Chunks`] iterates tag/length frames without allocating payloads or an index.
//!
//! ```
//! use wc3::model::common::binary::{AddressSpace, Output};
//!
//! let mut output = Output::new();
//! output.append(b"DATA")?;
//! let offset = output.reserve(4, 16)?;
//! output.value(offset, &42u32)?;
//! let bytes = output.finish();
//! assert_eq!(AddressSpace::new(&bytes).value::<u32>(offset)?, 42);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
use std::error::Error;
use std::fmt;

mod storage;
pub use storage::{
    align_position, append_bytes, patch_bytes, reserve_bytes, AddressSpace, Cursor, Output,
    OutputError, ValueError,
};

/// A failed byte-range read, independent of a particular model format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundsError {
    /// Absolute position where the requested read starts.
    pub offset: usize,
    /// Number of bytes requested.
    pub needed: usize,
    /// Number of bytes available at the requested position.
    pub remaining: usize,
}
impl fmt::Display for BoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "needed {} bytes at {}, but only {} remain",
            self.needed, self.offset, self.remaining
        )
    }
}
impl Error for BoundsError {}

/// Reads a bounded range and advances the position only on success.
///
/// `origin` locates the slice within a larger source for diagnostics; offsets
/// passed to this function always address the supplied slice itself.
pub fn read_bytes<'data>(
    data: &'data [u8],
    position: &mut usize,
    count: usize,
    origin: usize,
) -> Result<&'data [u8], BoundsError> {
    let error = BoundsError {
        offset: origin.saturating_add(*position),
        needed: count,
        remaining: data.len().saturating_sub(*position),
    };
    let end = position.checked_add(count).ok_or(error)?;
    let value = data.get(*position..end).ok_or(error)?;
    *position = end;
    Ok(value)
}

/// A four-byte tag followed by a little-endian payload length.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChunkHeader {
    /// Literal tag bytes in wire order.
    pub tag: [u8; 4],
    /// Number of bytes following this header.
    pub size: u32,
}
impl ChunkHeader {
    /// Number of bytes occupied by the tag and length.
    pub const SIZE: usize = 8;
    /// Decodes a header at the supplied position.
    pub fn read(data: &[u8], position: &mut usize, origin: usize) -> Result<Self, BoundsError> {
        let bytes = read_bytes(data, position, Self::SIZE, origin)?;
        let tag = bytes[..4].try_into().expect("four-byte tag");
        let size = u32::from_le_bytes(bytes[4..].try_into().expect("four-byte length"));
        Ok(Self { tag, size })
    }
    /// Produces the exact eight bytes of the header.
    pub fn to_bytes(self) -> [u8; 8] {
        let mut bytes = [0; 8];
        bytes[..4].copy_from_slice(&self.tag);
        bytes[4..].copy_from_slice(&self.size.to_le_bytes());
        bytes
    }
}

/// A framed payload borrowing the original stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Chunk<'data> {
    /// Decoded tag and payload size.
    pub header: ChunkHeader,
    /// Absolute position of the first payload byte.
    pub offset: usize,
    /// Bounded payload bytes.
    pub data: &'data [u8],
}

/// Reads ordered chunks without allocating payloads or a chunk index.
///
/// A malformed final frame yields one error and terminates the iterator.
#[derive(Clone, Debug)]
pub struct Chunks<'data> {
    data: &'data [u8],
    position: usize,
    origin: usize,
    failed: bool,
}
impl<'data> Chunks<'data> {
    /// Creates a stream iterator, associating its first byte with `origin`.
    pub fn new(data: &'data [u8], origin: usize) -> Self {
        Self {
            data,
            position: 0,
            origin,
            failed: false,
        }
    }
}
impl<'data> Iterator for Chunks<'data> {
    type Item = Result<Chunk<'data>, BoundsError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.failed || self.position == self.data.len() {
            return None;
        }
        let result = (|| {
            let header = ChunkHeader::read(self.data, &mut self.position, self.origin)?;
            let offset = self.origin.saturating_add(self.position);
            let data = read_bytes(
                self.data,
                &mut self.position,
                header.size as usize,
                self.origin,
            )?;
            Ok(Chunk {
                header,
                offset,
                data,
            })
        })();
        self.failed = result.is_err();
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{read_bytes, ChunkHeader, Chunks};

    #[test]
    fn failed_ranges_preserve_the_position_and_report_absolute_offsets() {
        let mut position = 1;
        let error = read_bytes(&[1, 2, 3], &mut position, usize::MAX, 20).unwrap_err();
        assert_eq!(position, 1);
        assert_eq!((error.offset, error.remaining), (21, 2));
        assert_eq!(
            read_bytes(&[1, 2, 3], &mut position, 2, 20).unwrap(),
            [2, 3]
        );
    }

    #[test]
    fn chunks_borrow_ordered_payloads_and_terminate_after_truncation() {
        let mut source = ChunkHeader {
            tag: *b"DATA",
            size: 3,
        }
        .to_bytes()
        .to_vec();
        source.extend_from_slice(&[5, 6, 7]);
        source.extend_from_slice(&[1, 2]);
        let mut chunks = Chunks::new(&source, 10);
        let chunk = chunks.next().unwrap().unwrap();
        assert_eq!(chunk.offset, 18);
        assert_eq!(chunk.data.as_ptr(), source[8..].as_ptr());
        assert_eq!(chunk.data, [5, 6, 7]);
        assert_eq!(chunks.next().unwrap().unwrap_err().offset, 21);
        assert!(chunks.next().is_none());
    }
}
