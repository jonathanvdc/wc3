//! Read, edit, and write Blizzard models, BLP textures, and MPQ archives.
//!
//! Use [`model`] for Warcraft III MDX/MDL and World of Warcraft M2/WMO models,
//! [`blp`] for textures, and [`mpq`] for archives. The library has no renderer
//! dependency; model poses and surfaces can be evaluated on the CPU.
//!
//! # Load a model
//!
//! For checked model editing, import a file into [`model::Model`]. This example
//! loads a self-contained Warcraft III model whose values outlive the input:
//!
//! ```
//! use wc3::model::{conversion::ImportOptions, io};
//!
//! let source = r#"Version { FormatVersion 800, } Model "Example" {}"#;
//! let imported = io::mdlx::decode_mdl(source, &ImportOptions::default())?;
//! assert_eq!(imported.value.name(), "Example");
//! # Ok::<(), wc3::model::io::mdlx::ReadError>(())
//! ```
//!
//! [`model::Model::edit`] validates changes together before publishing them.
//! [`model::io`] imports native documents and plans native output, with explicit
//! loss policies and reports from [`model::conversion`]. A model's
//! [`profile`](model::profiles) selects game semantics independently of the file
//! revision.
//!
//! For native record editing and unknown-data preservation, use
//! [`model::formats`] instead. Warcraft III uses [`model::formats::mdlx::Document`]
//! or [`model::formats::mdlx::DynamicDocument`]; MDX and MDL share their records.
//! WoW uses [`model::formats::m2`] and [`model::formats::wmo`], including bundles
//! for resolving and writing supporting files. Consult each format's reference
//! for supported layouts and round-trip guarantees.
//!
//! # Textures, archives, and features
//!
//! [`blp::Blp`] reads and writes BLP0, BLP1, and BLP2 encoded containers.
//! [`mpq::Archive`] reads seekable archives, [`mpq::SharedArchive`] supports
//! concurrent reads, and [`mpq::ArchiveWriter`] creates or edits archives.
//!
//! There are no default features. Enable `blp-decode` for RGBA pixels and the
//! `image` decoder adapter, or `blp-encode` to encode images. `mpq-decode` enables
//! compressed entry extraction; `mpq-encode` adds zlib/bzip2 writing. Model codecs,
//! stored MPQ entries, and encoded container editing need none of these features.
//! Enable `popcorn-jit` for optional native execution of baked Popcorn effects on
//! supported hosts; the Rust evaluator remains available without it.
//!
//! # Ownership and writing
//!
//! Native MDX, M2, and WMO documents and BLP containers can borrow their encoded
//! input. Convert them to owned storage when they must outlive it. MDL parsing
//! builds owned records. Checked model adapters offer their own borrowing and
//! owned import choices.
//!
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
