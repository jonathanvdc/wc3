//! Typed collections of independently identified graph objects.
use super::common::Detached;
use super::ids::Scope;
use super::{Error, Id};
use std::collections::{btree_map, BTreeMap};
use std::fmt::{self, Debug, Formatter};
use std::iter::Map;
use std::ops::Index;

/// Immutable access to one family of model-owned objects.
///
/// Keys describe identity kinds rather than payload types, so profile conversions
/// can replace values without changing their identities. Mutations are published
/// through a checked model builder.
///
/// A table accepts only IDs of its own object family:
/// ```compile_fail
/// use wc3::model::{MaterialId, MeshObject, Table};
/// fn wrong_family(meshes: &Table<MeshObject, u32>, material: MaterialId) {
///     meshes.get(material);
/// }
/// ```
pub struct Table<K, V> {
    entries: BTreeMap<Id<K>, V>,
}
impl<K, V> Default for Table<K, V> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}
impl<K, V: Clone> Clone for Table<K, V> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}
impl<K, V: Debug> Debug for Table<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.entries.fmt(f)
    }
}
impl<K, V> Table<K, V> {
    /// Transforms definitions without reallocating their graph identities.
    pub(crate) fn map_values<U>(self, mut map: impl FnMut(V) -> U) -> Table<K, U> {
        self.into_iter()
            .map(|(id, value)| (id, map(value)))
            .collect()
    }
    /// Number of definitions in this collection.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    /// Whether this collection has no definitions.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    /// Resolves an identity, returning `None` for absent or foreign identities.
    pub fn get(&self, id: Id<K>) -> Option<&V> {
        self.entries.get(&id)
    }
    /// Whether an identity has a definition in this collection.
    pub fn contains_key(&self, id: Id<K>) -> bool {
        self.entries.contains_key(&id)
    }
    /// Definitions in stable identity order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (Id<K>, &V)> + DoubleEndedIterator {
        self.entries.iter().map(|(id, value)| (*id, value))
    }
    /// Identities in stable order.
    pub fn keys(&self) -> impl ExactSizeIterator<Item = Id<K>> + DoubleEndedIterator + '_ {
        self.entries.keys().copied()
    }
    /// PropertyValues in stable identity order.
    pub fn values(&self) -> btree_map::Values<'_, Id<K>, V> {
        self.entries.values()
    }
    pub(crate) fn add(&mut self, scope: &Scope, value: V) -> Id<K> {
        let id = scope.allocate();
        self.entries.insert(id, value);
        id
    }
    pub(crate) fn replace(&mut self, id: Id<K>, value: V, context: &str) -> Result<(), Error> {
        let slot = self
            .entries
            .get_mut(&id)
            .ok_or_else(|| Error::new(context, "unknown identity"))?;
        *slot = value;
        Ok(())
    }
    pub(crate) fn insert(&mut self, id: Id<K>, value: V) -> Option<V> {
        self.entries.insert(id, value)
    }
    pub(crate) fn get_mut(&mut self, id: &Id<K>) -> Option<&mut V> {
        self.entries.get_mut(id)
    }
    pub(crate) fn remove(&mut self, id: &Id<K>) -> Option<V> {
        self.entries.remove(id)
    }
}
impl<'owned, K, V: Detached<'owned>> Detached<'owned> for Table<K, V> {
    type Detached = Table<K, V::Detached>;
    fn into_detached(self) -> Self::Detached {
        self.map_values(Detached::into_detached)
    }
}
impl<K, V> FromIterator<(Id<K>, V)> for Table<K, V> {
    fn from_iter<T: IntoIterator<Item = (Id<K>, V)>>(iter: T) -> Self {
        Self {
            entries: iter.into_iter().collect(),
        }
    }
}
impl<K, V> IntoIterator for Table<K, V> {
    type Item = (Id<K>, V);
    type IntoIter = btree_map::IntoIter<Id<K>, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}
impl<'s, K, V> IntoIterator for &'s Table<K, V> {
    type Item = (Id<K>, &'s V);
    type IntoIter = Map<btree_map::Iter<'s, Id<K>, V>, fn((&'s Id<K>, &'s V)) -> (Id<K>, &'s V)>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter().map(|(id, value)| (*id, value))
    }
}
impl<K, V> Index<&Id<K>> for Table<K, V> {
    type Output = V;
    fn index(&self, id: &Id<K>) -> &V {
        &self.entries[id]
    }
}
