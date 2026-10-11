//! Read, edit, and write Warcraft III models, BLP textures, and MPQ archives.
//!
//! Start with [`model::Model`] for a known format version or [`model::DynamicModel`]
//! for files whose version is determined at runtime. Both use the same model types
//! for binary MDX and text MDL; import the corresponding codec traits to enable I/O.
//! See [`model`] for a complete example and guidance on editing records.
//! [`blp`] reads and writes BLP0, BLP1, and BLP2 encoded texture containers.
//! [`mpq`] indexes, streams, creates, and edits MPQ v1–v4 archives.
//!
//! # Optional pixel and compression codecs
//!
//! The crate has no default features. Model codecs and encoded BLP/MPQ container
//! operations are always available. Enable `blp-decode` to read RGBA pixels and
//! use the `image` decoder adapter; `blp-encode` adds pixel encoding. `mpq-decode`
//! enables compressed entry extraction, and `mpq-encode` adds zlib/bzip2 writing.
//! Stored MPQ entries and encoded copying do not require those features.
//!
//! # Ownership and output
//!
//! MDX models and BLP containers can borrow their encoded input. Their owned
//! conversion methods detach data for longer-lived storage; MDL parsing builds
//! owned records. See each module for its editing and round-trip guarantees.
//! Writers can leave partial output after transport errors. Complete encoding or
//! archive finalization before replacing a source file.

#![deny(missing_docs)]

extern crate self as wc3;

/// Warcraft III models and their MDX and MDL codecs.
pub mod model;

/// BLP0, BLP1, and BLP2 texture containers.
pub mod blp;

/// MPQ archive indexing and streaming entry I/O.
pub mod mpq;
