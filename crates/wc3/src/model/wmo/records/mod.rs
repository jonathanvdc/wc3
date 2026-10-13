//! Fixed native records grouped by the documents that own them.
pub mod root;
pub use root::*;
pub mod group;
pub use group::*;

pub mod flags;
pub use flags::*;
pub mod values;
pub use values::*;
