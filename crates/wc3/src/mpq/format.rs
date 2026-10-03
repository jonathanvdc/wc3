use std::io::{Read, Seek, SeekFrom, Write};

use md5::compute;

use super::codec::decode;
use super::crypto::{crypt, hash};
use super::extended::ExtendedIndex;
use super::Error;

pub(super) const HEADER_SIZE: u32 = 32;
pub(super) const EMPTY: u32 = u32::MAX;
pub(super) const DELETED: u32 = u32::MAX - 1;

/// Raw MPQ block flags, retained even when the entry codec is unsupported.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FileFlags(
    /// Unmodified MPQ block flag bitmask, including any unknown bits.
    pub u32,
);

impl FileFlags {
    /// Entry sectors use PKWARE DCL compression.
    pub const IMPLODE: u32 = 0x100;
    /// Entry sectors use compression masks to select their codecs.
    pub const COMPRESS: u32 = 0x200;
    /// Entry payloads are encrypted using a filename-derived key.
    pub const ENCRYPTED: u32 = 0x10000;
    /// Encryption keys incorporate the entry offset and decoded size.
    pub const FIX_KEY: u32 = 0x20000;
    /// The payload is stored as one unit instead of sector framing.
    pub const SINGLE_UNIT: u32 = 0x1000000;
    /// The payload includes a sector checksum block.
    pub const SECTOR_CRC: u32 = 0x4000000;
    /// The block represents an existing file.
    pub const EXISTS: u32 = 0x80000000;

    /// Returns whether every bit in `bits` is set.
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

/// MPQ header, with offsets normalized to 64 bits.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Header {
    /// On-disk version: 0 (classic), 1, 2, or 3.
    pub version: u16,
    /// Original shortened v3 header size; zero selects the canonical size.
    pub shortened_header_size: u32,
    /// Declared archive region size in bytes.
    pub archive_size: u64,
    /// Sector size exponent: a sector contains `512 << sector_size_shift` bytes.
    pub sector_size_shift: u16,
    /// Classic hash table byte offset relative to the archive header.
    pub hash_table_offset: u64,
    /// Classic block table byte offset relative to the archive header.
    pub block_table_offset: u64,
    /// Number of slots in the classic hash table, including empty and deleted slots.
    pub hash_table_entries: u32,
    /// Number of records in the block table.
    pub block_table_entries: u32,
    /// High block-offset table byte offset relative to the archive header; zero if absent.
    pub hi_block_table_offset: u64,
    /// BET byte offset relative to the archive header; zero if absent.
    pub bet_table_offset: u64,
    /// HET byte offset relative to the archive header; zero if absent.
    pub het_table_offset: u64,
    /// Stored table sizes in bytes, ordered as hash, block, high-block, HET, BET.
    pub table_sizes: [u64; 5],
    /// Raw encoded chunk size in bytes for MD5 verification; zero disables chunk digests.
    pub raw_chunk_size: u32,
    /// Digests: block, hash, high-block, BET, HET, header.
    pub md5: [[u8; 16]; 6],
}

impl Header {
    /// Returns the header size in bytes, honoring a shortened v3 header.
    /// Returns zero for an unsupported version.
    pub fn header_size(&self) -> u32 {
        if self.version == 2 && self.shortened_header_size != 0 {
            return self.shortened_header_size;
        }
        [32, 44, 68, 208]
            .get(self.version as usize)
            .copied()
            .unwrap_or(0)
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 32 || &bytes[..4] != b"MPQ\x1a" {
            return Err(Error::InvalidArchive("missing header"));
        }
        let version = u16_at(bytes, 12);
        if version > 3 {
            return Err(Error::UnsupportedVersion(version));
        }
        let size = u32_at(bytes, 4) as usize;
        let valid = match version {
            0 => size == 32,
            1 => size == 44,
            2 => (44..=68).contains(&size),
            3 => size == 208,
            _ => false,
        };
        if !valid || bytes.len() < size {
            return Err(Error::InvalidArchive("header size"));
        }
        let mut h = Self {
            version,
            shortened_header_size: if version == 2 && size < 68 {
                size as u32
            } else {
                0
            },
            archive_size: u32_at(bytes, 8) as u64,
            sector_size_shift: u16_at(bytes, 14),
            hash_table_offset: u32_at(bytes, 16) as u64,
            block_table_offset: u32_at(bytes, 20) as u64,
            hash_table_entries: u32_at(bytes, 24),
            block_table_entries: u32_at(bytes, 28),
            ..Self::default()
        };
        if version >= 1 {
            h.hi_block_table_offset = u64_at(bytes, 32);
            h.hash_table_offset |= (u16_at(bytes, 40) as u64) << 32;
            h.block_table_offset |= (u16_at(bytes, 42) as u64) << 32;
        }
        if version >= 2 && size >= 68 {
            h.archive_size = u64_at(bytes, 44);
            h.bet_table_offset = u64_at(bytes, 52);
            h.het_table_offset = u64_at(bytes, 60);
        }
        if version == 3 {
            for (i, n) in h.table_sizes.iter_mut().enumerate() {
                *n = u64_at(bytes, 68 + i * 8);
            }
            h.raw_chunk_size = u32_at(bytes, 108);
            for (i, digest) in h.md5.iter_mut().enumerate() {
                digest.copy_from_slice(&bytes[112 + i * 16..128 + i * 16]);
            }
            verify_md5(&bytes[..192], &h.md5[5])?;
        }
        Ok(h)
    }
    pub(super) fn write(&self, output: &mut impl Write) -> Result<(), Error> {
        if self.version > 3 {
            return Err(Error::UnsupportedVersion(self.version));
        }
        if self.hash_table_offset >> 48 != 0
            || self.block_table_offset >> 48 != 0
            || (self.version == 0
                && (self.archive_size > u32::MAX as u64
                    || self.hash_table_offset > u32::MAX as u64
                    || self.block_table_offset > u32::MAX as u64))
        {
            return Err(Error::LimitExceeded("archive offsets"));
        }
        let mut b = Vec::new();
        b.extend_from_slice(b"MPQ\x1a");
        b.extend_from_slice(&self.header_size().to_le_bytes());
        b.extend_from_slice(&(self.archive_size as u32).to_le_bytes());
        b.extend_from_slice(&self.version.to_le_bytes());
        b.extend_from_slice(&self.sector_size_shift.to_le_bytes());
        for n in [
            self.hash_table_offset as u32,
            self.block_table_offset as u32,
            self.hash_table_entries,
            self.block_table_entries,
        ] {
            b.extend_from_slice(&n.to_le_bytes());
        }
        if self.version >= 1 {
            b.extend_from_slice(&self.hi_block_table_offset.to_le_bytes());
            b.extend_from_slice(&((self.hash_table_offset >> 32) as u16).to_le_bytes());
            b.extend_from_slice(&((self.block_table_offset >> 32) as u16).to_le_bytes());
        }
        if self.version >= 2 && self.header_size() >= 68 {
            for n in [
                self.archive_size,
                self.bet_table_offset,
                self.het_table_offset,
            ] {
                b.extend_from_slice(&n.to_le_bytes());
            }
        }
        if self.version == 3 {
            for n in self.table_sizes {
                b.extend_from_slice(&n.to_le_bytes());
            }
            b.extend_from_slice(&self.raw_chunk_size.to_le_bytes());
            for digest in &self.md5[..5] {
                b.extend_from_slice(digest);
            }
            b.extend_from_slice(&compute(&b).0);
        }
        b.resize(self.header_size() as usize, 0);
        output.write_all(&b)?;
        Ok(())
    }
    /// Returns the sector size in bytes, or `None` if the shift overflows `u32`.
    pub fn sector_size(&self) -> Option<u32> {
        1u32.checked_shl(9 + u32::from(self.sector_size_shift))
    }
}

pub(super) fn u64_at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

pub(super) fn verify_md5(bytes: &[u8], expected: &[u8; 16]) -> Result<(), Error> {
    if *expected != [0; 16] && compute(bytes).0 != *expected {
        return Err(Error::InvalidArchive("MD5 checksum mismatch"));
    }
    Ok(())
}

/// An on-disk filename hash and locale record. Filenames are not stored here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HashEntry {
    /// First classic MPQ filename hash.
    pub name_hash_a: u32,
    /// Second classic MPQ filename hash.
    pub name_hash_b: u32,
    /// Locale identifier; zero selects the neutral locale.
    pub locale: u16,
    /// Platform identifier; zero selects the neutral platform.
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
    /// Returns whether this slot is neither empty nor deleted.
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
    /// Payload byte offset relative to the archive header.
    pub offset: u64,
    /// Encoded payload size in bytes, including sector framing.
    pub stored_size: u32,
    /// Decoded file size in bytes.
    pub file_size: u32,
    /// Raw storage and existence flags for the block.
    pub flags: FileFlags,
}

impl BlockEntry {
    pub(super) fn bytes(self) -> [u8; 16] {
        let mut bytes = [0; 16];
        for (out, word) in bytes.as_chunks_mut::<4>().0.iter_mut().zip([
            self.offset as u32,
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
    /// Decoded archive header.
    pub header: Header,
    /// Classic filename hash slots, including empty and deleted slots.
    pub hashes: Vec<HashEntry>,
    /// Block metadata indexed by the IDs returned by filename lookup.
    pub blocks: Vec<BlockEntry>,
    /// Optional HET/BET filename lookup metadata.
    pub extended: Option<ExtendedIndex>,
}

impl Index {
    /// Looks up an exact locale/platform; no implicit locale fallback.
    pub fn find(&self, name: impl AsRef<[u8]>, locale: u16, platform: u16) -> Option<u32> {
        let name = name.as_ref();
        if let Some(index) = &self.extended {
            if locale == 0 && platform == 0 {
                if let Some(id) = index.find(name) {
                    return Some(id);
                }
            }
        }
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
    stored: u64,
    digest: &[u8; 16],
) -> Result<Vec<HashEntry>, Error> {
    let bytes = read_table(input, offset, count, stored, digest, b"(hash table)")?;
    Ok(bytes
        .as_chunks::<16>()
        .0
        .iter()
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
    stored: u64,
    digest: &[u8; 16],
) -> Result<Vec<BlockEntry>, Error> {
    let bytes = read_table(input, offset, count, stored, digest, b"(block table)")?;
    Ok(bytes
        .as_chunks::<16>()
        .0
        .iter()
        .map(|b| BlockEntry {
            offset: u32_at(b, 0) as u64,
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
    stored: u64,
    digest: &[u8; 16],
    name: &[u8],
) -> Result<Vec<u8>, Error> {
    let len = (count as usize)
        .checked_mul(16)
        .ok_or(Error::LimitExceeded("table size"))?;
    if stored > len as u64 {
        return Err(Error::InvalidArchive("table stored size"));
    }
    let mut bytes = vec![0; stored as usize];
    input.seek(SeekFrom::Start(offset))?;
    input.read_exact(&mut bytes)?;
    verify_md5(&bytes, digest)?;
    crypt(&mut bytes, hash(name, 3), true);
    if bytes.len() < len {
        bytes = decode(&bytes, len, false)?;
    }
    Ok(bytes)
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
