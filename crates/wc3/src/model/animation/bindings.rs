//! Typed read bindings, separate from writable animation destinations.
use super::{ChannelValue, Property, PropertySample, PropertyTarget};
use crate::model::profiles::{Extends, Profile, Universal};
use crate::model::{Link, Quaternion, TextureId, ValidateLinks};
use std::fmt::Debug;

/// Selects the read bindings supported by an animated value domain.
pub trait AnimatedValue: Clone + Debug + Send + Sync + Sized + 'static {
    /// Binding representation admitted for this value domain.
    type Binding<P: Profile>: BindingValue<Value = Self, Profile = P> + From<Property<Self, P>>;
}
/// Read binding selected by the value domain.
pub type Binding<T, P = Universal> = <T as AnimatedValue>::Binding<P>;

/// Operations shared by direct and explicitly converted read bindings.
pub trait BindingValue: Clone + Debug + Send + Sync + ValidateLinks {
    /// Value produced by evaluating this binding.
    type Value: AnimatedValue;
    /// Vocabulary containing the source address.
    type Profile: Profile;
    /// Direct property, absent when this binding converts its input.
    fn property(&self) -> Option<&Property<Self::Value, Self::Profile>>;
    /// Actual source address whose channels drive this binding.
    fn target(&self) -> &PropertyTarget<Self::Profile>;
    /// Evaluate the binding with the supplied channel selection and clocks.
    fn sample(&self, sample: &(impl PropertySample<Self::Profile> + ?Sized)) -> Option<Self::Value>
    where
        Self::Value: ChannelValue;
    /// Embed the binding without changing its source or conversion.
    fn widen<Q: Extends<Self::Profile>>(self) -> Binding<Self::Value, Q>;
    /// Compare sources and conversion semantics across profiles.
    fn equivalent<Q: Profile>(&self, other: &Binding<Self::Value, Q>) -> bool;
}

/// Scalar reads from a scalar destination or an explicitly converted boolean source.
///
/// Converted reads cannot be used as channel destinations:
///
/// ```compile_fail,E0308
/// use wc3::model::{Curve, ScalarBinding, Track, Clock};
/// fn track(binding: ScalarBinding, curve: Curve<'static, f32>) -> Track<'static, f32> {
///     Track { target: binding, curve, clock: Clock::Clip }
/// }
/// ```
#[derive(Clone, Debug)]
pub enum ScalarBinding<P: Profile = Universal> {
    /// Read a scalar animation destination directly.
    Property(Property<f32, P>),
    /// Read boolean animation as scalar zero or one.
    FromBoolean(Property<bool, P>),
}
impl<P: Profile> From<Property<f32, P>> for ScalarBinding<P> {
    fn from(property: Property<f32, P>) -> Self {
        Self::Property(property)
    }
}
impl<P: Profile> ValidateLinks for ScalarBinding<P> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Property(property) => property.visit_links(visitor),
            Self::FromBoolean(property) => property.visit_links(visitor),
        }
    }
}
impl<P: Profile> BindingValue for ScalarBinding<P> {
    type Value = f32;
    type Profile = P;
    fn property(&self) -> Option<&Property<f32, P>> {
        match self {
            Self::Property(property) => Some(property),
            Self::FromBoolean(_) => None,
        }
    }
    fn target(&self) -> &PropertyTarget<P> {
        match self {
            Self::Property(property) => property.target(),
            Self::FromBoolean(property) => property.target(),
        }
    }
    fn sample(&self, sample: &(impl PropertySample<P> + ?Sized)) -> Option<f32> {
        match self {
            Self::Property(property) => sample.property(property),
            Self::FromBoolean(property) => {
                sample
                    .property(property)
                    .map(|value| if value { 1. } else { 0. })
            }
        }
    }
    fn widen<Q: Extends<P>>(self) -> ScalarBinding<Q> {
        match self {
            Self::Property(property) => ScalarBinding::Property(property.widen()),
            Self::FromBoolean(property) => ScalarBinding::FromBoolean(property.widen()),
        }
    }
    fn equivalent<Q: Profile>(&self, other: &ScalarBinding<Q>) -> bool {
        match (self, other) {
            (Self::Property(left), ScalarBinding::Property(right)) => left == right,
            (Self::FromBoolean(left), ScalarBinding::FromBoolean(right)) => left == right,
            _ => false,
        }
    }
}
impl AnimatedValue for f32 {
    type Binding<P: Profile> = ScalarBinding<P>;
}
impl<T: AnimatedValue, P: Profile> BindingValue for Property<T, P> {
    type Value = T;
    type Profile = P;
    fn property(&self) -> Option<&Property<T, P>> {
        Some(self)
    }
    fn target(&self) -> &PropertyTarget<P> {
        self.target()
    }
    fn sample(&self, sample: &(impl PropertySample<P> + ?Sized)) -> Option<T>
    where
        T: ChannelValue,
    {
        sample.property(self)
    }
    fn widen<Q: Extends<P>>(self) -> Binding<T, Q> {
        self.widen::<Q>().into()
    }
    fn equivalent<Q: Profile>(&self, other: &Binding<T, Q>) -> bool {
        other.property().is_some_and(|other| self == other)
    }
}
macro_rules! direct_binding {
    ($($value:ty),* $(,)?) => { $(
        impl AnimatedValue for $value {
            type Binding<P: Profile> = Property<Self, P>;
        }
    )* };
}
direct_binding!(
    bool,
    f64,
    u8,
    u16,
    u32,
    i32,
    Quaternion,
    TextureId,
    Option<TextureId>
);
macro_rules! array_binding {
    ($($value:ty),*) => { $(
        impl<const N: usize> AnimatedValue for [$value; N] {
            type Binding<P: Profile> = Property<Self, P>;
        }
    )* };
}
array_binding!(f32, u32, u8);

impl<P: Profile, Q: Profile> PartialEq<ScalarBinding<Q>> for ScalarBinding<P> {
    fn eq(&self, other: &ScalarBinding<Q>) -> bool {
        self.equivalent::<Q>(other)
    }
}
impl<P: Profile> Eq for ScalarBinding<P> {}
