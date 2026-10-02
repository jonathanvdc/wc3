//! Index, extract, create, and edit MPQ archives (formats v1 through v4).
//!
//! [`Archive`](crate::mpq::Archive) loads the header and encrypted hash/block tables, then opens
//! payloads on demand. [`ArchiveWriter`](crate::mpq::ArchiveWriter) streams entries to a seekable sink and
//! commits the tables/header on `finish`. Both use the same [`Index`](crate::mpq::Index), hashing,
//! cryptography, and sector framing. Sources and sinks need seeking; streaming
//! means bounded payload memory, not forward-only archive I/O.
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
//! Writers default to classic headers; use [`WriteOptions`] to select newer
//! headers, HET/BET indexes, or v4 raw chunk digests. Patch-file semantics and
//! signature verification are not implemented. Protected/malformed map repair
//! is deliberately outside the strict reader's contract.
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
mod crypto;
mod error;
mod extended;
mod format;
#[cfg(feature = "mpq-decode")]
mod huffman;
#[cfg(feature = "mpq-decode")]
mod huffman_tables;
mod raw;
mod reader;
mod writer;

pub use error::Error;
pub use extended::ExtendedIndex;
pub use format::{BlockEntry, FileFlags, HashEntry, Header, Index};
pub use reader::{Archive, EntryReader, ReadOptions};
pub use writer::{ArchiveWriter, Compression, EntryWriter, FileOptions, WriteOptions};
