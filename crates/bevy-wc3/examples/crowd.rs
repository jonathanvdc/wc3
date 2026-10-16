//! Pipelined equipped-character throughput. See docs/tools.md for methodology.
#[path = "support/character.rs"]
mod character;
#[path = "support/profiling.rs"]
mod profiling;

use bevy::app::PluginsState;
use bevy::camera::{RenderTarget, ShadowLodOrigin};
use bevy::core_pipeline::prepass::{DepthPrepass, MotionVectorPrepass, NormalPrepass};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::prelude::*;
use bevy::render::diagnostic::RenderDiagnosticsPlugin;
use bevy::render::render_resource::TextureFormat;
use bevy::render::renderer::{RenderAdapterInfo, RenderDevice};
use bevy::time::TimeUpdateStrategy;
use bevy::window::{ExitCondition, WindowPlugin};
use bevy::winit::WinitPlugin;
use bevy_wc3::{
    spawn_model, Lod, LodSettings, ModelDraw, ModelMaterial, ModelNode, Playback, PlaybackError,
    Wc3BevyPlugin,
};
use serde::Serialize;
use std::collections::HashSet;
use std::error::Error;
use std::fs::write;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct Options {
    model: PathBuf,
    recipe: PathBuf,
    assets: PathBuf,
    output: PathBuf,
    count: usize,
    frames: usize,
    warmup: usize,
    sequence: usize,
    frozen: bool,
    prepasses: bool,
    shadows: bool,
    allow_loss: bool,
    lod: Lod,
}
impl Options {
    fn parse() -> Result<Self> {
        let mut args = std::env::args().skip(1);
        let help = "crowd MODEL RECIPE ASSET_ROOT OUTPUT.json [--count 100] [--frames 300] [--warmup 120] [--sequence 0] [--frozen] [--prepasses] [--shadows] [--allow-loss] [--lod 0|auto]";
        let Some(model) = args.next() else {
            return Err(help.into());
        };
        if model == "--help" {
            println!("{help}");
            std::process::exit(0);
        }
        let mut options = Self {
            model: model.into(),
            recipe: args.next().ok_or(help)?.into(),
            assets: args.next().ok_or(help)?.into(),
            output: args.next().ok_or(help)?.into(),
            count: 100,
            frames: 300,
            warmup: 120,
            sequence: 0,
            frozen: false,
            prepasses: false,
            shadows: false,
            allow_loss: false,
            lod: Lod::Fixed(0),
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--frozen" => options.frozen = true,
                "--prepasses" => options.prepasses = true,
                "--shadows" => options.shadows = true,
                "--allow-loss" => options.allow_loss = true,
                "--lod" => {
                    let value = args.next().ok_or("missing LOD")?;
                    options.lod = if value == "auto" {
                        Lod::Automatic
                    } else {
                        Lod::Fixed(value.parse()?)
                    };
                }
                "--count" => options.count = args.next().ok_or("missing count")?.parse()?,
                "--frames" => options.frames = args.next().ok_or("missing frames")?.parse()?,
                "--warmup" => options.warmup = args.next().ok_or("missing warmup")?.parse()?,
                "--sequence" => {
                    options.sequence = args.next().ok_or("missing sequence")?.parse()?
                }
                _ => return Err(format!("unknown argument {arg}").into()),
            }
        }
        if options.frames < 30 || options.warmup < 30 || options.count > 100 {
            return Err("use at least 30 measured/warmup frames and at most 100 characters".into());
        }
        Ok(options)
    }
}

#[derive(Serialize)]
struct Summary {
    median_ms: f64,
    p95_ms: f64,
}
fn summary(mut values: Vec<f64>) -> Summary {
    values.sort_by(f64::total_cmp);
    Summary {
        median_ms: values[values.len() / 2],
        p95_ms: values[(values.len() * 95 / 100).min(values.len() - 1)],
    }
}

fn check(app: &mut App) -> Result<()> {
    if let Some(exit) = app.should_exit() {
        return Err(format!("render application exited: {exit:?}").into());
    }
    let mut errors = app.world_mut().query::<&PlaybackError>();
    for error in errors.iter(app.world()) {
        if let Some(error) = &error.0 {
            return Err(error.clone().into());
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let options = Options::parse()?;
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                file_path: options
                    .assets
                    .canonicalize()?
                    .to_string_lossy()
                    .into_owned(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .disable::<WinitPlugin>(),
        Wc3BevyPlugin,
        RenderDiagnosticsPlugin,
    ));
    app.insert_resource(LodSettings {
        default_lod: options.lod,
        ..Default::default()
    });
    profiling::install(&mut app);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    let deadline = Instant::now() + Duration::from_secs(120);
    while app.plugins_state() != PluginsState::Ready {
        if Instant::now() > deadline {
            return Err("renderer initialization timed out".into());
        }
        sleep(Duration::from_millis(5));
    }
    app.finish();
    app.cleanup();
    let mut diagnostics = Vec::new();
    let prepared = {
        let compiled = {
            let mut images = app.world_mut().resource_mut::<Assets<Image>>();
            character::load(
                &options.recipe,
                &options.assets,
                &options.model,
                options.allow_loss,
                &mut images,
            )?
        };
        for diagnostic in compiled.diagnostics() {
            eprintln!("Character rendering: {}", diagnostic.message);
            diagnostics.push(diagnostic.message.clone());
        }
        app.world_mut()
            .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                let mut binds = world.resource_mut::<Assets<SkinnedMeshInverseBindposes>>();
                compiled.prepare(&mut meshes, &mut binds)
            })
    };
    let (clip, clip_name) = prepared
        .clips()
        .get(options.sequence)
        .ok_or("sequence out of range")?;
    let clip = *clip;
    let clip_name = clip_name.clone();
    let target = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            1280,
            720,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    let camera = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            Tonemapping::None,
            Msaa::Sample4,
            RenderTarget::Image(target.into()),
            ShadowLodOrigin,
            Transform::from_xyz(0., 20., 32.).looking_at(Vec3::new(0., 0.8, 0.), Vec3::Y),
        ))
        .id();
    if options.prepasses {
        app.world_mut().entity_mut(camera).insert((
            DepthPrepass,
            NormalPrepass,
            MotionVectorPrepass,
        ));
    }
    app.world_mut().spawn((
        DirectionalLight {
            illuminance: 10000.,
            shadow_maps_enabled: options.shadows,
            ..default()
        },
        Transform::from_xyz(1., 2., 1.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let columns = options.count.clamp(1, 10);
    let rows = options.count.div_ceil(columns).max(1);
    let mut roots = Vec::new();
    for i in 0..options.count {
        let transform = Transform::from_xyz(
            (i % columns) as f32 * 2.5 - (columns - 1) as f32 * 1.25,
            0.,
            (i / columns) as f32 * 2.5 - (rows - 1) as f32 * 1.25,
        );
        let root = app
            .world_mut()
            .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                world.resource_scope(|world, mut materials: Mut<Assets<ModelMaterial>>| {
                    spawn_model(
                        &mut world.commands(),
                        &mut meshes,
                        &mut materials,
                        &prepared,
                        transform,
                    )
                })
            });
        roots.push(root);
    }
    app.world_mut().flush();
    for (index, root) in roots.into_iter().enumerate() {
        let mut playback = app
            .world_mut()
            .get_mut::<Playback>(root)
            .ok_or("missing playback")?;
        playback.play(clip);
        playback.seek(0.5 + (index % 60) as f64 / 60.)?;
        playback.paused = options.frozen;
        playback.driving_camera = Some(camera);
    }
    let dt = Duration::from_secs_f64(1. / 60.);
    let deadline = Instant::now() + Duration::from_secs(180);
    let mut preparation_frames = 0;
    // Zero delta keeps the measured starting phase independent of preparation latency.
    while preparation_frames < 120 || !profiling::ready(&app)? {
        profiling::pace(&app)?;
        app.update();
        check(&mut app)?;
        preparation_frames += 1;
        if Instant::now() > deadline {
            return Err("pipeline preparation timed out".into());
        }
    }
    app.insert_resource(TimeUpdateStrategy::ManualDuration(dt));
    for _ in 0..options.warmup {
        profiling::pace(&app)?;
        app.update();
        check(&mut app)?;
        if Instant::now() > deadline {
            return Err("warmup timed out".into());
        }
    }
    profiling::synchronize(&mut app, preparation_frames + options.warmup)?;
    profiling::reset(&mut app);
    let start = Instant::now();
    let mut frames = Vec::new();
    for _ in 0..options.frames {
        let frame = Instant::now();
        profiling::pace(&app)?;
        app.update();
        frames.push(frame.elapsed().as_secs_f64() * 1000.);
        if let Some(exit) = app.should_exit() {
            return Err(format!("render application exited: {exit:?}").into());
        }
    }
    // A single terminal drain includes outstanding GPU work without serializing each frame.
    profiling::drain(&mut app)?;
    let elapsed = start.elapsed().as_secs_f64();
    check(&mut app)?;
    let mut nodes = app.world_mut().query_filtered::<Entity, With<ModelNode>>();
    let nodes = nodes.iter(app.world()).count();
    let mut draws=app.world_mut().query_filtered::<(&Mesh3d,&MeshMaterial3d<ModelMaterial>,&ViewVisibility),With<ModelDraw>>();
    let mut mesh_ids = HashSet::new();
    let mut material_ids = HashSet::new();
    let mut visible = 0;
    let mut draw_count = 0;
    let mut visible_triangles = 0;
    let meshes = app.world().resource::<Assets<Mesh>>();
    for (mesh, material, visibility) in draws.iter(app.world()) {
        draw_count += 1;
        mesh_ids.insert(mesh.id());
        material_ids.insert(material.id());
        visible += usize::from(visibility.get());
        if visibility.get() {
            if let Some(mesh) = meshes.get(mesh.id()) {
                visible_triangles += mesh
                    .indices()
                    .map_or_else(|| mesh.count_vertices(), |indices| indices.len())
                    / 3;
            }
        }
    }
    let mut skins = app.world_mut().query::<(&SkinnedMesh, &ViewVisibility)>();
    let skin_instances = skins.iter(app.world()).count();
    let joint_references: usize = skins
        .iter(app.world())
        .map(|(skin, _)| skin.joints.len())
        .sum();
    let visible_skins = skins
        .iter(app.world())
        .filter(|(_, visibility)| visibility.get())
        .count();
    let visible_joints: usize = skins
        .iter(app.world())
        .filter(|(_, visibility)| visibility.get())
        .map(|(skin, _)| skin.joints.len())
        .sum();
    let unique_joints: HashSet<_> = skins
        .iter(app.world())
        .flat_map(|(skin, _)| skin.joints.iter().copied())
        .collect();
    let unique_palettes: HashSet<_> = skins
        .iter(app.world())
        .map(|(skin, _)| skin.inverse_bindposes.id())
        .collect();
    let adapter = app.world().resource::<RenderAdapterInfo>();
    let features = app.world().resource::<RenderDevice>().features();
    let report = serde_json::json!({
        "sequence_index":options.sequence,"sequence_name":clip_name,"characters":options.count,"animated":!options.frozen,"prepasses":options.prepasses,"shadows":options.shadows,
        "lod_policy":format!("{:?}",options.lod),"visible_triangle_references":visible_triangles,"resolution":[1280,720],"msaa_samples":4,"frames":options.frames,"warmup_frames":options.warmup,
        "preparation_frames":preparation_frames,"rendering_diagnostics":diagnostics,"import_loss_accepted":options.allow_loss,"fixed_simulation_hz":60,"wall_ms_per_frame_including_terminal_drain":elapsed*1000./options.frames as f64,
        "update_wall":summary(frames),"adapter":format!("{} / {:?} / {:?}",adapter.name,adapter.backend,adapter.device_type),"device_features":format!("{features:?}"),
        "debug_assertions":cfg!(debug_assertions),"model_nodes":nodes,"draw_entities":draw_count,"visible_draw_entities":visible,
        "unique_meshes":mesh_ids.len(),"unique_materials":material_ids.len(),"skinned_draw_entities":skin_instances,"joint_references":joint_references,"visible_skinned_draw_entities":visible_skins,"visible_joint_references":visible_joints,"unique_joint_entities":unique_joints.len(),"unique_inverse_bind_palettes":unique_palettes.len(),
        "timings":profiling::report(&app),"note":"Draw entities are not hardware draw calls. CPU spans may overlap render work; GPU spans must not be summed across nested passes."
    });
    write(&options.output, serde_json::to_vec_pretty(&report)?)?;
    println!("{report}");
    Ok(())
}
