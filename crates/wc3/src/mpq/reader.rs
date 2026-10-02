use std::io::{self, Read, Seek, SeekFrom};

use super::codec::{decode, sector_checksum};
use super::crypto::{crypt, file_key};
use super::format::{
    offset_table_size, read_blocks, read_hashes, sector_count, u32_at, validate_name, HEADER_SIZE,
};
use super::{BlockEntry, Error, FileFlags, Header, Index};

/// Allocation and discovery bounds for untrusted archives.
#[derive(Clone, Debug)]
pub struct ReadOptions {
    pub max_table_entries: u32,
    pub max_sector_size: u32,
    pub max_sector_table_bytes: u32,
    pub max_file_size: u32,
    pub max_single_unit_bytes: u32,
    /// Maximum distance searched in 512-byte steps from the initial cursor.
    pub max_header_search_bytes: u64,
    /// Verify nonzero/non-sentinel checksums when an entry supplies them.
    pub verify_sector_checksums: bool,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            max_table_entries: 2 << 20,
            max_sector_size: 1 << 20,
            max_sector_table_bytes: 16 << 20,
            max_file_size: u32::MAX,
            max_single_unit_bytes: 64 << 20,
            max_header_search_bytes: 16 << 20,
            verify_sector_checksums: true,
        }
    }
}

/// A seekable MPQ source with an eagerly decoded index and lazy payloads.
pub struct Archive<R> {
    pub(super) source: R,
    pub(super) base: u64,
    pub(super) index: Index,
    pub(super) options: ReadOptions,
}

impl<R: Read + Seek> Archive<R> {
    pub fn open(source: R) -> Result<Self, Error> {
        Self::with_options(source, ReadOptions::default())
    }

    /// Discovers an MPQ header, including a user-data header or map prefix.
    /// Search begins at the source's current cursor, in 512-byte steps.
    pub fn with_options(mut source: R, options: ReadOptions) -> Result<Self, Error> {
        let start = source.stream_position()?;
        let len = source.seek(SeekFrom::End(0))?;
        let last = start
            .saturating_add(options.max_header_search_bytes)
            .min(len.saturating_sub(4));
        let mut at = start;
        while at <= last && at.checked_add(4).is_some_and(|end| end <= len) {
            source.seek(SeekFrom::Start(at))?;
            let mut magic = [0; 4];
            source.read_exact(&mut magic)?;
            if &magic == b"MPQ\x1a" {
                return Self::at(source, at, options);
            }
            if &magic == b"MPQ\x1b" {
                let mut data = [0; 12];
                source.read_exact(&mut data)?;
                let relative = u32_at(&data, 4) as u64;
                if relative < 16 {
                    return Err(Error::InvalidArchive("user-data header offset"));
                }
                let base = at
                    .checked_add(relative)
                    .ok_or(Error::InvalidArchive("header offset overflow"))?;
                return Self::at(source, base, options);
            }
            at = match at.checked_add(512) {
                Some(next) => next,
                None => break,
            };
        }
        Err(Error::InvalidArchive(
            "MPQ header not found within search limit",
        ))
    }

    /// Reads an MPQ at an explicit absolute offset, without scanning.
    pub fn at(mut source: R, base: u64, options: ReadOptions) -> Result<Self, Error> {
        let len = source.seek(SeekFrom::End(0))?;
        if base
            .checked_add(HEADER_SIZE as u64)
            .is_none_or(|end| end > len)
        {
            return Err(Error::InvalidArchive("header outside source"));
        }
        source.seek(SeekFrom::Start(base))?;
        let mut bytes = [0; 32];
        source.read_exact(&mut bytes)?;
        let header = Header::decode(&bytes)?;
        let size = header.archive_size as u64;
        if size < HEADER_SIZE as u64 || base.checked_add(size).is_none_or(|end| end > len) {
            return Err(Error::InvalidArchive("archive size outside source"));
        }
        let sector_size = header
            .sector_size()
            .ok_or(Error::InvalidArchive("sector size overflow"))?;
        if sector_size > options.max_sector_size {
            return Err(Error::LimitExceeded("sector size"));
        }
        if !header.hash_table_entries.is_power_of_two() {
            return Err(Error::InvalidArchive(
                "hash table size is not a power of two",
            ));
        }
        for (offset, count) in [
            (header.hash_table_offset, header.hash_table_entries),
            (header.block_table_offset, header.block_table_entries),
        ] {
            if count > options.max_table_entries {
                return Err(Error::LimitExceeded("table entries"));
            }
            if count != 0 && (offset < HEADER_SIZE || offset as u64 + count as u64 * 16 > size) {
                return Err(Error::InvalidArchive("index table outside archive"));
            }
        }
        let hashes = read_hashes(
            &mut source,
            base + header.hash_table_offset as u64,
            header.hash_table_entries,
        )?;
        let blocks = read_blocks(
            &mut source,
            base + header.block_table_offset as u64,
            header.block_table_entries,
        )?;
        for entry in &hashes {
            if entry.is_file()
                && (entry.block_index as usize >= blocks.len()
                    || !blocks[entry.block_index as usize]
                        .flags
                        .contains(FileFlags::EXISTS))
            {
                return Err(Error::InvalidArchive("hash references missing block"));
            }
        }
        for block in &blocks {
            let empty_at_zero = block.offset == 0 && block.file_size == 0 && block.stored_size == 0;
            if block.flags.contains(FileFlags::EXISTS)
                && ((block.offset < HEADER_SIZE && !empty_at_zero)
                    || block.offset as u64 + block.stored_size as u64 > size)
            {
                return Err(Error::InvalidArchive("file block outside archive"));
            }
        }
        Ok(Self {
            source,
            base,
            index: Index {
                header,
                hashes,
                blocks,
            },
            options,
        })
    }

    pub fn index(&self) -> &Index {
        &self.index
    }
    pub fn archive_offset(&self) -> u64 {
        self.base
    }
    pub fn into_inner(self) -> R {
        self.source
    }

    /// Opens a neutral-locale, platform-zero entry.
    pub fn open_file(&mut self, name: impl AsRef<[u8]>) -> Result<EntryReader<'_, R>, Error> {
        self.open_file_with_locale(name, 0, 0)
    }

    /// Opens an exact locale/platform match. Names use ASCII case-insensitive
    /// hashing and treat `/` and `\\` as equivalent; arbitrary non-NUL bytes work.
    pub fn open_file_with_locale(
        &mut self,
        name: impl AsRef<[u8]>,
        locale: u16,
        platform: u16,
    ) -> Result<EntryReader<'_, R>, Error> {
        let name = name.as_ref();
        validate_name(name)?;
        let id = self
            .index
            .find(name, locale, platform)
            .ok_or(Error::FileNotFound)?;
        let block = self.index.blocks[id as usize];
        block.flags.validate_codec()?;
        if block.file_size > self.options.max_file_size {
            return Err(Error::LimitExceeded("file size"));
        }
        let sector_size = self.index.header.sector_size().unwrap();
        if block.flags.contains(FileFlags::SINGLE_UNIT)
            && block.file_size.max(block.stored_size) > self.options.max_single_unit_bytes
        {
            return Err(Error::LimitExceeded("single-unit buffer"));
        }
        if !block.flags.compressed() && block.file_size != block.stored_size {
            return Err(Error::InvalidArchive("stored file size mismatch"));
        }
        let key = block.flags.contains(FileFlags::ENCRYPTED).then(|| {
            file_key(
                name,
                block.offset,
                block.file_size,
                block.flags.contains(FileFlags::FIX_KEY),
            )
        });
        let offsets = load_offsets(
            &mut self.source,
            self.base,
            block,
            sector_size,
            key,
            &self.options,
        )?;
        let checksums = if self.options.verify_sector_checksums
            && block.flags.contains(FileFlags::SECTOR_CRC)
            && !offsets.is_empty()
        {
            let count = sector_count(block.file_size, sector_size) as usize;
            let start = offsets[count];
            let stored = offsets[count + 1] - start;
            if stored == 0 {
                Vec::new()
            } else {
                let expected = count * 4;
                if stored as usize > expected {
                    return Err(Error::InvalidArchive("checksum table length"));
                }
                let mut bytes = vec![0; stored as usize];
                self.source.seek(SeekFrom::Start(
                    self.base + block.offset as u64 + start as u64,
                ))?;
                self.source.read_exact(&mut bytes)?;
                if bytes.len() < expected {
                    bytes = decode(&bytes, expected, false)?;
                }
                bytes.chunks_exact(4).map(|b| u32_at(b, 0)).collect()
            }
        } else {
            Vec::new()
        };
        Ok(EntryReader {
            source: &mut self.source,
            base: self.base,
            block,
            sector_size,
            key,
            offsets,
            checksums,
            sector: 0,
            delivered: 0,
            buffer: Vec::new(),
            cursor: 0,
            failed: false,
        })
    }

    /// Convenience extraction. Prefer `open_file` for bounded payload memory.
    pub fn read_file(&mut self, name: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        let mut reader = self.open_file(name)?;
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    /// Returns names supplied by `(listfile)`, without asserting completeness.
    /// Names remain bytes rather than imposing UTF-8 on old game archives.
    pub fn known_names(&mut self) -> Result<Vec<Vec<u8>>, Error> {
        let mut entry = match self.open_file(b"(listfile)") {
            Ok(entry) => entry,
            Err(Error::FileNotFound) => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };
        // Bound listfile allocation separately from streaming ordinary payloads.
        if entry.block.file_size > 16 << 20 {
            return Err(Error::LimitExceeded("listfile size"));
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        Ok(bytes
            .split(|b| matches!(b, b'\r' | b'\n'))
            .filter(|line| !line.is_empty())
            .map(<[u8]>::to_vec)
            .collect())
    }

    /// Reads encoded bytes by block index, including entries with unknown names
    /// or unsupported codecs. This never decompresses or decrypts the payload.
    pub fn encoded_file(&mut self, block_index: u32) -> Result<impl Read + '_, Error> {
        let block = *self
            .index
            .blocks
            .get(block_index as usize)
            .filter(|b| b.flags.contains(FileFlags::EXISTS))
            .ok_or(Error::FileNotFound)?;
        self.source
            .seek(SeekFrom::Start(self.base + block.offset as u64))?;
        Ok((&mut self.source).take(block.stored_size as u64))
    }
}

#[cfg(test)]
#[path = "reader_tests.rs"]
mod tests;

pub(super) fn load_offsets(
    source: &mut (impl Read + Seek),
    base: u64,
    block: BlockEntry,
    sector_size: u32,
    key: Option<u32>,
    options: &ReadOptions,
) -> Result<Vec<u32>, Error> {
    if !block.flags.compressed()
        || block.flags.contains(FileFlags::SINGLE_UNIT)
        || block.file_size == 0
    {
        return Ok(Vec::new());
    }
    let size = offset_table_size(
        block.file_size,
        sector_size,
        block.flags.contains(FileFlags::SECTOR_CRC),
    )?;
    if size > options.max_sector_table_bytes {
        return Err(Error::LimitExceeded("sector offset table"));
    }
    if size > block.stored_size {
        return Err(Error::InvalidArchive("truncated sector table"));
    }
    let mut bytes = vec![0; size as usize];
    source.seek(SeekFrom::Start(base + block.offset as u64))?;
    source.read_exact(&mut bytes)?;
    if let Some(key) = key {
        crypt(&mut bytes, key.wrapping_sub(1), true);
    }
    let offsets: Vec<u32> = bytes.chunks_exact(4).map(|b| u32_at(b, 0)).collect();
    if offsets[0] < size
        || offsets.last().is_none_or(|&n| n > block.stored_size)
        || offsets.windows(2).any(|pair| pair[0] > pair[1])
    {
        return Err(Error::InvalidArchive("invalid sector offsets"));
    }
    let count = sector_count(block.file_size, sector_size) as usize;
    for (i, pair) in offsets.windows(2).take(count).enumerate() {
        let expected =
            (block.file_size as u64 - i as u64 * sector_size as u64).min(sector_size as u64);
        let stored = pair[1] - pair[0];
        if stored == 0 || stored as u64 > expected {
            return Err(Error::InvalidArchive("invalid sector length"));
        }
    }
    Ok(offsets)
}

/// A sequential entry stream. The archive is exclusively borrowed until drop.
/// Payload buffers are bounded by the sector size (or a limited single-unit file).
pub struct EntryReader<'a, R> {
    source: &'a mut R,
    base: u64,
    block: BlockEntry,
    sector_size: u32,
    key: Option<u32>,
    offsets: Vec<u32>,
    checksums: Vec<u32>,
    sector: u32,
    delivered: u32,
    buffer: Vec<u8>,
    cursor: usize,
    failed: bool,
}

impl<R: Read + Seek> EntryReader<'_, R> {
    pub fn metadata(&self) -> &BlockEntry {
        &self.block
    }

    fn fill(&mut self) -> Result<(), Error> {
        let single = self.block.flags.contains(FileFlags::SINGLE_UNIT);
        let expected = if single {
            self.block.file_size
        } else {
            (self.block.file_size - self.delivered).min(self.sector_size)
        };
        let (offset, size) = if single {
            (0, self.block.stored_size)
        } else if self.offsets.is_empty() {
            (self.sector * self.sector_size, expected)
        } else {
            let at = self.sector as usize;
            (self.offsets[at], self.offsets[at + 1] - self.offsets[at])
        };
        if size > expected {
            return Err(Error::InvalidArchive("encoded unit exceeds decoded size"));
        }
        self.buffer.resize(size as usize, 0);
        self.source.seek(SeekFrom::Start(
            self.base + self.block.offset as u64 + offset as u64,
        ))?;
        self.source.read_exact(&mut self.buffer)?;
        if let Some(key) = self.key {
            crypt(
                &mut self.buffer,
                key.wrapping_add(if single { 0 } else { self.sector }),
                true,
            );
        }
        if let Some(&expected) = self.checksums.get(self.sector as usize) {
            if expected != 0 && expected != u32::MAX && sector_checksum(&self.buffer) != expected {
                return Err(Error::ChecksumMismatch(self.sector));
            }
        }
        if size < expected {
            if !self.block.flags.compressed() {
                return Err(Error::InvalidArchive("short stored sector"));
            }
            self.buffer = decode(
                &self.buffer,
                expected as usize,
                self.block.flags.contains(FileFlags::IMPLODE),
            )?;
        }
        self.cursor = 0;
        self.sector += 1;
        Ok(())
    }
}

impl<R: Read + Seek> Read for EntryReader<'_, R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(Error::InvalidArchive("entry reader previously failed").into());
        }
        if self.delivered == self.block.file_size {
            return Ok(0);
        }
        if self.cursor == self.buffer.len() {
            if let Err(e) = self.fill() {
                self.failed = true;
                return Err(e.into());
            }
        }
        let len = output.len().min(self.buffer.len() - self.cursor);
        output[..len].copy_from_slice(&self.buffer[self.cursor..self.cursor + len]);
        self.cursor += len;
        self.delivered += len as u32;
        Ok(len)
    }
}
