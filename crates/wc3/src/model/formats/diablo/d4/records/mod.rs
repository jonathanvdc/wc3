//! Concrete Diablo IV record layouts for the supported format hashes.
//! Maintained declarations select wire field order and descriptor encodings.
use super::codec::{Decoder, Encoder, Space};
use super::{EmptyTagMap, Read, ReadError, RecordLayout, Write, WriteError};
use crate::model::common::Ownership;
use crate::model::formats::diablo::d4;
use std::borrow::Cow;
use zerovec::ZeroVec;

/// Native appearance records.
pub mod appearance;
pub use appearance::*;
/// Native wind records.
pub mod wind;
pub use wind::*;
/// Native common records.
pub mod common;
pub use common::*;
/// Native animation records.
pub mod animation;
pub use animation::*;
/// Native physics records.
pub mod physics;
pub use physics::*;
/// Native actor records.
pub mod actor;
pub use actor::*;
/// Native events records.
pub mod events;
pub use events::*;
/// Native materials records.
pub mod materials;
pub use materials::*;
mod polymorphic;
pub use polymorphic::Polymorphic;
