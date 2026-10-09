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
use bevy::render::render_resource::{CachedPipelineState, PipelineCache, TextureFormat};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::render::{RenderApp, RenderPlugin};
use bevy::shader::ShaderCacheError;
use bevy::time::TimeUpdateStrategy;
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use bevy_wc3::{
    prepare_model_asset, PreparedModel, Wc3Animation, Wc3BevyPlugin, Wc3CameraBinding,
    Wc3CameraSample, Wc3LayerMaterial, Wc3Lod, Wc3LodOverride, Wc3LodSettings, Wc3LodState,
    Wc3Model, Wc3ModelAsset, Wc3ModelCameras, Wc3ModelInstance, Wc3NodeCamera, Wc3PoseBaker,
    Wc3PoseOptions, Wc3TextureBindings, Wc3TextureSlot,
};
use std::env::args;
use std::error::Error;
use std::fs::{create_dir_all, read};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, Instant};

type CaptureResult<T> = Result<T, Box<dyn Error>>;
const TIMEOUT: Duration = Duration::from_secs(30);
const HELP: &str = "Usage: capture MODEL.{mdx,mdl} OUTPUT_DIR [options]

Render offscreen and write frame-0000-1.000s.png, etc. Requires a GPU.
  --bake-pose           Render offline-baked geometry instead of the live hierarchy
  --prepasses           Enable depth/normal/motion prepasses and shadows
  --no-default-light    Omit the capture scene light (isolate model lights)
  --bevy-reference      Add a StandardMaterial sphere at (0, 2, 0)
  --tonemapping MODE    none or tony-mcmapface (default: none)
  --illuminance LUX     Scene directional light intensity (default: 5000)
  --times SECONDS,...    Increasing capture times (default: 1)
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
  --bitmap INDEX=PATH    Model bitmap override
  --particle2 INDEX=PATH PRE2 texture override
Texture override paths are relative to the asset root. Existing captures are overwritten.";

struct TextureChoice {
    index: u32,
    slot: Option<Wc3TextureSlot>,
    path: String,
}

struct PlaybackChange {
    time: f64,
    sequence: usize,
}

struct Options {
    bake_pose: bool,
    prepasses: bool,
    bevy_reference: bool,
    no_default_light: bool,
    tonemapping: Tonemapping,
    illuminance: f32,
    model: PathBuf,
    output: PathBuf,
    times: Vec<f64>,
    sequence: usize,
    lod: Wc3Lod,
    lod_settings: Wc3LodSettings,
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
            bake_pose: false,
            prepasses: false,
            bevy_reference: false,
            no_default_light: false,
            tonemapping: Tonemapping::None,
            illuminance: 5_000.0,
            model: model.into(),
            output: output.into(),
            times: vec![1.0],
            sequence: 0,
            lod: Wc3Lod::default(),
            lod_settings: Wc3LodSettings::default(),
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
                "--lod" => {
                    options.lod = if value == "auto" {
                        Wc3Lod::Automatic
                    } else {
                        Wc3Lod::Fixed(value.parse()?)
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
                "--replaceable" | "--bitmap" | "--particle2" => {
                    let (index, path) = value.split_once('=').ok_or("expected ID=PATH")?;
                    let index: u32 = index.parse()?;
                    if path.is_empty() {
                        return Err("texture path cannot be empty".into());
                    }
                    let slot = match option.as_str() {
                        "--bitmap" => Some(Wc3TextureSlot::Bitmap(index as usize)),
                        "--particle2" => Some(Wc3TextureSlot::Particle2(index as usize)),
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

    fn camera(&self, model: &Wc3Model) -> CaptureResult<Transform> {
        if let Some(index) = self.model_camera {
            let definitions = model.model.cameras();
            let definition = definitions.get(index).ok_or_else(|| {
                format!(
                    "model camera {index} is out of range ({} cameras)",
                    definitions.len()
                )
            })?;
            let sample = Wc3CameraSample {
                position: Vec3::from_array(definition.position),
                target: Vec3::from_array(definition.target_position),
                roll: 0.0,
                field_of_view: definition.field_of_view,
                near_clip: definition.near_clip,
                far_clip: definition.far_clip,
            };
            sample
                .perspective_projection(self.camera_fov_multiplier)
                .ok_or("model camera has invalid FOV or clipping distances")?;
            // Animated pose is resolved after the instance has loaded.
            return Ok(Transform::default());
        }
        let info = model.model.model_info();
        let center = info
            .as_ref()
            .map(|info| {
                (Vec3::from_array(info.minimum_extent) + Vec3::from_array(info.maximum_extent))
                    * 0.5
            })
            .filter(|center| center.is_finite())
            .unwrap_or(Vec3::ZERO);
        let radius = info
            .map(|info| {
                info.bounds_radius.max(
                    (Vec3::from_array(info.maximum_extent) - Vec3::from_array(info.minimum_extent))
                        .length()
                        * 0.5,
                )
            })
            .filter(|radius| radius.is_finite())
            .unwrap_or(100.0)
            .max(1.0);
        let target = self.target.unwrap_or(center);
        let aspect = self.size.x as f32 / self.size.y as f32;
        let eye = self.eye.unwrap_or(
            target
                + Vec3::new(0.7, -1.5, 0.55).normalize() * radius * 4.0 * (1.0 / aspect).max(1.0),
        );
        let direction = target - eye;
        if direction.length_squared() < 0.000001 {
            return Err("camera eye and target must differ".into());
        }
        let up = if direction.normalize().cross(Vec3::Z).length_squared() < 0.000001 {
            Vec3::Y
        } else {
            Vec3::Z
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
    prepared: &PreparedModel,
    baker: &mut Wc3PoseBaker,
    root: Entity,
    camera: Entity,
    entities: &mut Vec<Entity>,
) -> CaptureResult<()> {
    let world = app.world_mut();
    for entity in entities.drain(..) {
        world.despawn(entity);
    }
    let animation = world.get::<Wc3Animation>(root).unwrap();
    let options = Wc3PoseOptions {
        sequence: (!animation.sequences().is_empty()).then_some(animation.sequence()),
        elapsed_ms: animation.elapsed_ms(),
        global_elapsed_ms: animation.elapsed_ms(),
        camera: Some(
            world
                .get::<GlobalTransform>(camera)
                .unwrap()
                .compute_transform(),
        ),
    };
    let bindings = world.get::<Wc3TextureBindings>(root).unwrap().clone();
    let selected = world.get::<Wc3LodState>(root).unwrap().selected_level();
    let pose = world.resource_scope(|world, mut images: Mut<Assets<Image>>| {
        baker.bake(
            prepared,
            world.resource::<Assets<Mesh>>(),
            &mut images,
            &bindings,
            options,
        )
    })?;
    for part in pose
        .parts
        .into_iter()
        .filter(|part| part.visible && part.lod.is_none_or(|lod| lod == selected))
    {
        let mesh = world.resource_mut::<Assets<Mesh>>().add(part.mesh);
        let material = world
            .resource_mut::<Assets<Wc3LayerMaterial>>()
            .add(part.material.material);
        entities.push(
            world
                .spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::default()))
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
    let model_path = options.model.canonicalize()?;
    let asset_root = options
        .asset_root
        .clone()
        .unwrap_or_else(|| model_path.parent().unwrap().to_owned())
        .canonicalize()?;
    let asset_path = model_path
        .strip_prefix(&asset_root)
        .map_err(|_| "model must be inside the asset root")?
        .to_string_lossy()
        .into_owned();
    // Report parse/conversion errors before initializing the GPU.
    let source = Wc3Model::decode(&read(&model_path)?)?;
    let sequences = source.model.sequences();
    if options.sequence >= sequences.len() && !(sequences.is_empty() && options.sequence == 0) {
        return Err(format!(
            "sequence {} is out of range ({} sequences)",
            options.sequence,
            sequences.len()
        )
        .into());
    }
    for change in &options.plays {
        if change.sequence >= sequences.len() {
            return Err(format!("scheduled sequence {} is out of range", change.sequence).into());
        }
    }
    let camera = options.camera(&source)?;
    let mut app = App::new();
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
    let handle: Handle<Wc3ModelAsset> = assets.load(asset_path);
    let mut bindings = Wc3TextureBindings::default();
    let mut overrides = Vec::new();
    for choice in &options.textures {
        let image: Handle<Image> = assets.load(choice.path.clone());
        if let Some(slot) = choice.slot {
            bindings.set_slot(slot, image.clone());
        } else {
            bindings.set_replaceable(choice.index, image.clone());
        }
        overrides.push(image);
    }
    let root = app
        .world_mut()
        .spawn((
            Wc3ModelInstance::new(handle.clone()),
            bindings,
            options.lod,
            Wc3LodOverride(options.lod_settings.clone()),
        ))
        .id();
    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::new_target_texture(
            options.size.x,
            options.size.y,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
    let camera_entity = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            options.tonemapping,
            ShadowLodOrigin,
            Msaa::Sample4,
            RenderTarget::Image(image.clone().into()),
            camera,
        ))
        .id();
    if let Some(index) = options.model_camera {
        app.world_mut()
            .entity_mut(camera_entity)
            .insert(Wc3CameraBinding {
                fov_multiplier: options.camera_fov_multiplier,
                ..Wc3CameraBinding::new(root, index)
            });
        app.world_mut()
            .entity_mut(root)
            .insert(Wc3NodeCamera(camera_entity));
    }
    if options.prepasses {
        app.world_mut().entity_mut(camera_entity).insert((
            DepthPrepass,
            NormalPrepass,
            MotionVectorPrepass,
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
            Transform::from_xyz(0.0, 2.0, 0.0),
        ));
    }
    if !options.no_default_light {
        app.world_mut().spawn((
            DirectionalLight {
                illuminance: options.illuminance,
                shadow_maps_enabled: options.prepasses,
                ..default()
            },
            Transform::from_xyz(1.0, -1.0, 2.0).looking_at(Vec3::ZERO, Vec3::Z),
        ));
    }
    let deadline = Instant::now() + TIMEOUT;
    loop {
        step(&mut app, Duration::ZERO)?;
        check_asset(&assets, &handle)?;
        for image in &overrides {
            check_asset(&assets, image)?;
        }
        if assets.is_loaded_with_dependencies(&handle)
            && overrides
                .iter()
                .all(|image| assets.is_loaded_with_dependencies(image))
            && app.world().get::<Wc3Animation>(root).is_some()
        {
            break;
        }
        check_deadline(deadline, "loading model, spawning it, and loading textures")?;
        sleep(Duration::from_millis(5));
    }
    app.world_mut()
        .get_mut::<Wc3Animation>(root)
        .unwrap()
        .play_immediately(options.sequence);
    settle(&mut app)?;
    let prepared_pose = if options.bake_pose {
        Some(
            app.world_mut()
                .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                    world.resource_scope(
                        |world, mut binds: Mut<Assets<SkinnedMeshInverseBindposes>>| {
                            let models = world.resource::<Assets<Wc3ModelAsset>>();
                            prepare_model_asset(
                                &mut meshes,
                                &mut binds,
                                models.get(&handle).unwrap(),
                            )
                        },
                    )
                })?,
        )
    } else {
        None
    };
    let mut pose_baker = Wc3PoseBaker::default();
    let mut baked_entities = Vec::new();
    create_dir_all(&options.output)?;
    let mut elapsed = 0.0;
    let mut next_play = 0;
    for (index, &time) in options.times.iter().enumerate() {
        // Simulate from zero; seeking the animation alone would omit live particles.
        loop {
            if let Some(change) = options
                .plays
                .get(next_play)
                .filter(|change| change.time <= elapsed + 1e-9 && change.time <= time + 1e-9)
            {
                let mut animation = app.world_mut().get_mut::<Wc3Animation>(root).unwrap();
                if let Some(duration) = options.blend {
                    animation.play_with_blend(change.sequence, duration);
                } else {
                    animation.play(change.sequence);
                }
                next_play += 1;
                continue;
            }
            if elapsed + 1e-9 >= time {
                break;
            }
            let target = options
                .plays
                .get(next_play)
                .map_or(time, |change| time.min(change.time));
            let dt = Duration::try_from_secs_f64((target - elapsed).min(1.0 / options.fps))?;
            step(&mut app, dt)?;
            elapsed += dt.as_secs_f64();
        }
        settle(&mut app)?;
        if let Some(index) = options.model_camera {
            let world = app.world();
            let definitions = world
                .get::<Wc3ModelCameras>(root)
                .ok_or("model cameras are missing")?;
            let animation = world
                .get::<Wc3Animation>(root)
                .ok_or("model animation is missing")?;
            let sample = definitions
                .sample(index, animation)
                .ok_or("model camera is missing")?;
            let transform = world
                .get::<GlobalTransform>(root)
                .ok_or("model transform is missing")?;
            sample
                .world_transform(transform)
                .ok_or("model camera has an invalid animated view")?;
            sample
                .perspective_projection(options.camera_fov_multiplier)
                .ok_or("model camera has an invalid lens")?;
        }
        if let Some(prepared) = &prepared_pose {
            replace_with_baked_pose(
                &mut app,
                prepared,
                &mut pose_baker,
                root,
                camera_entity,
                &mut baked_entities,
            )?;
            settle(&mut app)?;
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
        println!(
            "Selected geometry LOD: {}",
            app.world()
                .get::<Wc3LodState>(root)
                .unwrap()
                .selected_level()
        );
        println!(
            "{} (sequence {}, time {time:.6}s)",
            path.display(),
            app.world().get::<Wc3Animation>(root).unwrap().sequence()
        );
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
    loop {
        step(app, Duration::ZERO)?;
        frames += 1;
        if frames >= 8
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
        check_deadline(deadline, "preparing render pipelines")?;
        sleep(Duration::from_millis(5));
    }
}

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
        assert_eq!(options.lod, Wc3Lod::Automatic);
        assert_eq!(options.lod_settings.quality_bias, 2.0);
        assert_eq!(options.lod_settings.minimum_level, 2);
        assert_eq!(options.lod_settings.thresholds, [300.0, 150.0]);
        assert_eq!(
            parse(&["--lod", "2"]).unwrap().unwrap().lod,
            Wc3Lod::Fixed(2)
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
    fn authored_camera_selection_validates_index_and_lens_before_loading_gpu() {
        let model = Wc3Model::decode_mdl(include_str!("../tests/fixtures/cameras.mdl")).unwrap();
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
        assert!(
            parse(&["--model-camera", "0", "--camera-fov-multiplier", "10"])
                .unwrap()
                .unwrap()
                .camera(&model)
                .is_err()
        );
    }
}
