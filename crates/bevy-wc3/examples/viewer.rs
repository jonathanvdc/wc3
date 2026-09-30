//! Run with `cargo run -p bevy-wc3 --example viewer -- path/to/model.mdx`.
use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use wc3::blp::BlpRef;
use bevy_wc3::{spawn_model, Wc3Animation, Wc3BevyPlugin, Wc3LayerMaterial, Wc3Model};

#[derive(Resource)]
struct Source(PathBuf);

fn main() {
    let path = std::env::args().nth(1).expect("pass an MDX path");
    App::new()
        .add_plugins((DefaultPlugins, Wc3BevyPlugin))
        .insert_resource(Source(path.into()))
        .add_systems(Startup, setup)
        .add_systems(Update, controls)
        .run();
}

fn setup(
    mut commands: Commands,
    source: Res<Source>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<Wc3LayerMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    let bytes = fs::read(&source.0).expect("read MDX");
    let model = Wc3Model::decode(&bytes).expect("decode and normalize MDX");
    let texture_files = find_textures(source.0.parent().unwrap_or(Path::new(".")));
    let mut loaded: HashMap<String, Handle<Image>> = HashMap::new();
    let root = spawn_model(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut inverse_bindposes,
        &model,
        |name| {
            let file_name = Path::new(name.replace('\\', "/").as_str())
                .file_name()?
                .to_string_lossy()
                .to_ascii_lowercase();
            if let Some(handle) = loaded.get(&file_name) {
                return Some(handle.clone());
            }
            let path = texture_files.get(&file_name)?;
            let image = load_image(path).ok()?;
            let handle = images.add(image);
            loaded.insert(file_name, handle.clone());
            Some(handle)
        },
    )
    .expect("compile model for Bevy");
    let info = model.model.model_info();
    let center = info
        .as_ref()
        .map(|info| {
            (Vec3::from_array(info.minimum_extent) + Vec3::from_array(info.maximum_extent)) * 0.5
        })
        .unwrap_or(Vec3::ZERO);
    let radius = info
        .map(|info| info.bounds_radius)
        .unwrap_or(100.0)
        .max(10.0);
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(center + Vec3::new(radius * 1.6, -radius * 2.2, radius * 0.9))
            .looking_at(center, Vec3::Z),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 20_000.0,
            ..default()
        },
        Transform::from_xyz(1.0, -1.0, 2.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    info!(
        "loaded {:?}, source version {}, root {:?}; press Space to change sequence",
        source.0, model.source_version, root
    );
}

fn controls(keys: Res<ButtonInput<KeyCode>>, mut animations: Query<&mut Wc3Animation>) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    for mut animation in &mut animations {
        let next = (animation.sequence + 1) % animation.sequences().len().max(1);
        animation.play(next);
        info!("sequence {}", next);
    }
}

fn find_textures(directory: &Path) -> HashMap<String, PathBuf> {
    let mut result = HashMap::new();
    fn visit(directory: &Path, result: &mut HashMap<String, PathBuf>) {
        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit(&path, result);
            } else if let Some(name) = path.file_name() {
                result.insert(name.to_string_lossy().to_ascii_lowercase(), path);
            }
        }
    }
    visit(directory, &mut result);
    result
}

fn load_image(path: &Path) -> Result<Image, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("blp"))
    {
        let rgba = BlpRef::read(&bytes)?.decode_mip(0)?;
        return Ok(Image::new(
            Extent3d {
                width: rgba.width(),
                height: rgba.height(),
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            rgba.into_raw(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        ));
    }
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("");
    Ok(Image::from_buffer(
        &bytes,
        ImageType::Extension(extension),
        CompressedImageFormats::BC,
        true,
        ImageSampler::Default,
        RenderAssetUsages::default(),
    )?)
}
