//! Explicit buffer ownership at the model boundary.
/// Creates a view borrowing the buffers held by a record, including owned buffers.
pub trait Borrowed<'view> {
    /// The record with its buffer lifetimes shortened to the view.
    type View;
    /// Borrows buffers and copies the record's metadata.
    fn as_borrowed(&'view self) -> Self::View;
}
/// Detaches a record from its source bytes, moving buffers that it already owns.
pub trait IntoOwned {
    /// The same record with independent buffer ownership.
    type Owned;
    /// Copies borrowed buffers and moves owned buffers.
    fn into_owned(self) -> Self::Owned;
}
impl<'view, T: Borrowed<'view>> Borrowed<'view> for Vec<T> {
    type View = Vec<T::View>;
    fn as_borrowed(&'view self) -> Self::View {
        self.iter().map(Borrowed::as_borrowed).collect()
    }
}
impl<'owned, T: Detached<'owned>> Detached<'owned> for Vec<T> {
    type Detached = Vec<T::Detached>;
    fn into_detached(self) -> Self::Detached {
        self.into_iter().map(Detached::into_detached).collect()
    }
}
impl<'view, T: Borrowed<'view>> Borrowed<'view> for Option<T> {
    type View = Option<T::View>;
    fn as_borrowed(&'view self) -> Self::View {
        self.as_ref().map(Borrowed::as_borrowed)
    }
}
impl<'owned, T: Detached<'owned>> Detached<'owned> for Option<T> {
    type Detached = Option<T::Detached>;
    fn into_detached(self) -> Self::Detached {
        self.map(Detached::into_detached)
    }
}
impl<'view, T: Borrowed<'view>> Borrowed<'view> for Box<T> {
    type View = Box<T::View>;
    fn as_borrowed(&'view self) -> Self::View {
        Box::new((**self).as_borrowed())
    }
}
impl<'owned, T: Detached<'owned>> Detached<'owned> for Box<T> {
    type Detached = Box<T::Detached>;
    fn into_detached(self) -> Self::Detached {
        Box::new((*self).into_detached())
    }
}

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
unchanged_ownership!(NoGeosetExtraSections, u8, u16, u32, i32, f32);
use super::materials::layer::{NoEmissiveGain, NoFresnel, NoLayerTextureSlots};
use super::scene::light::{NoFalloff, NoShadowRange};
unchanged_ownership!(
    NoEmissiveGain,
    NoFresnel,
    NoLayerTextureSlots,
    NoShadowRange,
    NoFalloff
);

/// Internal ownership conversion preserving an explicitly selected buffer lifetime.
#[doc(hidden)]
pub trait Detached<'owned> {
    /// Detached record type.
    type Detached;
    /// Moves owned buffers and copies borrowed buffers.
    fn into_detached(self) -> Self::Detached;
}
impl<T: Detached<'static>> IntoOwned for T {
    type Owned = T::Detached;
    fn into_owned(self) -> Self::Owned {
        self.into_detached()
    }
}

use super::animation::Sequence;
unchanged_ownership!(Sequence);
