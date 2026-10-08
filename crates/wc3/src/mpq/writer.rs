use std::io::{self, Read, Seek, SeekFrom, Write};

use md5::compute;

use super::codec::{check_encoder, encode, sector_checksum};
use super::crypto::{crypt, file_key, hash};
use super::extended::{encode_extended, ExtendedIndex};
use super::format::{offset_table_size, validate_name, DELETED};
use super::raw::{self, Digests};
use super::{Archive, BlockEntry, EncodedEntry, Error, FileFlags, HashEntry, Header};

/// Entry compression used by the writer. Compressed sectors that would grow
/// are stored raw, as required by MPQ's framing rules.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Compression {
    #[default]
    /// Store entry bytes without compression.
    Stored,
    /// Compress sectors with zlib; requires the `mpq-encode` feature.
    Zlib,
    /// Compress sectors with bzip2; requires the `mpq-encode` feature.
    Bzip2,
}

/// Per-entry storage and lookup settings.
#[derive(Clone, Copy, Debug, Default)]
pub struct FileOptions {
    /// Compression applied to entry sectors.
    pub compression: Compression,
    /// Encrypt payload bytes using the filename-derived MPQ key.
    pub encrypted: bool,
    /// Include the entry's archive-relative offset and decoded size in its key.
    pub adjusted_key: bool,
    /// Locale used for filename lookup; zero selects the neutral locale.
    pub locale: u16,
    /// Platform used for filename lookup; zero selects the neutral platform.
    pub platform: u16,
    /// Store Adler-32 records for compressed sector entries.
    pub sector_checksums: bool,
}

/// Archive construction settings.
#[derive(Clone, Debug)]
pub struct WriteOptions {
    /// Defaults to 4096-byte sectors (`512 << 3`). Maximum is 1 MiB.
    pub sector_size_shift: u16,
    /// Generate `(listfile)` on finish, including itself. Names remain bytes.
    pub listfile: bool,
    /// Maximum number of block records allowed in the archive.
    pub max_entries: u32,
    /// On-disk header version, 0 through 3. Defaults to classic MPQ.
    pub header_version: u16,
    /// Include HET/BET tables (requires header version 2 or 3).
    pub extended_index: bool,
    /// Raw encoded chunk MD5s; zero disables them. Requires header version 3.
    pub raw_chunk_size: u32,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            sector_size_shift: 3,
            listfile: true,
            max_entries: 1 << 20,
            header_version: 0,
            extended_index: false,
            raw_chunk_size: 0,
        }
    }
}

/// A streaming MPQ writer supporting all four header versions. `finish` is mandatory; dropping an entry
/// before its own `finish`, or encountering a payload I/O error, poisons it.
/// Memory holds the index, names, sector buffers, and the current sector table.
pub struct ArchiveWriter<W> {
    output: W,
    base: u64,
    options: WriteOptions,
    blocks: Vec<BlockEntry>,
    pending: Vec<(u32, HashEntry)>,
    /// Seeded table keeps unknown filenames in their original probe slots.
    seeded: Option<Vec<HashEntry>>,
    names: Vec<Vec<u8>>,
    name_bytes: usize,
    failed: bool,
    extended: Option<ExtendedIndex>,
    shortened_header_size: u32,
}

impl<W: Write + Seek> ArchiveWriter<W> {
    /// Starts an archive at the destination cursor, which must be 512-byte aligned.
    /// Returns an error for invalid options, exceeded limits, or destination I/O failures.
    pub fn new(output: W, options: WriteOptions) -> Result<Self, Error> {
        if options.header_version > 3 {
            return Err(Error::UnsupportedVersion(options.header_version));
        }
        if options.extended_index && options.header_version < 2 {
            return Err(Error::InvalidArchive("HET/BET requires MPQ v3"));
        }
        if options.raw_chunk_size != 0
            && (options.header_version != 3 || options.raw_chunk_size > 16 << 20)
        {
            return Err(Error::InvalidArchive(
                "raw chunks require MPQ v4 and at most 16 MiB",
            ));
        }
        if options.sector_size_shift > 11 {
            return Err(Error::LimitExceeded("writer sector size"));
        }
        let mut output = output;
        let base = output.stream_position()?;
        if base % 512 != 0 {
            return Err(Error::InvalidArchive(
                "writer base must be 512-byte aligned",
            ));
        }
        let header = Header {
            version: options.header_version,
            ..Header::default()
        };
        output.write_all(&vec![0; header.header_size() as usize])?;
        let extended = options.extended_index.then(|| ExtendedIndex {
            hash_bits: 64,
            slots: vec![None; 4],
        });
        Ok(Self {
            output,
            base,
            options,
            blocks: Vec::new(),
            pending: Vec::new(),
            seeded: None,
            names: Vec::new(),
            name_bytes: 0,
            failed: false,
            extended,
            shortened_header_size: 0,
        })
    }

    /// Creates an editable copy without decoding payloads. Encoded blocks keep
    /// their relative offsets, so unnamed entries, unknown codecs, and adjusted
    /// encryption keys survive. Replacements append data; this does not compact.
    /// The original hash table cannot grow without knowing all original names.
    /// Automatic listfile generation is disabled to preserve the original one.
    pub fn from_archive<R: Read + Seek>(
        output: W,
        archive: &mut Archive<R>,
    ) -> Result<Self, Error> {
        if !archive.diagnostics().is_empty() {
            return Err(Error::InvalidArchive("cannot edit a recovered archive"));
        }
        let options = WriteOptions {
            sector_size_shift: archive.index.header.sector_size_shift,
            listfile: false,
            header_version: archive.index.header.version,
            extended_index: archive.index.extended.is_some(),
            raw_chunk_size: archive.index.header.raw_chunk_size,
            ..WriteOptions::default()
        };
        let mut writer = Self::new(output, options)?;
        writer.shortened_header_size = archive.index.header.shortened_header_size;
        writer.output.seek(SeekFrom::Start(
            writer.base + archive.index.header.header_size() as u64,
        ))?;
        archive.source.seek(SeekFrom::Start(
            archive.base + archive.index.header.header_size() as u64,
        ))?;
        let size = archive.index.header.archive_size - archive.index.header.header_size() as u64;
        let copied = io::copy(&mut (&mut archive.source).take(size), &mut writer.output)?;
        if copied != size {
            return Err(Error::InvalidArchive("truncated archive copy"));
        }
        writer.blocks = archive.index.blocks.clone();
        writer.seeded = (!archive.index.hashes.is_empty()).then(|| archive.index.hashes.clone());
        writer.extended = archive.index.extended.clone();
        Ok(writer)
    }

    /// Starts an entry with a known decoded size, reserving its sector table.
    pub fn start_file(
        &mut self,
        name: impl AsRef<[u8]>,
        size: u32,
        options: FileOptions,
    ) -> Result<EntryWriter<'_, W>, Error> {
        self.start(name.as_ref(), size, options, false)
    }

    /// Streams exactly `size` bytes from `input`; remaining input is untouched.
    pub fn add_file(
        &mut self,
        name: impl AsRef<[u8]>,
        size: u32,
        input: &mut impl Read,
        options: FileOptions,
    ) -> Result<(), Error> {
        let mut entry = self.start_file(name, size, options)?;
        let copied = io::copy(&mut input.take(size as u64), &mut entry)?;
        if copied != size as u64 {
            return Err(Error::SizeMismatch);
        }
        entry.finish()
    }

    /// Imports an encoded entry without decompression or recompression.
    /// Rejects encrypted entries, unsupported flags, and incompatible sector
    /// framing before writing. Single-unit and uncompressed entries do not
    /// depend on the destination sector size. Payload contents are not decoded
    /// or validated; callers must supply metadata matching the encoded bytes.
    /// A short stream or I/O failure poisons the writer. Raw chunk digests are
    /// generated according to the destination archive's settings.
    pub fn add_encoded_file<R: Read>(
        &mut self,
        name: impl AsRef<[u8]>,
        mut entry: EncodedEntry<R>,
    ) -> Result<(), Error> {
        let name = name.as_ref();
        let metadata = *entry.metadata();
        self.validate_entry(name, metadata.locale, metadata.platform, false)?;
        let flags = metadata.flags;
        flags.validate_codec()?;
        if !flags.contains(FileFlags::EXISTS) || flags.contains(FileFlags::ENCRYPTED) {
            return Err(Error::UnsupportedFlags(flags.0));
        }
        if !metadata.sector_size.is_power_of_two()
            || metadata.sector_size < 512
            || metadata.sector_size > 1 << 20
        {
            return Err(Error::InvalidArchive("encoded sector size"));
        }
        if flags.compressed() && !flags.contains(FileFlags::SINGLE_UNIT) && metadata.file_size != 0
        {
            if metadata.sector_size != 512u32 << self.options.sector_size_shift {
                return Err(Error::InvalidArchive(
                    "encoded sector size differs from destination",
                ));
            }
            let table_size = offset_table_size(
                metadata.file_size,
                metadata.sector_size,
                flags.contains(FileFlags::SECTOR_CRC),
            )?;
            if table_size > 16 << 20 {
                return Err(Error::LimitExceeded("writer sector table"));
            }
            if metadata.stored_size < table_size {
                return Err(Error::SizeMismatch);
            }
        } else if !flags.compressed() && metadata.file_size != metadata.stored_size {
            return Err(Error::SizeMismatch);
        }
        if (metadata.file_size == 0) != (metadata.stored_size == 0) {
            return Err(Error::SizeMismatch);
        }
        let digest_size = if self.options.raw_chunk_size == 0 {
            0
        } else {
            (metadata.stored_size as u64).div_ceil(self.options.raw_chunk_size as u64) * 16
        };
        if digest_size > 16 << 20 {
            return Err(Error::LimitExceeded("writer raw digest table"));
        }
        let offset = self.relative_position()?;
        let max = if self.options.header_version == 0 {
            u32::MAX as u64
        } else {
            (1u64 << 48) - 1
        };
        offset
            .checked_add(metadata.stored_size as u64)
            .and_then(|n| n.checked_add(digest_size))
            .filter(|&n| n <= max)
            .ok_or(Error::LimitExceeded("archive size"))?;
        self.failed = true;
        let mut raw = (self.options.raw_chunk_size != 0)
            .then(|| Digests::new(self.options.raw_chunk_size, 0));
        let copied = if let Some(raw) = &mut raw {
            let mut count = 0;
            let mut buffer = [0; 64 * 1024];
            loop {
                let n = entry.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                self.output.write_all(&buffer[..n])?;
                raw.update(&buffer[..n]);
                count += n as u64;
            }
            count
        } else {
            io::copy(&mut entry, &mut self.output)?
        };
        if copied != metadata.stored_size as u64 {
            return Err(Error::SizeMismatch);
        }
        if let Some(raw) = raw {
            raw.finish(&[], &mut self.output)?;
        }
        self.commit_entry(
            name.to_vec(),
            metadata.locale,
            metadata.platform,
            BlockEntry {
                offset,
                stored_size: metadata.stored_size,
                file_size: metadata.file_size,
                flags,
            },
            false,
        )
    }

    /// Copies a named neutral entry into this archive, retaining its encoded representation.
    /// Uses `open_encoded_file` and `add_encoded_file`; encrypted entries are rejected.
    pub fn copy_file_from<R: Read + Seek>(
        &mut self,
        source: &mut Archive<R>,
        name: impl AsRef<[u8]>,
    ) -> Result<(), Error> {
        let name = name.as_ref();
        let entry = source.open_encoded_file(name)?;
        self.add_encoded_file(name, entry)
    }

    /// Copies a named entry for an exact locale/platform, preserving that lookup identity.
    pub fn copy_file_from_with_locale<R: Read + Seek>(
        &mut self,
        source: &mut Archive<R>,
        name: impl AsRef<[u8]>,
        locale: u16,
        platform: u16,
    ) -> Result<(), Error> {
        let name = name.as_ref();
        let entry = source.open_encoded_file_with_locale(name, locale, platform)?;
        self.add_encoded_file(name, entry)
    }

    /// Replaces an exact filename/locale/platform, or inserts if absent.
    pub fn replace_file(
        &mut self,
        name: impl AsRef<[u8]>,
        size: u32,
        input: &mut impl Read,
        options: FileOptions,
    ) -> Result<(), Error> {
        let mut entry = self.start(name.as_ref(), size, options, true)?;
        let copied = io::copy(&mut input.take(size as u64), &mut entry)?;
        if copied != size as u64 {
            return Err(Error::SizeMismatch);
        }
        entry.finish()
    }

    /// Deletes an exact match while retaining other locales and probe chains.
    /// Payload space is reclaimed only by an explicit future compaction.
    pub fn remove_file(
        &mut self,
        name: impl AsRef<[u8]>,
        locale: u16,
        platform: u16,
    ) -> Result<(), Error> {
        self.check_state()?;
        let name = name.as_ref();
        validate_name(name)?;
        let identity = (hash(name, 1), hash(name, 2), locale, platform);
        let mut found = if locale == 0 && platform == 0 {
            self.extended
                .as_mut()
                .is_some_and(|index| index.remove(name).is_some())
        } else {
            false
        };
        self.pending.retain(|(_, entry)| {
            let matches = Self::identity(entry) == identity;
            found |= matches;
            !matches
        });
        if let Some(table) = &mut self.seeded {
            for entry in table {
                if entry.is_file() && Self::identity(entry) == identity {
                    entry.block_index = DELETED;
                    found = true;
                }
            }
        }
        if !found {
            return Err(Error::FileNotFound);
        }
        let still_present = self
            .pending
            .iter()
            .map(|(_, e)| e)
            .chain(self.seeded.iter().flatten())
            .any(|e| e.is_file() && e.name_hash_a == identity.0 && e.name_hash_b == identity.1);
        if !still_present {
            self.names.retain(|n| {
                let keep = hash(n, 1) != identity.0 || hash(n, 2) != identity.1;
                if !keep {
                    self.name_bytes -= n.len();
                }
                keep
            });
        }
        self.mark_unreferenced();
        Ok(())
    }

    fn identity(entry: &HashEntry) -> (u32, u32, u16, u16) {
        (
            entry.name_hash_a,
            entry.name_hash_b,
            entry.locale,
            entry.platform,
        )
    }

    fn check_state(&self) -> Result<(), Error> {
        if self.failed {
            Err(Error::WriterFailed)
        } else {
            Ok(())
        }
    }

    fn validate_entry(
        &self,
        name: &[u8],
        locale: u16,
        platform: u16,
        replace: bool,
    ) -> Result<(), Error> {
        self.check_state()?;
        validate_name(name)?;
        if self
            .name_bytes
            .checked_add(name.len())
            .is_none_or(|n| n > 16 << 20)
        {
            return Err(Error::LimitExceeded("writer names"));
        }
        if self.options.listfile
            && hash(name, 1) == hash(b"(listfile)", 1)
            && hash(name, 2) == hash(b"(listfile)", 2)
        {
            return Err(Error::DuplicateFile);
        }
        let identity = (hash(name, 1), hash(name, 2), locale, platform);
        let exists = (locale == 0
            && platform == 0
            && self
                .extended
                .as_ref()
                .is_some_and(|i| i.find(name).is_some()))
            || self
                .pending
                .iter()
                .any(|(_, e)| Self::identity(e) == identity)
            || self.seeded.as_ref().is_some_and(|t| {
                t.iter()
                    .any(|e| e.is_file() && Self::identity(e) == identity)
            });
        if exists && !replace {
            return Err(Error::DuplicateFile);
        }
        if self.blocks.len() >= self.options.max_entries as usize {
            return Err(Error::LimitExceeded("writer entries"));
        }
        if let Some(table) = &self.seeded {
            let available = table.iter().filter(|e| !e.is_file()).count();
            let insertions = self.pending.len();
            if !exists && insertions >= available {
                return Err(Error::LimitExceeded("preserved hash table capacity"));
            }
        }
        Ok(())
    }

    fn start(
        &mut self,
        name: &[u8],
        size: u32,
        options: FileOptions,
        replace: bool,
    ) -> Result<EntryWriter<'_, W>, Error> {
        self.validate_entry(name, options.locale, options.platform, replace)?;
        check_encoder(options.compression)?;
        if options.adjusted_key && !options.encrypted {
            return Err(Error::UnsupportedFlags(FileFlags::FIX_KEY));
        }
        if options.sector_checksums && options.compression == Compression::Stored {
            return Err(Error::UnsupportedFlags(FileFlags::SECTOR_CRC));
        }
        let sector_size = 512u32 << self.options.sector_size_shift;
        let table_size = if options.compression != Compression::Stored && size != 0 {
            offset_table_size(size, sector_size, options.sector_checksums)?
        } else {
            0
        };
        if table_size > 16 << 20 {
            return Err(Error::LimitExceeded("writer sector table"));
        }
        let offset = self.relative_position()?;
        // Bound both the address space and raw digest allocation before writing.
        let checksum_size = if options.sector_checksums {
            size.div_ceil(sector_size) * 4
        } else {
            0
        };
        let stored = table_size as u64 + size as u64 + checksum_size as u64;
        if self.options.raw_chunk_size != 0
            && stored.div_ceil(self.options.raw_chunk_size as u64) * 16 > 16 << 20
        {
            return Err(Error::LimitExceeded("writer raw digest table"));
        }
        let max = if self.options.header_version == 0 {
            u32::MAX as u64
        } else {
            (1u64 << 48) - 1
        };
        offset
            .checked_add(table_size as u64)
            .and_then(|n| n.checked_add(size as u64))
            .and_then(|n| n.checked_add(checksum_size as u64))
            .filter(|&n| n <= max)
            .ok_or(Error::LimitExceeded("archive size"))?;
        self.failed = true;
        // Reserve without constructing a potentially large zero-filled buffer.
        let zeros = [0; 4096];
        let mut remaining = table_size;
        while remaining != 0 {
            let count = remaining.min(zeros.len() as u32);
            self.output.write_all(&zeros[..count as usize])?;
            remaining -= count;
        }
        let mut flags = FileFlags::EXISTS;
        if options.compression != Compression::Stored {
            flags |= FileFlags::COMPRESS;
        }
        if options.encrypted {
            flags |= FileFlags::ENCRYPTED;
        }
        if options.adjusted_key {
            flags |= FileFlags::FIX_KEY;
        }
        if options.sector_checksums {
            flags |= FileFlags::SECTOR_CRC;
        }
        let key = options
            .encrypted
            .then(|| file_key(name, offset as u32, size, options.adjusted_key));
        let raw = (self.options.raw_chunk_size != 0)
            .then(|| Digests::new(self.options.raw_chunk_size, table_size));
        Ok(EntryWriter {
            archive: self,
            name: name.to_vec(),
            options,
            block: BlockEntry {
                offset,
                stored_size: table_size,
                file_size: size,
                flags: FileFlags(flags),
            },
            key,
            raw,
            remaining: size,
            sector_size,
            sector: 0,
            buffer: Vec::new(),
            offsets: if table_size != 0 {
                vec![table_size]
            } else {
                Vec::new()
            },
            checksums: Vec::new(),
            replace,
            failed: false,
        })
    }

    fn commit_entry(
        &mut self,
        name: Vec<u8>,
        locale: u16,
        platform: u16,
        block: BlockEntry,
        replace: bool,
    ) -> Result<(), Error> {
        if replace {
            // Entry writing keeps the parent poisoned until its metadata commits.
            self.failed = false;
            match self.remove_file(&name, locale, platform) {
                Ok(()) | Err(Error::FileNotFound) => {}
                Err(e) => {
                    self.failed = true;
                    return Err(e);
                }
            }
            self.failed = true;
        }
        let block_index = self.blocks.len() as u32;
        self.blocks.push(block);
        if locale == 0 && platform == 0 {
            if let Some(index) = &mut self.extended {
                index.insert(&name, block_index)?;
            }
        }
        self.pending.push((
            hash(&name, 0),
            HashEntry {
                name_hash_a: hash(&name, 1),
                name_hash_b: hash(&name, 2),
                locale,
                platform,
                block_index,
            },
        ));
        self.names.push(name);
        self.name_bytes += self.names.last().unwrap().len();
        self.failed = false;
        Ok(())
    }

    fn relative_position(&mut self) -> Result<u64, Error> {
        let position = self
            .output
            .stream_position()?
            .checked_sub(self.base)
            .ok_or(Error::InvalidArchive("writer cursor before archive"))?;
        let max = if self.options.header_version == 0 {
            u32::MAX as u64
        } else {
            (1u64 << 48) - 1
        };
        if position > max {
            return Err(Error::LimitExceeded("archive size"));
        }
        Ok(position)
    }

    fn mark_unreferenced(&mut self) {
        let mut referenced = vec![false; self.blocks.len()];
        for entry in self
            .pending
            .iter()
            .map(|(_, e)| e)
            .chain(self.seeded.iter().flatten())
        {
            if entry.is_file() {
                referenced[entry.block_index as usize] = true;
            }
        }
        if let Some(index) = &self.extended {
            for (_, id) in index.slots.iter().flatten() {
                if *id != u32::MAX {
                    referenced[*id as usize] = true;
                }
            }
        }
        for (block, referenced) in self.blocks.iter_mut().zip(referenced) {
            if !referenced {
                block.flags.0 &= !FileFlags::EXISTS;
            }
        }
    }

    /// Writes encrypted index tables, patches the header, flushes, and returns
    /// the destination positioned immediately after the completed archive.
    pub fn finish(mut self) -> Result<W, Error> {
        self.check_state()?;
        if self.options.listfile {
            self.options.listfile = false;
            self.names.push(b"(listfile)".to_vec());
            self.names.sort();
            self.names.dedup();
            let mut listfile = Vec::new();
            for name in &self.names {
                listfile.extend_from_slice(name);
                listfile.extend_from_slice(b"\r\n");
                if listfile.len() > 16 << 20 {
                    return Err(Error::LimitExceeded("generated listfile"));
                }
            }
            self.add_file(
                b"(listfile)",
                listfile.len() as u32,
                &mut listfile.as_slice(),
                FileOptions::default(),
            )?;
        }
        let mut hashes = if let Some(table) = self.seeded.take() {
            table
        } else {
            let count = self
                .pending
                .len()
                .max(if self.extended.is_some() {
                    self.blocks.len()
                } else {
                    0
                })
                .checked_mul(2)
                .and_then(|n| n.max(4).checked_next_power_of_two())
                .ok_or(Error::LimitExceeded("hash table size"))?;
            vec![HashEntry::empty(); count]
        };
        for &(start, entry) in &self.pending {
            let mask = hashes.len() - 1;
            let slot = (0..hashes.len())
                .map(|step| (start as usize + step) & mask)
                .find(|&slot| !hashes[slot].is_file())
                .ok_or(Error::LimitExceeded("hash table capacity"))?;
            hashes[slot] = entry;
        }
        if let Some(index) = &mut self.extended {
            // Compact metadata after edits, retaining all unknown filename hashes
            // and payload offsets. BET record count must fit the classic hash table.
            let mut referenced = vec![false; self.blocks.len()];
            for entry in &hashes {
                if entry.is_file() {
                    referenced[entry.block_index as usize] = true;
                }
            }
            for (_, id) in index.slots.iter().flatten() {
                if *id != u32::MAX {
                    referenced[*id as usize] = true;
                }
            }
            let mut remap = vec![u32::MAX; self.blocks.len()];
            let mut blocks = Vec::new();
            for (id, block) in self.blocks.into_iter().enumerate() {
                if referenced[id] {
                    remap[id] = blocks.len() as u32;
                    blocks.push(block);
                }
            }
            self.blocks = blocks;
            for entry in &mut hashes {
                if entry.is_file() {
                    entry.block_index = remap[entry.block_index as usize];
                }
            }
            for (_, id) in index.slots.iter_mut().flatten() {
                if *id != u32::MAX {
                    *id = remap[*id as usize];
                }
            }
        }
        if self.extended.is_some() && self.blocks.len() > hashes.len() {
            return Err(Error::LimitExceeded(
                "preserved hash table capacity for BET records",
            ));
        }
        let mut header = Header {
            version: self.options.header_version,
            shortened_header_size: self.shortened_header_size,
            sector_size_shift: self.options.sector_size_shift,
            raw_chunk_size: self.options.raw_chunk_size,
            hash_table_entries: hashes.len() as u32,
            block_table_entries: self.blocks.len() as u32,
            ..Header::default()
        };
        if let Some(index) = &self.extended {
            let [het, bet] = encode_extended(index, &self.blocks)?;
            header.het_table_offset = self.relative_position()?;
            header.table_sizes[3] = het.len() as u64;
            header.md5[4] = compute(&het).0;
            self.output.write_all(&het)?;
            raw::write(&het, header.raw_chunk_size, &mut self.output)?;
            header.bet_table_offset = self.relative_position()?;
            header.table_sizes[4] = bet.len() as u64;
            header.md5[3] = compute(&bet).0;
            self.output.write_all(&bet)?;
            raw::write(&bet, header.raw_chunk_size, &mut self.output)?;
        }
        header.hash_table_offset = self.relative_position()?;
        let table = |records: Vec<[u8; 16]>, name: &[u8]| {
            let mut bytes: Vec<u8> = records.into_iter().flatten().collect();
            crypt(&mut bytes, hash(name, 3), false);
            bytes
        };
        let hash_bytes = table(
            hashes.iter().copied().map(HashEntry::bytes).collect(),
            b"(hash table)",
        );
        header.table_sizes[0] = hash_bytes.len() as u64;
        header.md5[1] = compute(&hash_bytes).0;
        self.output.write_all(&hash_bytes)?;
        header.block_table_offset = self.relative_position()?;
        let block_bytes = table(
            self.blocks.iter().copied().map(BlockEntry::bytes).collect(),
            b"(block table)",
        );
        header.table_sizes[1] = block_bytes.len() as u64;
        header.md5[0] = compute(&block_bytes).0;
        self.output.write_all(&block_bytes)?;
        if self.blocks.iter().any(|b| b.offset > u32::MAX as u64) {
            header.hi_block_table_offset = self.relative_position()?;
            let high: Vec<u8> = self
                .blocks
                .iter()
                .flat_map(|b| ((b.offset >> 32) as u16).to_le_bytes())
                .collect();
            header.table_sizes[2] = high.len() as u64;
            header.md5[2] = compute(&high).0;
            self.output.write_all(&high)?;
        }
        header.archive_size = self.relative_position()?;
        let size = header.archive_size;
        // Validate the final size even when writing the index crosses a limit.
        if self.options.header_version == 0 && size > u32::MAX as u64 {
            return Err(Error::LimitExceeded("archive size"));
        }
        self.output.seek(SeekFrom::Start(self.base))?;
        header.write(&mut self.output)?;
        self.output.seek(SeekFrom::Start(self.base + size))?;
        self.output.flush()?;
        Ok(self.output)
    }
}

/// An entry sink implementing `io::Write`. Call `finish` after exactly the
/// declared number of bytes; dropping it leaves the parent writer poisoned.
pub struct EntryWriter<'a, W> {
    archive: &'a mut ArchiveWriter<W>,
    name: Vec<u8>,
    options: FileOptions,
    block: BlockEntry,
    raw: Option<Digests>,
    key: Option<u32>,
    remaining: u32,
    sector_size: u32,
    sector: u32,
    buffer: Vec<u8>,
    offsets: Vec<u32>,
    checksums: Vec<u32>,
    replace: bool,
    failed: bool,
}

impl<W: Write + Seek> EntryWriter<'_, W> {
    fn flush_sector(&mut self) -> Result<(), Error> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        let mut bytes = encode(&self.buffer, self.options.compression)?;
        if self.options.sector_checksums {
            self.checksums.push(sector_checksum(&bytes));
        }
        if let Some(key) = self.key {
            crypt(&mut bytes, key.wrapping_add(self.sector), false);
        }
        self.archive.output.write_all(&bytes)?;
        if let Some(raw) = &mut self.raw {
            raw.update(&bytes);
        }
        self.block.stored_size = self
            .block
            .stored_size
            .checked_add(bytes.len() as u32)
            .ok_or(Error::LimitExceeded("entry stored size"))?;
        if !self.offsets.is_empty() {
            self.offsets.push(self.block.stored_size);
        }
        self.sector += 1;
        self.buffer.clear();
        Ok(())
    }

    /// Flushes the final sector and commits this entry to its archive writer.
    /// Returns an error if the supplied byte count differs from the declared size or
    /// if encoding or I/O fails. The archive cannot finish after an entry failure.
    pub fn finish(mut self) -> Result<(), Error> {
        if self.failed {
            return Err(Error::WriterFailed);
        }
        if self.remaining != 0 {
            return Err(Error::SizeMismatch);
        }
        self.flush_sector()?;
        if !self.checksums.is_empty() {
            for checksum in &self.checksums {
                self.archive.output.write_all(&checksum.to_le_bytes())?;
                if let Some(raw) = &mut self.raw {
                    raw.update(&checksum.to_le_bytes());
                }
            }
            self.block.stored_size = self
                .block
                .stored_size
                .checked_add(self.checksums.len() as u32 * 4)
                .ok_or(Error::LimitExceeded("checksum storage"))?;
            self.offsets.push(self.block.stored_size);
        }
        let mut table = Vec::new();
        if !self.offsets.is_empty() {
            let end = self.archive.output.stream_position()?;
            let mut bytes: Vec<u8> = self.offsets.iter().flat_map(|n| n.to_le_bytes()).collect();
            if let Some(key) = self.key {
                crypt(&mut bytes, key.wrapping_sub(1), false);
            }
            self.archive
                .output
                .seek(SeekFrom::Start(self.archive.base + self.block.offset))?;
            self.archive.output.write_all(&bytes)?;
            self.archive.output.seek(SeekFrom::Start(end))?;
            table = bytes;
        }
        if let Some(raw) = self.raw.take() {
            raw.finish(&table, &mut self.archive.output)?;
        }
        self.archive.commit_entry(
            self.name,
            self.options.locale,
            self.options.platform,
            self.block,
            self.replace,
        )
    }
}

impl<W: Write + Seek> Write for EntryWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(Error::WriterFailed.into());
        }
        if bytes.len() > self.remaining as usize {
            self.failed = true;
            return Err(Error::SizeMismatch.into());
        }
        let count = bytes
            .len()
            .min(self.sector_size as usize - self.buffer.len());
        self.buffer.extend_from_slice(&bytes[..count]);
        self.remaining -= count as u32;
        if self.buffer.len() == self.sector_size as usize {
            if let Err(e) = self.flush_sector() {
                self.failed = true;
                return Err(e.into());
            }
        }
        Ok(count)
    }

    /// Flushes the underlying sink without finalizing a partial sector.
    fn flush(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(Error::WriterFailed.into());
        }
        let result = self.archive.output.flush();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}

#[cfg(test)]
#[path = "writer_tests.rs"]
mod tests;
