//! Geometry failures in checked semantic models.
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// A geometry constraint that prevents checked model use.
#[derive(Clone, Debug)]
pub enum GeometryError {}
impl Display for GeometryError {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}
impl StdError for GeometryError {}
