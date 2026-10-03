//! Read, edit, and write Warcraft III models, BLP textures, and MPQ archives.
//!
//! Start with [`model::Model`] for a known format version or [`model::DynamicModel`]
//! for files whose version is determined at runtime. Both use the same model types
//! for binary MDX and text MDL; import the corresponding codec traits to enable I/O.
//! See [`model`] for a complete example and guidance on editing records.
//! [`blp`] reads and writes BLP1 and BLP2 encoded texture containers.
//! [`mpq`] indexes, streams, creates, and edits MPQ v1–v4 archives.

#![deny(missing_docs)]

extern crate self as wc3;

/// Warcraft III models and their MDX and MDL codecs.
pub mod model;

/// BLP1 and BLP2 texture containers.
pub mod blp;

/// MPQ archive indexing and streaming entry I/O.
pub mod mpq;
