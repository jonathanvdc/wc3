//! HET/BET bit-packed indexes, introduced by MPQ v3.
use std::io::{Read, Seek, SeekFrom};

use super::codec::decode;
use super::crypto::{crypt, hash, jenkins};
use super::format::{u32_at, verify_md5};
use super::{BlockEntry, Error, FileFlags, Header, ReadOptions};

#[derive(Clone, Debug, Eq, PartialEq)]
/// Decoded HET/BET filename lookup metadata for neutral locale and platform entries.
pub struct ExtendedIndex {
    pub(super) hash_bits: u32,
    pub(super) slots: Vec<Option<(u64, u32)>>,
}

impl ExtendedIndex {
    /// Returns the block index for a filename, using MPQ case and slash normalization.
    /// Returns `None` when no matching filename hash is present.
    pub fn find(&self, name: &[u8]) -> Option<u32> {
        if self.slots.is_empty() {
            return None;
        }
        let mask = u64::MAX >> (64 - self.hash_bits);
        let value = (jenkins(name) & mask) | (1 << (self.hash_bits - 1));
        let start = (value % self.slots.len() as u64) as usize;
        for step in 0..self.slots.len() {
            match self.slots[(start + step) % self.slots.len()] {
                None => return None,
                Some((h, index)) if h == value => return Some(index),
                _ => {}
            }
        }
        None
    }
}

pub(super) fn bits(data: &[u8], start: u64, count: u32) -> Result<u64, Error> {
    if count > 64
        || start
            .checked_add(count as u64)
            .is_none_or(|n| n > data.len() as u64 * 8)
    {
        return Err(Error::InvalidArchive("packed table bit range"));
    }
    let mut result = 0;
    for bit in 0..count {
        let at = start + bit as u64;
        result |= ((data[(at / 8) as usize] >> (at % 8)) as u64 & 1) << bit;
    }
    Ok(result)
}

pub(super) fn read_extended(
    source: &mut (impl Read + Seek),
    base: u64,
    header: &Header,
    options: &ReadOptions,
) -> Result<(ExtendedIndex, Vec<BlockEntry>), Error> {
    let het = read_ext(
        source,
        base + header.het_table_offset,
        header.table_sizes[3],
        b"HET\x1a",
        b"(hash table)",
        &header.md5[4],
        options,
    )?;
    let bet = read_ext(
        source,
        base + header.bet_table_offset,
        header.table_sizes[4],
        b"BET\x1a",
        b"(block table)",
        &header.md5[3],
        options,
    )?;
    if het.len() < 32 || bet.len() < 76 {
        return Err(Error::InvalidArchive("extended table header"));
    }
    let h: Vec<u32> = het[..32]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| u32_at(b, 0))
        .collect();
    let b: Vec<u32> = bet[..76]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| u32_at(b, 0))
        .collect();
    if h[0] as usize > het.len()
        || h[0] < 32
        || b[0] as usize > bet.len()
        || b[0] < 76
        || h[1] > options.max_table_entries
        || h[2] > options.max_table_entries
        || b[1] > options.max_table_entries
        || !(8..=64).contains(&h[3])
        || h[6] > 32
        || h[6] > h[4]
        || h[5] != h[4] - h[6]
        || b[16] > 56
        || b[16] + 8 != h[3]
        || b[16] > b[14]
        || b[15] != b[14] - b[16]
    {
        return Err(Error::InvalidArchive("extended table dimensions"));
    }
    let slot_count = h[2] as usize;
    let index_bytes = (h[4] as u64 * h[2] as u64).div_ceil(8);
    if index_bytes != h[7] as u64 || 32 + slot_count as u64 + index_bytes != h[0] as u64 {
        return Err(Error::InvalidArchive("HET size"));
    }
    let count = b[1] as usize;
    let flag_bytes = b[18] as u64 * 4;
    let record_bytes = (b[3] as u64 * count as u64).div_ceil(8);
    let hash_bytes = (b[14] as u64 * count as u64).div_ceil(8);
    if hash_bytes != b[17] as u64 || 76 + flag_bytes + record_bytes + hash_bytes > b[0] as u64 {
        return Err(Error::InvalidArchive("BET size"));
    }
    for field in 0..5 {
        if b[9 + field] > if field == 0 { 64 } else { 32 }
            || b[4 + field]
                .checked_add(b[9 + field])
                .is_none_or(|n| n > b[3])
        {
            return Err(Error::InvalidArchive("BET record field"));
        }
    }
    let flags_end = 76 + flag_bytes as usize;
    let records_end = flags_end + record_bytes as usize;
    let flags = &bet[76..flags_end];
    let records = &bet[flags_end..records_end];
    let hashes = &bet[records_end..records_end + hash_bytes as usize];
    let mut blocks = Vec::with_capacity(count);
    for i in 0..count {
        let field = |f: usize| bits(records, i as u64 * b[3] as u64 + b[4 + f] as u64, b[9 + f]);
        let flag = field(3)?;
        let flags = if b[18] == 0 {
            0
        } else {
            if flag >= b[18] as u64 {
                return Err(Error::InvalidArchive("BET flag index"));
            }
            u32_at(flags, flag as usize * 4)
        };
        blocks.push(BlockEntry {
            offset: field(0)?,
            file_size: field(1)? as u32,
            stored_size: field(2)? as u32,
            flags: FileFlags(flags),
        });
    }
    let mut slots = Vec::with_capacity(slot_count);
    let indices = &het[32 + slot_count..];
    for i in 0..slot_count {
        let high = het[32 + i];
        if high == 0 {
            slots.push(None);
            continue;
        }
        let id = bits(indices, i as u64 * h[4] as u64, h[6])?;
        if id >= count as u64 {
            if high == 0x80 && id == ((1u64 << h[6]) - 1) {
                slots.push(Some((0, u32::MAX)));
                continue;
            }
            return Err(Error::InvalidArchive("HET block index"));
        }
        let low = bits(hashes, id * b[14] as u64, b[16])?;
        slots.push(Some(((high as u64) << b[16] | low, id as u32)));
    }
    Ok((
        ExtendedIndex {
            hash_bits: h[3],
            slots,
        },
        blocks,
    ))
}

fn read_ext(
    source: &mut (impl Read + Seek),
    offset: u64,
    stored: u64,
    signature: &[u8; 4],
    key_name: &[u8],
    digest: &[u8; 16],
    options: &ReadOptions,
) -> Result<Vec<u8>, Error> {
    let limit = options.max_table_bytes;
    if stored < 12 || stored > limit {
        return Err(Error::LimitExceeded("extended table size"));
    }
    let mut bytes = vec![0; stored as usize];
    source.seek(SeekFrom::Start(offset))?;
    source.read_exact(&mut bytes)?;
    verify_md5(&bytes, digest)?;
    if &bytes[..4] != signature || u32_at(&bytes, 4) != 1 {
        return Err(Error::InvalidArchive("extended table signature/version"));
    }
    let size = u32_at(&bytes, 8) as usize;
    if size as u64 > limit || size < bytes.len() - 12 {
        return Err(Error::LimitExceeded("extended table size"));
    }
    let mut data = bytes.split_off(12);
    crypt(&mut data, hash(key_name, 3), true);
    if data.len() < size {
        data = decode(&data, size, false)?;
    }
    Ok(data)
}

impl ExtendedIndex {
    pub(super) fn insert(&mut self, name: &[u8], id: u32) -> Result<(), Error> {
        let value =
            (jenkins(name) & (u64::MAX >> (64 - self.hash_bits))) | (1 << (self.hash_bits - 1));
        let capacity = ((id as usize + 1) * 2).max(4).next_power_of_two();
        if capacity > self.slots.len() {
            let entries: Vec<_> = self
                .slots
                .iter()
                .flatten()
                .copied()
                .filter(|(_, id)| *id != u32::MAX)
                .collect();
            self.slots = vec![None; capacity];
            for entry in entries {
                self.insert_hash(entry)?;
            }
        }
        self.insert_hash((value, id))
    }

    fn insert_hash(&mut self, entry: (u64, u32)) -> Result<(), Error> {
        let start = (entry.0 % self.slots.len() as u64) as usize;
        for step in 0..self.slots.len() {
            let slot = (start + step) % self.slots.len();
            if self.slots[slot].is_none_or(|(h, id)| h == entry.0 || id == u32::MAX) {
                self.slots[slot] = Some(entry);
                return Ok(());
            }
        }
        Err(Error::LimitExceeded("extended index capacity"))
    }

    pub(super) fn remove(&mut self, name: &[u8]) -> Option<u32> {
        let id = self.find(name)?;
        let entries: Vec<_> = self
            .slots
            .iter()
            .flatten()
            .copied()
            .filter(|(_, n)| *n != id && *n != u32::MAX)
            .collect();
        self.slots.fill(None);
        for entry in entries {
            let start = (entry.0 % self.slots.len() as u64) as usize;
            for step in 0..self.slots.len() {
                let slot = (start + step) % self.slots.len();
                if self.slots[slot].is_none() {
                    self.slots[slot] = Some(entry);
                    break;
                }
            }
        }
        Some(id)
    }
}

pub(super) fn encode_extended(
    index: &ExtendedIndex,
    blocks: &[BlockEntry],
) -> Result<[Vec<u8>; 2], Error> {
    let count = blocks.len() as u32;
    let index_bits = 32 - count.leading_zeros();
    let hash_bits = index.hash_bits - 8;
    let mut indices = vec![0; (index.slots.len() * index_bits as usize).div_ceil(8)];
    let mut hashes = vec![0; (blocks.len() * hash_bits as usize).div_ceil(8)];
    let mut high = Vec::new();
    for (slot, entry) in index.slots.iter().enumerate() {
        if let Some((value, id)) = entry {
            if *id == u32::MAX {
                high.push(0x80);
                set_bits(
                    &mut indices,
                    slot * index_bits as usize,
                    index_bits,
                    u32::MAX as u64,
                );
                continue;
            }
            if *id >= count {
                return Err(Error::InvalidArchive("extended writer block index"));
            }
            high.push((value >> hash_bits) as u8);
            set_bits(
                &mut indices,
                slot * index_bits as usize,
                index_bits,
                *id as u64,
            );
            set_bits(
                &mut hashes,
                *id as usize * hash_bits as usize,
                hash_bits,
                *value,
            );
        } else {
            high.push(0);
        }
    }
    let mut het = Vec::new();
    for n in [
        32 + high.len() as u32 + indices.len() as u32,
        count,
        high.len() as u32,
        index.hash_bits,
        index_bits,
        0,
        index_bits,
        indices.len() as u32,
    ] {
        het.extend_from_slice(&n.to_le_bytes());
    }
    het.extend(high);
    het.extend(indices);
    let mut bet = Vec::new();
    // Pad small BET name-hash areas to at least 12 bytes for reader compatibility.
    let size = 76 + blocks.len() * 24 + hashes.len().max(12);
    for n in [
        size as u32,
        count,
        0x10,
        160,
        0,
        64,
        96,
        128,
        160,
        64,
        32,
        32,
        32,
        0,
        hash_bits,
        0,
        hash_bits,
        hashes.len() as u32,
        count,
    ] {
        bet.extend_from_slice(&n.to_le_bytes());
    }
    for block in blocks {
        bet.extend_from_slice(&block.flags.0.to_le_bytes());
    }
    for (id, block) in blocks.iter().enumerate() {
        bet.extend_from_slice(&block.offset.to_le_bytes());
        for n in [block.file_size, block.stored_size, id as u32] {
            bet.extend_from_slice(&n.to_le_bytes());
        }
    }
    bet.extend(hashes);
    bet.resize(size, 0);
    let wrap = |mut data: Vec<u8>, signature: &[u8], key: &[u8]| {
        let mut bytes = signature.to_vec();
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
        crypt(&mut data, hash(key, 3), false);
        bytes.extend(data);
        bytes
    };
    Ok([
        wrap(het, b"HET\x1a", b"(hash table)"),
        wrap(bet, b"BET\x1a", b"(block table)"),
    ])
}

fn set_bits(data: &mut [u8], start: usize, count: u32, value: u64) {
    for bit in 0..count as usize {
        if value & (1 << bit) != 0 {
            data[(start + bit) / 8] |= 1 << ((start + bit) % 8);
        }
    }
}

#[cfg(test)]
#[path = "extended_tests.rs"]
mod tests;
