use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::scene::popcorn::{EffectInstance, EffectProgram, SceneInputs};

#[path = "support/particle_runtime.rs"]
mod support;
use support::{assert_stable, fixture, limits, verify_trajectory, warm, DT, STEPS, WARMUP};

fn advance_block(program: &EffectProgram, instances: &mut [EffectInstance], children: usize) {
    for _ in 0..STEPS {
        for instance in &mut *instances {
            instance
                .advance(program, DT, SceneInputs::default())
                .unwrap();
        }
    }
    // These O(1) assertions ARE inside the measured block. Per-layer population
    // scans and changing-output validation occur only during untimed preparation.
    for instance in instances {
        assert_stable(instance, children);
        assert_eq!(instance.time(), (WARMUP + STEPS) as f64 * DT);
        black_box(&*instance);
    }
}

fn preparation(c: &mut Criterion) {
    let document = Document::read(fixture(1024)).unwrap();
    let (root, _) = document.effect_roots().next().unwrap().unwrap();
    let options = EffectCompileOptions::default();
    c.bench_function("particle_runtime/compile_native_1024_with_drop", |b| {
        b.iter(|| black_box(document.compile_effect(black_box(root), &options).unwrap()));
    });
    let program = document.compile_effect(root, &options).unwrap();
    let mut group = c.benchmark_group("particle_runtime/instantiate_with_drop");
    for capacity in [128, 1024] {
        group.bench_with_input(
            BenchmarkId::from_parameter(capacity),
            &capacity,
            |b, &capacity| {
                b.iter(|| {
                    black_box(
                        program
                            .instantiate(black_box(limits(capacity)), 42)
                            .unwrap(),
                    )
                });
            },
        );
    }
    group.finish();
}

fn steady(c: &mut Criterion) {
    let mut group = c.benchmark_group("particle_runtime/warmed_64_ticks_with_validation");
    for (children, capacity) in [
        (1, 128),
        (8, 128),
        (1, 1024),
        (8, 1024),
        (128, 128),
        (1024, 1024),
    ] {
        let document = Document::read(fixture(children)).unwrap();
        let (root, _) = document.effect_roots().next().unwrap().unwrap();
        let program = document
            .compile_effect(root, &EffectCompileOptions::default())
            .unwrap();
        // Full public canonical output and all64 interpolating frames are
        // validated independently BEFORE Criterion calibration/warmup/timing.
        verify_trajectory(&program, children, capacity);
        for count in [1, 8] {
            // Elements are active-particle ticks, including the controller.
            group.throughput(Throughput::Elements(
                ((children + 1) * count * STEPS) as u64,
            ));
            group.bench_with_input(
                BenchmarkId::new(format!("children_{children}_capacity_{capacity}"), count),
                &count,
                |b, &count| {
                    b.iter_batched_ref(
                        || {
                            (0..count)
                                .map(|_| warm(&program, children, capacity))
                                .collect::<Vec<_>>()
                        },
                        |instances| advance_block(black_box(&program), instances, children),
                        BatchSize::PerIteration,
                    );
                },
            );
        }
    }
    group.finish();
}

criterion_group!(benches, preparation, steady);
criterion_main!(benches);
