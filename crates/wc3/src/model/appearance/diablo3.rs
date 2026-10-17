//! Named material slots and actual variants own Diablo III appearance assignments.
use crate::model::materials::diablo3::{AssetRef, Group};
use crate::model::{
    AppearanceSchema, Error, Id, Link, Look as DrawLook, LookId, MaterialId, MaterialSlotId, Scope,
    ValidateLinks,
};
use std::{borrow::Cow, iter::empty};

/// Identity marker for a material variant owned by one slot.
pub enum VariantObject {}
/// Stable slot-scoped material variant identity.
pub type VariantId = Id<VariantObject>;
/// Identity marker for one appearance-wide named look.
pub enum AlternativeObject {}
/// Stable appearance-scoped named look identity.
pub type AlternativeId = Id<AlternativeObject>;

/// External resources and qualifiers selected by one appearance variant.
#[derive(Clone, Debug)]
pub struct Selection {
    /// Independently authored subobject appearance qualifier.
    pub appearance_control: i32,
    /// External material resource selection.
    pub base_material: AssetRef,
    /// External cloth resource selection.
    pub cloth: AssetRef,
}
impl Default for Selection {
    fn default() -> Self {
        Self {
            appearance_control: 0,
            base_material: AssetRef {
                id: 0,
                group: Group::Material,
            },
            cloth: AssetRef {
                id: 0,
                group: Group::Cloth,
            },
        }
    }
}
impl ValidateLinks for Selection {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.base_material.visit_links(visitor);
        self.cloth.visit_links(visitor);
    }
}
/// One actual appearance variant and its independent external selections.
#[derive(Clone, Debug)]
pub struct Variant {
    /// Shader material executed by this variant.
    pub material: MaterialId,
    /// External resources and qualifiers belonging to this variant.
    pub selection: Selection,
}
/// Named native material slot with independently authored variants.
#[derive(Clone, Debug, Default)]
pub struct Slot<'a> {
    /// Authored material-slot name, independent of material display labels.
    pub name: Cow<'a, str>,
    scope: Scope,
    variants: Vec<(VariantId, Variant)>,
}
impl<'a> Slot<'a> {
    /// Creates an empty named material slot.
    pub fn named(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
    /// Iterates actual variants in authored order, including variants unused by any look.
    pub fn variants(&self) -> impl ExactSizeIterator<Item = (VariantId, MaterialId)> + '_ {
        self.variants
            .iter()
            .map(|(id, variant)| (*id, variant.material))
    }
    /// Replaces an actual variant assignment while preserving its scoped identity.
    pub fn set_variant(&mut self, id: VariantId, material: MaterialId) -> Result<(), Error> {
        let value = self
            .variants
            .iter_mut()
            .find(|(key, _)| *key == id)
            .ok_or_else(|| {
                Error::new("d3.appearance.variant", "variant belongs to another slot")
            })?;
        value.1.material = material;
        Ok(())
    }
    /// Removes a variant; checked publication rejects looks that still select it.
    pub fn remove_variant(&mut self, id: VariantId) -> Option<MaterialId> {
        self.variants
            .iter()
            .position(|(key, _)| *key == id)
            .map(|index| self.variants.remove(index).1.material)
    }
    /// Resolves a variant only when this actual slot owns its identity.
    pub fn variant(&self, id: VariantId) -> Option<MaterialId> {
        self.variants
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value.material)
    }
    /// Reads the complete variant owned by this slot.
    pub fn variant_definition(&self, id: VariantId) -> Option<&Variant> {
        self.variants
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Edits shader assignment and external selections without changing identity.
    pub fn variant_definition_mut(&mut self, id: VariantId) -> Option<&mut Variant> {
        self.variants
            .iter_mut()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
}
/// Named look assigning actual variants to actual material slots.
#[derive(Clone, Debug)]
pub struct Alternative<'a> {
    /// Authored alternative name.
    pub name: Cow<'a, str>,
    bindings: Vec<(MaterialSlotId, VariantId)>,
}
impl Alternative<'_> {
    /// Iterates checked slot-to-variant selections in authored order.
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = (MaterialSlotId, VariantId)> + '_ {
        self.bindings.iter().copied()
    }
}
/// Diablo III appearance vocabulary with scoped slots, variants, and named looks.
#[derive(Clone, Debug, Default)]
pub struct Appearance<'a> {
    scope: Scope,
    slots: Vec<(MaterialSlotId, Slot<'a>)>,
    alternatives: Vec<(AlternativeId, Alternative<'a>)>,
}
impl<'a> Appearance<'a> {
    /// Adds an independent material slot without changing existing identities.
    pub fn add_slot(&mut self, slot: Slot<'a>) -> MaterialSlotId {
        let id = self.scope.allocate();
        self.slots.push((id, slot));
        id
    }
    /// Adds an actual variant to a slot owned by this appearance.
    pub fn add_variant(
        &mut self,
        slot: MaterialSlotId,
        material: MaterialId,
    ) -> Result<VariantId, Error> {
        self.add_variant_with_selection(slot, material, Selection::default())
    }
    /// Adds an actual variant with independently authored external selections.
    pub fn add_variant_with_selection(
        &mut self,
        slot: MaterialSlotId,
        material: MaterialId,
        selection: Selection,
    ) -> Result<VariantId, Error> {
        let slot = self
            .slots
            .iter_mut()
            .find(|(key, _)| *key == slot)
            .map(|(_, value)| value)
            .ok_or_else(|| {
                Error::new("d3.appearance.slot", "slot belongs to another appearance")
            })?;
        let id = slot.scope.allocate();
        slot.variants.push((
            id,
            Variant {
                material,
                selection,
            },
        ));
        Ok(id)
    }
    /// Adds a named look after checking every selected slot and variant.
    pub fn add_look(
        &mut self,
        name: impl Into<Cow<'a, str>>,
        bindings: Vec<(MaterialSlotId, VariantId)>,
    ) -> Result<AlternativeId, Error> {
        self.validate_bindings(&bindings)?;
        let id = self.scope.allocate();
        self.alternatives.push((
            id,
            Alternative {
                name: name.into(),
                bindings,
            },
        ));
        Ok(id)
    }
    /// Iterates actual slots in authored order.
    pub fn slots(&self) -> impl ExactSizeIterator<Item = (MaterialSlotId, &Slot<'a>)> {
        self.slots.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves a slot only when this appearance owns its identity.
    pub fn slot(&self, id: MaterialSlotId) -> Option<&Slot<'a>> {
        self.slots
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Edits an actual slot while preserving its variants' identities.
    pub fn slot_mut(&mut self, id: MaterialSlotId) -> Option<&mut Slot<'a>> {
        self.slots
            .iter_mut()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Removes a slot; checked publication rejects remaining references to it.
    pub fn remove_slot(&mut self, id: MaterialSlotId) -> Option<Slot<'a>> {
        self.slots
            .iter()
            .position(|(key, _)| *key == id)
            .map(|index| self.slots.remove(index).1)
    }
    /// Selects an actual slot-owned variant on an existing named look.
    pub fn bind(
        &mut self,
        look: AlternativeId,
        slot: MaterialSlotId,
        variant: VariantId,
    ) -> Result<(), Error> {
        self.validate_bindings(&[(slot, variant)])?;
        let look = self
            .alternatives
            .iter_mut()
            .find(|(key, _)| *key == look)
            .map(|(_, value)| value)
            .ok_or_else(|| {
                Error::new("d3.appearance.look", "look belongs to another appearance")
            })?;
        if let Some((_, selected)) = look.bindings.iter_mut().find(|(key, _)| *key == slot) {
            *selected = variant;
        } else {
            look.bindings.push((slot, variant));
        }
        Ok(())
    }
    /// Edits an actual named look's authored name.
    pub fn alternative_mut(&mut self, id: AlternativeId) -> Option<&mut Alternative<'a>> {
        self.alternatives
            .iter_mut()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Removes a named look without removing its independently authored variants.
    pub fn remove_look(&mut self, id: AlternativeId) -> Option<Alternative<'a>> {
        self.alternatives
            .iter()
            .position(|(key, _)| *key == id)
            .map(|index| self.alternatives.remove(index).1)
    }
    /// Iterates named looks in authored order.
    pub fn alternatives(&self) -> impl ExactSizeIterator<Item = (AlternativeId, &Alternative<'a>)> {
        self.alternatives.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves a named look only when this appearance owns its identity.
    pub fn alternative(&self, id: AlternativeId) -> Option<&Alternative<'a>> {
        self.alternatives
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    fn validate_bindings(&self, bindings: &[(MaterialSlotId, VariantId)]) -> Result<(), Error> {
        for (index, (slot, variant)) in bindings.iter().enumerate() {
            if self
                .slot(*slot)
                .and_then(|slot| slot.variant(*variant))
                .is_none()
                || bindings[..index].iter().any(|(key, _)| key == slot)
            {
                return Err(Error::new(
                    "d3.appearance.bindings",
                    "look selects a foreign slot or variant, or assigns the same slot twice",
                ));
            }
        }
        Ok(())
    }
}
impl ValidateLinks for Appearance<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for (_, slot) in &self.slots {
            for (_, variant) in &slot.variants {
                variant.material.visit_links(visitor);
                variant.selection.visit_links(visitor);
            }
        }
    }
}
impl<'a> AppearanceSchema<'a> for Appearance<'a> {
    fn looks(&self) -> Box<dyn ExactSizeIterator<Item = (LookId, &DrawLook<'a>)> + '_> {
        Box::new(empty())
    }
    fn look(&self, _: LookId) -> Option<&DrawLook<'a>> {
        None
    }
    fn default_look(&self) -> Option<LookId> {
        None
    }
    fn contains_slot(&self, id: MaterialSlotId) -> bool {
        self.slot(id).is_some()
    }
    fn validate(&self) -> Result<(), Error> {
        for (_, alternative) in &self.alternatives {
            self.validate_bindings(&alternative.bindings)?;
        }
        Ok(())
    }
}
