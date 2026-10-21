//! The existing recipe path and the ordinary asset-backed model path for crowds.
use super::{character, loading};
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use bevy_wc3::{
    spawn_model, ModelAsset, ModelInstance, ModelMaterial, ModelRig, Playback, PreparedModel,
};
use serde::Serialize;
use std::error::Error;
use std::path::Path;
use wc3::model::ClipId;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkloadKind {
    CharacterRecipe,
    Model,
}
pub enum CrowdModel {
    Character(Box<PreparedModel>),
    Model(Handle<ModelAsset>),
}
impl CrowdModel {
    pub fn load(
        app: &mut App,
        recipe: Option<&Path>,
        assets: &Path,
        model: &Path,
        allow_loss: bool,
    ) -> Result<Self> {
        if let Some(recipe) = recipe {
            let compiled = character::load(
                recipe,
                assets,
                model,
                allow_loss,
                &mut app.world_mut().resource_mut::<Assets<Image>>(),
            )?;
            let prepared =
                app.world_mut()
                    .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                        let mut binds = world.resource_mut::<Assets<SkinnedMeshInverseBindposes>>();
                        compiled.prepare(&mut meshes, &mut binds)
                    });
            Ok(Self::Character(Box::new(prepared)))
        } else {
            let model = model.canonicalize()?;
            let path = model
                .strip_prefix(assets)
                .map_err(|_| "model must be inside the asset root")?
                .to_string_lossy()
                .into_owned();
            Ok(Self::Model(
                app.world().resource::<AssetServer>().load(path),
            ))
        }
    }
    pub fn kind(&self) -> WorkloadKind {
        match self {
            Self::Character(_) => WorkloadKind::CharacterRecipe,
            Self::Model(_) => WorkloadKind::Model,
        }
    }
    pub fn spawn(&self, app: &mut App, transform: Transform) -> Entity {
        match self {
            Self::Character(prepared) => {
                app.world_mut()
                    .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                        world.resource_scope(|world, mut materials: Mut<Assets<ModelMaterial>>| {
                            spawn_model(
                                &mut world.commands(),
                                &mut meshes,
                                &mut materials,
                                prepared,
                                transform,
                            )
                        })
                    })
            }
            Self::Model(handle) => app
                .world_mut()
                .spawn((
                    ModelInstance(handle.clone()),
                    transform,
                    Visibility::Inherited,
                    Playback::default(),
                ))
                .id(),
        }
    }
    /// Return None until both source dependencies and every requested rig are ready.
    /// Loading uses zero delta; initialization installs the same staggered phases once.
    pub fn initialize(
        &self,
        app: &mut App,
        roots: &[Entity],
        camera: Entity,
        sequence: usize,
        frozen: bool,
    ) -> Result<Option<(String, Vec<String>)>> {
        if let Self::Model(handle) = self {
            let server = app.world().resource::<AssetServer>();
            loading::check_asset(server, handle)?;
            if !server.is_loaded_with_dependencies(handle) {
                return Ok(None);
            }
        }
        if roots
            .iter()
            .any(|root| app.world().get::<ModelRig>(*root).is_none())
        {
            return Ok(None);
        }
        let (clip, name, diagnostics) = match self {
            Self::Character(prepared) => {
                let (clip, name) = select(prepared.clips(), sequence, false)?;
                (
                    clip,
                    name,
                    prepared
                        .diagnostics()
                        .iter()
                        .map(|diagnostic| diagnostic.message.clone())
                        .collect(),
                )
            }
            Self::Model(handle) => {
                let assets = app.world().resource::<Assets<ModelAsset>>();
                let Some(asset) = assets.get(handle) else {
                    return Ok(None);
                };
                if !asset.unresolved_images.is_empty() {
                    return Err(format!(
                        "model workload has unresolved images: {:?}",
                        asset.unresolved_images
                    )
                    .into());
                }
                let (clip, name) = select(asset.compiled.clips(), sequence, true)?;
                (
                    clip,
                    name,
                    asset
                        .compiled
                        .diagnostics()
                        .iter()
                        .map(|diagnostic| diagnostic.message.clone())
                        .collect(),
                )
            }
        };
        for (index, root) in roots.iter().enumerate() {
            let mut playback = app
                .world_mut()
                .get_mut::<Playback>(*root)
                .ok_or("missing playback")?;
            if let Some(clip) = clip {
                playback.play(clip);
            }
            playback.seek(0.5 + (index % 60) as f64 / 60.)?;
            playback.paused = frozen;
            playback.driving_camera = Some(camera);
        }
        Ok(Some((name, diagnostics)))
    }
}
fn select(
    clips: &[(ClipId, String)],
    sequence: usize,
    allow_rest: bool,
) -> Result<(Option<ClipId>, String)> {
    if allow_rest && clips.is_empty() && sequence == 0 {
        return Ok((None, "rest".into()));
    }
    let (clip, name) = clips.get(sequence).ok_or("sequence out of range")?;
    Ok((Some(*clip), name.clone()))
}

/// This headless application loads only its workload closure. Check all loaded
/// model assets before/after timing, including ready children not yet populated.
pub fn validate_images(app: &App) -> Result<()> {
    for (identity, asset) in app.world().resource::<Assets<ModelAsset>>().iter() {
        if !asset.unresolved_images.is_empty() {
            return Err(format!(
                "workload model {identity:?} has unresolved images: {:?}",
                asset.unresolved_images
            )
            .into());
        }
    }
    Ok(())
}
