//! Independent programme declarations, selections, and shared playback layers.
mod codec;
mod controllers;
mod declarations;
mod inventory;
mod physics;
mod read;
mod write;
mod write_physics;
pub(in crate::model::conversion::wem) use inventory::prepare_clips;
pub(in crate::model::conversion::wem) use read::{decode_model, install, SourceTables};
pub(in crate::model::conversion::wem) use write::{write_clips, write_model};

#[cfg(test)]
mod tests;
