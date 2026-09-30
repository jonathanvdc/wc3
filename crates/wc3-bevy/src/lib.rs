//! Bevy integration for Warcraft III models.
//!
//! `Wc3Model::decode` normalizes MDX versions at the boundary. `spawn_model`
//! creates an independently animated instance with GPU skinned meshes.

mod animation;
mod material;
mod mesh;
mod model;
mod spawn;

use bevy::prelude::*;

pub use animation::Wc3Animation;
pub use material::{Wc3LayerMaterial, Wc3LayerState};
pub use model::{ModelError, Wc3Model};
pub use spawn::spawn_model;

pub struct Wc3BevyPlugin;

impl Plugin for Wc3BevyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<Wc3LayerMaterial>::default());
        app.add_systems(
            Update,
            (
                animation::advance_animation,
                animation::animate_nodes,
                animation::animate_layers,
            )
                .chain(),
        );
    }
}
