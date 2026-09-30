//! Validate model compilation without starting a GPU renderer.
use bevy::ecs::world::{CommandQueue, World};
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::error::Error;
use std::fs;
use bevy_wc3::{spawn_model, Wc3LayerMaterial, Wc3Model};

fn main() -> Result<(), Box<dyn Error>> {
    for path in std::env::args().skip(1) {
        let model = Wc3Model::decode(&fs::read(&path)?)?;
        let world = World::new();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<Wc3LayerMaterial>::default();
        let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
        spawn_model(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut bindposes,
            &model,
            |_| None,
        )?;
        println!(
            "{path}: compiled {} meshes and {} material passes",
            meshes.len(),
            materials.len()
        );
    }
    Ok(())
}
