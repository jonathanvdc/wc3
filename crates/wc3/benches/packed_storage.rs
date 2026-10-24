use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::{Quat, Vec3};
use std::hint::black_box;
use wc3::model::common::{Borrowed, ZeroVec};
use wc3::model::PackedArray;
use wc3::model::{
    AnimationProgram, AnimationSample, Curve, DynamicTrack, ModelBuilder, ModelMetadata, Time,
    TimeSource, Track,
};

fn packed_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("packed_storage");
    for count in [256, 16_384] {
        let components: ZeroVec<[f32; 3]> =
            (0..count).map(|i| [i as f32 * 0.01, 2., -3.]).collect();
        let values = PackedArray::<Vec3>::from_storage(components.as_slice().as_zerovec());
        let owned = values.clone().into_owned();
        let rotation = Quat::from_rotation_y(0.7);
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("zerovec_transform", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let rotation = black_box(rotation);
                    let sum = black_box(&components)
                        .iter()
                        .map(|value| rotation * Vec3::from_array(value))
                        .sum::<Vec3>();
                    black_box(sum)
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("packed_transform", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let rotation = black_box(rotation);
                    let sum = black_box(&values)
                        .iter()
                        .map(|value| rotation * value)
                        .sum::<Vec3>();
                    black_box(sum)
                });
            },
        );
        group.bench_with_input(BenchmarkId::new("zerovec_index", count), &count, |b, _| {
            b.iter(|| {
                let values = black_box(&components);
                let mut sum = Vec3::ZERO;
                for index in 0..values.len() {
                    sum += Vec3::from_array(values.get(index).unwrap());
                }
                black_box(sum)
            });
        });
        group.bench_with_input(BenchmarkId::new("packed_index", count), &count, |b, _| {
            b.iter(|| {
                let values = black_box(&values);
                let mut sum = Vec3::ZERO;
                for index in 0..values.len() {
                    sum += values.get(index).unwrap();
                }
                black_box(sum)
            });
        });
        group.bench_with_input(BenchmarkId::new("owned_index", count), &count, |b, _| {
            b.iter(|| {
                let values = black_box(&owned);
                let mut sum = Vec3::ZERO;
                for index in 0..values.len() {
                    sum += values.get(index).unwrap();
                }
                black_box(sum)
            });
        });
        group.bench_with_input(
            BenchmarkId::new("owned_transform", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let rotation = black_box(rotation);
                    let sum = black_box(&owned)
                        .iter()
                        .map(|value| rotation * value)
                        .sum::<Vec3>();
                    black_box(sum)
                });
            },
        );
    }
    group.finish();
}

fn packed_ownership(c: &mut Criterion) {
    let mut group = c.benchmark_group("packed_ownership");
    for count in [0, 1, 16, 256, 16_384] {
        let values: PackedArray<Vec3> = (0..count).map(|i| Vec3::splat(i as f32)).collect();
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::new("clone", count), &count, |b, _| {
            b.iter(|| black_box(black_box(&values).clone()));
        });
        group.bench_with_input(BenchmarkId::new("clone_and_edit", count), &count, |b, _| {
            b.iter(|| {
                let mut edited = black_box(&values).clone();
                black_box(edited.set(0, black_box(Vec3::Y)));
                black_box(edited)
            });
        });
    }
    group.finish();
}
fn time(seconds: f64) -> Time {
    Time::seconds(seconds).unwrap()
}

fn packed_sampling(c: &mut Criterion) {
    let mut group = c.benchmark_group("packed_sampling");
    for count in [2, 16, 256] {
        let times: PackedArray<Time> = (0..count).map(|i| time(i as f64)).collect();
        let values: PackedArray<Vec3> = (0..count).map(|i| Vec3::splat(i as f32)).collect();
        let controls: PackedArray<Vec3> = (0..count).map(|_| Vec3::ONE).collect();
        let linear = Curve::linear(times.as_borrowed(), values.as_borrowed()).unwrap();
        let hermite = Curve::hermite(
            times.as_borrowed(),
            values.as_borrowed(),
            controls.as_borrowed(),
            controls.as_borrowed(),
        )
        .unwrap();
        let sample_time = time((count - 1) as f64 * 0.373);
        let interval = time(0.)..=time((count - 1) as f64);
        for (shape, borrowed) in [("linear", linear), ("hermite", hermite)] {
            let owned = borrowed.clone().into_owned();
            for (storage, curve) in [("borrowed", &borrowed), ("owned", &owned)] {
                group.bench_with_input(
                    BenchmarkId::new(format!("{shape}_{storage}"), count),
                    &count,
                    |b, _| b.iter(|| black_box(black_box(curve).sample(black_box(sample_time)))),
                );
                group.bench_with_input(
                    BenchmarkId::new(format!("{shape}_interval_{storage}"), count),
                    &count,
                    |b, _| {
                        b.iter(|| {
                            black_box(
                                black_box(curve)
                                    .sample_in(black_box(sample_time), black_box(interval.clone())),
                            )
                        })
                    },
                );
            }
        }
    }
    group.finish();
    let mut group = c.benchmark_group("packed_program");
    for key_count in [2, 16] {
        let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
        let mut properties = Vec::new();
        let curve = Curve::linear(
            (0..key_count).map(|i| time(i as f64)).collect::<Vec<_>>(),
            (0..key_count)
                .map(|i| Vec3::splat(i as f32))
                .collect::<Vec<_>>(),
        )
        .unwrap();
        for _ in 0..128 {
            let node = builder.add_named_node("bone").unwrap();
            let property = builder.model().node(node).unwrap().transform.translation;
            builder
                .add_global_track(DynamicTrack::Vector3(Track {
                    target: property,
                    clock: TimeSource::Animation,
                    curve: curve.clone(),
                }))
                .unwrap();
            properties.push(property);
        }
        let model = builder.finish().unwrap();
        let program = AnimationProgram::compile(&model);
        let selection = AnimationSample {
            clip_time: time((key_count - 1) as f64 * 0.373),
            ..Default::default()
        };
        group.throughput(Throughput::Elements(128));
        group.bench_with_input(
            BenchmarkId::new("128_translations", key_count),
            &key_count,
            |b, _| {
                b.iter(|| {
                    let sampled = black_box(&program).sample(black_box(selection)).unwrap();
                    let mut sum = Vec3::ZERO;
                    for property in black_box(&properties) {
                        sum += sampled.override_value(black_box(property)).unwrap();
                    }
                    black_box(sum)
                });
            },
        );
    }
    group.finish();
}
criterion_group!(benches, packed_access, packed_ownership, packed_sampling);
criterion_main!(benches);
