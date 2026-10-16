//! Capture an MDX or MDL offscreen at fixed animation times.
//! Run `cargo run -p bevy-wc3 --example capture -- --help` for options.
use bevy::app::PluginsState;
use bevy::asset::{AssetPlugin, LoadState, RecursiveDependencyLoadState};
use bevy::camera::{RenderTarget, ShadowLodOrigin};
use bevy::core_pipeline::prepass::{DepthPrepass, MotionVectorPrepass, NormalPrepass};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::render_resource::{CachedPipelineState, PipelineCache, PollType, TextureFormat};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::render::{RenderApp, RenderPlugin};
use bevy::shader::ShaderCacheError;
use bevy::time::TimeUpdateStrategy;
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use bevy_wc3::{
    spawn_universal_model, PreparedUniversalModel, UniversalAnimationError, UniversalAssetSettings,
    UniversalBakeSpace, UniversalCameraBinding, UniversalInstance, UniversalLod,
    UniversalLodOverride, UniversalLodSettings, UniversalMaterial, UniversalModelAsset,
    UniversalModelInstance, UniversalModelInstanceError, UniversalPlayback, UniversalPoseBaker,
    UniversalPoseOptions, UniversalWorldSelection, Wc3BevyPlugin,
};
use std::collections::{BTreeMap, HashMap};
use std::env::args;
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use wc3::model::formats::wmo::Profile as WmoProfile;
#[path = "support/resolver.rs"]
mod resolver;
use resolver::{parse_wmo_profile, ModelResolver};

type CaptureResult<T> = Result<T, Box<dyn Error>>;
const TIMEOUT: Duration = Duration::from_secs(30);
const HELP: &str = "Usage: capture MODEL OUTPUT_DIR [options]

Render offscreen and write frame-0000-1.000s.png, etc. Requires a GPU.
  --allow-loss          Explicitly discard unsupported native data, retaining diagnostics
  --wmo-profile NAME    classic, legion, bfa, shadowlands-pre92, shadowlands, dragonflight, warwithin
  --file-id ID=PATH     Map a native file-data identity to an asset-root-relative file
  --bake-pose           Render offline-baked geometry instead of the live hierarchy
  --prepasses           Enable depth/normal/motion prepasses and shadows
  --no-default-light    Omit the capture scene light (isolate model lights)
  --bevy-reference      Add a StandardMaterial sphere at (0, 2, 0)
  --tonemapping MODE    none or tony-mcmapface (default: none)
  --illuminance LUX     Scene directional light intensity (default: 5000)
  --times SECONDS,...    Increasing capture times (default: 1)
  --benchmark-frames N   Measure N synchronized frames at each captured pose; write CSV
  --lod LEVEL|auto       Geometry LOD (default: 0)
  --lod-bias NUMBER       Automatic quality bias (default: 1)
  --lod-thresholds PIXELS,...  Descending LOD transition diameters
  --lod-hysteresis NUMBER  LOD dead band fraction (default: 0.15)
  --lod-minimum LEVEL     Finest permitted authored level (default: 0)
  --sequence INDEX       Animation sequence index (default: 0)
  --play SECONDS=INDEX   Switch sequence at this simulation time (repeatable)
  --blend-ms NUMBER      Override BlendTime for scheduled switches (0: immediate)
  --fps NUMBER           Simulation steps per second (default: 60)
  --size WIDTHxHEIGHT    PNG dimensions (default: 640x480)
  --eye X,Y,Z            Camera position (default: frame model bounds)
  --target X,Y,Z         Camera target (default: model bounds center)
  --model-camera INDEX   Play an authored model camera (excludes --eye/--target)
  --camera-fov-multiplier NUMBER  Authored FOV scale (default: 1; portraits: 0.75)
  --asset-root PATH      Texture asset root (default: model's directory)
  --replaceable ID=PATH  Replaceable texture override
  --bitmap INDEX=PATH    Semantic texture override (including effect textures)
Texture override paths are relative to the asset root. Existing captures are overwritten.";

struct TextureChoice {
    index: u32,
    slot: Option<usize>,
    path: String,
}

struct PlaybackChange {
    time: f64,
    sequence: usize,
}

struct Options {
    allow_loss: bool,
    wmo_profile: Option<WmoProfile>,
    file_ids: HashMap<u32, String>,
    bake_pose: bool,
    prepasses: bool,
    bevy_reference: bool,
    no_default_light: bool,
    tonemapping: Tonemapping,
    illuminance: f32,
    model: PathBuf,
    output: PathBuf,
    times: Vec<f64>,
    benchmark_frames: Option<usize>,
    sequence: usize,
    lod: UniversalLod,
    lod_settings: UniversalLodSettings,
    plays: Vec<PlaybackChange>,
    blend: Option<Duration>,
    fps: f64,
    size: UVec2,
    eye: Option<Vec3>,
    target: Option<Vec3>,
    model_camera: Option<usize>,
    camera_fov_multiplier: f32,
    asset_root: Option<PathBuf>,
    textures: Vec<TextureChoice>,
}

impl Options {
    fn parse(arguments: impl IntoIterator<Item = String>) -> CaptureResult<Option<Self>> {
        let mut arguments = arguments.into_iter();
        let Some(model) = arguments.next() else {
            return Ok(None);
        };
        if model == "--help" || model == "-h" {
            return Ok(None);
        }
        let output = arguments
            .next()
            .ok_or("pass an output directory after the model path")?;
        let mut options = Self {
            allow_loss: false,
            wmo_profile: None,
            file_ids: HashMap::new(),
            bake_pose: false,
            prepasses: false,
            bevy_reference: false,
            no_default_light: false,
            tonemapping: Tonemapping::None,
            illuminance: 5_000.0,
            model: model.into(),
            output: output.into(),
            times: vec![1.0],
            benchmark_frames: None,
            sequence: 0,
            lod: UniversalLod::Fixed(0),
            lod_settings: UniversalLodSettings::default(),
            plays: Vec::new(),
            blend: None,
            fps: 60.0,
            size: UVec2::new(640, 480),
            eye: None,
            target: None,
            model_camera: None,
            camera_fov_multiplier: 1.0,
            asset_root: None,
            textures: Vec::new(),
        };
        while let Some(option) = arguments.next() {
            if option == "--help" || option == "-h" {
                return Ok(None);
            }
            if option == "--no-default-light" {
                options.no_default_light = true;
                continue;
            }
            if option == "--allow-loss" {
                options.allow_loss = true;
                continue;
            }
            if option == "--bake-pose" {
                options.bake_pose = true;
                continue;
            }
            if option == "--prepasses" {
                options.prepasses = true;
                continue;
            }
            if option == "--bevy-reference" {
                options.bevy_reference = true;
                continue;
            }
            let value = arguments
                .next()
                .ok_or_else(|| format!("missing value for {option}"))?;
            match option.as_str() {
                "--wmo-profile" => {
                    options.wmo_profile = Some(parse_wmo_profile(&value)?);
                }
                "--file-id" => {
                    let (id, path) = value.split_once('=').ok_or("file-id must be ID=PATH")?;
                    options.file_ids.insert(id.parse()?, path.to_owned());
                }
                "--tonemapping" => {
                    options.tonemapping = match value.as_str() {
                        "tony-mcmapface" => Tonemapping::TonyMcMapface,
                        "none" => Tonemapping::None,
                        _ => return Err("tonemapping must be none or tony-mcmapface".into()),
                    };
                }
                "--illuminance" => options.illuminance = value.parse()?,
                "--times" => {
                    options.times = value.split(',').map(str::parse).collect::<Result<_, _>>()?
                }
                "--sequence" => options.sequence = value.parse()?,
                "--benchmark-frames" => {
                    let count = value.parse()?;
                    if count == 0 {
                        return Err("benchmark-frames must be positive".into());
                    }
                    options.benchmark_frames = Some(count);
                }
                "--lod" => {
                    options.lod = if value == "auto" {
                        UniversalLod::Automatic
                    } else {
                        UniversalLod::Fixed(value.parse()?)
                    }
                }
                "--lod-bias" => options.lod_settings.quality_bias = value.parse()?,
                "--lod-thresholds" => {
                    options.lod_settings.thresholds =
                        value.split(',').map(str::parse).collect::<Result<_, _>>()?
                }
                "--lod-hysteresis" => options.lod_settings.hysteresis = value.parse()?,
                "--lod-minimum" => options.lod_settings.minimum_level = value.parse()?,
                "--play" => {
                    let (time, sequence) = value.split_once('=').ok_or("expected SECONDS=INDEX")?;
                    let time: f64 = time.parse()?;
                    if !time.is_finite() || time < 0.0 {
                        return Err("play time must be finite and nonnegative".into());
                    }
                    if options
                        .plays
                        .last()
                        .is_some_and(|previous| previous.time >= time)
                    {
                        return Err("play times must be strictly increasing".into());
                    }
                    options.plays.push(PlaybackChange {
                        time,
                        sequence: sequence.parse()?,
                    });
                }
                "--blend-ms" => options.blend = Some(Duration::from_millis(value.parse()?)),
                "--fps" => options.fps = value.parse()?,
                "--size" => {
                    let (width, height) = value.split_once('x').ok_or("expected WIDTHxHEIGHT")?;
                    options.size = UVec2::new(width.parse()?, height.parse()?);
                }
                "--eye" => options.eye = Some(parse_vector(&value)?),
                "--target" => options.target = Some(parse_vector(&value)?),
                "--model-camera" => options.model_camera = Some(value.parse()?),
                "--camera-fov-multiplier" => options.camera_fov_multiplier = value.parse()?,
                "--asset-root" => options.asset_root = Some(value.into()),
                "--replaceable" | "--bitmap" => {
                    let (index, path) = value.split_once('=').ok_or("expected ID=PATH")?;
                    let index: u32 = index.parse()?;
                    if path.is_empty() {
                        return Err("texture path cannot be empty".into());
                    }
                    let slot = match option.as_str() {
                        "--bitmap" => Some(index as usize),
                        _ => None,
                    };
                    options.textures.push(TextureChoice {
                        index,
                        slot,
                        path: path.to_owned(),
                    });
                }
                _ => return Err(format!("unknown option: {option}").into()),
            }
        }
        if options.bake_pose && !options.plays.is_empty() {
            return Err("bake-pose does not support pose transitions (--play)".into());
        }
        options.lod_settings.validate()?;
        if !options.fps.is_finite() || !(1.0..=1000.0).contains(&options.fps) {
            return Err("fps must be between 1 and 1000".into());
        }
        if options.model_camera.is_some() && (options.eye.is_some() || options.target.is_some()) {
            return Err("model-camera cannot be combined with eye or target".into());
        }
        if !options.camera_fov_multiplier.is_finite() || options.camera_fov_multiplier <= 0.0 {
            return Err("camera-fov-multiplier must be finite and positive".into());
        }
        if options.model_camera.is_none() && options.camera_fov_multiplier != 1.0 {
            return Err("camera-fov-multiplier requires model-camera".into());
        }
        if !options.illuminance.is_finite() || options.illuminance < 0.0 {
            return Err("illuminance must be finite and nonnegative".into());
        }
        if options.size.min_element() == 0 {
            return Err("image dimensions must be positive".into());
        }
        if options.times.is_empty()
            || options
                .times
                .iter()
                .any(|&time| !time.is_finite() || time < 0.0)
            || options.times.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(
                "capture times must be finite, nonnegative, and strictly increasing".into(),
            );
        }
        for &time in &options.times {
            Duration::try_from_secs_f64(time)?;
        }
        Ok(Some(options))
    }

    fn camera(&self, model: &PreparedUniversalModel) -> CaptureResult<Transform> {
        if let Some(index) = self.model_camera {
            if model.cameras().nth(index).is_none() {
                return Err(format!("model camera {index} is out of range").into());
            }
            return Ok(Transform::IDENTITY);
        }
        let bounds = model.rest_bounds();
        let center = bounds.map_or(Vec3::ZERO, |bounds| bounds.center.into());
        let radius = bounds
            .map_or(1., |bounds| bounds.half_extents.length())
            .max(0.1);
        let target = self.target.unwrap_or(center);
        let aspect = self.size.x as f32 / self.size.y as f32;
        let eye = self.eye.unwrap_or(
            target + Vec3::new(0.7, 0.55, 1.5).normalize() * radius * 4. * (1. / aspect).max(1.),
        );
        let direction = target - eye;
        if direction.length_squared() < 0.000001 {
            return Err("camera eye and target must differ".into());
        }
        let up = if direction.normalize().cross(Vec3::Y).length_squared() < 0.000001 {
            Vec3::Z
        } else {
            Vec3::Y
        };
        Ok(Transform::from_translation(eye).looking_at(target, up))
    }
}

fn parse_vector(value: &str) -> CaptureResult<Vec3> {
    let components: Vec<f32> = value.split(',').map(str::parse).collect::<Result<_, _>>()?;
    if components.len() != 3 || components.iter().any(|component| !component.is_finite()) {
        return Err("expected three finite coordinates: X,Y,Z".into());
    }
    Ok(Vec3::new(components[0], components[1], components[2]))
}

#[derive(Resource, Default)]
struct ScreenshotResult(Option<Result<(), String>>);

fn replace_with_baked_pose(
    app: &mut App,
    prepared: &PreparedUniversalModel,
    baker: &mut UniversalPoseBaker,
    root: Entity,
    camera: Entity,
    entities: &mut Vec<Entity>,
) -> CaptureResult<()> {
    let world = app.world_mut();
    for entity in entities.drain(..) {
        world.despawn(entity);
    }
    let playback = world.get::<UniversalPlayback>(root).unwrap();
    let camera_world = world.get::<GlobalTransform>(camera).unwrap().to_matrix();
    let camera =
        Transform::from_matrix(prepared.model_to_bevy().to_matrix().inverse() * camera_world);
    let pose = baker.bake(
        prepared,
        world.resource::<Assets<Mesh>>(),
        world.resource::<Assets<SkinnedMeshInverseBindposes>>(),
        UniversalPoseOptions {
            playback: Some(playback),
            world_selection: world.get::<UniversalWorldSelection>(root),
            camera: Some(camera),
            lod: Some(playback.lod),
            space: UniversalBakeSpace::Bevy,
            ..Default::default()
        },
    )?;
    for part in pose.parts.into_iter().filter(|part| part.visible) {
        let Some(material) = part.material else {
            continue;
        };
        let mesh = world.resource_mut::<Assets<Mesh>>().add(part.mesh);
        let material = world
            .resource_mut::<Assets<UniversalMaterial>>()
            .add(material);
        entities.push(
            world
                .spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY))
                .id(),
        );
    }
    world.entity_mut(root).insert(Visibility::Hidden);
    Ok(())
}

fn main() -> CaptureResult<()> {
    let Some(options) = Options::parse(args().skip(1))? else {
        println!("{HELP}");
        return Ok(());
    };
    capture(options)
}

fn capture(options: Options) -> CaptureResult<()> {
    let model = options.model.canonicalize()?;
    let asset_root = options
        .asset_root
        .clone()
        .unwrap_or_else(|| model.parent().unwrap().to_owned())
        .canonicalize()?;
    let path = model
        .strip_prefix(&asset_root)
        .map_err(|_| "model must be inside the asset root")?
        .to_string_lossy()
        .into_owned();
    let mut app = App::new();
    app.insert_resource(UniversalAssetSettings {
        resolver: Arc::new(ModelResolver {
            allow_loss: options.allow_loss,
            file_ids: options.file_ids.clone(),
            bitmaps: options
                .textures
                .iter()
                .filter_map(|choice| choice.slot.map(|index| (index, choice.path.clone())))
                .collect(),
            replaceables: options
                .textures
                .iter()
                .filter(|choice| choice.slot.is_none())
                .map(|choice| (choice.index, choice.path.clone()))
                .collect(),
            wmo_profile: options.wmo_profile,
        }),
    });
    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                file_path: asset_root.to_string_lossy().into_owned(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .set(RenderPlugin {
                synchronous_pipeline_compilation: true,
                ..default()
            })
            .disable::<WinitPlugin>()
            .disable::<PipelinedRenderingPlugin>(),
        Wc3BevyPlugin,
    ));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app.insert_resource(options.lod_settings.clone());
    app.init_resource::<ScreenshotResult>();
    let deadline = Instant::now() + TIMEOUT;
    while app.plugins_state() != PluginsState::Ready {
        check_deadline(deadline, "initializing renderer")?;
        sleep(Duration::from_millis(5));
    }
    app.finish();
    app.cleanup();
    let limit = app
        .world()
        .resource::<RenderDevice>()
        .limits()
        .max_texture_dimension_2d;
    if options.size.max_element() > limit {
        return Err(format!("image dimensions exceed the GPU limit of {limit}").into());
    }
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(1));
    let assets = app.world().resource::<AssetServer>().clone();
    let handle: Handle<UniversalModelAsset> = assets.load(path);
    let deadline = Instant::now() + TIMEOUT;
    loop {
        step(&mut app, Duration::ZERO)?;
        check_asset(&assets, &handle)?;
        if assets.is_loaded_with_dependencies(&handle) {
            break;
        }
        check_deadline(deadline, "loading Universal model and textures")?;
        sleep(Duration::from_millis(5));
    }
    let prepared = app
        .world_mut()
        .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(
                |world, mut binds: Mut<Assets<SkinnedMeshInverseBindposes>>| {
                    let models = world.resource::<Assets<UniversalModelAsset>>();
                    let asset = models.get(&handle).unwrap();
                    let mut diagnostics = BTreeMap::new();
                    for diagnostic in asset.compiled.diagnostics() {
                        *diagnostics.entry(&diagnostic.message).or_insert(0usize) += 1;
                    }
                    for (message, count) in diagnostics {
                        eprintln!("Universal rendering ({count} occurrences): {message}");
                    }
                    for resource in &asset.unresolved_images {
                        eprintln!("Unresolved image: {resource:?}");
                    }
                    asset.compiled.prepare(&mut meshes, &mut binds)
                },
            )
        });
    let clips = prepared.clips();
    if options.sequence >= clips.len() && !(clips.is_empty() && options.sequence == 0) {
        return Err("Universal sequence index is out of range".into());
    }
    for change in &options.plays {
        if change.sequence >= clips.len() {
            return Err("scheduled Universal sequence index is out of range".into());
        }
    }
    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            options.size.x,
            options.size.y,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    let camera = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            options.tonemapping,
            ShadowLodOrigin,
            Msaa::Sample4,
            RenderTarget::Image(image.clone().into()),
            options.camera(&prepared)?,
        ))
        .id();
    if options.prepasses {
        app.world_mut().entity_mut(camera).insert((
            DepthPrepass,
            NormalPrepass,
            MotionVectorPrepass,
        ));
    }
    if !options.no_default_light {
        app.world_mut().spawn((
            DirectionalLight {
                illuminance: options.illuminance,
                shadow_maps_enabled: options.prepasses,
                ..default()
            },
            Transform::from_xyz(1., 2., 1.).looking_at(Vec3::ZERO, Vec3::Y),
        ));
    }
    if options.bevy_reference {
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Sphere::new(0.5));
        let material = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.15, 0.05),
                metallic: 0.6,
                perceptual_roughness: 0.3,
                ..default()
            });
        app.world_mut().spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0., 2., 0.),
        ));
    }
    let root = app
        .world_mut()
        .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(|world, mut materials: Mut<Assets<UniversalMaterial>>| {
                let mut commands = world.commands();
                spawn_universal_model(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &prepared,
                    Transform::IDENTITY,
                )
            })
        });
    app.world_mut().flush();
    app.world_mut().entity_mut(root).insert((
        options.lod,
        UniversalLodOverride(options.lod_settings.clone()),
    ));
    if let Some(index) = options.model_camera {
        let feature = prepared
            .cameras()
            .nth(index)
            .ok_or("model camera index out of range")?;
        app.world_mut()
            .entity_mut(camera)
            .insert(UniversalCameraBinding {
                fov_multiplier: options.camera_fov_multiplier,
                ..UniversalCameraBinding::new(root, feature.id)
            });
    }
    {
        let mut playback = app
            .world_mut()
            .get_mut::<UniversalPlayback>(root)
            .ok_or("Universal playback was not spawned")?;
        playback.clip = clips.get(options.sequence).map(|(id, _)| *id);
        playback.driving_camera = Some(camera);
        if let UniversalLod::Fixed(level) = options.lod {
            playback.lod = level;
        }
    }
    settle(&mut app)?;
    create_dir_all(&options.output)?;
    let mut elapsed = 0.;
    let mut next_play = 0;
    let mut baker = UniversalPoseBaker::default();
    let mut baked_entities = Vec::new();
    for (index, &time) in options.times.iter().enumerate() {
        loop {
            if let Some(change) = options
                .plays
                .get(next_play)
                .filter(|change| change.time <= elapsed + 1e-9 && change.time <= time + 1e-9)
            {
                let mut playback = app.world_mut().get_mut::<UniversalPlayback>(root).unwrap();
                let clip = clips[change.sequence].0;
                if let Some(duration) = options.blend {
                    playback.play_with_blend(clip, duration);
                } else {
                    playback.play(clip);
                }
                next_play += 1;
                continue;
            }
            if elapsed + 1e-9 >= time {
                break;
            }
            let target_time = options
                .plays
                .get(next_play)
                .map_or(time, |change| time.min(change.time));
            let dt = Duration::try_from_secs_f64((target_time - elapsed).min(1. / options.fps))?;
            step(&mut app, dt)?;
            elapsed += dt.as_secs_f64();
        }
        settle(&mut app)?;
        if let Some(error) = app
            .world()
            .get::<UniversalAnimationError>(root)
            .and_then(|error| error.0.as_ref())
        {
            return Err(format!("Universal animation failed: {error}").into());
        }
        if let Some(camera_index) = options.model_camera {
            let instance = app.world().get::<UniversalInstance>(root).unwrap();
            let playback = app.world().get::<UniversalPlayback>(root).unwrap();
            let feature = prepared.cameras().nth(camera_index).unwrap();
            let sample = instance
                .sample_camera(
                    feature.id,
                    playback,
                    options.size.x as f32 / options.size.y as f32,
                )?
                .ok_or("authored camera is unavailable")?;
            sample
                .perspective_projection(options.camera_fov_multiplier)
                .ok_or("authored camera has invalid lens")?;
            let root_world = app
                .world()
                .get::<GlobalTransform>(root)
                .copied()
                .unwrap_or_default();
            sample
                .world_transform(root_world.mul_transform(prepared.model_to_bevy()))
                .ok_or("authored camera has a degenerate view transform")?;
        }
        if options.bake_pose {
            replace_with_baked_pose(
                &mut app,
                &prepared,
                &mut baker,
                root,
                camera,
                &mut baked_entities,
            )?;
            settle(&mut app)?;
        }
        if let Some(frames) = options.benchmark_frames {
            benchmark_pose(&mut app, &options, index, time, frames)?;
        }
        let path = options
            .output
            .join(format!("frame-{index:04}-{time:.3}s.png"));
        app.world_mut().resource_mut::<ScreenshotResult>().0 = None;
        let destination = path.clone();
        app.world_mut()
            .spawn(Screenshot::image(image.clone()))
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
        let deadline = Instant::now() + TIMEOUT;
        loop {
            step(&mut app, Duration::ZERO)?;
            if let Some(result) = app.world_mut().resource_mut::<ScreenshotResult>().0.take() {
                result.map_err(|error| format!("saving {}: {error}", path.display()))?;
                break;
            }
            check_deadline(deadline, "waiting for screenshot readback")?;
            sleep(Duration::from_millis(5));
        }
        println!("{} (Universal, time {time:.6}s)", path.display());
    }
    Ok(())
}

fn check_asset<A: Asset>(assets: &AssetServer, handle: &Handle<A>) -> CaptureResult<()> {
    if let LoadState::Failed(error) = assets.load_state(handle) {
        return Err(format!("asset load failed: {error}").into());
    }
    if let RecursiveDependencyLoadState::Failed(error) =
        assets.recursive_dependency_load_state(handle)
    {
        return Err(format!("texture dependency load failed: {error}").into());
    }
    Ok(())
}

fn benchmark_pose(
    app: &mut App,
    options: &Options,
    index: usize,
    time: f64,
    frames: usize,
) -> CaptureResult<()> {
    let path = options
        .output
        .join(format!("timings-{index:04}-{time:.3}s.csv"));
    let mut output = File::create(&path)?;
    writeln!(output, "frame,pose_seconds,wall_ms")?;
    let synchronize = |app: &App| -> CaptureResult<()> {
        app.sub_app(RenderApp)
            .world()
            .resource::<RenderDevice>()
            .poll(PollType::Wait {
                submission_index: None,
                timeout: Some(TIMEOUT),
            })?;
        Ok(())
    };
    synchronize(app)?;
    // Warm resource residency and driver caches at the measured pose.
    for _ in 0..8 {
        step(app, Duration::ZERO)?;
        synchronize(app)?;
    }
    let mut samples = Vec::with_capacity(frames);
    for frame in 0..frames {
        let start = Instant::now();
        step(app, Duration::ZERO)?;
        synchronize(app)?;
        let milliseconds = start.elapsed().as_secs_f64() * 1000.0;
        samples.push(milliseconds);
        // File writes and image readback are outside the measured interval.
        writeln!(output, "{frame},{time:.9},{milliseconds:.6}")?;
    }
    samples.sort_by(f64::total_cmp);
    println!(
        "{}: {frames} synchronized frames, median {:.3} ms, p95 {:.3} ms",
        path.display(),
        samples[frames / 2],
        samples[frames - frames / 20 - 1]
    );
    Ok(())
}

fn check_deadline(deadline: Instant, operation: &str) -> CaptureResult<()> {
    if Instant::now() >= deadline {
        return Err(format!("timed out {operation}").into());
    }
    Ok(())
}

fn step(app: &mut App, dt: Duration) -> CaptureResult<()> {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() = TimeUpdateStrategy::ManualDuration(dt);
    app.update();
    let cache = app.sub_app(RenderApp).world().resource::<PipelineCache>();
    for pipeline in cache.pipelines() {
        if let CachedPipelineState::Err(error) = &pipeline.state {
            // Missing shader assets/imports are retried by Bevy while loading.
            if !matches!(
                error,
                ShaderCacheError::ShaderNotLoaded(_)
                    | ShaderCacheError::ShaderImportNotYetAvailable
            ) {
                return Err(format!("GPU pipeline failed: {error:?}").into());
            }
        }
    }
    Ok(())
}

fn settle(app: &mut App) -> CaptureResult<()> {
    let deadline = Instant::now() + TIMEOUT;
    let mut frames = 0;
    let mut requests = app
        .world_mut()
        .query_filtered::<(), With<UniversalModelInstance>>();
    let mut errors = app.world_mut().query::<&UniversalModelInstanceError>();
    loop {
        step(app, Duration::ZERO)?;
        if let Some(error) = errors.iter(app.world()).next() {
            return Err(format!("model instance failed: {error:?}").into());
        }
        frames += 1;
        if frames == 8 {
            let cache = app.sub_app(RenderApp).world().resource::<PipelineCache>();
            if cache
                .pipelines()
                .any(|pipeline| matches!(pipeline.state, CachedPipelineState::Err(_)))
            {
                for (_, shader) in app.world().resource::<Assets<Shader>>().iter() {
                    if shader.path.contains("universal") {
                        eprintln!(
                            "Pending shader {}: {:?}; imports {:?}",
                            shader.path, shader.import_path, shader.imports
                        );
                    }
                }
                for pipeline in cache.pipelines() {
                    if let CachedPipelineState::Err(error) = &pipeline.state {
                        eprintln!("Pending pipeline: {error:?}");
                    }
                }
            }
        }
        if frames >= 8
            && requests.iter(app.world()).next().is_none()
            && app
                .sub_app(RenderApp)
                .world()
                .resource::<PipelineCache>()
                .waiting_pipelines()
                .next()
                .is_none()
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            let cache = app.sub_app(RenderApp).world().resource::<PipelineCache>();
            let errors: Vec<_> = cache
                .pipelines()
                .filter_map(|pipeline| {
                    if let CachedPipelineState::Err(error) = &pipeline.state {
                        Some(format!("{error:?}"))
                    } else {
                        None
                    }
                })
                .collect();
            return Err(format!(
                "timed out preparing render pipelines; unresolved shader errors: {}",
                errors.join("; ")
            )
            .into());
        }
        sleep(Duration::from_millis(5));
    }
}

#[cfg(test)]
#[path = "support/model.rs"]
mod semantic;

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(options: &[&str]) -> CaptureResult<Option<Options>> {
        Options::parse(
            ["test.mdl", "captures"]
                .into_iter()
                .chain(options.iter().copied())
                .map(str::to_owned),
        )
    }

    #[test]
    fn validates_benchmark_frame_count() {
        assert_eq!(
            parse(&["--benchmark-frames", "120"])
                .unwrap()
                .unwrap()
                .benchmark_frames,
            Some(120)
        );
        for value in ["0", "-1", "1.5"] {
            assert!(parse(&["--benchmark-frames", value]).is_err());
        }
    }

    #[test]
    fn validates_lod_capture_configuration() {
        let options = parse(&[
            "--lod",
            "auto",
            "--lod-bias",
            "2",
            "--lod-thresholds",
            "300,150",
            "--lod-minimum",
            "2",
        ])
        .unwrap()
        .unwrap();
        assert_eq!(options.lod, UniversalLod::Automatic);
        assert_eq!(options.lod_settings.quality_bias, 2.0);
        assert_eq!(options.lod_settings.minimum_level, 2);
        assert_eq!(options.lod_settings.thresholds, [300.0, 150.0]);
        assert_eq!(
            parse(&["--lod", "2"]).unwrap().unwrap().lod,
            UniversalLod::Fixed(2)
        );
        for arguments in [
            vec!["--lod", "bad"],
            vec!["--lod-bias", "NaN"],
            vec!["--lod-thresholds", "100,200"],
            vec!["--lod-hysteresis", "1"],
        ] {
            assert!(parse(&arguments).is_err());
        }
    }

    #[test]
    fn validates_scheduled_sequence_switches() {
        for invalid in ["NaN=1", "-1=1", "inf=1", "0.5", "0.5=x"] {
            assert!(parse(&["--play", invalid]).is_err());
        }
        assert!(parse(&["--play", "1=0", "--play", "0.5=1"]).is_err());
        assert!(parse(&["--play", "1=0", "--play", "1=1"]).is_err());
        let options = parse(&["--play", "0=1", "--play", "0.75=0", "--blend-ms", "0"])
            .unwrap()
            .unwrap();
        assert_eq!(options.plays.len(), 2);
        assert_eq!(options.plays[1].time, 0.75);
        assert_eq!(options.blend, Some(Duration::ZERO));
    }

    #[test]
    fn validates_capture_schedule_and_camera_arguments() {
        for times in ["NaN", "-1", "1,0", "1,1", "inf"] {
            assert!(parse(&["--times", times]).is_err());
        }
        assert!(parse(&["--size", "0x480"]).is_err());
        assert!(parse(&["--fps", "0"]).is_err());
        assert!(parse(&["--eye", "1,2"]).is_err());
        assert!(parse(&["--eye", "1,2,NaN"]).is_err());
        assert!(parse(&["--times"]).is_err());
        for illuminance in ["NaN", "inf", "-1"] {
            assert!(parse(&["--illuminance", illuminance]).is_err());
        }
        assert!(parse(&["--tonemapping", "unknown"]).is_err());
        assert!(parse(&["--model-camera", "0", "--eye", "0,-10,0"]).is_err());
        assert!(parse(&["--model-camera", "0", "--target", "0,0,0"]).is_err());
        for multiplier in ["0", "-1", "NaN", "inf"] {
            assert!(
                parse(&["--model-camera", "0", "--camera-fov-multiplier", multiplier]).is_err()
            );
        }
        assert!(parse(&["--camera-fov-multiplier", "0.75"]).is_err());
        let options = parse(&[
            "--times", "0,0.5,1", "--size", "320x240", "--eye", "0,-15,5",
        ])
        .unwrap()
        .unwrap();
        assert_eq!(options.times, vec![0.0, 0.5, 1.0]);
        assert_eq!(options.size, UVec2::new(320, 240));
        assert_eq!(options.eye, Some(Vec3::new(0.0, -15.0, 5.0)));
        assert_eq!(options.tonemapping, Tonemapping::None);
        assert_eq!(options.illuminance, 5_000.0);
        let original = parse(&["--tonemapping", "tony-mcmapface", "--illuminance", "20000"])
            .unwrap()
            .unwrap();
        assert_eq!(original.tonemapping, Tonemapping::TonyMcMapface);
        assert_eq!(original.illuminance, 20_000.0);
    }

    #[test]
    fn authored_camera_selection_validates_index_before_loading_gpu() {
        let model = semantic::decode(include_bytes!("../tests/fixtures/cameras.mdl")).unwrap();
        let compiled = bevy_wc3::compile_universal_model(&model, |_| None).unwrap();
        let model = compiled.prepare(
            &mut Assets::<Mesh>::default(),
            &mut Assets::<SkinnedMeshInverseBindposes>::default(),
        );
        let options = parse(&["--model-camera", "0", "--camera-fov-multiplier", "0.75"])
            .unwrap()
            .unwrap();
        assert_eq!(options.model_camera, Some(0));
        assert_eq!(options.camera_fov_multiplier, 0.75);
        assert!(options.camera(&model).is_ok());
        assert!(parse(&["--model-camera", "2"])
            .unwrap()
            .unwrap()
            .camera(&model)
            .is_err());
    }
}
