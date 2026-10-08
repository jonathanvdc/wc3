//! Portable metadata paired with a bounded encoded payload stream.
use super::FileFlags;
use std::io::{self, Read, Take};

/// Storage metadata for an encoded file, without archive-relative addresses.
/// Compression masks and sector tables remain part of the payload itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncodedFileMetadata {
    /// Size of the decoded file.
    pub file_size: u32,
    /// Number of encoded payload bytes, excluding archive-level raw chunk digests.
    pub stored_size: u32,
    /// Original block flags describing the encoded representation.
    pub flags: FileFlags,
    /// Source archive sector size in bytes.
    pub sector_size: u32,
    /// Exact locale used for lookup in the destination archive.
    pub locale: u16,
    /// Exact platform used for lookup in the destination archive.
    pub platform: u16,
}

/// An encoded payload stream paired with its storage metadata.
/// Reading never decompresses or decrypts the payload, and stops at `stored_size`.
/// Constructing this value does not validate the payload's contents.
pub struct EncodedEntry<R> {
    metadata: EncodedFileMetadata,
    payload: Take<R>,
}
impl<R: Read> EncodedEntry<R> {
    /// Wraps an encoded stream, bounding reads to the declared stored size.
    /// This also allows importing payloads from a standalone encoded cache.
    pub fn new(metadata: EncodedFileMetadata, source: R) -> Self {
        Self {
            metadata,
            payload: source.take(metadata.stored_size as u64),
        }
    }
    /// Returns metadata describing this stream, independent of its original offset.
    pub fn metadata(&self) -> &EncodedFileMetadata {
        &self.metadata
    }
}
impl<R: Read> Read for EncodedEntry<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.payload.read(output)
    }
}
