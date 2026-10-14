//! Complete common and game-specific attached authoring payloads.
pub mod common;
pub use common::{
    AssetAddress, Attachment, Camera, DepthOfField, Event, EventCategory, EventCode, Light,
    LightKind, LightVolume,
};

/// Warcraft III-specific emitter and facial animation declarations.
pub mod warcraft;

/// World of Warcraft-specific particle authoring declarations.
pub mod wow;

/// StarCraft particle and ribbon authoring snapshots.
pub mod starcraft;
