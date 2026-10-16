//! Run with `cargo run -p bevy-wc3 --example viewer -- path/to/model.mdx` (or `.mdl`).
//! Use `--lod auto --lod-bias 2` to retain more detail during automatic LOD selection.
//! Use `--tonemapping tony-mcmapface` to enable tonemapping and `--illuminance LUX` to set lighting.
use bevy::asset::AssetPlugin;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy_wc3::{
    AssetSettings, Lod, LodSettings, ModelAsset, ModelInstance, ModelRig, Playback, Wc3BevyPlugin,
};
use std::sync::Arc;
#[path = "support/resolver.rs"]
mod resolver;
use resolver::{parse_wmo_profile, ModelResolver};
use std::f32::consts::FRAC_PI_2;
use std::path::PathBuf;

#[derive(Resource)]
struct Source {
    path: PathBuf,
    handle: Handle<ModelAsset>,
    tonemapping: Tonemapping,
}

#[derive(Component)]
struct OrbitCamera {
    target: Vec3,
    distance: f32,
    yaw: f32,
    pitch: f32,
    min_distance: f32,
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let path = PathBuf::from(arguments.next().expect("pass an MDX or MDL path"))
        .canonicalize()
        .expect("find model file");
    let mut resolver = ModelResolver::default();
    let mut lod = Lod::Fixed(0);
    let mut lod_settings = LodSettings::default();
    let mut tonemapping = Tonemapping::None;
    let mut illuminance = 5_000.0;
    while let Some(option) = arguments.next() {
        if option.starts_with("--lod") {
            let value = arguments.next().expect("pass a value after LOD option");
            match option.as_str() {
                "--lod" => {
                    lod = if value == "auto" {
                        Lod::Automatic
                    } else {
                        Lod::Fixed(value.parse().expect("LOD must be auto or a level number"))
                    }
                }
                "--lod-bias" => {
                    lod_settings.quality_bias = value.parse().expect("LOD bias must be numeric")
                }
                "--lod-thresholds" => {
                    lod_settings.thresholds = value
                        .split(',')
                        .map(|value| value.parse().expect("LOD thresholds must be numeric"))
                        .collect()
                }
                "--lod-minimum" => {
                    lod_settings.minimum_level =
                        value.parse().expect("LOD minimum must be a level number")
                }
                "--lod-hysteresis" => {
                    lod_settings.hysteresis = value.parse().expect("LOD hysteresis must be numeric")
                }
                _ => panic!("unknown option {option}"),
            }
            continue;
        }
        if option == "--allow-loss" {
            resolver.allow_loss = true;
            continue;
        }
        if option == "--wmo-profile" {
            resolver.wmo_profile = Some(
                parse_wmo_profile(&arguments.next().expect("pass WMO profile"))
                    .expect("valid WMO profile"),
            );
            continue;
        }
        if option == "--tonemapping" {
            tonemapping = match arguments.next().as_deref() {
                Some("none") => Tonemapping::None,
                Some("tony-mcmapface") => Tonemapping::TonyMcMapface,
                _ => panic!("pass none or tony-mcmapface after --tonemapping"),
            };
            continue;
        }
        if option == "--illuminance" {
            illuminance = arguments
                .next()
                .expect("pass lux after --illuminance")
                .parse::<f32>()
                .expect("illuminance must be numeric");
            assert!(
                illuminance.is_finite() && illuminance >= 0.0,
                "illuminance must be finite and nonnegative"
            );
            continue;
        }
        let choice = arguments.next().expect("pass ID=path after texture option");
        let (index, texture_path) = choice.split_once('=').expect("expected ID=path");
        let index: usize = index.parse().expect("expected numeric texture ID or slot");
        match option.as_str() {
            "--replaceable" => {
                resolver
                    .replaceables
                    .insert(index as u32, texture_path.to_owned());
            }
            "--bitmap" => {
                resolver.bitmaps.insert(index, texture_path.to_owned());
            }
            "--file-id" => {
                resolver
                    .file_ids
                    .insert(index as u32, texture_path.to_owned());
            }
            _ => panic!("unknown option {option}"),
        }
    }
    lod_settings.validate().expect("valid LOD quality settings");
    let directory = path.parent().expect("model parent directory");
    let filename = path
        .file_name()
        .expect("model filename")
        .to_string_lossy()
        .into_owned();
    App::new()
        .insert_resource(lod_settings)
        .insert_resource(AssetSettings {
            resolver: Arc::new(resolver),
        })
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: directory.to_string_lossy().into_owned(),
                ..default()
            }),
            Wc3BevyPlugin,
        ))
        .add_systems(
            Startup,
            move |mut commands: Commands, assets: Res<AssetServer>| {
                let handle = assets.load(filename.clone());
                commands.spawn((ModelInstance::new(handle.clone()), lod));
                commands.insert_resource(Source {
                    path: path.clone(),
                    handle,
                    tonemapping,
                });
                commands.spawn((
                    DirectionalLight {
                        illuminance,
                        ..default()
                    },
                    Transform::from_xyz(1.0, 2.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
                ));
            },
        )
        .add_systems(Update, (frame_model, orbit_camera, controls).chain())
        .run();
}

fn frame_model(
    mut commands: Commands,
    source: Res<Source>,
    models: Res<Assets<ModelAsset>>,
    cameras: Query<Entity, With<OrbitCamera>>,
) {
    if !cameras.is_empty() {
        return;
    }
    let Some(asset) = models.get(&source.handle) else {
        return;
    };
    let bounds = asset.compiled.rest_bounds();
    let center = bounds.map_or(Vec3::ZERO, |bounds| bounds.center.into());
    let radius = bounds.map_or(1., |bounds| bounds.half_extents.length());
    let radius = radius.max(0.1);
    let offset = Vec3::new(0.7, 0.55, 1.5).normalize();
    let distance = radius * 4.0;
    commands.spawn((
        Camera3d::default(),
        source.tonemapping,
        Msaa::Sample4,
        Transform::from_translation(center + offset * distance).looking_at(center, Vec3::Y),
        OrbitCamera {
            target: center,
            distance,
            yaw: offset.x.atan2(offset.z),
            pitch: offset.y.asin(),
            min_distance: radius * 0.1,
        },
    ));
    info!(
        "loaded {:?}, {} authored clips; left drag rotates, right drag pans, scroll zooms, Space changes sequence",
        source.path, asset.compiled.clips().len()
    );
}

fn orbit_camera(
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut cameras: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    let Ok((mut transform, mut orbit)) = cameras.single_mut() else {
        return;
    };
    let delta = motion.delta;
    if buttons.pressed(MouseButton::Left) {
        orbit.yaw -= delta.x * 0.005;
        orbit.pitch = (orbit.pitch + delta.y * 0.005).clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);
    } else if buttons.pressed(MouseButton::Right) {
        let scale = orbit.distance * 0.0015;
        orbit.target += transform.rotation * Vec3::new(-delta.x * scale, delta.y * scale, 0.0);
    }
    let wheel = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / 100.0,
    };
    orbit.distance = (orbit.distance * (-wheel * 0.15).exp())
        .clamp(orbit.min_distance, orbit.min_distance * 500.0);
    let (sin_yaw, cos_yaw) = orbit.yaw.sin_cos();
    let (sin_pitch, cos_pitch) = orbit.pitch.sin_cos();
    let offset = Vec3::new(sin_yaw * cos_pitch, sin_pitch, cos_yaw * cos_pitch);
    transform.translation = orbit.target + offset * orbit.distance;
    transform.look_at(orbit.target, Vec3::Y);
}

fn controls(keys: Res<ButtonInput<KeyCode>>, mut animations: Query<(&ModelRig, &mut Playback)>) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    for (instance, mut animation) in &mut animations {
        let clips = instance.model().clips();
        if clips.is_empty() {
            continue;
        }
        let current = clips
            .iter()
            .position(|(id, _)| Some(*id) == animation.clip)
            .unwrap_or(0);
        let next = (current + 1) % clips.len();
        animation.play(clips[next].0);
        info!("sequence {}", next);
    }
}
