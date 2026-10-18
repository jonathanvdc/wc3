use bevy::asset::{AssetApp, AssetPlugin, LoadState};
use bevy::camera::visibility::VisibilityPlugin;
use bevy::ecs::world::CommandQueue;
use bevy::image::{CompressedImageFormats, ImageLoader, ImagePlugin};
use bevy::mesh::{skinning::SkinnedMeshInverseBindposes, MeshPlugin};
use bevy::prelude::*;
use bevy::shader::Shader;
use bevy::time::TimeUpdateStrategy;
use bevy_wc3::{
    compile_model_with_options, spawn_model, CompileOptions, ModelAsset, ModelMaterial, Playback,
    PreparedModel, Wc3BevyPlugin,
};
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use mdx::Write as _;
use std::{
    env,
    hint::black_box,
    thread,
    time::{Duration, Instant},
};
use wc3::model::formats::mdlx::mdx;
use wc3::model::{Model, NodeFeature, ParticleEmitter};

#[path = "../examples/support/model.rs"]
mod semantic;

#[path = "../../../benchmarks/support/model.rs"]
mod fixtures;

struct Harness {
    app: App,
    prepared: PreparedModel,
    roots: Vec<Entity>,
}

impl Harness {
    fn new(source: &Model<'_>, count: usize, child: bool) -> Self {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                file_path: format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR")),
                ..default()
            },
            TransformPlugin,
            VisibilityPlugin,
            MeshPlugin,
            ImagePlugin::default(),
        ));
        app.init_asset::<Shader>();
        app.register_asset_loader(ImageLoader::new(CompressedImageFormats::NONE));
        app.add_plugins(Wc3BevyPlugin);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        app.finish();
        app.cleanup();
        let child_handle = child.then(|| {
            app.world()
                .resource::<AssetServer>()
                .load::<ModelAsset>("attachment_capture_child.mdl")
        });
        if let Some(handle) = &child_handle {
            let deadline = Instant::now() + Duration::from_secs(30);
            loop {
                app.update();
                match app
                    .world()
                    .resource::<AssetServer>()
                    .load_state(handle.id())
                {
                    LoadState::Loaded => break,
                    LoadState::Failed(error) => panic!("child fixture failed: {error}"),
                    _ => {
                        assert!(Instant::now() < deadline, "child fixture timed out");
                        thread::sleep(Duration::from_millis(1));
                    }
                }
            }
        }
        let mut meshes = Assets::<Mesh>::default();
        let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
        let mut options = CompileOptions::default();
        if let Some(handle) = child_handle {
            for (id, feature) in source.features() {
                if matches!(
                    &feature.payload,
                    NodeFeature::Particle(ParticleEmitter::MdlxEmitter1(_))
                ) {
                    options.child_models.insert(id, handle.clone());
                }
            }
        }
        let compiled = compile_model_with_options(source, options, |_| None).unwrap();
        let prepared = compiled.prepare(&mut meshes, &mut bindposes);
        app.insert_resource(meshes);
        app.insert_resource(Assets::<ModelMaterial>::default());
        app.insert_resource(bindposes);
        let mut harness = Self {
            app,
            prepared,
            roots: Vec::new(),
        };
        harness.spawn(count);
        harness.app.update();
        assert_eq!(
            harness
                .roots
                .iter()
                .filter(|&&root| harness.app.world().get::<Playback>(root).is_some())
                .count(),
            count
        );
        harness
    }

    fn spawn(&mut self, count: usize) {
        self.app
            .world_mut()
            .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                world.resource_scope(|world, mut materials: Mut<Assets<ModelMaterial>>| {
                    let mut queue = CommandQueue::default();
                    let mut commands = Commands::new(&mut queue, world);
                    for _ in 0..count {
                        self.roots.push(spawn_model(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &self.prepared,
                            Transform::IDENTITY,
                        ));
                    }
                    queue.apply(world);
                    if let Some((clip, _)) = self.prepared.clips().first() {
                        for &root in &self.roots {
                            world
                                .get_mut::<Playback>(root)
                                .unwrap()
                                .play_immediately(*clip);
                        }
                    }
                });
            });
    }

    fn step(&mut self) {
        self.app.update();
        black_box(self.app.world().entities().len());
    }
}

fn runtime(c: &mut Criterion) {
    for profile in fixtures::profiles() {
        let source = semantic::decode(&fixtures::modern(profile).encode_mdx().unwrap()).unwrap();
        let mut group = c.benchmark_group(format!("runtime/{}", profile.name));
        let counts = if env::var_os("WC3_BENCH_QUICK").is_some() {
            vec![32]
        } else {
            vec![1, 32]
        };
        for count in counts {
            group.throughput(Throughput::Elements(count as u64));
            group.bench_function(BenchmarkId::new("spawn", count), |b| {
                b.iter_batched_ref(
                    || Harness::new(&source, 0, false),
                    |h| {
                        h.spawn(count);
                        black_box(&h.roots);
                    },
                    BatchSize::PerIteration,
                )
            });
            group.bench_function(BenchmarkId::new("despawn", count), |b| {
                b.iter_batched_ref(
                    || Harness::new(&source, count, false),
                    |h| {
                        for root in h.roots.drain(..) {
                            h.app.world_mut().despawn(root);
                        }
                        black_box(h.app.world().entities().len());
                    },
                    BatchSize::PerIteration,
                )
            });
            let mut harness = Harness::new(&source, count, false);
            harness
                .app
                .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                    1.0 / 60.0,
                )));
            for _ in 0..120 {
                harness.step();
            }
            group.bench_function(BenchmarkId::new("update/frame-60hz", count), |b| {
                b.iter(|| harness.step())
            });
            for &root in &harness.roots {
                let mut playback = harness.app.world_mut().get_mut::<Playback>(root).unwrap();
                playback.seek(0.5).unwrap();
                playback.paused = true;
            }
            harness.step();
            group.bench_function(BenchmarkId::new("update/paused", count), |b| {
                b.iter(|| harness.step())
            });
            if let Some(&root) = harness.roots.first() {
                harness
                    .app
                    .world_mut()
                    .get_mut::<Playback>(root)
                    .unwrap()
                    .paused = false;
            }
            group.bench_function(BenchmarkId::new("update/one-animated", count), |b| {
                b.iter(|| harness.step())
            });
        }
        group.finish();
    }
    let mut group = c.benchmark_group("runtime/effects");
    for (name, source, child) in [
        (
            "prem",
            include_str!("../tests/fixtures/prem_capture.mdl"),
            true,
        ),
        (
            "pre2",
            include_str!("../tests/fixtures/particle_capture.mdl"),
            false,
        ),
        (
            "ribbon",
            include_str!("../tests/fixtures/ribbon_capture.mdl"),
            false,
        ),
    ] {
        let source = semantic::decode(source.as_bytes()).unwrap();
        let counts = if env::var_os("WC3_BENCH_QUICK").is_some() {
            vec![32]
        } else if env::var_os("WC3_BENCH_LARGE").is_some() {
            vec![1, 32, 128]
        } else {
            vec![1, 32]
        };
        for count in counts {
            let mut harness = Harness::new(&source, count, child);
            harness
                .app
                .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                    1.0 / 60.0,
                )));
            for _ in 0..180 {
                harness.step();
            }
            let baseline = harness.app.world().entities().len();
            if child {
                let mut animations = harness.app.world_mut().query::<&Playback>();
                assert!(
                    animations.iter(harness.app.world()).count() > count,
                    "PREM child models must spawn"
                );
            }
            black_box(baseline);
            group.throughput(Throughput::Elements(count as u64));
            group.bench_function(BenchmarkId::new(format!("{name}/frame-60hz"), count), |b| {
                b.iter(|| harness.step())
            });
        }
    }
    group.finish();
}

criterion_group!(benches, runtime);
criterion_main!(benches);
