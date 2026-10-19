use super::CharacterCatalog;
use crate::assembly::TextureReplacement;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use wc3::model::{ClipId, NodeId, ResourceLocator, SocketIdentity};

/// Sex category used by the game database when choosing character assets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sex {
    /// Male character model category.
    Male,
    /// Female character model category.
    Female,
}
/// One table-defined customization option and selected choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Customization {
    /// External customization option ID.
    pub option: u32,
    /// External choice ID, not a presumed zero-based ordinal.
    pub choice: u32,
}
/// Supported equipment destinations; ordering does not determine texture layering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EquipmentSlot {
    /// Head armor.
    Head,
    /// Shoulder armor, potentially several model parts.
    Shoulders,
    /// Underlying shirt.
    Shirt,
    /// Chest armor or robe.
    Chest,
    /// Waist armor.
    Waist,
    /// Leg armor.
    Legs,
    /// Foot armor.
    Feet,
    /// Wrist armor.
    Wrists,
    /// Gloves.
    Hands,
    /// Back/cape.
    Back,
    /// Tabard.
    Tabard,
    /// Main-hand equipment.
    MainHand,
    /// Off-hand equipment.
    OffHand,
    /// Ranged or back-mounted weapon.
    Ranged,
}
/// Item appearance identity supplied by the caller or resolved from its item ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Equipment {
    /// Destination slot.
    pub slot: EquipmentSlot,
    /// ItemDisplayInfo identity, not an item-table identity.
    pub display: u32,
}
/// Desired character appearance, independent of any client table binary layout.
#[derive(Clone, Debug)]
pub struct CharacterRecipe {
    /// External race ID.
    pub race: u32,
    /// Model sex selection.
    pub sex: Sex,
    /// Explicit table-defined customization choices.
    pub customization: Vec<Customization>,
    /// Equipment appearance IDs, at most one per destination slot.
    pub equipment: Vec<Equipment>,
}
impl CharacterRecipe {
    /// Rejects conflicting entries before requesting game data.
    pub fn validate(&self) -> Result<(), RecipeError> {
        let mut options = BTreeSet::new();
        for choice in &self.customization {
            if !options.insert(choice.option) {
                return Err(RecipeError::DuplicateCustomization(choice.option));
            }
        }
        let mut slots = BTreeSet::new();
        for equipment in &self.equipment {
            if !slots.insert(equipment.slot) {
                return Err(RecipeError::DuplicateEquipment(equipment.slot));
            }
        }
        Ok(())
    }
    /// Resolves game data, checking both the request and the returned definition.
    pub fn resolve<D: CharacterDatabase>(
        &self,
        database: &D,
    ) -> Result<CharacterDefinition, RecipeError> {
        self.validate()?;
        let definition = database
            .resolve(self)
            .map_err(|error| RecipeError::Database(error.to_string()))?;
        definition.validate()?;
        Ok(definition)
    }
}
/// Application adapter for a particular client build's tables and asset manifests.
///
/// Resolving item IDs, customization layouts, helmet hide rules, robe/glove/boot
/// geoset precedence and texture layer order belongs here, before renderer setup.
pub trait CharacterDatabase {
    /// Error emitted by this game-data source.
    type Error: Error;
    /// Resolves the complete appearance using one coherent client-build dataset.
    fn resolve(&self, recipe: &CharacterRecipe) -> Result<CharacterDefinition, Self::Error>;
}
/// Explicit native geoset visibility, including all its passes and LODs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GeosetSelection {
    /// Unknown identifiers are rejected instead of silently producing bare armor.
    pub visible: BTreeSet<u16>,
}
impl GeosetSelection {
    /// Checks selected geosets against the imported default appearance.
    pub fn validate(&self, catalog: &CharacterCatalog) -> Result<(), RecipeError> {
        for geoset in &self.visible {
            if !catalog.geosets.iter().any(|draw| draw.geoset == *geoset) {
                return Err(RecipeError::UnknownGeoset(*geoset));
            }
        }
        Ok(())
    }
    /// Selects visible logical draws, preserving every authored LOD.
    pub fn draws(&self, catalog: &CharacterCatalog) -> Result<Vec<super::GeosetDraw>, RecipeError> {
        self.validate(catalog)?;
        Ok(catalog
            .geosets
            .iter()
            .copied()
            .filter(|draw| self.visible.contains(&draw.geoset))
            .collect())
    }
}
/// How a separately loaded model participates in the character hierarchy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EquipmentBinding {
    /// Rigid equipment follows the native attachment point's authored frame.
    Attachment(SocketIdentity<'static>),
    /// Deforming equipment uses explicitly validated native CRC/key-bone matching.
    SharedSkeleton,
}
/// One resolved armor, shoulder, weapon, cape or auxiliary model.
#[derive(Clone, Debug)]
pub struct ModelPart {
    /// Model resource identity.
    pub model: ResourceLocator<'static>,
    /// Rigid attachment or shared skeleton.
    pub binding: EquipmentBinding,
    /// Explicit source-to-character joint mappings, overriding automatic native matching.
    pub joints: BTreeMap<NodeId, NodeId>,
    /// Explicitly retain unmatched child joints and their local animation.
    /// Defaults should remain false for armor whose whole skeleton must match.
    pub retain_unmatched_joints: bool,
    /// Host-to-part clip bindings for extra joints and animated equipment materials.
    /// Both clips sample the host time in seconds; no implicit retiming occurs.
    pub clips: BTreeMap<ClipId, ClipId>,
    /// Explicit geoset visibility; absent retains this model's authored geometry.
    pub geosets: Option<GeosetSelection>,
    /// Per-part native replacement categories, independent of the base character.
    pub textures: BTreeMap<u32, TextureReplacement>,
}
impl ModelPart {
    /// Creates a part with authored geometry, no replacement textures and strict native joint matching.
    pub fn new(model: ResourceLocator<'static>, binding: EquipmentBinding) -> Self {
        Self {
            model,
            binding,
            joints: BTreeMap::new(),
            retain_unmatched_joints: false,
            clips: BTreeMap::new(),
            geosets: None,
            textures: BTreeMap::new(),
        }
    }
}
/// Fully resolved appearance, ready for model loading and renderer composition.
#[derive(Clone, Debug)]
pub struct CharacterDefinition {
    /// Root character model resource.
    pub model: ResourceLocator<'static>,
    /// Exact visible native geosets, including hair/facial features and clothing.
    pub geosets: GeosetSelection,
    /// Native replacement categories; numeric identities remain build-defined.
    pub textures: BTreeMap<u32, TextureReplacement>,
    /// Additional rigid and deforming model parts.
    pub parts: Vec<ModelPart>,
    /// Selected native BONE resources; an empty set retains the base face.
    pub bone_sets: HashSet<ResourceLocator<'static>>,
}
impl CharacterDefinition {
    /// Validates atlas layouts before loading models or mutating a live character.
    pub fn validate(&self) -> Result<(), RecipeError> {
        for replacement in self
            .textures
            .values()
            .chain(self.parts.iter().flat_map(|part| part.textures.values()))
        {
            replacement
                .validate()
                .map_err(|_| RecipeError::InvalidAtlas)?;
        }
        Ok(())
    }
}
/// An invalid or unresolved character recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecipeError {
    /// One customization option was selected several times.
    DuplicateCustomization(u32),
    /// More than one display was assigned to one slot.
    DuplicateEquipment(EquipmentSlot),
    /// A selected geoset is absent from the imported model.
    UnknownGeoset(u16),
    /// Atlas dimensions or destination rectangles are invalid.
    InvalidAtlas,
    /// The application database could not resolve the complete appearance.
    Database(String),
}
impl Display for RecipeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "invalid WoW character recipe: {self:?}")
    }
}
impl Error for RecipeError {}

#[cfg(test)]
#[path = "recipe_tests.rs"]
mod tests;
