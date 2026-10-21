use super::*;
fn sample(resource: &Turbulence, point: Vec3, time: f64) -> Vec3 {
    let mut cache = TemporalCache::default();
    resource
        .sample(point, cache.phase(0, resource, time).unwrap())
        .unwrap()
}
fn resource(parameters: Parameters) -> Turbulence {
    Turbulence::new(parameters).unwrap()
}
fn near(a: Vec3, b: Vec3, tolerance: f32) {
    assert!((a - b).length() <= tolerance, "{a:?} != {b:?}");
}
#[test]
fn deterministic_field_does_not_depend_on_query_or_instance_order() {
    let p = Parameters {
        time_scale: 0.2,
        ..Default::default()
    };
    let a = resource(p);
    let b = resource(p);
    let point = Vec3::new(0.23, -0.71, 0.32);
    let expected = sample(&a, point, 1.25);
    let mut first = TemporalCache::default();
    let mut second = TemporalCache::default();
    for time in [3., 0., 1.25, 9., 1.25] {
        let phase = first.phase(0, &a, time).unwrap();
        let _ = a.sample(Vec3::splat(0.4), phase).unwrap();
    }
    assert_eq!(
        a.sample(point, first.phase(0, &a, 1.25).unwrap()).unwrap(),
        expected
    );
    assert_eq!(
        b.sample(point, second.phase(0, &b, 1.25).unwrap()).unwrap(),
        expected
    );
    let changed = resource(Parameters {
        seed: p.seed + 1,
        ..p
    });
    assert_ne!(sample(&changed, point, 1.25), expected);
}
#[test]
fn analytic_gradient_matches_own_scalar_potential_and_curl_has_negligible_divergence() {
    for interpolator in [0, 1, 2] {
        let field = resource(Parameters {
            interpolator,
            octaves: 1,
            ..Default::default()
        });
        let phase = field.phase(0.).unwrap();
        for point in [DVec3::new(0.31, 0.42, -0.23), DVec3::new(1., 0.42, -0.23)] {
            if interpolator == 0 && point.x == 1. {
                continue;
            }
            let (_, gradient) = field.potential(point, &phase, 1);
            for axis in [DVec3::X, DVec3::Y, DVec3::Z] {
                let h = 1e-6;
                let derivative = (field.potential(point + axis * h, &phase, 1).0
                    - field.potential(point - axis * h, &phase, 1).0)
                    / (2. * h);
                assert!((derivative - gradient.dot(axis)).abs() < 2e-5);
            }
        }
        let point = Vec3::new(0.135, 0.215, -0.115);
        let h = 1e-4;
        let divergence = [Vec3::X, Vec3::Y, Vec3::Z]
            .into_iter()
            .map(|axis| {
                (sample(&field, point + axis * h, 0.) - sample(&field, point - axis * h, 0.))
                    .dot(axis)
                    / (2. * h)
            })
            .sum::<f32>();
        assert!(divergence.abs() < 0.002, "{divergence}");
    }
}
#[test]
fn strength_scale_wavelength_and_octaves_preserve_dimensional_conventions() {
    let p = Parameters::default();
    let x = Vec3::new(0.123, -0.237, 0.379);
    let base = sample(&resource(p), x, 0.);
    near(
        sample(
            &resource(Parameters {
                strength: p.strength * 2.,
                ..p
            }),
            x,
            0.,
        ),
        base * 2.,
        1e-6,
    );
    assert_eq!(
        sample(&resource(Parameters { strength: 0., ..p }), x, 0.),
        Vec3::ZERO
    );
    near(
        sample(&resource(Parameters { scale: 2., ..p }), x * 2., 0.),
        base * 2.,
        1e-6,
    );
    near(
        sample(
            &resource(Parameters {
                wavelength: p.wavelength * 2.,
                ..p
            }),
            x * 2.,
            0.,
        ),
        base,
        1e-6,
    );
    near(
        sample(
            &resource(Parameters {
                strength: -p.strength,
                ..p
            }),
            x,
            0.,
        ),
        -base,
        1e-6,
    );
    let one = Parameters { octaves: 1, ..p };
    near(
        sample(
            &resource(Parameters {
                lacunarity: 1.,
                gain: 1.,
                ..p
            }),
            x,
            0.,
        ),
        sample(&resource(one), x, 0.) * 2.,
        1e-6,
    );
    let second = Parameters {
        octaves: 1,
        wavelength: p.wavelength * p.lacunarity,
        strength: p.strength * p.gain,
        ..p
    };
    near(
        base,
        sample(&resource(one), x, 0.) + sample(&resource(second), x, 0.),
        1e-6,
    );
    near(
        sample(&resource(Parameters { gain: 0., ..p }), x, 0.),
        sample(&resource(one), x, 0.),
        1e-6,
    );
    near(
        sample(&resource(Parameters { octaves: 0, ..p }), x, 0.),
        sample(&resource(one), x, 0.),
        1e-6,
    );
}
#[test]
fn time_is_effect_global_frozen_reversible_and_phase_offset_is_temporal() {
    let point = Vec3::new(0.23, 0.31, -0.47);
    let p = Parameters::default();
    let frozen = resource(p);
    assert_eq!(sample(&frozen, point, 0.), sample(&frozen, point, 1e15));
    let moving = resource(Parameters {
        time_scale: 0.2,
        ..p
    });
    assert_ne!(sample(&moving, point, 0.), sample(&moving, point, 1.));
    let reversed = resource(Parameters {
        time_scale: -0.2,
        ..p
    });
    assert_eq!(sample(&moving, point, -1.), sample(&reversed, point, 1.));
    let shifted = resource(Parameters {
        time_base: 0.2,
        ..p
    });
    assert_eq!(sample(&moving, point, 1.), sample(&shifted, point, 0.));
    let periodic = resource(Parameters {
        time_scale: 1.,
        time_variation: 0.,
        ..p
    });
    near(
        sample(&periodic, point, 0.25),
        sample(&periodic, point, 1.25),
        1e-6,
    );
}
#[test]
fn real_tiny_scale_controls_and_checked_failures_are_explicit() {
    let p = Parameters {
        scale: 0.01,
        strength: 0.01,
        wavelength: 0.1,
        lacunarity: 0.018,
        gain: 0.025,
        ..Default::default()
    };
    assert!(sample(&resource(p), Vec3::new(0.2, -0.3, 0.4), 0.).is_finite());
    for p in [
        Parameters { scale: 0., ..p },
        Parameters {
            wavelength: -1.,
            ..p
        },
        Parameters { octaves: 25, ..p },
        Parameters {
            interpolator: 3,
            ..p
        },
        Parameters {
            time_variation: -0.1,
            ..p
        },
        Parameters {
            strength: f32::NAN,
            ..p
        },
    ] {
        assert!(Turbulence::new(p).is_err());
    }
    let field = resource(Parameters {
        time_scale: 1.,
        ..Default::default()
    });
    let mut cache = TemporalCache::default();
    assert!(cache.phase(0, &field, f64::NAN).is_err());
    assert!(cache.phase(0, &field, 1e20).is_err());
    let phase = cache.phase(0, &field, 0.).unwrap();
    assert_eq!(
        field.sample(Vec3::splat(f32::INFINITY), phase).unwrap(),
        Vec3::splat(f32::INFINITY)
    );
    for point in [
        Vec3::new(f32::INFINITY, 0., 0.),
        Vec3::splat(f32::NAN),
        Vec3::splat(f32::MAX),
    ] {
        assert!(field.sample(point, phase).is_err());
    }
}

#[test]
fn hot_queries_have_bounded_corners_and_zero_trigonometry() {
    let point = Vec3::new(0.1, -0.2, 0.3);
    let moving = resource(Parameters {
        time_scale: 0.2,
        ..Default::default()
    });
    let mut cache = TemporalCache::default();
    CORNER_VISITS.with(|count| count.set(0));
    OSCILLATOR_PAIRS.with(|count| count.set(0));
    for _ in 0..100 {
        moving
            .sample(point, cache.phase(0, &moving, 1.).unwrap())
            .unwrap();
    }
    assert_eq!(CORNER_VISITS.with(Cell::get), 100 * 2 * 3 * 8);
    assert_eq!(OSCILLATOR_PAIRS.with(Cell::get), FREQUENCY_BANDS);
    moving
        .sample(point, cache.phase(0, &moving, 2.).unwrap())
        .unwrap();
    assert_eq!(OSCILLATOR_PAIRS.with(Cell::get), 2 * FREQUENCY_BANDS);
    let frozen = resource(Parameters::default());
    CORNER_VISITS.with(|count| count.set(0));
    OSCILLATOR_PAIRS.with(|count| count.set(0));
    for time in 0..100 {
        frozen
            .sample(point, cache.phase(1, &frozen, time as f64).unwrap())
            .unwrap();
    }
    assert_eq!(CORNER_VISITS.with(Cell::get), 100 * 2 * 3 * 8);
    assert_eq!(OSCILLATOR_PAIRS.with(Cell::get), 0);
    let maximum = resource(Parameters {
        octaves: MAX_OCTAVES,
        ..Default::default()
    });
    CORNER_VISITS.with(|count| count.set(0));
    maximum
        .sample(point, cache.phase(2, &maximum, 0.).unwrap())
        .unwrap();
    assert_eq!(CORNER_VISITS.with(Cell::get), MAX_OCTAVES as usize * 3 * 8);
}
