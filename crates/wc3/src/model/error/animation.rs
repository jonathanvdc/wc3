//! Animation failures in checked semantic models.
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// A animation constraint that prevents checked model use.
#[derive(Clone, Debug)]
pub enum AnimationError {}
impl Display for AnimationError {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}
impl StdError for AnimationError {}
