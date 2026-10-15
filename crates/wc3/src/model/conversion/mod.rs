//! Explicit policies, diagnostics, and plans for translating model meaning.
//!
//! Native document adapters live in [`model::io`](crate::model::io). Profiles describe the
//! semantic vocabulary independently of the dialect selected for export.
mod context;
mod plan;
mod policy;
mod report;
pub use context::*;
pub use plan::*;
pub use policy::*;
pub use report::*;
