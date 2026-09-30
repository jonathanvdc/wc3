//! File-backed model and texture loading through Bevy's asset server.
use bevy::asset::RenderAssetUsages;
use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::path::Path;
use wc3::blp::BlpRef;

use crate::model::{ModelError, Wc3Model};

/// An MDX file with its resolved image dependencies.
#[derive(Asset, TypePath)]
pub struct Wc3ModelAsset {
    pub(crate) source: Wc3Model,
    pub(crate) textures: Vec<Option<Handle<Image>>>,
}

#[derive(Default, TypePath)]
pub(crate) struct Wc3ModelLoader;

impl AssetLoader for Wc3ModelLoader {
    type Asset = Wc3ModelAsset;
    type Settings = ();
    type Error = ModelError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        context: &mut LoadContext<'_>,
    ) -> Result<Wc3ModelAsset, ModelError> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|error| ModelError(error.to_string()))?;
        let source = Wc3Model::decode(&bytes)?;
        let model_path = context.path().path().to_owned();
        let mut textures = Vec::new();
        for texture in source.model.textures() {
            if texture.replaceable_id != 0 {
                textures.push(None);
                continue;
            }
            let mut resolved = None;
            for path in texture_paths(&model_path, &texture.path.text()) {
                if context.read_asset_bytes(path.clone()).await.is_ok() {
                    resolved = Some(context.load(path));
                    break;
                }
            }
            if resolved.is_none() && !texture.path.text().is_empty() {
                warn!(
                    "Could not resolve WC3 texture {} for {}",
                    texture.path.text(),
                    model_path.display()
                );
            }
            textures.push(resolved);
        }
        Ok(Wc3ModelAsset { source, textures })
    }

    fn extensions(&self) -> &[&str] {
        &["mdx"]
    }
}

/// Resolve beside the MDX first, then from the Bevy asset root.
fn texture_paths(model_path: &Path, name: &str) -> Vec<String> {
    let normalized = name.replace('\\', "/");
    if normalized.is_empty() {
        return Vec::new();
    }
    let path = Path::new(&normalized);
    if path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Vec::new();
    }
    let local = model_path.parent().unwrap_or(Path::new("")).join(path);
    let local = local.to_string_lossy().replace('\\', "/");
    if local == normalized {
        vec![normalized]
    } else {
        vec![local, normalized]
    }
}

#[derive(Default, TypePath)]
pub(crate) struct BlpImageLoader;

impl AssetLoader for BlpImageLoader {
    type Asset = Image;
    type Settings = ();
    type Error = ModelError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _context: &mut LoadContext<'_>,
    ) -> Result<Image, ModelError> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|error| ModelError(error.to_string()))?;
        let blp = BlpRef::read(&bytes).map_err(|error| ModelError(error.to_string()))?;
        let rgba = blp
            .decode_mip(0)
            .map_err(|error| ModelError(error.to_string()))?;
        Ok(Image::new(
            Extent3d {
                width: rgba.width(),
                height: rgba.height(),
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            rgba.into_raw(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        ))
    }

    fn extensions(&self) -> &[&str] {
        &["blp"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::AssetPlugin;
    use std::fs;
    use std::time::{Duration, Instant};

    #[test]
    fn texture_paths_try_model_then_asset_root() {
        let model = Path::new("units/footman.mdx");
        assert_eq!(
            texture_paths(model, "body.blp"),
            ["units/body.blp", "body.blp"]
        );
        assert_eq!(
            texture_paths(model, "Textures\\armor.blp"),
            ["units/Textures/armor.blp", "Textures/armor.blp"]
        );
        assert!(texture_paths(model, "../secret.blp").is_empty());
    }

    #[test]
    fn asset_server_loads_mdx_file() {
        let directory = std::env::temp_dir().join(format!(
            "bevy-wc3-assets-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join("quad.mdx"),
            include_bytes!("../../wc3/tests/fixtures/mdl/quad_model.mdx"),
        )
        .unwrap();
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                file_path: directory.to_string_lossy().into_owned(),
                ..default()
            },
        ));
        app.init_asset::<Image>();
        app.init_asset::<Wc3ModelAsset>();
        app.init_asset_loader::<Wc3ModelLoader>();
        let handle = app.world().resource::<AssetServer>().load("quad.mdx");
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            app.update();
            if app
                .world()
                .resource::<Assets<Wc3ModelAsset>>()
                .contains(&handle)
            {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(app
            .world()
            .resource::<Assets<Wc3ModelAsset>>()
            .contains(&handle));
        fs::remove_dir_all(directory).unwrap();
    }
}
