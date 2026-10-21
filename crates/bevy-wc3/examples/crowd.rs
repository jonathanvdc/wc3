//! Repeated model or equipped-character throughput. See docs/tools.md for methodology.
#[path = "support/character.rs"]
mod character;
#[path = "support/completion.rs"]
mod completion;
#[path = "support/crowd_model.rs"]
mod crowd_model;
#[path = "support/effect_populations.rs"]
mod effect_populations;
#[path = "support/loading.rs"]
mod loading;
#[path = "support/profiling.rs"]
mod profiling;
#[path = "support/resolver.rs"]
mod resolver;

use bevy::app::PluginsState;
use bevy::camera::{RenderTarget, ShadowLodOrigin};
use bevy::core_pipeline::prepass::{DepthPrepass, MotionVectorPrepass, NormalPrepass};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::ecs::schedule::{Schedules, SingleThreadedExecutor};
use bevy::mesh::skinning::SkinnedMesh;
use bevy::pbr::material_uses_bindless_resources;
use bevy::prelude::*;
use bevy::render::diagnostic::RenderDiagnosticsPlugin;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::render_resource::TextureFormat;
use bevy::render::renderer::{RenderAdapterInfo, RenderDevice};
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::render::RenderApp;
use bevy::time::TimeUpdateStrategy;
use bevy::window::{ExitCondition, WindowPlugin};
use bevy::winit::WinitPlugin;
use bevy_wc3::{
    AssetSettings, Lod, LodSettings, ModelDraw, ModelInstanceError, ModelMaterial, ModelNode,
    ParticleProgramEmitter, PlaybackError, Wc3BevyPlugin,
};
use completion::CompletionWait;
use crowd_model::CrowdModel;
use profiling::FramePolicy;
use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use wc3::model::formats::wmo::Profile as WmoProfile;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct Options {
    model: PathBuf,
    recipe: Option<PathBuf>,
    assets: PathBuf,
    output: PathBuf,
    screenshot: Option<PathBuf>,
    count: usize,
    frames: usize,
    warmup: usize,
    sequence: usize,
    frozen: bool,
    prepasses: bool,
    shadows: bool,
    allow_loss: bool,
    lod: Lod,
    frame_policy: FramePolicy,
    render_stages: bool,
    single_threaded: bool,
    msaa: Msaa,
    file_ids: HashMap<u32, String>,
    wmo_profile: Option<WmoProfile>,
}
impl Options {
    fn parse() -> Result<Self> {
        Self::parse_args(std::env::args().skip(1))
    }
    fn parse_args(arguments: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut args = arguments.into_iter();
        let help = "crowd MODEL RECIPE_OR_- ASSET_ROOT OUTPUT.json [--count 100] [--frames 300] [--warmup 120] [--sequence 0] [--frozen] [--prepasses] [--shadows] [--allow-loss] [--lod 0|auto] [--msaa 1|2|4|8] [--serial-completed-frames [--completion-wait spin|blocking]] [--file-id ID=PATH] [--wmo-profile PROFILE] [--screenshot OUTPUT.png] [--render-stages] [--single-threaded]";
        let Some(model) = args.next() else {
            return Err(help.into());
        };
        if model == "--help" {
            println!("{help}");
            std::process::exit(0);
        }
        let recipe = args.next().ok_or(help)?;
        let mut serial = false;
        let mut wait = None;
        let mut options = Self {
            model: model.into(),
            recipe: (recipe != "-").then(|| recipe.into()),
            assets: args.next().ok_or(help)?.into(),
            output: args.next().ok_or(help)?.into(),
            screenshot: None,
            count: 100,
            frames: 300,
            warmup: 120,
            sequence: 0,
            frozen: false,
            prepasses: false,
            shadows: false,
            allow_loss: false,
            lod: Lod::Fixed(0),
            frame_policy: FramePolicy::Pipelined,
            render_stages: false,
            single_threaded: false,
            msaa: Msaa::Sample4,
            file_ids: HashMap::new(),
            wmo_profile: None,
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--screenshot" => {
                    options.screenshot = Some(args.next().ok_or("missing screenshot path")?.into());
                }
                "--frozen" => options.frozen = true,
                "--render-stages" => options.render_stages = true,
                "--single-threaded" => options.single_threaded = true,
                "--serial-completed-frames" => serial = true,
                "--completion-wait" => {
                    wait = Some(
                        match args.next().ok_or("missing completion wait")?.as_str() {
                            "spin" => CompletionWait::SpinPoll,
                            "blocking" => CompletionWait::BlockingPoll,
                            _ => return Err("completion wait must be spin or blocking".into()),
                        },
                    )
                }
                "--msaa" => {
                    options.msaa = match args.next().ok_or("missing MSAA")?.as_str() {
                        "1" => Msaa::Off,
                        "2" => Msaa::Sample2,
                        "4" => Msaa::Sample4,
                        "8" => Msaa::Sample8,
                        _ => return Err("MSAA must be 1, 2, 4 or 8".into()),
                    }
                }
                "--file-id" => {
                    let value = args.next().ok_or("missing FileDataID alias")?;
                    let (id, path) = value.split_once('=').ok_or("use --file-id ID=PATH")?;
                    if path.is_empty() {
                        return Err("empty FileDataID path".into());
                    }
                    options.file_ids.insert(id.parse()?, path.to_owned());
                }
                "--wmo-profile" => {
                    options.wmo_profile = Some(resolver::parse_wmo_profile(
                        &args.next().ok_or("missing WMO profile")?,
                    )?)
                }
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
        options.frame_policy = if serial {
            FramePolicy::SerialCompleted {
                wait: wait.unwrap_or_default(),
            }
        } else {
            if wait.is_some() {
                return Err("--completion-wait requires --serial-completed-frames".into());
            }
            FramePolicy::Pipelined
        };
        if options.frames < 30 || options.warmup < 30 || options.count > 100 {
            return Err("use at least 30 measured/warmup frames and at most 100 instances".into());
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
    let mut loading = app.world_mut().query::<&ModelInstanceError>();
    if let Some(error) = loading.iter(app.world()).next() {
        return Err(format!("model request failed: {error:?}").into());
    }
    let mut programmes = app.world_mut().query::<&ParticleProgramEmitter>();
    for emitter in programmes.iter(app.world()) {
        if let Some(error) = &emitter.error {
            return Err(error.clone().into());
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let options = Options::parse()?;
    let assets = options.assets.canonicalize()?;
    let mut app = App::new();
    app.insert_resource(AssetSettings {
        resolver: Arc::new(resolver::ModelResolver {
            allow_loss: options.allow_loss,
            file_ids: options.file_ids.clone(),
            wmo_profile: options.wmo_profile,
            ..Default::default()
        }),
    });
    let plugins = DefaultPlugins
        .set(AssetPlugin {
            file_path: assets.to_string_lossy().into_owned(),
            ..default()
        })
        .set(WindowPlugin {
            primary_window: None,
            exit_condition: ExitCondition::DontExit,
            ..default()
        })
        .disable::<WinitPlugin>();
    let plugins = if options.frame_policy.is_serial() {
        plugins.disable::<PipelinedRenderingPlugin>()
    } else {
        plugins
    };
    app.add_plugins((plugins, Wc3BevyPlugin, RenderDiagnosticsPlugin));
    app.insert_resource(LodSettings {
        default_lod: options.lod,
        ..Default::default()
    });
    profiling::install(&mut app, options.render_stages);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    let deadline = Instant::now() + Duration::from_secs(120);
    while app.plugins_state() != PluginsState::Ready {
        if Instant::now() > deadline {
            return Err("renderer initialization timed out".into());
        }
        sleep(Duration::from_millis(5));
    }
    app.finish();
    if options.single_threaded {
        for (_, schedule) in app.world_mut().resource_mut::<Schedules>().iter_mut() {
            schedule.set_executor(SingleThreadedExecutor::new());
        }
        // Pipelined rendering transfers RenderApp during cleanup.
        for (_, schedule) in app
            .sub_app_mut(RenderApp)
            .world_mut()
            .resource_mut::<Schedules>()
            .iter_mut()
        {
            schedule.set_executor(SingleThreadedExecutor::new());
        }
    }
    app.cleanup();
    let source = CrowdModel::load(
        &mut app,
        options.recipe.as_deref(),
        &assets,
        &options.model,
        options.allow_loss,
    )?;
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
            options.msaa,
            RenderTarget::Image(target.clone().into()),
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
        let root = source.spawn(&mut app, transform);
        roots.push(root);
    }
    app.world_mut().flush();
    let dt = Duration::from_secs_f64(1. / 60.);
    let deadline = Instant::now() + Duration::from_secs(180);
    let mut preparation_frames = 0;
    // Initialize prepared characters before their first frame, as before. Ordinary
    // ModelInstance requests use the same zero-delta loop until ready.
    let mut initialized =
        source.initialize(&mut app, &roots, camera, options.sequence, options.frozen)?;
    let mut initialized_frames = 0;
    while initialized_frames < 120 || !profiling::ready(&app)? {
        profiling::frame(&mut app, options.frame_policy)?;
        check(&mut app)?;
        preparation_frames += 1;
        if initialized.is_some() {
            initialized_frames += 1;
        }
        if initialized.is_none() {
            initialized =
                source.initialize(&mut app, &roots, camera, options.sequence, options.frozen)?;
        }
        if Instant::now() > deadline {
            return Err("pipeline preparation timed out".into());
        }
    }
    let (clip_name, diagnostics) = initialized.unwrap();
    for diagnostic in &diagnostics {
        eprintln!("Rendering: {diagnostic}");
    }
    app.insert_resource(TimeUpdateStrategy::ManualDuration(dt));
    for _ in 0..options.warmup {
        profiling::frame(&mut app, options.frame_policy)?;
        check(&mut app)?;
        if Instant::now() > deadline {
            return Err("warmup timed out".into());
        }
    }
    crowd_model::validate_images(&app)?;
    profiling::synchronize(&mut app, preparation_frames + options.warmup)?;
    profiling::reset(&mut app);
    let start = Instant::now();
    let mut frames = Vec::new();
    let mut update_times = Vec::new();
    let mut wait_times = Vec::new();
    for _ in 0..options.frames {
        let frame = profiling::frame(&mut app, options.frame_policy)?;
        frames.push(frame.wall_ms);
        update_times.push(frame.update_ms);
        wait_times.push(frame.completion_wait_ms);
        if let Some(exit) = app.should_exit() {
            return Err(format!("render application exited: {exit:?}").into());
        }
    }
    // Include outstanding work and publish final diagnostics without another simulation step.
    profiling::drain(&mut app)?;
    let elapsed = start.elapsed().as_secs_f64();
    check(&mut app)?;
    crowd_model::validate_images(&app)?;
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
    let effects = effect_populations::collect(app.world_mut());
    let adapter = app.world().resource::<RenderAdapterInfo>();
    let features = app.world().resource::<RenderDevice>().features();
    let mut report = serde_json::json!({
        "sequence_index":options.sequence,"sequence_name":clip_name,"characters":options.recipe.as_ref().map(|_| options.count),"animated":!options.frozen,"prepasses":options.prepasses,"shadows":options.shadows,
        "lod_policy":format!("{:?}",options.lod),"visible_triangle_references":visible_triangles,"resolution":[1280,720],"msaa_samples":options.msaa.samples(),"frames":options.frames,"warmup_frames":options.warmup,
        "preparation_frames":preparation_frames,"rendering_diagnostics":diagnostics,"import_loss_accepted":options.allow_loss,"fixed_simulation_hz":60,"wall_ms_per_frame_including_terminal_drain":elapsed*1000./options.frames as f64,
        "update_wall":summary(frames),"adapter":format!("{} / {:?} / {:?}",adapter.name,adapter.backend,adapter.device_type),"device_features":format!("{features:?}"),
        "debug_assertions":cfg!(debug_assertions),"model_nodes":nodes,"draw_entities":draw_count,"visible_draw_entities":visible,
        "unique_meshes":mesh_ids.len(),"unique_materials":material_ids.len(),"skinned_draw_entities":skin_instances,"joint_references":joint_references,"visible_skinned_draw_entities":visible_skins,"visible_joint_references":visible_joints,"unique_joint_entities":unique_joints.len(),"unique_inverse_bind_palettes":unique_palettes.len(),
        "timings":profiling::report(&app, options.frame_policy),"note":"Draw entities are not hardware draw calls. CPU spans may overlap render work; GPU spans must not be summed across nested passes."
    });
    let Value::Object(workload) = serde_json::json!({
        "workload":source.kind(), "model":&options.model, "recipe":&options.recipe,
        "instances":options.count, "frame_policy":options.frame_policy, "effects":effects,
        "render_stage_windows": options.render_stages,
        "single_threaded_schedules": options.single_threaded,
        "bindless_model_materials": material_uses_bindless_resources::<ModelMaterial>(app.world().resource::<RenderDevice>()),
        "completion_wait_wall":options.frame_policy.is_serial().then(|| summary(wait_times)),
        "update_and_backpressure_wall":summary(update_times),
    }) else {
        unreachable!("workload metadata is an object")
    };
    report.as_object_mut().unwrap().extend(workload);
    write(&options.output, serde_json::to_vec_pretty(&report)?)?;
    println!("{report}");
    // The terminal drain, all timing calculations and the report write precede
    // screenshot setup. Readback uses zero simulation delta at the final pose.
    if let Some(path) = &options.screenshot {
        save_final_frame(&mut app, &target, path, options.frame_policy)?;
        eprintln!("Final crowd screenshot: {}", path.display());
    }
    Ok(())
}

#[derive(Resource, Default)]
struct ScreenshotResult(Option<std::result::Result<(), String>>);

/// Uses the same Screenshot::image observer/readback path as capture.rs.
/// This helper runs only after the measured report has been finalized.
fn save_final_frame(
    app: &mut App,
    target: &Handle<Image>,
    path: &Path,
    policy: FramePolicy,
) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        create_dir_all(parent)?;
    }
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    profiling::resume_rendering(app)?;
    app.init_resource::<ScreenshotResult>();
    let destination = path.to_path_buf();
    app.world_mut()
        .spawn(Screenshot::image(target.clone()))
        .observe(
            move |event: On<ScreenshotCaptured>, mut result: ResMut<ScreenshotResult>| {
                result.0 = Some(
                    event
                        .image
                        .clone()
                        .try_into_dynamic()
                        .map_err(|error| error.to_string())
                        .and_then(|image| {
                            image
                                .to_rgb8()
                                .save(&destination)
                                .map_err(|error| error.to_string())
                        }),
                );
            },
        );
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        profiling::frame(app, policy)?;
        check(app)?;
        if let Some(result) = app.world_mut().resource_mut::<ScreenshotResult>().0.take() {
            result.map_err(|error| format!("saving {}: {error}", path.display()))?;
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("waiting for final crowd screenshot readback timed out".into());
        }
        sleep(Duration::from_millis(5));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn options(extra: &[&str]) -> Result<Options> {
        Options::parse_args(
            ["model.mdl", "-", "/assets", "/tmp/report.json"]
                .into_iter()
                .chain(extra.iter().copied())
                .map(str::to_owned),
        )
    }
    #[test]
    fn ordinary_models_keep_the_default_pipeline_and_quality() {
        let options = options(&[]).unwrap();
        assert!(options.recipe.is_none());
        assert_eq!(options.frame_policy, FramePolicy::Pipelined);
        assert!(!options.single_threaded);
        assert_eq!(options.msaa, Msaa::Sample4);
        assert_eq!(
            (options.count, options.frames, options.warmup),
            (100, 300, 120)
        );
        let character = Options::parse_args(
            ["body.m2", "recipe.json", "/assets", "/tmp/report.json"].map(str::to_owned),
        )
        .unwrap();
        assert_eq!(character.recipe.as_deref(), Some(Path::new("recipe.json")));
    }
    #[test]
    fn serial_completion_requires_an_explicit_policy_boundary() {
        let serial = options(&["--serial-completed-frames", "--msaa", "1"]).unwrap();
        assert_eq!(
            serial.frame_policy,
            FramePolicy::SerialCompleted {
                wait: CompletionWait::SpinPoll
            }
        );
        assert_eq!(serial.msaa, Msaa::Off);
        let blocking =
            options(&["--completion-wait", "blocking", "--serial-completed-frames"]).unwrap();
        assert_eq!(
            blocking.frame_policy,
            FramePolicy::SerialCompleted {
                wait: CompletionWait::BlockingPoll
            }
        );
        assert!(options(&["--completion-wait", "spin"]).is_err());
        assert!(options(&["--serial-completed-frames", "--completion-wait", "event"]).is_err());
    }
    #[test]
    fn single_threaded_schedules_preserve_the_selected_frame_boundary() {
        let pipelined = options(&["--single-threaded", "--count", "0"]).unwrap();
        assert!(pipelined.single_threaded);
        assert_eq!(pipelined.count, 0);
        assert_eq!(pipelined.frame_policy, FramePolicy::Pipelined);
        let serial = options(&[
            "--single-threaded",
            "--serial-completed-frames",
            "--completion-wait",
            "spin",
        ])
        .unwrap();
        assert!(serial.single_threaded);
        assert_eq!(
            serial.frame_policy,
            FramePolicy::SerialCompleted {
                wait: CompletionWait::SpinPoll
            }
        );
    }

    #[test]
    fn invalid_quality_or_resource_controls_fail_before_loading() {
        assert!(options(&["--msaa", "3"]).is_err());
        assert!(options(&["--file-id", "123="]).is_err());
        assert!(options(&["--file-id", "missing-path"]).is_err());
        assert!(options(&["--wmo-profile", "guess"]).is_err());
        let aliases = options(&["--file-id", "123=filedata/123.m2", "--count", "0"]).unwrap();
        assert_eq!(
            aliases.file_ids.get(&123).map(String::as_str),
            Some("filedata/123.m2")
        );
        assert_eq!(aliases.count, 0);
    }
}
