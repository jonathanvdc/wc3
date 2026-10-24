use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::{Quat, Vec3};
use std::hint::black_box;
use wc3::model::{
    Animation, AnimationProgram, AnimationSample, Clip, Curve, DynamicTrack, ModelBuilder,
    ModelMetadata, Time, TimeSource, Track,
};

fn time(seconds: f64) -> Time {
    Time::seconds(seconds).unwrap()
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
criterion_group!(benches, packed_interval_program);
criterion_main!(benches);
