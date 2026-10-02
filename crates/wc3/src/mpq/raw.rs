//! Raw-data chunk digests, separate from decoded sector checksums.
use std::io::{Read, Seek, SeekFrom, Write};

use md5::compute;

use super::{Error, Header, ReadOptions};

pub(super) fn verify(
    source: &mut (impl Read + Seek),
    base: u64,
    offset: u64,
    size: u64,
    header: &Header,
    options: &ReadOptions,
) -> Result<(), Error> {
    let chunk = header.raw_chunk_size as u64;
    if chunk == 0 || size == 0 {
        return Ok(());
    }
    if chunk > options.max_table_bytes {
        return Err(Error::LimitExceeded("raw chunk size"));
    }
    let count = size.div_ceil(chunk);
    let digest_offset = offset
        .checked_add(size)
        .ok_or(Error::InvalidArchive("raw chunk range"))?;
    if count
        .checked_mul(16)
        .and_then(|n| digest_offset.checked_add(n))
        .is_none_or(|end| end > header.archive_size)
    {
        return Err(Error::InvalidArchive("raw chunk digests outside archive"));
    }
    let mut buffer = vec![0; chunk.min(size) as usize];
    for i in 0..count {
        let size = chunk.min(size - i * chunk) as usize;
        source.seek(SeekFrom::Start(base + offset + i * chunk))?;
        source.read_exact(&mut buffer[..size])?;
        let mut digest = [0; 16];
        source.seek(SeekFrom::Start(base + digest_offset + i * 16))?;
        source.read_exact(&mut digest)?;
        if compute(&buffer[..size]).0 != digest {
            return Err(Error::InvalidArchive("raw chunk MD5 mismatch"));
        }
    }
    Ok(())
}

/// Retain chunks overlapping a sector table until its offsets are patched.
/// Other chunks are hashed while writing, keeping payload memory bounded.
pub(super) struct Digests {
    chunk: usize,
    prefix_capacity: usize,
    prefix: Vec<u8>,
    pending: Vec<u8>,
    digests: Vec<[u8; 16]>,
}

impl Digests {
    pub(super) fn new(chunk: u32, table_size: u32) -> Self {
        let chunk = chunk as usize;
        let prefix_capacity = (table_size as usize).div_ceil(chunk) * chunk;
        Self {
            chunk,
            prefix_capacity,
            prefix: vec![0; table_size as usize],
            pending: Vec::new(),
            digests: Vec::new(),
        }
    }

    pub(super) fn update(&mut self, mut bytes: &[u8]) {
        let n = (self.prefix_capacity - self.prefix.len()).min(bytes.len());
        self.prefix.extend_from_slice(&bytes[..n]);
        bytes = &bytes[n..];
        while !bytes.is_empty() {
            let n = (self.chunk - self.pending.len()).min(bytes.len());
            self.pending.extend_from_slice(&bytes[..n]);
            bytes = &bytes[n..];
            if self.pending.len() == self.chunk {
                self.digests.push(compute(&self.pending).0);
                self.pending.clear();
            }
        }
    }

    pub(super) fn finish(mut self, table: &[u8], output: &mut impl Write) -> Result<(), Error> {
        self.prefix[..table.len()].copy_from_slice(table);
        for chunk in self.prefix.chunks(self.chunk) {
            output.write_all(&compute(chunk).0)?;
        }
        for digest in &self.digests {
            output.write_all(digest)?;
        }
        if !self.pending.is_empty() {
            output.write_all(&compute(&self.pending).0)?;
        }
        Ok(())
    }
}

pub(super) fn write(bytes: &[u8], chunk: u32, output: &mut impl Write) -> Result<(), Error> {
    if chunk != 0 {
        for bytes in bytes.chunks(chunk as usize) {
            output.write_all(&compute(bytes).0)?;
        }
    }
    Ok(())
}
