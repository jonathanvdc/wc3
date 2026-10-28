//! Explicit policies and diagnostics for translating model meaning.
//!
//! Native document adapters live in [`model::io`](crate::model::io). Profiles describe the
//! semantic vocabulary independently of the dialect selected for export.
mod context;
mod error;
pub use error::Error;
mod policy;
mod report;
mod semantic;
pub use context::*;
pub use policy::*;
pub use report::*;
pub use semantic::*;
