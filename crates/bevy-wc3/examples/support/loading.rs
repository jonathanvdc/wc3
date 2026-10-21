//! Root and recursive asset failure checks shared by offscreen tools.
use bevy::asset::{LoadState, RecursiveDependencyLoadState};
use bevy::prelude::{Asset, AssetServer, Handle};
use std::error::Error;

/// Check the same root/dependency failure boundary for loading tools.
pub fn check_asset<A: Asset>(
    assets: &AssetServer,
    handle: &Handle<A>,
) -> Result<(), Box<dyn Error>> {
    if let LoadState::Failed(error) = assets.load_state(handle) {
        return Err(format!("asset load failed: {error}").into());
    }
    if let RecursiveDependencyLoadState::Failed(error) =
        assets.recursive_dependency_load_state(handle)
    {
        return Err(format!("asset dependency load failed: {error}").into());
    }
    Ok(())
}
