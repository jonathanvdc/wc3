use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{env, hint::black_box};
use wc3::model::{Curve, Quat, SampleValue, SplineValue, Time, Vec3};

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

criterion_group!(benches, animation);
criterion_main!(benches);
