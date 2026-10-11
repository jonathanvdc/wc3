use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{env, hint::black_box};
use wc3::model::animation::{
    AnimationTime, Interpolate, Sequence, TangentKeyframe, Track, ValueKeyframe,
};
use wc3::model::scene::{EventObject, Node};
use wc3::model::ZeroVec;

fn tracks<T: Interpolate>(c: &mut Criterion, name: &str, value: impl Fn(usize) -> T) {
    let mut group = c.benchmark_group(format!("animation/{name}"));
    let sizes = if env::var_os("WC3_BENCH_QUICK").is_some() {
        vec![64]
    } else if env::var_os("WC3_BENCH_LARGE").is_some() {
        vec![8, 64, 256, 4096]
    } else {
        vec![8, 64, 256]
    };
    for count in sizes {
        let values: ZeroVec<_> = (0..count)
            .map(|i| ValueKeyframe {
                frame: (i * 33 + i % 7) as i32,
                value: value(i),
            })
            .collect();
        let tangents: ZeroVec<_> = values
            .iter()
            .map(|k| TangentKeyframe {
                frame: k.frame,
                value: k.value,
                in_tangent: k.value,
                out_tangent: k.value,
            })
            .collect();
        for (mode, track) in [
            ("step", Track::step(values.clone(), None).unwrap()),
            ("linear", Track::linear(values.clone(), None).unwrap()),
            ("hermite", Track::hermite(tangents.clone(), None).unwrap()),
            ("bezier", Track::bezier(tangents, None).unwrap()),
        ] {
            for scattered in [false, true] {
                let times: Vec<_> = (0..128)
                    .map(|i| {
                        ((if scattered { i * 73 % 128 } else { i }) as f64 / 128.0)
                            * ((count - 1) * 33) as f64
                            + 0.5
                    })
                    .collect();
                assert!(times.iter().all(|&time| track.evaluate(time).is_some()));
                group.throughput(Throughput::Elements(times.len() as u64));
                group.bench_with_input(
                    BenchmarkId::new(
                        format!(
                            "{mode}/{}",
                            if scattered { "scattered" } else { "sequential" }
                        ),
                        count,
                    ),
                    &track,
                    |b, track| {
                        b.iter(|| {
                            for &time in &times {
                                black_box(track.evaluate(black_box(time)));
                            }
                        })
                    },
                );
            }
        }
        let track = Track::linear(values, Some(0)).unwrap();
        let sequence = Sequence {
            interval: [0, (count * 33) as u32],
            ..Default::default()
        };
        for global in [false, true] {
            let sampled = if global {
                track.clone()
            } else {
                Track::linear(
                    (track.linear_keys().unwrap().to_vec())
                        .into_iter()
                        .collect(),
                    None,
                )
                .unwrap()
            };
            group.throughput(Throughput::Elements(6));
            group.bench_function(
                BenchmarkId::new(
                    if global {
                        "global-boundaries"
                    } else {
                        "sequence-boundaries"
                    },
                    count,
                ),
                |b| {
                    b.iter(|| {
                        for elapsed in [
                            -1.0,
                            0.0,
                            1.0,
                            (count * 33) as f64 - 0.5,
                            (count * 33) as f64,
                            (count * 66) as f64,
                        ] {
                            black_box(sampled.sample(&AnimationTime {
                                sequence: Some(&sequence),
                                elapsed_ms: black_box(elapsed),
                                global_sequences: &[(count * 33) as u32],
                            }));
                        }
                    })
                },
            );
        }
    }
    group.finish();
}

fn animation(c: &mut Criterion) {
    tracks(c, "scalar", |i| (i % 17) as f32 / 17.0);
    tracks(c, "vector", |i| [i as f32 * 0.01, (i % 7) as f32, 1.0]);
    tracks(c, "quaternion", |i| {
        let a = i as f32 * 0.03;
        [0.0, 0.0, a.sin(), a.cos()]
    });
    let sequence = Sequence {
        interval: [0, 2000],
        ..Default::default()
    };
    let mut group = c.benchmark_group("animation/events");
    let counts = if env::var_os("WC3_BENCH_QUICK").is_some() {
        vec![64]
    } else {
        vec![8, 64, 256]
    };
    for count in counts {
        let frames: Vec<_> = (0..count).map(|i| i * 2000 / count).collect();
        for global in [false, true] {
            let event = EventObject::new(
                Node::new("Event", 0).unwrap(),
                if global { 0 } else { u32::MAX },
                &frames,
            );
            for (name, from, to) in [
                ("frame", 900.0, 916.667),
                ("boundary", 1990.0, 2010.0),
                ("loops", 0.0, 8000.0),
            ] {
                let time = AnimationTime {
                    sequence: Some(&sequence),
                    elapsed_ms: to,
                    global_sequences: &[2000],
                };
                group.throughput(Throughput::Elements(count as u64));
                group.bench_function(
                    BenchmarkId::new(
                        format!("{name}/{}", if global { "global" } else { "sequence" }),
                        count,
                    ),
                    |b| {
                        b.iter(|| black_box(time.event_occurrences(black_box(&event), from, false)))
                    },
                );
            }
        }
    }
    group.finish();
}

criterion_group!(benches, animation);
criterion_main!(benches);
