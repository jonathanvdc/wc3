use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::{Quat, Vec3};
use std::hint::black_box;
use std::ops::RangeInclusive;
use wc3::model::common::{Borrowed, ZeroVec};
use wc3::model::PackedArray;
use wc3::model::{
    Animation, AnimationProgram, AnimationSample, Clip, Curve, DynamicTrack, ModelBuilder,
    ModelMetadata, SampleValue, Time, TimeSource, Track,
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
// Deliberately separate the two interval partitions from the public unbounded
// sampler. These controls diagnose components; their times are not additive.
#[inline(never)]
fn interval_bounds(
    times: &PackedArray<'_, Time>,
    interval: RangeInclusive<Time>,
) -> (usize, usize) {
    let partition = |boundary: Time, inclusive: bool| {
        let mut low = 0;
        let mut high = times.len();
        while low < high {
            let middle = low + (high - low) / 2;
            let key = times.get(middle).unwrap();
            if key < boundary || (inclusive && key == boundary) {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        low
    };
    (
        partition(*interval.start(), false),
        partition(*interval.end(), true),
    )
}

// Taking the sampler's address prevents private-ABI argument promotion for the
// indirect target. It is a diagnostic control, not a proposed runtime API.
type IntervalSampler<'data> = fn(&Curve<'data, Vec3>, Time, RangeInclusive<Time>) -> Option<Vec3>;

fn sampling_diagnostic(c: &mut Criterion) {
    let mut group = c.benchmark_group("sampling_diagnostic");
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
            if shape == "hermite" && count != 16 {
                continue;
            }
            let owned = borrowed.clone().into_owned();
            for (storage, curve) in [("borrowed", &borrowed), ("owned", &owned)] {
                let name = format!("{shape}_{storage}");
                assert_eq!(interval_bounds(curve.times(), interval.clone()), (0, count));
                assert_eq!(
                    curve.sample(sample_time),
                    curve.sample_in(sample_time, interval.clone())
                );
                group.bench_function(BenchmarkId::new(format!("{name}_full"), count), |b| {
                    b.iter(|| {
                        black_box(
                            black_box(curve)
                                .sample_in(black_box(sample_time), black_box(interval.clone())),
                        )
                    })
                });
                group.bench_function(
                    BenchmarkId::new(format!("{name}_sample_only"), count),
                    |b| b.iter(|| black_box(black_box(curve).sample(black_box(sample_time)))),
                );
                group.bench_function(
                    BenchmarkId::new(format!("{name}_bounds_only"), count),
                    |b| {
                        b.iter(|| {
                            black_box(interval_bounds(
                                black_box(curve.times()),
                                black_box(interval.clone()),
                            ))
                        })
                    },
                );
                let sampler: IntervalSampler<'_> = Curve::sample_in;
                let sampler = black_box(sampler);
                group.bench_function(BenchmarkId::new(format!("{name}_indirect"), count), |b| {
                    b.iter(|| {
                        black_box(sampler(
                            black_box(curve),
                            black_box(sample_time),
                            black_box(interval.clone()),
                        ))
                    })
                });
            }
        }
    }
    group.bench_function("vec3_interpolation_only", |b| {
        b.iter(|| {
            black_box(Vec3::interpolate(
                black_box(Vec3::ZERO),
                black_box(Vec3::ONE),
                black_box(0.373),
                false,
            ))
        })
    });
    group.finish();
}

fn packed_interval_program(c: &mut Criterion) {
    let mut group = c.benchmark_group("packed_interval_program");
    // Defaults are synthetic controls around the all-clip average inventory.
    // The environment can supply measured active-property profiles without
    // changing either matched binary: label,clips,vectors,scalars,rotations,keys.
    let profiles = std::env::var("SAMPLING_PROFILES")
        .unwrap_or_else(|_| "synthetic128,400,48,32,48,16;synthetic256,400,96,64,96,16;synthetic256_one_key,400,96,64,96,1".to_owned());
    for profile in profiles.split(';') {
        let parts = profile.split(',').collect::<Vec<_>>();
        assert_eq!(parts.len(), 6);
        let label = parts[0];
        let counts = parts[1..5]
            .iter()
            .map(|v| v.parse::<usize>().unwrap())
            .collect::<Vec<_>>();
        let [clip_count, vector_count, scalar_count, rotation_count] = counts[..] else {
            unreachable!()
        };
        let key_patterns = parts[5]
            .split('/')
            .map(|pattern| {
                pattern
                    .split('+')
                    .map(|v| {
                        (
                            v.trim_end_matches('S').parse::<usize>().unwrap(),
                            v.ends_with('S'),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert!(clip_count > 0 && matches!(key_patterns.len(), 1 | 3));
        assert!(key_patterns.iter().flatten().all(|&(count, _)| count > 0));
        let keys = |kind: usize, index: usize| {
            let pattern = &key_patterns[if key_patterns.len() == 1 { 0 } else { kind }];
            pattern[index % pattern.len()]
        };
        let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
        let vectors = (0..vector_count)
            .map(|_| builder.add_value(Vec3::ZERO))
            .collect::<Vec<_>>();
        let scalars = (0..scalar_count)
            .map(|_| builder.add_value(0.0f32))
            .collect::<Vec<_>>();
        let rotations = (0..rotation_count)
            .map(|_| builder.add_value(Quat::IDENTITY))
            .collect::<Vec<_>>();
        let mut clips = Vec::new();
        for clip_index in 0..clip_count {
            let mut tracks = Vec::new();
            // Separate encoded allocations model independent track buffers.
            let times = |key_count: usize| {
                (0..key_count)
                    .map(|i| time(i as f64 / (key_count - 1).max(1) as f64))
                    .collect::<Vec<_>>()
            };
            for (index, &target) in vectors.iter().enumerate() {
                let (key_count, step) = keys(0, index);
                let values = (0..key_count)
                    .map(|i| Vec3::new(i as f32, index as f32 * 0.01, clip_index as f32 * 0.001))
                    .collect::<Vec<_>>();
                tracks.push(DynamicTrack::Vector3(Track {
                    target,
                    clock: TimeSource::Animation,
                    curve: if step {
                        Curve::step(times(key_count), values)
                    } else {
                        Curve::linear(times(key_count), values)
                    }
                    .unwrap(),
                }));
            }
            for (index, &target) in scalars.iter().enumerate() {
                let (key_count, step) = keys(1, index);
                let values = (0..key_count)
                    .map(|i| (i + index + clip_index) as f32 * 0.01)
                    .collect::<Vec<_>>();
                tracks.push(DynamicTrack::Scalar(Track {
                    target,
                    clock: TimeSource::Animation,
                    curve: if step {
                        Curve::step(times(key_count), values)
                    } else {
                        Curve::linear(times(key_count), values)
                    }
                    .unwrap(),
                }));
            }
            for (index, &target) in rotations.iter().enumerate() {
                let (key_count, step) = keys(2, index);
                let values = (0..key_count)
                    .map(|i| Quat::from_rotation_y((i + index + clip_index) as f32 * 0.01))
                    .collect::<Vec<_>>();
                tracks.push(DynamicTrack::Rotation(Track {
                    target,
                    clock: TimeSource::Animation,
                    curve: if step {
                        Curve::step(times(key_count), values)
                    } else {
                        Curve::linear(times(key_count), values)
                    }
                    .unwrap(),
                }));
            }
            let animation = builder.add_animation(Animation {
                tracks,
                ..Default::default()
            });
            clips.push(
                builder
                    .add_clip(
                        Clip::named(format!("clip-{clip_index}"))
                            .with_animation(animation)
                            .with_duration(time(1.)),
                    )
                    .unwrap(),
            );
        }
        let model = builder.finish().unwrap();
        let program = AnimationProgram::compile(&model);
        drop(model);
        let property_count = vector_count + scalar_count + rotation_count;
        group.throughput(Throughput::Elements(property_count as u64));
        // Each iteration is one sampled animation frame. It includes clip lookup,
        // clock mapping, property index lookup, interval search and interpolation.
        // It does not include pose evaluation, Bevy ECS, geometry or GPU work.
        let mut frame = 0usize;
        group.bench_function(BenchmarkId::new(label, property_count), |b| {
            b.iter(|| {
                frame = (frame + 1) % (clip_count * 60);
                let selection = AnimationSample {
                    clip: Some(clips[frame / 60]),
                    clip_time: time((frame % 60) as f64 / 60. + 0.003),
                    ..Default::default()
                };
                let sampled = black_box(&program).sample(black_box(selection)).unwrap();
                let mut sum = Vec3::ZERO;
                for property in black_box(&vectors) {
                    sum += sampled.override_value(property).unwrap();
                }
                let mut scalar_sum = 0.;
                for property in black_box(&scalars) {
                    scalar_sum += sampled.override_value(property).unwrap();
                }
                let mut rotation_sum = 0.;
                for property in black_box(&rotations) {
                    rotation_sum += sampled.override_value(property).unwrap().w;
                }
                black_box((sum, scalar_sum, rotation_sum))
            });
        });
    }
    group.finish();
}
criterion_group!(
    benches,
    packed_access,
    packed_ownership,
    packed_sampling,
    sampling_diagnostic,
    packed_interval_program
);
criterion_main!(benches);
