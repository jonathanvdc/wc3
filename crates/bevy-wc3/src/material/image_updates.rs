//! Refresh ordinary material allocations when a referenced GPU image is replaced.
use super::ModelMaterial;
use bevy::asset::{AssetEvent, AssetId, Assets, VisitAssetDependencies};
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::{
    App, Image, IntoScheduleConfigs, Last, Local, MessageReader, MessageWriter, Res, ResMut,
    Resource, SystemSet,
};
use bevy::render::erased_render_asset::prepare_erased_assets;
use bevy::render::render_asset::prepare_assets;
use bevy::render::texture::GpuImage;
use bevy::render::{Render, RenderApp, RenderSystems};
use std::collections::{HashMap, HashSet};

#[derive(Resource, Default)]
struct MaterialImages {
    materials: HashMap<AssetId<ModelMaterial>, Vec<AssetId<Image>>>,
    owners: HashMap<AssetId<Image>, HashSet<AssetId<ModelMaterial>>>,
    scratch: Vec<AssetId<Image>>,
    initialized: bool,
}
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ImageMaterialPreparation;

impl MaterialImages {
    fn remove(&mut self, id: AssetId<ModelMaterial>) {
        let Some(images) = self.materials.remove(&id) else {
            return;
        };
        for image in images {
            if let Some(owners) = self.owners.get_mut(&image) {
                owners.remove(&id);
                if owners.is_empty() {
                    self.owners.remove(&image);
                }
            }
        }
    }
    fn update(&mut self, id: AssetId<ModelMaterial>, material: &ModelMaterial) {
        self.scratch.clear();
        material.base.visit_dependencies(&mut |dependency| {
            if let Ok(image) = dependency.try_typed::<Image>() {
                self.scratch.push(image);
            }
        });
        let extension = &material.extension;
        self.scratch.extend(
            [
                &extension.secondary,
                &extension.orm,
                &extension.team_color,
                &extension.environment,
                &extension.tertiary,
                &extension.fourth,
            ]
            .into_iter()
            .filter_map(|image| image.as_ref().map(|image| image.id())),
        );
        self.scratch.sort_unstable();
        self.scratch.dedup();
        if self
            .materials
            .get(&id)
            .is_some_and(|images| *images == self.scratch)
        {
            return;
        }
        self.remove(id);
        for image in &self.scratch {
            self.owners.entry(*image).or_default().insert(id);
        }
        self.materials.insert(id, self.scratch.clone());
    }
}
/// Main-world invalidation leaves bindless/non-bindless allocation and device
/// recovery with Bevy. No GPU handles or strong asset handles are retained here.
pub(crate) fn register_material_image_updates(app: &mut App) {
    app.init_resource::<MaterialImages>().add_systems(
        Last,
        (index_material_images, invalidate_image_materials).chain(),
    );
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render.configure_sets(
            Render,
            ImageMaterialPreparation
                .in_set(RenderSystems::PrepareAssets)
                .after(prepare_assets::<GpuImage>)
                .before(prepare_erased_assets::<MeshMaterial3d<ModelMaterial>>),
        );
    }
}
// Bevy's dependency flattening connects each incoming/outgoing edge of an empty
// set, so this expresses ordering without a runtime system or duplicate preparation.
fn index_material_images(
    mut events: MessageReader<AssetEvent<ModelMaterial>>,
    materials: Res<Assets<ModelMaterial>>,
    mut index: ResMut<MaterialImages>,
    mut changed: Local<HashSet<AssetId<ModelMaterial>>>,
) {
    if !index.initialized {
        for (id, material) in materials.iter() {
            index.update(id, material);
        }
        index.initialized = true;
    }
    changed.clear();
    for event in events.read() {
        match event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::Removed { id }
            | AssetEvent::Unused { id } => {
                changed.insert(*id);
            }
            AssetEvent::LoadedWithDependencies { .. } => {}
        }
    }
    for id in changed.iter() {
        if let Some(material) = materials.get(*id) {
            index.update(*id, material);
        } else {
            index.remove(*id);
        }
    }
}
fn invalidate_image_materials(
    mut events: MessageReader<AssetEvent<Image>>,
    index: Res<MaterialImages>,
    materials: Res<Assets<ModelMaterial>>,
    mut updates: MessageWriter<AssetEvent<ModelMaterial>>,
    mut invalidated: Local<HashSet<AssetId<ModelMaterial>>>,
) {
    invalidated.clear();
    for event in events.read() {
        let (AssetEvent::Added { id }
        | AssetEvent::Modified { id }
        | AssetEvent::Removed { id }
        | AssetEvent::Unused { id }) = event
        else {
            continue;
        };
        if let Some(owners) = index.owners.get(id) {
            invalidated.extend(owners);
        }
    }
    for id in invalidated.iter() {
        if materials.get(*id).is_some() {
            updates.write(AssetEvent::Modified { id: *id });
        }
    }
}
#[cfg(test)]
#[path = "image_updates_tests.rs"]
mod tests;
