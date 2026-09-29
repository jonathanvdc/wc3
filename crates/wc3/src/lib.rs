//! Read, edit, and write Warcraft III models.
//!
//! Start with [`model::Model`] for a known format version or [`model::DynamicModel`]
//! for files whose version is determined at runtime. Both use the same model types
//! for binary MDX and text MDL; import the corresponding codec traits to enable I/O.
//! See [`model`] for a complete example and guidance on editing records.

extern crate self as wc3;

/// Warcraft III models and their MDX and MDL codecs.
pub mod model;
