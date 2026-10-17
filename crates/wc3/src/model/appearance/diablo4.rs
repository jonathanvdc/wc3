//! Hashed material slots and actual variants own Diablo IV appearance assignments.
use crate::model::{
    AppearanceSchema, Error, Id, Link, Look as DrawLook, LookId, MaterialId, MaterialSlotId, Scope,
    ValidateLinks,
};
use std::iter::empty;

/// Identity marker for a material variant owned by one slot.
pub enum VariantObject {}
/// Stable slot-scoped material variant identity.
pub type VariantId = Id<VariantObject>;
/// Identity marker for one appearance-wide hashed look.
pub enum AlternativeObject {}
/// Stable appearance-scoped hashed look identity.
pub type AlternativeId = Id<AlternativeObject>;

/// Appearance-local references that select an external material and attached cloth/effects.
#[derive(Clone, Debug, Default)]
pub struct Selection {
    /// Appearance material options.
    pub flags: u32,
    /// External base-material resource, including negative absent selections.
    pub base_material: i32,
    /// External override-material resource.
    pub override_material: i32,
    /// External cloth resource.
    pub cloth: i32,
    /// External high-quality cloth override.
    pub high_quality_cloth_override: i32,
    /// Mounted cloth overrides in their authored slots.
    pub mounted_cloth_overrides: [i32; 4],
    /// External effect-group resource.
    pub effect_group: i32,
    /// Ordered external variant-material resources.
    pub variant_materials: Vec<i32>,
}
impl ValidateLinks for Selection {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
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
pub struct Slot {
    /// Authored material-slot name hash.
    pub hash: u32,
    /// Persona-specific material declaration, including unresolved selectors.
    pub persona_material: i32,
    /// Authored persona selection.
    pub persona: u8,
    scope: Scope,
    variants: Vec<(VariantId, Variant)>,
}
impl Slot {
    /// Creates an empty slot with authored identity and persona declarations.
    pub fn new(hash: u32, persona_material: i32, persona: u8) -> Self {
        Self {
            hash,
            persona_material,
            persona,
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
                Error::new("d4.appearance.variant", "variant belongs to another slot")
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
/// Hashed look assigning actual variants to actual material slots.
#[derive(Clone, Debug)]
pub struct Alternative {
    /// Authored look-name hash.
    pub hash: u32,
    bindings: Vec<(MaterialSlotId, VariantId)>,
}
impl Alternative {
    /// Iterates checked slot-to-variant selections in authored order.
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = (MaterialSlotId, VariantId)> + '_ {
        self.bindings.iter().copied()
    }
}
/// Diablo IV appearance vocabulary with scoped slots, variants, and hashed looks.
#[derive(Clone, Debug, Default)]
pub struct Appearance {
    scope: Scope,
    slots: Vec<(MaterialSlotId, Slot)>,
    alternatives: Vec<(AlternativeId, Alternative)>,
}
impl Appearance {
    /// Adds an independent material slot without changing existing identities.
    pub fn add_slot(&mut self, slot: Slot) -> MaterialSlotId {
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
                Error::new("d4.appearance.slot", "slot belongs to another appearance")
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
    /// Adds a hashed look after checking every selected slot and variant.
    pub fn add_look(
        &mut self,
        hash: u32,
        bindings: Vec<(MaterialSlotId, VariantId)>,
    ) -> Result<AlternativeId, Error> {
        self.validate_bindings(&bindings)?;
        let id = self.scope.allocate();
        self.alternatives.push((id, Alternative { hash, bindings }));
        Ok(id)
    }
    /// Iterates actual slots in authored order.
    pub fn slots(&self) -> impl ExactSizeIterator<Item = (MaterialSlotId, &Slot)> {
        self.slots.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves a slot only when this appearance owns its identity.
    pub fn slot(&self, id: MaterialSlotId) -> Option<&Slot> {
        self.slots
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Edits an actual slot while preserving its variants' identities.
    pub fn slot_mut(&mut self, id: MaterialSlotId) -> Option<&mut Slot> {
        self.slots
            .iter_mut()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Removes a slot; checked publication rejects remaining references to it.
    pub fn remove_slot(&mut self, id: MaterialSlotId) -> Option<Slot> {
        self.slots
            .iter()
            .position(|(key, _)| *key == id)
            .map(|index| self.slots.remove(index).1)
    }
    /// Selects an actual slot-owned variant on an existing hashed look.
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
                Error::new("d4.appearance.look", "look belongs to another appearance")
            })?;
        if let Some((_, selected)) = look.bindings.iter_mut().find(|(key, _)| *key == slot) {
            *selected = variant;
        } else {
            look.bindings.push((slot, variant));
        }
        Ok(())
    }
    /// Edits an actual hashed look's authored hash.
    pub fn alternative_mut(&mut self, id: AlternativeId) -> Option<&mut Alternative> {
        self.alternatives
            .iter_mut()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Removes a hashed look without removing its independently authored variants.
    pub fn remove_look(&mut self, id: AlternativeId) -> Option<Alternative> {
        self.alternatives
            .iter()
            .position(|(key, _)| *key == id)
            .map(|index| self.alternatives.remove(index).1)
    }
    /// Iterates hashed looks in authored order.
    pub fn alternatives(&self) -> impl ExactSizeIterator<Item = (AlternativeId, &Alternative)> {
        self.alternatives.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves a hashed look only when this appearance owns its identity.
    pub fn alternative(&self, id: AlternativeId) -> Option<&Alternative> {
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
                    "d4.appearance.bindings",
                    "look selects a foreign slot or variant, or assigns the same slot twice",
                ));
            }
        }
        Ok(())
    }
}
impl ValidateLinks for Appearance {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for (_, slot) in &self.slots {
            for (_, variant) in &slot.variants {
                variant.material.visit_links(visitor);
                variant.selection.visit_links(visitor);
            }
        }
    }
}
impl<'a> AppearanceSchema<'a> for Appearance {
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
