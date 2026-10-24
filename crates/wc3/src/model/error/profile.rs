//! Profile failures in checked semantic models.
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// A profile constraint that prevents checked model use.
#[derive(Clone, Debug)]
pub enum ProfileError {}
impl Display for ProfileError {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}
impl StdError for ProfileError {}
