//! Validate model compilation without starting a GPU renderer.
use bevy::ecs::world::{CommandQueue, World};
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use bevy_wc3::{prepare_model, spawn_prepared_model, Wc3LayerMaterial, Wc3Model};
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    for path in std::env::args().skip(1) {
        let model = Wc3Model::decode(&fs::read(&path)?)?;
        let world = World::new();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<Wc3LayerMaterial>::default();
        let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
        let prepared = prepare_model(&mut meshes, &mut bindposes, &model, |_| None)?;
        spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
        spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
        println!(
            "{path}: two instances, {} shared meshes and {} material assets",
            meshes.len(),
            materials.len()
        );
    }
    Ok(())
}
