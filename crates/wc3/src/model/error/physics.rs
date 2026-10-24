//! Physics failures in checked semantic models.
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// A physics constraint that prevents checked model use.
#[derive(Clone, Debug)]
pub enum PhysicsError {}
impl Display for PhysicsError {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}
impl StdError for PhysicsError {}
