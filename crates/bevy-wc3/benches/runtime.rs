use bevy::asset::{AssetApp, AssetPlugin, LoadState};
use bevy::camera::visibility::VisibilityPlugin;
use bevy::ecs::world::CommandQueue;
use bevy::image::ImagePlugin;
use bevy::mesh::{skinning::SkinnedMeshInverseBindposes, MeshPlugin};
use bevy::prelude::*;
use bevy::shader::Shader;
use bevy::time::TimeUpdateStrategy;
use bevy_wc3::{
    prepare_model_with_resources, spawn_prepared_model, PreparedModel, Wc3Animation, Wc3BevyPlugin,
    Wc3LayerMaterial, Wc3Model, Wc3ModelAsset,
};
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use mdx::Write as _;
use std::{
    env,
    hint::black_box,
    thread,
    time::{Duration, Instant},
};
use wc3::model::mdx;

#[path = "../../../benchmarks/support/model.rs"]
mod fixtures;

struct Harness {
    app: App,
    prepared: PreparedModel,
    roots: Vec<Entity>,
}

impl Harness {
    fn new(source: &Wc3Model, count: usize, child: bool) -> Self {
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
        app.add_plugins(Wc3BevyPlugin);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        app.finish();
        app.cleanup();
        let child_handle = child.then(|| {
            app.world()
                .resource::<AssetServer>()
                .load::<Wc3ModelAsset>("attachment_capture_child.mdl")
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
        let prepared = prepare_model_with_resources(
            &mut meshes,
            &mut bindposes,
            source,
            |_| None,
            |_| child_handle.clone(),
        )
        .unwrap();
        app.insert_resource(meshes);
        app.insert_resource(Assets::<Wc3LayerMaterial>::default());
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
                .filter(|&&root| harness.app.world().get::<Wc3Animation>(root).is_some())
                .count(),
            count
        );
        harness
    }

    fn spawn(&mut self, count: usize) {
        self.app
            .world_mut()
            .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                world.resource_scope(|world, mut materials: Mut<Assets<Wc3LayerMaterial>>| {
                    let mut queue = CommandQueue::default();
                    let mut commands = Commands::new(&mut queue, world);
                    for _ in 0..count {
                        self.roots.push(spawn_prepared_model(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &self.prepared,
                        ));
                    }
                    queue.apply(world);
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
        let source =
            Wc3Model::decode_mdx(&fixtures::modern(profile).encode_mdx().unwrap()).unwrap();
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
        let source = Wc3Model::decode_mdl(source).unwrap();
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
                let mut animations = harness.app.world_mut().query::<&Wc3Animation>();
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
