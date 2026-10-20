//! ECS services shared by the single semantic effect simulation system.
use super::super::{ModelAsset, ModelMaterial};
use super::age_sprites_gpu::AgeSpriteData;
use super::{ModelEmitter, RibbonEmitter, SpriteEmitter};
use crate::effects::sprites::ParticleInstances as SpriteInstances;
use crate::effects::trails::RibbonInstances;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

#[derive(SystemParam)]
pub(crate) struct EffectWorld<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub models: Res<'w, Assets<ModelAsset>>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<ModelMaterial>>,
    pub age_statistics: Query<'w, 's, &'static mut SpriteEmitter>,
    pub age_sprites: Query<'w, 's, &'static mut AgeSpriteData>,
    pub sprites: Query<'w, 's, &'static mut SpriteInstances>,
    pub trails: Query<'w, 's, &'static mut RibbonInstances>,
    pub ribbon_statistics: Query<'w, 's, &'static mut RibbonEmitter>,
    pub model_statistics: Query<'w, 's, &'static mut ModelEmitter>,
    pub globals: Query<'w, 's, &'static GlobalTransform>,
    pub hierarchy: Query<'w, 's, (&'static Visibility, Option<&'static ChildOf>)>,
}
