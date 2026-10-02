//! Run with `cargo run -p bevy-wc3 --example viewer -- path/to/model.mdx` (or `.mdl`).
//! Use `--tonemapping tony-mcmapface` to enable tonemapping and `--illuminance LUX` to set lighting.
use bevy::asset::AssetPlugin;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy_wc3::{
    Wc3Animation, Wc3BevyPlugin, Wc3ModelAsset, Wc3ModelInstance, Wc3TextureBindings,
    Wc3TextureSlot,
};
use std::f32::consts::FRAC_PI_2;
use std::path::PathBuf;

#[derive(Resource)]
struct Source {
    path: PathBuf,
    handle: Handle<Wc3ModelAsset>,
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
    let mut choices = Vec::new();
    let mut tonemapping = Tonemapping::None;
    let mut illuminance = 5_000.0;
    while let Some(option) = arguments.next() {
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
        let slot = match option.as_str() {
            "--replaceable" => None,
            "--bitmap" => Some(Wc3TextureSlot::Bitmap(index)),
            "--particle2" => Some(Wc3TextureSlot::Particle2(index)),
            _ => panic!("unknown option {option}"),
        };
        choices.push((index, slot, texture_path.to_owned()));
    }
    let directory = path.parent().expect("model parent directory");
    let filename = path
        .file_name()
        .expect("model filename")
        .to_string_lossy()
        .into_owned();
    App::new()
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
                let mut bindings = Wc3TextureBindings::default();
                for (index, slot, texture_path) in &choices {
                    let image = assets.load(texture_path.clone());
                    if let Some(slot) = slot {
                        bindings.set_slot(*slot, image);
                    } else {
                        bindings.set_replaceable(*index as u32, image);
                    }
                }
                commands.spawn((Wc3ModelInstance::new(handle.clone()), bindings));
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
                    Transform::from_xyz(1.0, -1.0, 2.0).looking_at(Vec3::ZERO, Vec3::Z),
                ));
            },
        )
        .add_systems(Update, (frame_model, orbit_camera, controls).chain())
        .run();
}

fn frame_model(
    mut commands: Commands,
    source: Res<Source>,
    models: Res<Assets<Wc3ModelAsset>>,
    cameras: Query<Entity, With<OrbitCamera>>,
) {
    if !cameras.is_empty() {
        return;
    }
    let Some(asset) = models.get(&source.handle) else {
        return;
    };
    let model = asset.source();
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
    let offset = Vec3::new(0.7, -1.5, 0.55).normalize();
    let distance = radius * 4.0;
    commands.spawn((
        Camera3d::default(),
        source.tonemapping,
        Msaa::Sample4,
        Transform::from_translation(center + offset * distance).looking_at(center, Vec3::Z),
        OrbitCamera {
            target: center,
            distance,
            yaw: offset.x.atan2(-offset.y),
            pitch: offset.z.asin(),
            min_distance: radius * 0.1,
        },
    ));
    info!(
        "loaded {:?}, source version {}; left drag rotates, right drag pans, scroll zooms, Space changes sequence",
        source.path, model.source_version
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
    let offset = Vec3::new(sin_yaw * cos_pitch, -cos_yaw * cos_pitch, sin_pitch);
    transform.translation = orbit.target + offset * orbit.distance;
    transform.look_at(orbit.target, Vec3::Z);
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
