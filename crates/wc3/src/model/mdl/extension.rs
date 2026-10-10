//! Application block dispatch.
use super::Parser;
use crate::model::mdl;

/// Dispatches application block names to their MDL readers.
pub trait Extension: Sized {
    /// Reads a recognized block with the parser positioned at its opening identifier.
    ///
    /// Returns `None` without consuming input for an unrecognized name, and an
    /// error for a malformed recognized block.
    fn read_extension(name: &str, parser: &mut Parser<'_>) -> Result<Option<Self>, mdl::ReadError>;
}
