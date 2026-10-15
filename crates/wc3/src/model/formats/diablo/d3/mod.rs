//! Diablo III model SNO documents with independently selected native revisions.
//!
//! `Appearance<V260>`, `Animation<V118>`, and the other document families check
//! their own header revision. They do not infer a layout from a game patch.
//! Fixed leaf arrays borrow encoded bytes; metadata records resolve their
//! descriptors and writers relocate every supported reference.
//!
//! ```
//! use wc3::model::formats::diablo::d3::{Appearance, V260};
//! let source = Appearance::<V260>::default();
//! let bytes = source.write()?;
//! let decoded = Appearance::<V260>::read(&bytes)?;
//! assert_eq!(decoded.root.bones.len(), 0);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! A revision marker from another document family cannot select its layout:
//!
//! ```compile_fail
//! use wc3::model::formats::diablo::d3::{Appearance, V118};
//! let _ = Appearance::<V118>::default();
//! ```
pub mod codec;
pub use codec::{Read, RecordLayout, Write};
mod document;
pub mod records;
pub use document::*;
mod error;
pub use error::{ReadError, WriteError};
mod geometry;

/// Creates a detached native document or a borrowed view using shared model ownership.
pub use crate::model::common::{Borrowed, IntoOwned};
