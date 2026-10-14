//! Optional native fields compose independently selected record layouts.
use crate::model::m3::{self as native, records as nr};
/// Tests unresolved source fields without interpreting their contents.
pub(super) trait Unresolved {
    fn populated(self) -> bool;
}
impl Unresolved for () {
    fn populated(self) -> bool {
        false
    }
}
impl<const N: usize> Unresolved for nr::FixedBytes<N> {
    fn populated(self) -> bool {
        self.0.iter().any(|byte| *byte != 0)
    }
}
impl<T: native::Target> Unresolved for native::Ref<T> {
    fn populated(self) -> bool {
        !self.is_empty()
    }
}
impl<const N: usize> Unresolved for [u32; N] {
    fn populated(self) -> bool {
        self.iter().any(|word| *word != 0)
    }
}
impl Unresolved for u32 {
    fn populated(self) -> bool {
        self != 0
    }
}
pub(super) trait Slot<T> {
    fn slot(self) -> Option<T>;
}
impl<T> Slot<T> for () {
    fn slot(self) -> Option<T> {
        None
    }
}
macro_rules! scalar {($($t:ty),*)=>{$(impl Slot<$t> for $t {fn slot(self)->Option<Self>{Some(self)}})*};}
scalar!(f32, u32, i32);
macro_rules! enums {($($t:ty),*)=>{$(impl Slot<u32> for $t {fn slot(self)->Option<u32>{Some(self.raw())}})*};}
enums!(nr::Smoothing, nr::Variation);
macro_rules! flags {($($t:ty),*)=>{$(impl Slot<u32> for $t {fn slot(self)->Option<u32>{Some(self.bits())}})*};}
flags!(
    nr::ParticleAdditionalFlags,
    nr::ParticleRotationFlags,
    nr::RibbonAdditionalFlags
);
impl<T> Slot<nr::AnimationValue<T>> for nr::AnimationValue<T> {
    fn slot(self) -> Option<Self> {
        Some(self)
    }
}
impl<T: native::Target> Slot<native::Ref<T>> for native::Ref<T> {
    fn slot(self) -> Option<Self> {
        Some(self)
    }
}

pub(super) trait Assign<T> {
    const PRESENT: bool;
    fn assign(value: T) -> Self;
}
impl<T> Assign<T> for () {
    const PRESENT: bool = false;
    fn assign(_: T) {}
}
macro_rules! assign_scalar {($($t:ty),*)=>{$(impl Assign<$t> for $t {const PRESENT:bool=true; fn assign(value:Self)->Self{value}})*};}
assign_scalar!(f32, u32, i32);
macro_rules! assign_enum {($($t:ty),*)=>{$(impl Assign<u32> for $t {const PRESENT:bool=true;fn assign(value:u32)->Self{Self::from_raw(value)}})*};}
assign_enum!(nr::Smoothing, nr::Variation);
macro_rules! assign_flags {($($t:ty),*)=>{$(impl Assign<u32> for $t {const PRESENT:bool=true;fn assign(value:u32)->Self{value.into()}})*};}
assign_flags!(
    nr::ParticleAdditionalFlags,
    nr::ParticleRotationFlags,
    nr::RibbonAdditionalFlags
);
impl<T> Assign<nr::AnimationValue<T>> for nr::AnimationValue<T> {
    const PRESENT: bool = true;
    fn assign(value: Self) -> Self {
        value
    }
}
impl<T: native::Target> Assign<native::Ref<T>> for native::Ref<T> {
    const PRESENT: bool = true;
    fn assign(value: Self) -> Self {
        value
    }
}

impl<const N: usize> Slot<[f32; N]> for [f32; N] {
    fn slot(self) -> Option<Self> {
        Some(self)
    }
}
