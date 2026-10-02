use crate::assets::Wc3ModelAsset;
use crate::preparation::PreparedModel;
use bevy::asset::AssetId;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Resource, Default)]
pub(crate) struct PreparedModelCache {
    pub(super) prepared: HashMap<AssetId<Wc3ModelAsset>, PreparedModel>,
    pub(super) failed: HashSet<AssetId<Wc3ModelAsset>>,
}
