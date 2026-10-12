//! Read, edit, and write Warcraft models, BLP textures, and MPQ archives.
//!
//! Start with [`model::mdlx::Model`] for a known format version or [`model::mdlx::DynamicModel`]
//! for files whose version is determined at runtime. Both use the same model types
//! for binary MDX and text MDL; import the corresponding codec traits to enable I/O.
//! See [`model::mdlx`] for a complete example and guidance on editing records.
//!
//! World of Warcraft models use [`model::m2::Model`] or [`model::m2::DynamicModel`].
//! Their native skin, animation, skeleton, bone, and physics codecs retain separate
//! documents; [`model::m2::Bundle`] resolves declared references and coordinates
//! external animation output through caller-owned storage.
//!
//! World models use [`model::wmo::Root`] and [`model::wmo::Group`], with explicit
//! client profiles and [`model::wmo::Bundle`] for shared root/group references.
//! StarCraft II and Heroes of the Storm models use [`model::m3::Model`], whose
//! MD33/MD34 envelopes keep independently versioned indexed sections and external animations.
//!
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
//! MDX, M2, WMO, and M3 documents and BLP containers can borrow their encoded input. Their
//! owned conversion methods detach data for longer-lived storage; MDL parsing builds
//! owned records. See each module for its editing and round-trip guarantees.
//! Writers can leave partial output after transport errors. Complete encoding or
//! archive finalization before replacing a source file.

#![deny(missing_docs)]

extern crate self as wc3;

/// Warcraft III MDX/MDL, World of Warcraft M2/WMO, and StarCraft II M3 formats.
pub mod model;

/// BLP0, BLP1, and BLP2 texture containers.
pub mod blp;

/// MPQ archive indexing and streaming entry I/O.
pub mod mpq;
