use super::{
    CharacterDatabase, CharacterDefinition, CharacterRecipe, EquipmentSlot, ModelPart, RecipeError,
    Sex,
};
use crate::assembly::{TextureLayer, TextureReplacement};
use std::collections::{BTreeMap, BTreeSet};

/// Explicit visibility change over a table-defined set of mutually exclusive geosets.
#[derive(Clone, Debug)]
pub struct GeosetRule {
    /// Every identifier controlled by this rule; no numeric category convention is inferred.
    pub alternatives: BTreeSet<u16>,
    /// Members of `alternatives` to leave visible after this rule.
    pub visible: BTreeSet<u16>,
}
/// One resolved customization or equipment contribution from a client dataset.
#[derive(Clone, Debug, Default)]
pub struct AppearancePatch {
    /// Increasing application order. Database adapters supply native precedence.
    pub order: i32,
    /// Ordered visibility replacements, including helmet hide and robe rules.
    pub geosets: Vec<GeosetRule>,
    /// Whole replacement slots applied before additional atlas layers.
    pub textures: BTreeMap<u32, TextureReplacement>,
    /// Back-to-front contributions appended to existing atlas slots.
    pub atlas_layers: BTreeMap<u32, Vec<TextureLayer>>,
    /// Additional rigid or deforming equipment models.
    pub parts: Vec<ModelPart>,
}
/// Build-specific resolved table rows with deterministic appearance assembly.
///
/// An application can populate these rows from DB2, an exported manifest, or its
/// own item catalog. Table decoding and item-to-display lookup remain outside the
/// renderer. Missing exact race/sex/display combinations fail instead of silently
/// using a model for another race or sex. Cloning a resolved definition is setup
/// work; instances reuse its compiled geometry and composed images.
#[derive(Clone, Debug, Default)]
pub struct CharacterTables {
    /// Baseline character by race and model sex.
    pub bodies: BTreeMap<(u32, Sex), CharacterDefinition>,
    /// Exact race, sex, customization option and choice rows.
    pub customization: BTreeMap<(u32, Sex, u32, u32), AppearancePatch>,
    /// Exact race, sex, equipment slot and ItemDisplayInfo rows.
    pub equipment: BTreeMap<(u32, Sex, EquipmentSlot, u32), AppearancePatch>,
}
impl CharacterDatabase for CharacterTables {
    type Error = RecipeError;
    fn resolve(&self, recipe: &CharacterRecipe) -> Result<CharacterDefinition, RecipeError> {
        recipe.validate()?;
        let mut definition = self
            .bodies
            .get(&(recipe.race, recipe.sex))
            .cloned()
            .ok_or_else(|| {
                RecipeError::Database(format!(
                    "missing body for race {} {:?}",
                    recipe.race, recipe.sex
                ))
            })?;
        let mut patches = Vec::new();
        for customization in &recipe.customization {
            let key = (
                recipe.race,
                recipe.sex,
                customization.option,
                customization.choice,
            );
            let patch = self
                .customization
                .get(&key)
                .ok_or_else(|| RecipeError::Database(format!("missing customization {key:?}")))?;
            patches.push((
                patch.order,
                0,
                customization.option,
                customization.choice,
                patch,
            ));
        }
        for equipment in &recipe.equipment {
            let key = (recipe.race, recipe.sex, equipment.slot, equipment.display);
            let patch = self
                .equipment
                .get(&key)
                .ok_or_else(|| RecipeError::Database(format!("missing equipment {key:?}")))?;
            patches.push((
                patch.order,
                1,
                equipment.slot as u32,
                equipment.display,
                patch,
            ));
        }
        // Explicit order wins; stable table keys resolve ties independently of
        // the order in which a caller provided customization or equipment.
        patches.sort_by_key(|(order, kind, slot, choice, _)| (*order, *kind, *slot, *choice));
        for (_, _, _, _, patch) in patches {
            for rule in &patch.geosets {
                if !rule.visible.is_subset(&rule.alternatives) {
                    return Err(RecipeError::Database(
                        "geoset rule selects a value outside its controlled alternatives".into(),
                    ));
                }
                definition
                    .geosets
                    .visible
                    .retain(|id| !rule.alternatives.contains(id));
                definition.geosets.visible.extend(&rule.visible);
            }
            definition.textures.extend(patch.textures.clone());
            for (category, layers) in &patch.atlas_layers {
                match definition.textures.get_mut(category) {
                    Some(TextureReplacement::Atlas { layers: target, .. }) => {
                        target.extend(layers.iter().cloned())
                    }
                    _ => {
                        return Err(RecipeError::Database(format!(
                            "atlas contributions require atlas replacement slot {category}"
                        )))
                    }
                }
            }
            definition.parts.extend(patch.parts.iter().cloned());
        }
        definition.validate()?;
        Ok(definition)
    }
}

#[cfg(test)]
#[path = "database_tests.rs"]
mod tests;
