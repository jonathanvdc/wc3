//! Resolving material assignments without erasing family-specific appearance identities.
use super::{diablo3, diablo4, UniversalAppearance};
use crate::model::profiles::Universal;
use crate::model::{
    DrawId, Error, InstanceId, LookId, MaterialId, MaterialSlotId, MeshControls, Model,
};

/// An authored appearance selection for resolving a draw's material.
///
/// Defaults use a published draw look or an unambiguous slot assignment. Multiple
/// Diablo alternatives require a caller selection; authored ordering is not a default.
#[derive(Clone, Copy, Debug, Default)]
pub enum MaterialSelection {
    /// Use the authored default, or an unambiguous slot assignment.
    #[default]
    Default,
    /// Select a complete draw-based look.
    Draw(LookId),
    /// Select a Diablo III named alternative.
    Diablo3(diablo3::AlternativeId),
    /// Select a Diablo IV hashed alternative.
    Diablo4(diablo4::AlternativeId),
    /// Select an actual Diablo III slot-owned variant explicitly.
    Diablo3Variant {
        /// Slot that owns the selected variant.
        slot: MaterialSlotId,
        /// Actual variant identity within that slot.
        variant: diablo3::VariantId,
    },
    /// Select an actual Diablo IV slot-owned variant explicitly.
    Diablo4Variant {
        /// Slot that owns the selected variant.
        slot: MaterialSlotId,
        /// Actual variant identity within that slot.
        variant: diablo4::VariantId,
    },
}

impl Model<'_, Universal> {
    /// Resolves one draw under a typed appearance selection.
    ///
    /// Slot-based appearances retain their actual slot and variant identities.
    /// Ambiguous defaults, missing bindings, foreign identities, and selections
    /// from another appearance family return an error rather than choosing a material.
    pub fn resolve_material(
        &self,
        instance: InstanceId,
        draw: DrawId,
        selection: MaterialSelection,
    ) -> Result<MaterialId, Error> {
        self.instance(instance)
            .filter(|value| value.draw(draw).is_some())
            .ok_or_else(|| Error::new("appearance.draw", "instance draw is absent"))?;
        match (self.appearance(), selection) {
            (
                UniversalAppearance::Single(_) | UniversalAppearance::Draw(_),
                MaterialSelection::Default,
            ) => self.material_for_draw(instance, draw, None),
            (
                UniversalAppearance::Single(_) | UniversalAppearance::Draw(_),
                MaterialSelection::Draw(look),
            ) => self.material_for_draw(instance, draw, Some(look)),
            _ => self.resolve_instance_material(instance, selection),
        }
    }

    /// Resolves the slot assignment of an entire mesh placement without inventing
    /// a draw identity. Diablo appearances assign mesh-owned slots and their
    /// native instances intentionally have no explicit draw bindings.
    pub fn resolve_instance_material(
        &self,
        instance: InstanceId,
        selection: MaterialSelection,
    ) -> Result<MaterialId, Error> {
        let placement = self
            .instance(instance)
            .ok_or_else(|| Error::new("appearance.instance", "mesh placement is absent"))?;
        match (self.appearance(), selection) {
            (UniversalAppearance::D3(appearance), selection) => {
                let slot = match self
                    .mesh(placement.mesh)
                    .and_then(|mesh| mesh.controls.as_ref())
                {
                    Some(MeshControls::D3(value)) => value.material_slot,
                    _ => None,
                }
                .ok_or_else(missing_slot)?;
                let definition = appearance.slot(slot).ok_or_else(missing_slot)?;
                let variant = match selection {
                    MaterialSelection::Diablo3(id) => appearance
                        .alternative(id)
                        .ok_or_else(foreign_selection)?
                        .bindings()
                        .find(|(key, _)| *key == slot)
                        .map(|(_, value)| value),
                    MaterialSelection::Diablo3Variant {
                        slot: selected,
                        variant,
                    } if selected == slot => Some(variant),
                    MaterialSelection::Default => {
                        let mut alternatives = appearance.alternatives();
                        match (alternatives.next(), alternatives.next()) {
                            (Some((_, alternative)), None) => alternative
                                .bindings()
                                .find(|(key, _)| *key == slot)
                                .map(|(_, value)| value),
                            (None, None) => only(definition.variants().map(|(id, _)| id)),
                            _ => return Err(ambiguous()),
                        }
                    }
                    _ => return Err(foreign_selection()),
                }
                .ok_or_else(ambiguous)?;
                definition.variant(variant).ok_or_else(foreign_selection)
            }
            (UniversalAppearance::D4(appearance), selection) => {
                let slot = match self
                    .mesh(placement.mesh)
                    .and_then(|mesh| mesh.controls.as_ref())
                {
                    Some(MeshControls::D4(value)) => value.material_slot,
                    _ => None,
                }
                .ok_or_else(missing_slot)?;
                let definition = appearance.slot(slot).ok_or_else(missing_slot)?;
                let variant = match selection {
                    MaterialSelection::Diablo4(id) => appearance
                        .alternative(id)
                        .ok_or_else(foreign_selection)?
                        .bindings()
                        .find(|(key, _)| *key == slot)
                        .map(|(_, value)| value),
                    MaterialSelection::Diablo4Variant {
                        slot: selected,
                        variant,
                    } if selected == slot => Some(variant),
                    MaterialSelection::Default => {
                        let mut alternatives = appearance.alternatives();
                        match (alternatives.next(), alternatives.next()) {
                            (Some((_, alternative)), None) => alternative
                                .bindings()
                                .find(|(key, _)| *key == slot)
                                .map(|(_, value)| value),
                            (None, None) => only(definition.variants().map(|(id, _)| id)),
                            _ => return Err(ambiguous()),
                        }
                    }
                    _ => return Err(foreign_selection()),
                }
                .ok_or_else(ambiguous)?;
                definition.variant(variant).ok_or_else(foreign_selection)
            }
            _ => Err(foreign_selection()),
        }
    }
}

fn only<T>(mut values: impl Iterator<Item = T>) -> Option<T> {
    let value = values.next()?;
    values.next().is_none().then_some(value)
}
fn missing_slot() -> Error {
    Error::new(
        "appearance.slot",
        "draw mesh has no slot owned by this appearance",
    )
}
fn foreign_selection() -> Error {
    Error::new(
        "appearance.selection",
        "selection is absent, foreign, or incompatible with this appearance",
    )
}
fn ambiguous() -> Error {
    Error::new(
        "appearance.selection",
        "slot assignment is missing or ambiguous; select an actual alternative or variant",
    )
}

#[cfg(test)]
#[path = "resolution_tests.rs"]
mod tests;
