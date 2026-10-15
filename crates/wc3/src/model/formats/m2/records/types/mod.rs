//! Typed native wire choices and behavior bits.
use crate::model::{common::Ownership, formats::m2};
use bitfield::bitfield;
use zerovec::ule::AsULE;

macro_rules! wire_ule {
    ($ty:ty, $raw:ty, $get:ident, $make:ident) => {
        impl AsULE for $ty {
            type ULE = <$raw as AsULE>::ULE;
            fn to_unaligned(self) -> Self::ULE {
                self.$get().to_unaligned()
            }
            fn from_unaligned(value: Self::ULE) -> Self {
                Self::$make(<$raw>::from_unaligned(value))
            }
        }
    };
}

mod animation;
pub use animation::*;
mod materials;
pub use materials::*;
mod scene;
pub use scene::*;
mod particles;
pub use particles::*;
mod physics;
pub use physics::*;
mod flags;
pub use flags::*;
