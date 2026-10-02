use std::io::{Read, Seek, SeekFrom, Write};

use super::crypto::{crypt, hash};
use super::Error;

pub(super) const HEADER_SIZE: u32 = 32;
pub(super) const EMPTY: u32 = u32::MAX;
pub(super) const DELETED: u32 = u32::MAX - 1;

/// Raw MPQ block flags, retained even when the entry codec is unsupported.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FileFlags(pub u32);

impl FileFlags {
    pub const IMPLODE: u32 = 0x100;
    pub const COMPRESS: u32 = 0x200;
    pub const ENCRYPTED: u32 = 0x10000;
    pub const FIX_KEY: u32 = 0x20000;
    pub const SINGLE_UNIT: u32 = 0x1000000;
    pub const SECTOR_CRC: u32 = 0x4000000;
    pub const EXISTS: u32 = 0x80000000;

    pub fn contains(self, bits: u32) -> bool {
        self.0 & bits == bits
    }
    pub(super) fn compressed(self) -> bool {
        self.0 & (Self::COMPRESS | Self::IMPLODE) != 0
    }
    pub(super) fn validate_codec(self) -> Result<(), Error> {
        let supported = Self::EXISTS
            | Self::IMPLODE
            | Self::COMPRESS
            | Self::ENCRYPTED
            | Self::FIX_KEY
            | Self::SINGLE_UNIT
            | Self::SECTOR_CRC;
        if self.0 & !supported != 0
            || self.contains(Self::COMPRESS | Self::IMPLODE)
            || (self.contains(Self::FIX_KEY) && !self.contains(Self::ENCRYPTED))
            || (self.contains(Self::SECTOR_CRC)
                && (!self.compressed() || self.contains(Self::SINGLE_UNIT)))
        {
            return Err(Error::UnsupportedFlags(self.0));
        }
        Ok(())
    }
}

/// Classic MPQ (header version zero) container layout.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Header {
    pub archive_size: u32,
    /// Sector size is `512 << sector_size_shift`.
    pub sector_size_shift: u16,
    pub hash_table_offset: u32,
    pub block_table_offset: u32,
    pub hash_table_entries: u32,
    pub block_table_entries: u32,
}

impl Header {
    pub(super) fn decode(bytes: &[u8; 32]) -> Result<Self, Error> {
        if &bytes[..4] != b"MPQ\x1a" {
            return Err(Error::InvalidArchive("missing header"));
        }
        let version = u16_at(bytes, 12);
        if version != 0 {
            return Err(Error::UnsupportedVersion(version));
        }
        if u32_at(bytes, 4) != HEADER_SIZE {
            return Err(Error::InvalidArchive("header size"));
        }
        Ok(Self {
            archive_size: u32_at(bytes, 8),
            sector_size_shift: u16_at(bytes, 14),
            hash_table_offset: u32_at(bytes, 16),
            block_table_offset: u32_at(bytes, 20),
            hash_table_entries: u32_at(bytes, 24),
            block_table_entries: u32_at(bytes, 28),
        })
    }
    pub(super) fn write(&self, output: &mut impl Write) -> Result<(), Error> {
        output.write_all(b"MPQ\x1a")?;
        output.write_all(&HEADER_SIZE.to_le_bytes())?;
        output.write_all(&self.archive_size.to_le_bytes())?;
        output.write_all(&0u16.to_le_bytes())?;
        output.write_all(&self.sector_size_shift.to_le_bytes())?;
        for word in [
            self.hash_table_offset,
            self.block_table_offset,
            self.hash_table_entries,
            self.block_table_entries,
        ] {
            output.write_all(&word.to_le_bytes())?;
        }
        Ok(())
    }
    pub fn sector_size(&self) -> Option<u32> {
        1u32.checked_shl(9 + u32::from(self.sector_size_shift))
    }
}

/// An on-disk filename hash and locale record. Filenames are not stored here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HashEntry {
    pub name_hash_a: u32,
    pub name_hash_b: u32,
    pub locale: u16,
    pub platform: u16,
    /// `u32::MAX` is empty and `u32::MAX - 1` is deleted.
    pub block_index: u32,
}

impl HashEntry {
    pub(super) fn empty() -> Self {
        Self {
            name_hash_a: EMPTY,
            name_hash_b: EMPTY,
            locale: u16::MAX,
            platform: u16::MAX,
            block_index: EMPTY,
        }
    }
    pub fn is_file(&self) -> bool {
        self.block_index < DELETED
    }
    pub(super) fn bytes(self) -> [u8; 16] {
        let mut bytes = [0; 16];
        bytes[0..4].copy_from_slice(&self.name_hash_a.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.name_hash_b.to_le_bytes());
        bytes[8..10].copy_from_slice(&self.locale.to_le_bytes());
        bytes[10..12].copy_from_slice(&self.platform.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.block_index.to_le_bytes());
        bytes
    }
}

/// Encoded storage metadata, independent of entry decompression.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockEntry {
    pub offset: u32,
    pub stored_size: u32,
    pub file_size: u32,
    pub flags: FileFlags,
}

impl BlockEntry {
    pub(super) fn bytes(self) -> [u8; 16] {
        let mut bytes = [0; 16];
        for (out, word) in bytes.chunks_exact_mut(4).zip([
            self.offset,
            self.stored_size,
            self.file_size,
            self.flags.0,
        ]) {
            out.copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }
}

/// Decoded container tables. Reading these does not decode file payloads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Index {
    pub header: Header,
    pub hashes: Vec<HashEntry>,
    pub blocks: Vec<BlockEntry>,
}

impl Index {
    /// Looks up an exact locale/platform; no implicit locale fallback.
    pub fn find(&self, name: impl AsRef<[u8]>, locale: u16, platform: u16) -> Option<u32> {
        let name = name.as_ref();
        if self.hashes.is_empty() {
            return None;
        }
        let start = hash(name, 0) as usize & (self.hashes.len() - 1);
        let a = hash(name, 1);
        let b = hash(name, 2);
        for step in 0..self.hashes.len() {
            let entry = self.hashes[(start + step) & (self.hashes.len() - 1)];
            if entry.block_index == EMPTY {
                break;
            }
            if entry.is_file()
                && entry.name_hash_a == a
                && entry.name_hash_b == b
                && entry.locale == locale
                && entry.platform == platform
            {
                return Some(entry.block_index);
            }
        }
        None
    }
}

pub(super) fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap())
}

pub(super) fn read_hashes(
    input: &mut (impl Read + Seek),
    offset: u64,
    count: u32,
) -> Result<Vec<HashEntry>, Error> {
    let bytes = read_table(input, offset, count, b"(hash table)")?;
    Ok(bytes
        .chunks_exact(16)
        .map(|b| HashEntry {
            name_hash_a: u32_at(b, 0),
            name_hash_b: u32_at(b, 4),
            locale: u16_at(b, 8),
            platform: u16_at(b, 10),
            block_index: u32_at(b, 12),
        })
        .collect())
}

pub(super) fn read_blocks(
    input: &mut (impl Read + Seek),
    offset: u64,
    count: u32,
) -> Result<Vec<BlockEntry>, Error> {
    let bytes = read_table(input, offset, count, b"(block table)")?;
    Ok(bytes
        .chunks_exact(16)
        .map(|b| BlockEntry {
            offset: u32_at(b, 0),
            stored_size: u32_at(b, 4),
            file_size: u32_at(b, 8),
            flags: FileFlags(u32_at(b, 12)),
        })
        .collect())
}

fn read_table(
    input: &mut (impl Read + Seek),
    offset: u64,
    count: u32,
    name: &[u8],
) -> Result<Vec<u8>, Error> {
    let len = (count as usize)
        .checked_mul(16)
        .ok_or(Error::LimitExceeded("table size"))?;
    let mut bytes = vec![0; len];
    input.seek(SeekFrom::Start(offset))?;
    input.read_exact(&mut bytes)?;
    crypt(&mut bytes, hash(name, 3), true);
    Ok(bytes)
}

pub(super) fn write_table(
    output: &mut impl Write,
    records: impl Iterator<Item = [u8; 16]>,
    name: &[u8],
) -> Result<(), Error> {
    let mut bytes: Vec<u8> = records.flatten().collect();
    crypt(&mut bytes, hash(name, 3), false);
    output.write_all(&bytes)?;
    Ok(())
}

pub(super) fn validate_name(name: &[u8]) -> Result<(), Error> {
    if name.is_empty() || name.iter().any(|b| matches!(b, 0 | b'\r' | b'\n')) {
        Err(Error::InvalidName)
    } else {
        Ok(())
    }
}

pub(super) fn sector_count(size: u32, sector_size: u32) -> u32 {
    size.div_ceil(sector_size)
}

pub(super) fn offset_table_size(
    size: u32,
    sector_size: u32,
    checksums: bool,
) -> Result<u32, Error> {
    sector_count(size, sector_size)
        .checked_add(if checksums { 2 } else { 1 })
        .and_then(|n| n.checked_mul(4))
        .ok_or(Error::LimitExceeded("sector table size"))
}

#[cfg(test)]
#[path = "format_tests.rs"]
mod tests;
