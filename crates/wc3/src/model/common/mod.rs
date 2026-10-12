//! Buffer ownership and encoded array helpers shared by model formats.
mod ownership;
pub use ownership::{Borrowed, Detached, IntoOwned};
mod into_zero_vec;
pub use into_zero_vec::IntoZeroVec;
pub use zerovec::{ule::AsULE, ZeroVec};

pub mod binary;

pub use wc3_derive::{CommonPartialEq, Ownership};

mod io_error;
pub use io_error::IoError;
