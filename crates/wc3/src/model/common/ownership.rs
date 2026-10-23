//! Explicit buffer ownership at the model boundary.
use glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};
use std::array::from_fn;
use std::borrow::Cow;
use zerovec::{ule::AsULE, ZeroVec};
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
unchanged_ownership!(
    (),
    bool,
    char,
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
    usize,
    isize,
    f32,
    f64,
    Vec2,
    Vec3,
    Vec4,
    Quat,
    Mat3,
    Mat4
);

impl<'owned> Detached<'owned> for Cow<'_, str> {
    type Detached = Cow<'owned, str>;
    fn into_detached(self) -> Self::Detached {
        Cow::Owned(self.into_owned())
    }
}

/// Converts component storage to an explicitly selected buffer lifetime.
///
/// This is the extension contract behind [`IntoOwned`]. Implementations copy
/// borrowed buffers and move owned buffers while preserving authored values and
/// identities. A custom profile payload `Payload<'data>` maps to
/// `Payload<'owned>`; a generic container maps each component that can borrow.
/// Implementing `Detached<'static>` also provides the blanket `IntoOwned` impl.
/// The resulting type can retain unchanged generic parameters; the profile's
/// exact payload-family bounds enforce the owned model's storage lifetime.
///
/// ```
/// use std::borrow::Cow;
/// use wc3::model::common::{Detached, IntoOwned};
///
/// struct Wrapper<T>(T);
/// impl<'owned, T: Detached<'owned>> Detached<'owned> for Wrapper<T> {
///     type Detached = Wrapper<T::Detached>;
///     fn into_detached(self) -> Self::Detached {
///         Wrapper(self.0.into_detached())
///     }
/// }
///
/// let source = vec![1_u8, 2, 3];
/// let view = Wrapper(Cow::Borrowed(source.as_slice()));
/// let owned: Wrapper<Cow<'static, [u8]>> = view.into_owned();
/// drop(source);
/// assert_eq!(owned.0.as_ref(), &[1, 2, 3]);
/// ```
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

impl<'view, T: AsULE + 'view> Borrowed<'view> for ZeroVec<'_, T> {
    type View = ZeroVec<'view, T>;
    fn as_borrowed(&'view self) -> Self::View {
        self.as_slice().as_zerovec()
    }
}
impl<'owned, T: AsULE + 'owned> Detached<'owned> for ZeroVec<'_, T> {
    type Detached = ZeroVec<'owned, T>;
    fn into_detached(self) -> Self::Detached {
        self.into_owned()
    }
}
impl<'view, T: Clone + 'view> Borrowed<'view> for Cow<'_, [T]> {
    type View = Cow<'view, [T]>;
    fn as_borrowed(&'view self) -> Self::View {
        Cow::Borrowed(self.as_ref())
    }
}
impl<'owned, T: Clone + 'owned> Detached<'owned> for Cow<'_, [T]> {
    type Detached = Cow<'owned, [T]>;
    fn into_detached(self) -> Self::Detached {
        Cow::Owned(self.into_owned())
    }
}

impl<'view, T: Borrowed<'view> + 'view, const N: usize> Borrowed<'view> for [T; N] {
    type View = [T::View; N];
    fn as_borrowed(&'view self) -> Self::View {
        from_fn(|index| self[index].as_borrowed())
    }
}
impl<'owned, T: Detached<'owned>, const N: usize> Detached<'owned> for [T; N] {
    type Detached = [T::Detached; N];
    fn into_detached(self) -> Self::Detached {
        self.map(Detached::into_detached)
    }
}

macro_rules! tuple_ownership {
    ($($ty:ident : $index:tt),+) => {
        impl<'view, $($ty: Borrowed<'view>),+> Borrowed<'view> for ($($ty,)+) {
            type View = ($($ty::View,)+);
            fn as_borrowed(&'view self) -> Self::View { ($(self.$index.as_borrowed(),)+) }
        }
        impl<'owned, $($ty: Detached<'owned>),+> Detached<'owned> for ($($ty,)+) {
            type Detached = ($($ty::Detached,)+);
            fn into_detached(self) -> Self::Detached { ($(self.$index.into_detached(),)+) }
        }
    };
}
tuple_ownership!(A: 0);
tuple_ownership!(A: 0, B: 1);
tuple_ownership!(A: 0, B: 1, C: 2);
tuple_ownership!(A: 0, B: 1, C: 2, D: 3);

use std::collections::BTreeMap;
impl<'view, K: Ord + Clone, V: Borrowed<'view>> Borrowed<'view> for BTreeMap<K, V> {
    type View = BTreeMap<K, V::View>;
    fn as_borrowed(&'view self) -> Self::View {
        self.iter()
            .map(|(key, value)| (key.clone(), value.as_borrowed()))
            .collect()
    }
}
impl<'owned, K: Ord, V: Detached<'owned>> Detached<'owned> for BTreeMap<K, V> {
    type Detached = BTreeMap<K, V::Detached>;
    fn into_detached(self) -> Self::Detached {
        self.into_iter()
            .map(|(key, value)| (key, value.into_detached()))
            .collect()
    }
}
