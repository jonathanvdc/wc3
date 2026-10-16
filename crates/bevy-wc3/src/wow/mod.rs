//! World of Warcraft character assembly, separate from model rendering.
//!
//! Game tables resolve race, customization and item choices into an explicit
//! recipe. No client-build-specific geoset defaults or texture layouts are guessed.
mod assembly;
mod atlas;
mod catalog;
mod database;
mod recipe;

pub use assembly::{compile_character, CharacterModels};
pub use atlas::{compose_atlas, Atlas, AtlasError, AtlasLayer, PixelRect, RgbaImage};
pub use catalog::{BoneIdentity, CharacterCatalog, GeosetDraw, JointMatchError};
pub use database::{AppearancePatch, CharacterTables, GeosetRule};
pub use recipe::{
    CharacterDatabase, CharacterDefinition, CharacterRecipe, Customization, Equipment,
    EquipmentBinding, EquipmentSlot, GeosetSelection, ModelPart, RecipeError, Sex, TextureLayer,
    TextureReplacement,
};
