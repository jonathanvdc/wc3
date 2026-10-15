//! Read, edit, and write Blizzard models, BLP/TEX textures, and MPQ archives.
//!
//! Start with [`model::Model`] for checked authoring values, relationships, and
//! shared resources. Its [`model::profiles`] select game-specific semantics independently
//! of native file layouts. [`model::io`] imports native documents and plans exports;
//! [`model::conversion`] supplies explicit representability policies and diagnostics.
//!
//! Native documents live under [`model::formats`]. Use
//! [`model::formats::mdlx::Document`] for a known Warcraft III format version or
//! [`model::formats::mdlx::DynamicDocument`] for runtime version selection. Both share
//! native record types for binary MDX and text MDL; import their codec traits for I/O.
//! Retain native documents for record inspection, opaque data, and codec round trips.
//!
//! World of Warcraft models use [`model::formats::m2::Document`] or [`model::formats::m2::DynamicDocument`].
//! Their native skin, animation, skeleton, bone, and physics codecs retain separate
//! documents; [`model::formats::m2::Bundle`] resolves declared references and coordinates
//! external animation output through caller-owned storage.
//!
//! World models use [`model::formats::wmo::Root`] and [`model::formats::wmo::Group`], with explicit
//! client profiles and [`model::formats::wmo::Bundle`] for shared root/group references.
//! StarCraft II and Heroes of the Storm models use [`model::formats::m3::Document`], whose
//! MD33/MD34 envelopes keep independently versioned indexed sections and external animations.
//!
//! Diablo III and IV resources use [`model::formats::diablo`], with independent native
//! root layouts and caller-supplied external payloads. [`tex`] handles their
//! encoded texture containers and optional pixel conversion.
//!
//! [`blp`] reads and writes BLP0, BLP1, and BLP2 encoded texture containers.
//! [`mpq`] indexes, streams, creates, and edits MPQ v1–v4 archives.
//!
//! # Optional pixel and compression codecs
//!
//! The crate has no default features. Document codecs and encoded BLP/MPQ container
//! and TEX operations are always available. Enable `blp-decode` to read RGBA pixels and
//! use the `image` decoder adapter; `blp-encode` adds pixel encoding. `mpq-decode`
//! enables compressed entry extraction, and `mpq-encode` adds zlib/bzip2 writing.
//! Stored MPQ entries and encoded copying do not require those features.
//! `tex-decode` and `tex-encode` enable supported Diablo pixel conversions.
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

/// Checked model graphs, authoring profiles, and native format codecs.
pub mod model;

/// BLP0, BLP1, and BLP2 texture containers.
pub mod blp;

/// MPQ archive indexing and streaming entry I/O.
pub mod mpq;

/// Diablo III and Diablo IV TEX texture containers.
pub mod tex;
