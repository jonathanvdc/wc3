//! Named material slots and per-family appearance choices are actual authored domains.
use super::Metadata;
use crate::model::semantic::AppearanceSchema;
use crate::model::semantic::Look as DrawLook;
use crate::model::semantic::Scope;
use crate::model::semantic::{Error, Id, Link, LookId, MaterialId, MaterialSlotId, ValidateLinks};
use std::borrow::Cow;
use std::iter::empty;
/// Declared shader/actor convention belonging to an appearance set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Family {
    /// Common WEM semantics.
    Common,
    /// Warcraft III classic semantics.
    WarcraftClassic,
    /// Warcraft III Reforged semantics.
    WarcraftReforged,
    /// Warcraft III Definitive semantics.
    WarcraftDefinitive,
    /// World of Warcraft semantics.
    WarcraftOnline,
    /// StarCraft II semantics.
    StarCraft,
    /// Heroes of the Storm semantics.
    Heroes,
    /// Diablo III semantics.
    Diablo,
    /// An unrecognized authored convention.
    Unknown(u32),
}
/// One named material alternative within a declared family.
#[derive(Clone, Debug)]
pub struct Look<'a> {
    /// Display name of this alternative.
    pub name: Cow<'a, str>,
    /// Relative selection weight.
    pub weight: i32,
    bindings: Vec<(MaterialSlotId, Option<MaterialId>)>,
}
impl<'a> Look<'a> {
    /// Creates an alternative without assigned materials.
    pub fn named(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            weight: 0,
            bindings: Vec::new(),
        }
    }
    /// Reads authored slot/material relationships, including explicitly unbound slots.
    pub fn bindings(
        &self,
    ) -> impl ExactSizeIterator<Item = (MaterialSlotId, Option<MaterialId>)> + '_ {
        self.bindings.iter().copied()
    }
}
/// Identity marker for an appearance-owned shader convention set.
pub enum SetObject {}
/// Stable shader convention set belonging to one appearance vocabulary.
pub type SetId = Id<SetObject>;
/// Identity marker for a set-owned material alternative.
pub enum AlternativeObject {}
/// Stable appearance alternative belonging to one actual set.
pub type AlternativeId = Id<AlternativeObject>;
/// One shader convention's alternatives and independently authored materials.
#[derive(Clone, Debug)]
pub struct Set<'a> {
    /// Game shader/actor convention.
    pub family: Family,
    /// All authored materials, including currently unused declarations.
    pub materials: Vec<MaterialId>,
    /// Ordered authoring properties associated with this family.
    pub metadata: Metadata<'a>,
    scope: Scope,
    looks: Vec<(AlternativeId, Look<'a>)>,
    default: Option<AlternativeId>,
}
impl<'a> Set<'a> {
    /// Creates a family vocabulary with no material alternatives.
    pub fn new(family: Family) -> Self {
        Self {
            family,
            materials: Vec::new(),
            metadata: Metadata::default(),
            scope: Scope::default(),
            looks: Vec::new(),
            default: None,
        }
    }
    /// Adds a named alternative without changing existing identities.
    pub fn add_look(&mut self, look: Look<'a>) -> AlternativeId {
        let id = self.scope.allocate();
        self.looks.push((id, look));
        id
    }
    /// Borrows an alternative only when this set owns its identity.
    pub fn look(&self, id: AlternativeId) -> Option<&Look<'a>> {
        self.looks
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, v)| v)
    }
    /// Iterates actual alternatives in authoring order.
    pub fn looks(&self) -> impl ExactSizeIterator<Item = (AlternativeId, &Look<'a>)> {
        self.looks.iter().map(|(id, value)| (*id, value))
    }
    /// Selects a default belonging to this set, or clears the selection.
    pub fn set_default_look(&mut self, id: Option<AlternativeId>) -> Result<(), Error> {
        if id.is_some_and(|id| self.look(id).is_none()) {
            return Err(Error::new(
                "wem.appearance.default",
                "alternative belongs to another set",
            ));
        }
        self.default = id;
        Ok(())
    }
    /// Actual default alternative, when selected.
    pub fn default_look(&self) -> Option<AlternativeId> {
        self.default
    }
}
/// Appearance-owned slots and per-family material alternatives.
#[derive(Clone, Debug, Default)]
pub struct Appearance<'a> {
    scope: Scope,
    slots: Vec<(MaterialSlotId, Cow<'a, str>)>,
    sets: Vec<(SetId, Set<'a>)>,
}
impl<'a> Appearance<'a> {
    /// Creates an independent slot vocabulary.
    pub fn new() -> Self {
        Self::default()
    }
    /// Adds a slot without copying its borrowed name.
    pub fn add_slot(&mut self, name: impl Into<Cow<'a, str>>) -> MaterialSlotId {
        let id = self.scope.allocate();
        self.slots.push((id, name.into()));
        id
    }
    /// Iterates named slots in authoring order.
    pub fn slots(&self) -> impl ExactSizeIterator<Item = (MaterialSlotId, &str)> {
        self.slots.iter().map(|(id, value)| (*id, value.as_ref()))
    }
    /// Borrows a slot only when this vocabulary owns its identity.
    pub fn slot(&self, id: MaterialSlotId) -> Option<&str> {
        self.slots
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, v)| v.as_ref())
    }
    /// Iterates actual shader sets in authoring order.
    pub fn sets(&self) -> impl ExactSizeIterator<Item = (SetId, &Set<'a>)> {
        self.sets.iter().map(|(id, value)| (*id, value))
    }
    /// Borrows a shader set only when this vocabulary owns its identity.
    pub fn set(&self, id: SetId) -> Option<&Set<'a>> {
        self.sets
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Adds a family after checking its slot and default ownership.
    pub fn add_set(&mut self, set: Set<'a>) -> Result<SetId, Error> {
        if !self.valid_set(&set) {
            return Err(Error::new(
                "wem.appearance.set",
                "invalid default or foreign material slot",
            ));
        }
        let id = self.scope.allocate();
        self.sets.push((id, set));
        Ok(id)
    }
    /// Assigns a material to an actual alternative and slot.
    pub fn bind(
        &mut self,
        set: SetId,
        look: AlternativeId,
        slot: MaterialSlotId,
        material: Option<MaterialId>,
    ) -> Result<(), Error> {
        if self.slot(slot).is_none() {
            return Err(Error::new(
                "wem.appearance.slot",
                "slot belongs to another appearance",
            ));
        }
        let look = self
            .sets
            .iter_mut()
            .find(|(id, _)| *id == set)
            .and_then(|(_, set)| set.looks.iter_mut().find(|(id, _)| *id == look))
            .map(|(_, value)| value)
            .ok_or_else(|| {
                Error::new("wem.appearance.look", "alternative belongs to another set")
            })?;
        if let Some(existing) = look.bindings.iter_mut().find(|(id, _)| *id == slot) {
            existing.1 = material;
        } else {
            look.bindings.push((slot, material));
        }
        Ok(())
    }
    fn valid_set(&self, set: &Set<'_>) -> bool {
        set.default.is_none_or(|id| set.look(id).is_some())
            && set
                .looks
                .iter()
                .flat_map(|(_, look)| &look.bindings)
                .all(|(slot, material)| {
                    self.slot(*slot).is_some()
                        && material.is_none_or(|id| set.materials.contains(&id))
                })
    }
}
impl ValidateLinks for Appearance<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for (_, set) in &self.sets {
            set.materials.visit_links(visitor);
            for (_, look) in &set.looks {
                for (_, material) in &look.bindings {
                    material.visit_links(visitor);
                }
            }
        }
    }
}
impl<'a> AppearanceSchema<'a> for Appearance<'a> {
    fn looks(&self) -> Box<dyn ExactSizeIterator<Item = (LookId, &DrawLook<'a>)> + '_> {
        Box::new(empty())
    }
    fn look(&self, _id: LookId) -> Option<&DrawLook<'a>> {
        None
    }
    fn default_look(&self) -> Option<LookId> {
        None
    }
    fn contains_slot(&self, id: MaterialSlotId) -> bool {
        self.slot(id).is_some()
    }
    fn validate(&self) -> Result<(), Error> {
        if self.sets.iter().all(|(_, set)| self.valid_set(set)) {
            Ok(())
        } else {
            Err(Error::new(
                "wem.appearance",
                "invalid family appearance binding",
            ))
        }
    }
}
