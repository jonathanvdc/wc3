//! Bounded recovery of malformed classic MPQs. Valid extended formats stay strict.
//!
//! Recovery preserves logical hash slots and resolves classic wrapping before
//! validating physical ranges. It never changes codecs or bypasses resource limits.
use std::io::{ErrorKind, Read, Seek, SeekFrom};

use super::codec::decode;
use super::crypto::{content_key, crypt, hash, known_word_keys};
use super::format::{offset_table_size, sector_count, u32_at, DELETED};
use super::reader::load_offsets;
use super::{BlockEntry, Error, FileFlags, HashEntry, Header, Index, ReadOptions};

/// Interpretation policy for malformed archives.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ReadMode {
    #[default]
    /// Reject malformed archive layouts without recovery.
    Strict,
    /// Recover known classic MPQ quirks, retaining all resource limits.
    Permissive,
}

/// A recovery applied while indexing or opening an entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryDiagnostic {
    /// Description of the recovery applied to the archive or entry.
    pub reason: &'static str,
    /// Table slot or block ID, when the recovery concerns a record.
    pub entry: Option<u32>,
}

pub(super) fn report(out: &mut Vec<RecoveryDiagnostic>, reason: &'static str, entry: Option<u32>) {
    let diagnostic = RecoveryDiagnostic { reason, entry };
    // Bound both memory and work for malicious tables with millions of bad records.
    if out.len() < 1024 {
        out.push(diagnostic);
    } else if out.len() == 1024 {
        out.push(RecoveryDiagnostic {
            reason: "further recovery diagnostics omitted",
            entry: None,
        });
    }
}

/// Classic Storm arithmetic wraps the absolute address at 32 bits.
/// Range checking always follows address resolution, never precedes it.
pub(super) fn address(
    base: u64,
    relative: u64,
    size: u64,
    len: u64,
    wrapping: bool,
) -> Result<u64, Error> {
    let at = if wrapping {
        (base as u32).wrapping_add(relative as u32) as u64
    } else {
        base.checked_add(relative)
            .ok_or(Error::InvalidArchive("offset overflow"))?
    };
    if at.checked_add(size).is_none_or(|end| end > len) {
        return Err(Error::InvalidArchive("read outside source"));
    }
    Ok(at)
}

pub(super) fn read_index(
    source: &mut (impl Read + Seek),
    base: u64,
    len: u64,
    bytes: &[u8],
    options: &ReadOptions,
    diagnostics: &mut Vec<RecoveryDiagnostic>,
) -> Result<Index, Error> {
    let mut normalized = bytes[..32].to_vec();
    if u32_at(bytes, 4) != 32 || bytes[12..14] != [0, 0] {
        report(diagnostics, "classic header size/version overridden", None);
    }
    normalized[4..8].copy_from_slice(&32u32.to_le_bytes());
    normalized[12..14].fill(0);
    if normalized[15] != 0 {
        report(diagnostics, "sector shift high byte ignored", None);
        normalized[15] = 0;
    }
    let mut header = Header::decode(&normalized)?;
    if header
        .sector_size()
        .is_none_or(|size| size > options.max_sector_size)
    {
        return Err(Error::LimitExceeded("sector size"));
    }
    if header.archive_size != len - base {
        report(
            diagnostics,
            "archive extent replaced by source extent",
            None,
        );
    }
    header.archive_size = len - base;
    let mut tables = Vec::new();
    for (offset, count, name, is_hash) in [
        (
            header.hash_table_offset,
            header.hash_table_entries,
            b"(hash table)".as_slice(),
            true,
        ),
        (
            header.block_table_offset,
            header.block_table_entries,
            b"(block table)".as_slice(),
            false,
        ),
    ] {
        let masked = count & 0x0fff_ffff;
        if masked != count {
            report(diagnostics, "table count high bits ignored", None);
        }
        let at = if offset == u32::MAX as u64 {
            report(
                diagnostics,
                "sentinel table offset uses current cursor",
                None,
            );
            source.stream_position()?
        } else {
            address(base, offset, 0, len, true)?
        };
        if at > len {
            return Err(Error::InvalidArchive("table outside source"));
        }
        if at < base {
            report(diagnostics, "table offset wrapped before header", None);
        }
        let available = ((len - at) / 16).min(masked as u64) as u32;
        let logical = if is_hash { masked } else { available };
        if logical > options.max_table_entries || logical as u64 * 16 > options.max_table_bytes {
            return Err(Error::LimitExceeded("table allocation"));
        }
        if is_hash && (logical == 0 || !logical.is_power_of_two()) {
            return Err(Error::InvalidArchive(
                "hash table size is not a power of two",
            ));
        }
        if available < masked {
            report(diagnostics, "classic table truncated", None);
        }
        let mut table = vec![0; available as usize * 16];
        source.seek(SeekFrom::Start(at))?;
        source.read_exact(&mut table)?;
        crypt(&mut table, hash(name, 3), true);
        tables.push((table, logical));
    }
    let (hash_bytes, hash_count) = &tables[0];
    let (block_bytes, block_count) = &tables[1];
    let blocks: Vec<_> = block_bytes
        .as_chunks::<16>()
        .0
        .iter()
        .map(|b| BlockEntry {
            offset: u32_at(b, 0) as u64,
            stored_size: u32_at(b, 4),
            file_size: u32_at(b, 8),
            flags: FileFlags(u32_at(b, 12)),
        })
        .collect();
    let mut hashes = Vec::with_capacity(*hash_count as usize);
    for (slot, b) in hash_bytes.as_chunks::<16>().0.iter().enumerate() {
        let mut id = u32_at(b, 12);
        if id < DELETED {
            let masked = id & 0x0fff_ffff;
            if masked != id {
                report(
                    diagnostics,
                    "block index high bits ignored",
                    Some(slot as u32),
                );
            }
            id = masked;
            if blocks
                .get(id as usize)
                .is_none_or(|b| !b.flags.contains(FileFlags::EXISTS))
            {
                report(
                    diagnostics,
                    "invalid hash reference ignored",
                    Some(slot as u32),
                );
                id = DELETED;
            }
        }
        if b[11] != 0 && id < DELETED {
            report(diagnostics, "hash reserved byte ignored", Some(slot as u32));
        }
        hashes.push(HashEntry {
            name_hash_a: u32_at(b, 0),
            name_hash_b: u32_at(b, 4),
            locale: u16::from_le_bytes(b[8..10].try_into().unwrap()),
            platform: b[10] as u16,
            block_index: id,
        });
    }
    hashes.resize(
        *hash_count as usize,
        HashEntry {
            block_index: DELETED,
            ..HashEntry::empty()
        },
    );
    for (id, block) in blocks.iter().enumerate() {
        if block.flags.contains(FileFlags::EXISTS)
            && address(base, block.offset, 0, len, true)
                .is_ok_and(|at| at < base + 32 && block.file_size != 0)
        {
            report(
                diagnostics,
                "block offset wrapped before header",
                Some(id as u32),
            );
        }
        if block.flags.contains(FileFlags::EXISTS)
            && address(base, block.offset, block.stored_size as u64, len, true).is_err()
        {
            report(
                diagnostics,
                "invalid block extent deferred",
                Some(id as u32),
            );
        }
    }
    header.hash_table_entries = *hash_count;
    header.block_table_entries = *block_count;
    header.table_sizes[0] = hash_bytes.len() as u64;
    header.table_sizes[1] = block_bytes.len() as u64;
    Ok(Index {
        header,
        hashes,
        blocks,
        extended: None,
    })
}

pub(super) struct LayoutRequest<'a> {
    pub base: u64,
    pub len: u64,
    pub block: BlockEntry,
    pub id: u32,
    pub sector_size: u32,
    pub key: Option<u32>,
    pub wrapping: bool,
    pub options: &'a ReadOptions,
}

pub(super) struct EntryLayout {
    pub key: Option<u32>,
    pub offsets: Vec<u32>,
    pub checksums: Vec<u32>,
}

pub(super) fn load_layout(
    source: &mut (impl Read + Seek),
    request: LayoutRequest<'_>,
    diagnostics: &mut Vec<RecoveryDiagnostic>,
) -> Result<EntryLayout, Error> {
    let LayoutRequest {
        base,
        len,
        block,
        id,
        sector_size,
        mut key,
        wrapping,
        options,
    } = request;
    let mut payload_options = options.clone();
    if !wrapping {
        payload_options.mode = ReadMode::Strict;
    }
    let offsets = match load_offsets(
        &mut *source,
        base,
        block,
        sector_size,
        key,
        &payload_options,
    ) {
        Ok(offsets) => offsets,
        Err(original)
            if wrapping
                && block.flags.contains(FileFlags::ENCRYPTED)
                && block.flags.compressed()
                && !block.flags.contains(FileFlags::SINGLE_UNIT) =>
        {
            let size = offset_table_size(
                block.file_size,
                sector_size,
                block.flags.contains(FileFlags::SECTOR_CRC),
            )?;
            if size > options.max_sector_table_bytes {
                return Err(Error::LimitExceeded("sector offset table"));
            }
            let at = address(base, block.offset, size as u64, len, true)?;
            let mut encrypted = vec![0; size as usize];
            source.seek(SeekFrom::Start(at))?;
            source.read_exact(&mut encrypted)?;
            let mut recovered = None;
            'candidates: for first in size..size.saturating_add(4) {
                for candidate in known_word_keys(&encrypted, first) {
                    let candidate = candidate.wrapping_add(1);
                    if let Ok(offsets) = load_offsets(
                        &mut *source,
                        base,
                        block,
                        sector_size,
                        Some(candidate),
                        &payload_options,
                    ) {
                        recovered = Some((candidate, offsets));
                        break 'candidates;
                    }
                }
            }
            let (candidate, offsets) = recovered.ok_or(original)?;
            key = Some(candidate);
            report(
                diagnostics,
                "encryption key recovered from sector table",
                Some(id),
            );
            offsets
        }
        Err(error) => return Err(error),
    };
    if wrapping
        && block.flags.contains(FileFlags::ENCRYPTED)
        && offsets.is_empty()
        && (!block.flags.compressed()
            || (block.flags.contains(FileFlags::SINGLE_UNIT)
                && block.stored_size == block.file_size))
        && block.file_size >= 8
    {
        let size = block.file_size.min(12);
        let at = address(base, block.offset, size as u64, len, true)?;
        let mut encrypted = vec![0; size as usize];
        source.seek(SeekFrom::Start(at))?;
        source.read_exact(&mut encrypted)?;
        if let Some(candidate) = content_key(&encrypted, block.file_size) {
            if key != Some(candidate) {
                key = Some(candidate);
                report(
                    diagnostics,
                    "encryption key recovered from known content",
                    Some(id),
                );
            }
        }
    }
    if block.flags.contains(FileFlags::ENCRYPTED) && key.is_none() && block.file_size != 0 {
        return Err(Error::InvalidArchive("unknown encryption key"));
    }
    if wrapping && offsets.first().is_some_and(|offset| *offset > 0x8000_0000) {
        report(diagnostics, "sector offsets wrapped before table", Some(id));
    }
    if wrapping
        && offsets
            .first()
            .is_some_and(|offset| *offset < 0x8000_0000 && *offset as usize > offsets.len() * 4)
    {
        report(diagnostics, "extra sector table bytes skipped", Some(id));
    }
    let checksum_result = (|| -> Result<Vec<u32>, Error> {
        Ok(
            if options.verify_sector_checksums
                && block.flags.contains(FileFlags::SECTOR_CRC)
                && !offsets.is_empty()
            {
                let count = sector_count(block.file_size, sector_size) as usize;
                let start = offsets[count];
                let stored = offsets[count + 1]
                    .checked_sub(start)
                    .ok_or(Error::InvalidArchive("checksum offsets"))?;
                if stored == 0 {
                    Vec::new()
                } else {
                    let expected = count * 4;
                    if stored as usize > expected {
                        return Err(Error::InvalidArchive("checksum table length"));
                    }
                    let mut bytes = vec![0; stored as usize];
                    source.seek(SeekFrom::Start(address(
                        base,
                        block.offset + start as u64,
                        stored as u64,
                        len,
                        wrapping,
                    )?))?;
                    source.read_exact(&mut bytes)?;
                    if bytes.len() < expected {
                        bytes = decode(&bytes, expected, false)?;
                    }
                    bytes
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .map(|b| u32_at(b, 0))
                        .collect()
                }
            } else {
                Vec::new()
            },
        )
    })();
    let checksums = match checksum_result {
        Ok(checksums) => checksums,
        Err(error)
            if wrapping
                && (matches!(
                    &error,
                    Error::InvalidArchive(_) | Error::UnsupportedCompression(_)
                ) || matches!(&error, Error::Io(io) if io.kind() == ErrorKind::UnexpectedEof)) =>
        {
            report(
                diagnostics,
                "unusable sector checksum metadata ignored",
                Some(id),
            );
            Vec::new()
        }
        Err(error) => return Err(error),
    };
    Ok(EntryLayout {
        key,
        offsets,
        checksums,
    })
}
