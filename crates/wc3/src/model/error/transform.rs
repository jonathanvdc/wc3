//! Transform failures in checked semantic models.
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// A transform constraint that prevents checked model use.
#[derive(Clone, Debug)]
pub enum TransformError {}
impl Display for TransformError {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}
impl StdError for TransformError {}
