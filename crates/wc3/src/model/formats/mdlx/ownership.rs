//! Ownership implementations for MDLX records.
use crate::model::common::{Borrowed, Detached};
macro_rules! unchanged_ownership {
    ($($ty:ty),* $(,)?) => {$ (
        impl<'view> Borrowed<'view> for $ty {
            type View=Self;
            fn as_borrowed(&'view self)->Self { self.clone() }
        }
        impl<'owned> Detached<'owned> for $ty {
            type Detached=Self;
            fn into_detached(self)->Self {self}
        }
    )*};
}
use super::NoGeosetExtraSections;
unchanged_ownership!(NoGeosetExtraSections);
use super::materials::layer::{NoEmissiveGain, NoFresnel, NoLayerTextureSlots};
use super::scene::light::{NoFalloff, NoShadowRange};
unchanged_ownership!(
    NoEmissiveGain,
    NoFresnel,
    NoLayerTextureSlots,
    NoShadowRange,
    NoFalloff
);

use super::animation::Sequence;
unchanged_ownership!(Sequence);
