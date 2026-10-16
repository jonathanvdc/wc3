//! ECS services shared by the single semantic effect simulation system.
use super::super::{UniversalMaterial, UniversalModelAsset};
use super::{UniversalModelParticleEmitter, UniversalParticleEmitter, UniversalRibbonEmitter};
use crate::effects::sprites::ParticleInstances as SpriteInstances;
use crate::effects::trails::RibbonInstances;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

#[derive(SystemParam)]
pub(crate) struct EffectWorld<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub models: Res<'w, Assets<UniversalModelAsset>>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<UniversalMaterial>>,
    pub age_statistics: Query<'w, 's, &'static mut UniversalParticleEmitter>,
    pub sprites: Query<'w, 's, &'static mut SpriteInstances>,
    pub trails: Query<'w, 's, &'static mut RibbonInstances>,
    pub ribbon_statistics: Query<'w, 's, &'static mut UniversalRibbonEmitter>,
    pub model_statistics: Query<'w, 's, &'static mut UniversalModelParticleEmitter>,
    pub globals: Query<'w, 's, &'static GlobalTransform>,
    pub hierarchy: Query<'w, 's, (&'static Visibility, Option<&'static ChildOf>)>,
}
