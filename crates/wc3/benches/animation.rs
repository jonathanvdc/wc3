use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{env, hint::black_box};
use wc3::model::profiles::Universal;
use wc3::model::{
    Animation, AnimationProgram, AnimationSample, Clip, Curve, DynamicTrack, Model, ModelMetadata,
    Quat, SampleValue, SplineValue, Time, TimeSource, Track, Vec3,
};

fn tracks<T: SplineValue + SampleValue>(c: &mut Criterion, name: &str, value: impl Fn(usize) -> T) {
    let mut group = c.benchmark_group(format!("animation/{name}"));
    let sizes = if env::var_os("WC3_BENCH_QUICK").is_some() {
        vec![64]
    } else if env::var_os("WC3_BENCH_LARGE").is_some() {
        vec![8, 64, 256, 4096]
    } else {
        vec![8, 64, 256]
    };
    for count in sizes {
        let times: Vec<_> = (0..count)
            .map(|i| Time::seconds((i * 33 + i % 7) as f64 / 1000.).unwrap())
            .collect();
        let values: Vec<_> = (0..count).map(&value).collect();
        for (mode, curve) in [
            ("step", Curve::step(times.clone(), values.clone()).unwrap()),
            (
                "linear",
                Curve::linear(times.clone(), values.clone()).unwrap(),
            ),
            (
                "hermite",
                Curve::hermite(
                    times.clone(),
                    values.clone(),
                    values
                        .iter()
                        .map(|v| (*v).into_derivative())
                        .collect::<Vec<_>>(),
                    values
                        .iter()
                        .map(|v| (*v).into_derivative())
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            ),
            (
                "bezier",
                Curve::bezier(
                    times.clone(),
                    values.clone(),
                    values
                        .iter()
                        .map(|v| (*v).into_control_point())
                        .collect::<Vec<_>>(),
                    values
                        .iter()
                        .map(|v| (*v).into_control_point())
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            ),
        ] {
            for scattered in [false, true] {
                let times: Vec<_> = (0..128)
                    .map(|i| {
                        Time::seconds(
                            ((if scattered { i * 73 % 128 } else { i }) as f64 / 128.)
                                * ((count - 1) * 33) as f64
                                / 1000.
                                + 0.0005,
                        )
                        .unwrap()
                    })
                    .collect();
                assert!(times.iter().all(|&time| curve.sample(time).is_some()));
                group.throughput(Throughput::Elements(times.len() as u64));
                group.bench_with_input(
                    BenchmarkId::new(
                        format!(
                            "{mode}/{}",
                            if scattered { "scattered" } else { "sequential" }
                        ),
                        count,
                    ),
                    &curve,
                    |b, curve| {
                        b.iter(|| {
                            for &time in &times {
                                black_box(curve.sample(black_box(time)));
                            }
                        })
                    },
                );
            }
        }
    }
    group.finish();
}

fn animation(c: &mut Criterion) {
    tracks(c, "scalar", |i| (i % 17) as f32 / 17.);
    tracks(c, "vector", |i| {
        Vec3::new(i as f32 * 0.01, (i % 7) as f32, 1.)
    });
    tracks(c, "quaternion", |i| Quat::from_rotation_z(i as f32 * 0.06));
}

fn selected_tracks(c: &mut Criterion) {
    let mut group = c.benchmark_group("animation/selected_tracks");
    // Keep the property workload fixed while varying the number of reusable
    // animations traversed to resolve it. Some slots have no writer; others
    // have several writers, including an empty highest-priority curve.
    for count in [1, 16, 64] {
        let mut builder = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
        let properties: Vec<_> = (0..64)
            .map(|index| builder.values_mut().add(index as f32))
            .collect();
        let mut clip = Clip::named("selection").with_duration(Time::seconds(1.).unwrap());
        for index in 0..count {
            let tracks = properties[..48]
                .iter()
                .enumerate()
                .filter(|(ordinal, _)| ordinal % count == index || ordinal % 8 == 0)
                .map(|(ordinal, &target)| {
                    let empty = index + 1 == count && ordinal % 8 == 0;
                    DynamicTrack::Scalar(Track {
                        target,
                        clock: TimeSource::Animation,
                        curve: if empty {
                            Curve::step(Vec::<Time>::new(), Vec::<f32>::new()).unwrap()
                        } else {
                            Curve::linear(
                                [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
                                [ordinal as f32, index as f32],
                            )
                            .unwrap()
                        },
                    })
                })
                .collect();
            let animation = builder.add_animation(Animation {
                tracks,
                ..Default::default()
            });
            clip = clip.with_animation(animation);
        }
        let clip = builder.add_clip(clip).unwrap();
        let model = builder.finish().unwrap();
        let program = AnimationProgram::compile(&model);
        let selection = AnimationSample {
            clip: Some(clip),
            clip_time: Time::seconds(0.375).unwrap(),
            ..Default::default()
        };
        let compiled = program.sample(selection).unwrap();
        let direct = model.animation_sample(selection).unwrap();
        for property in &properties {
            assert_eq!(compiled.animated(property), direct.animated(property));
        }
        group.throughput(Throughput::Elements(properties.len() as u64));
        group.bench_function(BenchmarkId::new("compiled", count), |b| {
            b.iter(|| {
                let sample = program.sample(black_box(selection)).unwrap();
                for property in &properties {
                    black_box(sample.animated(black_box(property)));
                }
            });
        });
    }
    group.finish();
}

criterion_group!(benches, animation, selected_tracks);
criterion_main!(benches);
