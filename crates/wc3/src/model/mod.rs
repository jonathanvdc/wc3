//! Model formats and shared encoded storage utilities.
pub mod common;
/// World of Warcraft M2 model documents and supporting file codecs.
pub mod m2;
/// StarCraft II and Heroes of the Storm indexed M3 model documents.
pub mod m3;
/// Warcraft III models with binary MDX and text MDL codecs.
pub mod mdlx;
/// World of Warcraft world-model root and group documents.
pub mod wmo;

/// Explicit model conversion planning, policies, and native adapters.
pub mod conversion;

/// Checked logical model graphs with borrowed storage and typed relationships.
pub mod semantic;

/// Diablo III and Diablo IV model documents and supporting SNO resources.
pub mod diablo;
