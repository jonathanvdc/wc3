//! Typed, model-scoped storage shared by authored and evaluated animation values.
use crate::model::ids::Scope;
use crate::model::ValueSample;
use crate::model::{CurveValue, Error, Id, Link, Quaternion, Table, Texture, ValidateLinks};
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Index;

mod private {
    pub trait Sealed {}
}

/// A supported slot value type. The implementation selects its typed subtable.
///
/// This trait is sealed: adding a value type also requires adding its storage and
/// evaluation support. Slot storage and identities do not depend on model profiles.
pub trait PropertyValue: private::Sealed + CurveValue {
    /// Selects the subtable containing this value type.
    fn table(values: &PropertyValues) -> &Table<ValueObject<Self>, Self>;
    /// Selects mutable storage for this value type.
    fn table_mut(values: &mut PropertyValues) -> &mut Table<ValueObject<Self>, Self>;
    /// Erases the type while retaining the exact slot identity.
    fn slot(value: Property<Self>) -> ValueSlot;
    /// Resolves an authored value or explicit converted source.
    fn resolve(
        values: &PropertyValues,
        slot: Property<Self>,
        sample: &(impl ValueSample + ?Sized),
    ) -> Self;
}

/// Identity family for slots containing `T`.
pub struct ValueObject<T>(PhantomData<fn() -> T>);

/// A reference to a typed, model-owned value slot.
///
/// Copying this reference shares the slot. Allocate another slot to create an
/// independently animated value. Constants live in the model's [`PropertyValues`] table.
pub struct Property<T: PropertyValue>(Id<ValueObject<T>>);
impl<T: PropertyValue> Copy for Property<T> {}
impl<T: PropertyValue> Clone for Property<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: PropertyValue> Debug for Property<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl<T: PropertyValue> PartialEq for Property<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T: PropertyValue> Eq for Property<T> {}
impl<T: PropertyValue> Hash for Property<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl<T: PropertyValue> Property<T> {
    /// Exact slot identity, including its value type.
    pub fn slot(self) -> ValueSlot {
        T::slot(self)
    }
}
impl<T: PropertyValue> ValidateLinks for Property<T> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        visitor(Link::Value(self.slot()));
    }
}

macro_rules! resolve_slot {
    (scalars, $values:ident, $slot:ident, $sample:ident) => {
        if let Some(source) = $values.boolean_reads.get(&$slot) {
            if $sample.animated(source) {
                1.
            } else {
                0.
            }
        } else {
            *$values.get($slot).expect("slot belongs to sampled model")
        }
    };
    ($field:ident, $values:ident, $slot:ident, $sample:ident) => {
        *$values.get($slot).expect("slot belongs to sampled model")
    };
}
macro_rules! tables {
    ($($field:ident: $ty:ty => $variant:ident),* $(,)?) => {
        /// One subtable per supported type, used for authored constants and evaluated instances.
        #[derive(Clone, Debug, Default)]
        pub struct PropertyValues { scope: Scope, boolean_reads: HashMap<Property<f32>, Property<bool>>, $($field: Table<ValueObject<$ty>, $ty>,)* }
        /// A heterogeneous slot identity used by validation and channel indexing.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum ValueSlot { $(#[doc = concat!("`", stringify!($ty), "`")] $variant(Property<$ty>),)* }
        $(impl private::Sealed for $ty {}
        impl PropertyValue for $ty {
            fn table(values: &PropertyValues) -> &Table<ValueObject<Self>, Self> { &values.$field }
            fn table_mut(values: &mut PropertyValues) -> &mut Table<ValueObject<Self>, Self> { &mut values.$field }
            fn slot(value: Property<Self>) -> ValueSlot { ValueSlot::$variant(value) }
            fn resolve(values: &PropertyValues, slot: Property<Self>, _sample: &(impl ValueSample + ?Sized)) -> Self { resolve_slot!($field, values, slot, _sample) }
        })*
        impl PropertyValues {
            /// Whether this table contains the exact typed identity.
            pub fn contains(&self, slot: ValueSlot) -> bool { match slot { $(ValueSlot::$variant(id) => self.$field.contains_key(id.0),)* } }
            /// Number of allocated slots across all types.
            pub fn len(&self) -> usize { 0 $(+ self.$field.len())* }
            /// Whether no values have been allocated.
            pub fn is_empty(&self) -> bool { self.len() == 0 }
            /// Validates constants and visits any resources they reference.
            pub(crate) fn visit_values(&self, visitor: &mut dyn FnMut(Link)) {
                for source in self.boolean_reads.values() { source.visit_links(visitor); }
                $(for value in self.$field.values() { if !value.is_valid() { visitor(Link::InvalidValue); } value.visit_value_links(visitor); })*
            }
            pub(crate) fn evaluate_into(&self, sample: &impl ValueSample, output: &mut Self) {
                output.clone_from(self);
                $(for (id, _) in self.$field.iter() { let slot = Property(id); output.$field.insert(id, sample.animated(&slot)); })*
            }
            /// Copies disjoint source slots while retaining their identities.
            pub(crate) fn extend(&mut self, source: &Self) -> Result<(), Error> {
                $(for (id, value) in &source.$field {
                    if self.$field.get(id).is_some_and(|existing| existing != value) { return Err(Error::new("values", "slot identity has conflicting constants")); }
                })*
                for (target, input) in &source.boolean_reads { if self.boolean_reads.get(target).is_some_and(|existing| existing != input) { return Err(Error::new("values", "slot has conflicting converted sources")); } }
                $(for (id, value) in &source.$field { self.$field.insert(id, *value); })*
                self.boolean_reads.extend(source.boolean_reads.iter().map(|(a,b)| (*a,*b)));
                Ok(())
            }
        }
    }
}
tables! {
    scalars: f32 => Scalar,
    vectors2: [f32; 2] => Vector2,
    vectors3: [f32; 3] => Vector3,
    vectors4: [f32; 4] => Vector4,
    rotations: Quaternion => Rotation,
    unsigned: u32 => Unsigned,
    signed: i32 => Signed,
    shorts: u16 => Unsigned16,
    bytes: u8 => Byte,
    booleans: bool => Boolean,
    unsigned4: [u32; 4] => Unsigned4,
    bytes4: [u8; 4] => Bytes4,
    textures: Texture => Texture,
    optional_textures: Option<Texture> => OptionalTexture,
}
impl PropertyValues {
    /// Allocates a scalar output explicitly reading a boolean slot as zero or one.
    pub fn scalar_from_boolean(&mut self, source: Property<bool>) -> Result<Property<f32>, Error> {
        let value = *self
            .get(source)
            .ok_or_else(|| Error::new("values", "boolean source is absent"))?;
        let slot = self.add(if value { 1. } else { 0. });
        self.boolean_reads.insert(slot, source);
        Ok(slot)
    }
    /// Source of an explicitly converted scalar output.
    pub fn boolean_source(&self, slot: Property<f32>) -> Option<Property<bool>> {
        self.boolean_reads.get(&slot).copied()
    }
    pub(crate) fn scoped(scope: Scope) -> Self {
        Self {
            scope,
            ..Self::default()
        }
    }
    /// Allocates a new independently animatable value.
    pub fn add<T: PropertyValue>(&mut self, value: T) -> Property<T> {
        let scope = self.scope.clone();
        Property(T::table_mut(self).add(&scope, value))
    }
    /// Resolves an authored or evaluated value. Foreign and absent IDs return `None`.
    pub fn get<T: PropertyValue>(&self, value: Property<T>) -> Option<&T> {
        T::table(self).get(value.0)
    }
    /// Changes an existing slot without changing its identity.
    pub fn set<T: PropertyValue>(&mut self, slot: Property<T>, value: T) -> Result<(), Error> {
        T::table_mut(self).replace(slot.0, value, "values")?;
        if let ValueSlot::Boolean(source) = slot.slot() {
            let scalar = if self[source] { 1. } else { 0. };
            let targets: Vec<_> = self
                .boolean_reads
                .iter()
                .filter_map(|(target, input)| (*input == source).then_some(*target))
                .collect();
            for target in targets {
                self.scalars.replace(target.0, scalar, "values")?;
            }
        }
        Ok(())
    }
    /// Removes a slot without changing any other identity. Model publication checks references.
    pub fn remove<T: PropertyValue>(&mut self, slot: Property<T>) -> Option<T> {
        if let ValueSlot::Scalar(target) = slot.slot() {
            self.boolean_reads.remove(&target);
        }
        T::table_mut(self).remove(&slot.0)
    }
    /// Iterates one typed subtable in stable identity order.
    pub fn iter<T: PropertyValue>(
        &self,
    ) -> impl ExactSizeIterator<Item = (Property<T>, &T)> + DoubleEndedIterator {
        T::table(self)
            .iter()
            .map(|(id, value)| (Property(id), value))
    }
}

/// Constructs a model definition, allocating any required animation slots.
/// Types without slots retain their ordinary defaults.
pub trait Construct: Sized {
    /// Creates a definition in the supplied value allocation context.
    fn construct(values: &mut PropertyValues) -> Self;
}
impl<T: Default> Construct for T {
    fn construct(_: &mut PropertyValues) -> Self {
        Self::default()
    }
}
impl<T: PropertyValue + Default> Construct for Property<T> {
    fn construct(values: &mut PropertyValues) -> Self {
        values.add(T::default())
    }
}
pub use wc3_derive::Construct;

impl<T: PropertyValue> Index<Property<T>> for PropertyValues {
    type Output = T;
    fn index(&self, slot: Property<T>) -> &T {
        self.get(slot).expect("slot belongs to this value table")
    }
}
impl<T: PropertyValue> Index<&Property<T>> for PropertyValues {
    type Output = T;
    fn index(&self, slot: &Property<T>) -> &T {
        &self[*slot]
    }
}
