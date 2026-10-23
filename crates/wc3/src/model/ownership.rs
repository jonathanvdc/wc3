//! Ownership of shared semantic values and composed authored records.
use crate::model::common::Detached;
use crate::model::profiles::{Either, NoProperties, Unavailable};
use crate::model::{
    Animation, Bounds, ClipProperties, Curve, CurveValue, DynamicTrack, Id, MeshProperties,
    NodeProperties, Property, PropertyValue, Sampler, SharedClock, Texture, TextureTransform, Time,
    TimeSource, Track,
};

macro_rules! unchanged {
    ($($ty:ty),* $(,)?) => {$ (
        impl<'owned> Detached<'owned> for $ty {
            type Detached = Self;
            fn into_detached(self) -> Self { self }
        }
    )*};
}
unchanged!(
    Bounds,
    Time,
    TimeSource,
    SharedClock,
    Texture,
    Sampler,
    TextureTransform,
    NoProperties,
    Unavailable,
    NodeProperties,
    MeshProperties,
    ClipProperties,
);
impl<'owned, K> Detached<'owned> for Id<K> {
    type Detached = Self;
    fn into_detached(self) -> Self {
        self
    }
}
impl<'owned, T: PropertyValue> Detached<'owned> for Property<T> {
    type Detached = Self;
    fn into_detached(self) -> Self {
        self
    }
}
impl<'owned, A: Detached<'owned>, B: Detached<'owned>> Detached<'owned> for Either<A, B> {
    type Detached = Either<A::Detached, B::Detached>;
    fn into_detached(self) -> Self::Detached {
        match self {
            Self::Left(value) => Either::Left(value.into_detached()),
            Self::Right(value) => Either::Right(value.into_detached()),
        }
    }
}
impl<'owned, T: CurveValue> Detached<'owned> for Curve<'_, T> {
    type Detached = Curve<'owned, T>;
    fn into_detached(self) -> Self::Detached {
        self.into_owned()
    }
}
impl<'owned, T: CurveValue + PropertyValue> Detached<'owned> for Track<'_, T> {
    type Detached = Track<'owned, T>;
    fn into_detached(self) -> Self::Detached {
        self.into_owned()
    }
}
impl<'owned> Detached<'owned> for DynamicTrack<'_> {
    type Detached = DynamicTrack<'owned>;
    fn into_detached(self) -> Self::Detached {
        self.into_owned()
    }
}
impl<'owned> Detached<'owned> for Animation<'_> {
    type Detached = Animation<'owned>;
    fn into_detached(self) -> Self::Detached {
        Animation {
            tracks: self.tracks.into_detached(),
            events: self.events,
        }
    }
}
