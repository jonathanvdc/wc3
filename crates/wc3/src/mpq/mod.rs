//! Index, extract, create, and edit MPQ archives (formats v1 through v4).
//!
//! [`Archive`](crate::mpq::Archive) reads entries from a seekable source, while [`SharedArchive`](crate::mpq::SharedArchive) opens
//! independent readers for concurrent extraction from a random-access source.
//! Both decode the archive index when opened and read payloads on demand.
//! [`ArchiveWriter`](crate::mpq::ArchiveWriter) writes entries to a seekable destination and commits the
//! tables and header on `finish`. Entry streaming bounds payload memory; archive
//! sources and destinations still require random access.
//!
//! Container indexing, stored entries, encryption, and encoded copying have no
//! optional dependencies. `mpq-decode` adds zlib, bzip2, PKWARE DCL, sparse,
//! Huffman, mono/stereo ADPCM, and LZMA decoding (including supported compression
//! chains). `mpq-encode` adds zlib and bzip2 encoding. All compression
//! dependencies use Rust implementations.
//! Sector checksums are verified by default and can be written for compressed
//! entries. All four header versions, 64-bit offsets, compressed index tables,
//! and HET/BET lookup are supported. V4 header/table MD5s are checked when present;
//! raw file chunks are checked when opening a file. Compressed tables require
//! `mpq-decode`. HET/BET filename lookup is neutral-locale/platform only.
//! Writers default to classic headers; use [`WriteOptions`](crate::mpq::WriteOptions) to select newer
//! headers, HET/BET indexes, or v4 raw chunk digests. Patch-file semantics and
//! signature verification are not implemented. Use [`ReadMode::Permissive`](crate::mpq::ReadMode::Permissive) for
//! bounded recovery of known malformed classic MPQs. Valid extended formats
//! retain strict checks; recovered archives cannot be edited. Recovery does not
//! restore missing filenames or deleted editor data. [`Archive::diagnostics`](crate::mpq::Archive::diagnostics)
//! reports indexing and payload recoveries (at most 1024 records plus a notice).
//! [`Archive::open_file_by_index`](crate::mpq::Archive::open_file_by_index) can extract unnamed entries, with limited
//! encryption-key recovery from sector tables or RIFF/EXE/XML signatures in
//! permissive mode. Recovery is limited to these documented cases.
//!
//! # Streaming a file
//!
//! ```
//! use std::io::{Cursor, Read};
//! use wc3::mpq::{Archive, ArchiveWriter, FileOptions, WriteOptions};
//!
//! let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default())?;
//! writer.add_file(b"war3map.j", 5, &mut b"hello".as_slice(), FileOptions::default())?;
//! let bytes = writer.finish()?.into_inner();
//! let mut archive = Archive::open(Cursor::new(bytes))?;
//! let mut entry = archive.open_file(b"WAR3MAP.J")?;
//! let mut text = String::new();
//! entry.read_to_string(&mut text)?;
//! assert_eq!(text, "hello");
//! # Ok::<(), wc3::mpq::Error>(())
//! ```
//!
//! Entry names are byte strings, ASCII case-insensitive, with slash normalization.
//! Lookup is exact by locale/platform (neutral defaults); names in `(listfile)`
//! are hints, not a complete directory. Index metadata can be inspected without
//! extracting any file, and [`Archive::encoded_file`](crate::mpq::Archive::encoded_file) accesses raw payloads by
//! block index even if their names or codecs are unknown.
//!
//! # Concurrent extraction
//!
//! Use [`SharedArchive`](crate::mpq::SharedArchive) when multiple entries need to be read at the same time.
//! Its clones share the parsed index, and `open_file` takes `&self`. Each entry
//! reader owns its decoding state and keeps the source alive until it is dropped.
//! The application supplies threads or other worker scheduling and controls how
//! many reads run at once.
//!
//! Files use positional reads on Unix and Windows. `Arc<[u8]>` provides a shared
//! memory source; implement [`ReadAt`](crate::mpq::ReadAt) for other sources. The following example
//! reads two entries on separate threads from the same archive:
//!
//! ```no_run
//! use std::fs::File;
//! use std::thread;
//! use wc3::mpq::{Error, SharedArchive};
//!
//! let archive = SharedArchive::open(File::open("map.w3x")?)?;
//! let (script, strings) = thread::scope(|scope| {
//!     let script = scope.spawn(|| archive.read_file("war3map.j"));
//!     let strings = scope.spawn(|| archive.read_file("war3map.wts"));
//!     Ok::<_, Error>((script.join().unwrap()?, strings.join().unwrap()?))
//! })?;
//! # Ok::<(), Error>(())
//! ```
//!
//! `read_file` allocates the complete decoded entry. For streaming, use `open_file`
//! and the standard I/O `Read` trait. Configure [`ReadOptions`](crate::mpq::ReadOptions) through
//! [`SharedArchive::with_options`](crate::mpq::SharedArchive::with_options) to limit allocations and select recovery policy.
//! Keep source contents unchanged for the lifetime of the archive and its readers.
//! Indexing recoveries are available from [`SharedArchive::diagnostics`](crate::mpq::SharedArchive::diagnostics); payload
//! recoveries are reported by [`SharedEntryReader::diagnostics`](crate::mpq::SharedEntryReader::diagnostics).
//!
//! For a source that only implements `Read + Seek`, [`Archive::into_shared`](crate::mpq::Archive::into_shared)
//! preserves its parsed index and options. The [`SeekSource`](crate::mpq::SeekSource) adapter serializes
//! source seek/read operations while allowing entry decompression to overlap.
//!
//! # Copying encoded files into a fresh archive
//!
//! [`Archive::open_encoded_file`](crate::mpq::Archive::open_encoded_file) pairs a bounded raw payload stream with
//! [`EncodedFileMetadata`](crate::mpq::EncodedFileMetadata), which carries sizes, storage flags, source sector
//! size, and locale/platform without retaining original offsets or block IDs.
//! [`ArchiveWriter::add_encoded_file`](crate::mpq::ArchiveWriter::add_encoded_file) imports that representation into a new
//! archive. [`ArchiveWriter::copy_file_from`](crate::mpq::ArchiveWriter::copy_file_from) combines lookup and import when
//! the destination name is unchanged. Both operations are available without
//! compression features, including for unsupported compression masks.
//!
//! ```
//! use std::io::Cursor;
//! use wc3::mpq::{Archive, ArchiveWriter, FileOptions, WriteOptions};
//!
//! let mut original = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default())?;
//! original.add_file("file.txt", 5, &mut b"hello".as_slice(), FileOptions::default())?;
//! let mut source = Archive::open(Cursor::new(original.finish()?.into_inner()))?;
//! let mut destination = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default())?;
//! destination.copy_file_from(&mut source, "file.txt")?;
//! let bytes = destination.finish()?.into_inner();
//! assert_eq!(Archive::open(Cursor::new(bytes))?.read_file("file.txt")?, b"hello");
//! # Ok::<(), wc3::mpq::Error>(())
//! ```
//!
//! Encoded import rejects encrypted entries and incompatible sector framing.
//! Uncompressed and single-unit files can move between sector sizes; compressed
//! sector files require matching sector sizes. Destination indexes and raw chunk
//! digests are generated normally. Entries retain no obsolete payloads from the
//! source archive. Copied compressed contents are not decoded or validated;
//! callers must supply metadata matching the payload and verify content as needed.
//! Source raw chunk MD5s are checked by `open_encoded_file`. Short streams and
//! payload I/O failures prevent destination completion, just as decoded writes do.
//! [`EncodedEntry::new`](crate::mpq::EncodedEntry::new) also supports importing standalone encoded cache entries.
//!
//! # Editing without recompressing
//!
//! [`ArchiveWriter::from_archive`](crate::mpq::ArchiveWriter::from_archive) copies the original encoded archive region
//! with bounded memory and retains its hash slots and block offsets. Replacement
//! files append new blocks; deletions retain tombstones. Unknown names/codecs and
//! offset-adjusted encrypted files survive without decryption. This preserves
//! unused bytes rather than compacting, cannot grow the original hash table,
//! and leaves the original listfile unchanged unless you replace it explicitly.
//! HET/BET edits compact block records and remap IDs while retaining payload
//! offsets and unknown filename hashes.
//! Outer map prefixes, user-data wrappers, and signatures outside the archive
//! region are not copied. Existing signatures/attributes may become stale after
//! edits; managing those special files is the caller's responsibility.
//!
//! Successful `finish` is required on both entry and archive writers. A failed
//! or abandoned entry prevents archive completion. Output is not transactional:
//! use a separate destination and replace a source file only after success.

#[cfg(feature = "mpq-decode")]
mod adpcm;
mod codec;
#[cfg(all(test, feature = "mpq-decode"))]
mod codec_test_vectors;
mod compat;
mod crypto;
mod encoded;
mod error;
mod extended;
mod format;
#[cfg(feature = "mpq-decode")]
mod huffman;
#[cfg(feature = "mpq-decode")]
mod huffman_tables;
mod raw;
mod reader;
mod shared;
mod writer;

pub use compat::{ReadMode, RecoveryDiagnostic};
pub use encoded::{EncodedEntry, EncodedFileMetadata};
pub use error::Error;
pub use extended::ExtendedIndex;
pub use format::{BlockEntry, FileFlags, HashEntry, Header, Index};
pub use reader::{Archive, EntryReader, ReadOptions};
pub use shared::{ReadAt, SeekSource, SharedArchive, SharedEntryReader};
pub use writer::{ArchiveWriter, Compression, EntryWriter, FileOptions, WriteOptions};

#[cfg(test)]
mod permissive_tests;
