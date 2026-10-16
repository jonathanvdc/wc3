//! Errors shared by semantic compilation and asset decoding.
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A model decoding, strict conversion, or geometry preparation failure.
#[derive(Debug)]
pub struct ModelError(pub(crate) String);

impl Display for ModelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.0)
    }
}

impl Error for ModelError {}
