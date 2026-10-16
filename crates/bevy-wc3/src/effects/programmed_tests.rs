use super::*;
use wc3::model::{AnimationProgram, AnimationSample, Model};
#[test]
fn collapsed_age_stages_preserve_explicit_endpoints() {
    for middle in [0., 1.] {
        for smoothing in [Smoothing::Linear, Smoothing::LinearSmooth] {
            assert_eq!(stage([2., 4., 8.], middle, 0., smoothing), 2.);
            assert_eq!(stage([2., 4., 8.], middle, 1., smoothing), 8.);
        }
        let mut appearance = ProgrammedAppearance::constant(1.);
        appearance.cells = [2, 3, 5, 7];
        appearance.cell_middle = middle;
        assert_eq!(appearance.sample(0.).cell, 2);
        assert_eq!(appearance.sample(1.).cell, 7);
    }
}

#[test]
fn birth_freezes_motion_and_style_in_the_selected_space() {
    let model = Model::empty("programmed sprite controls").unwrap();
    let animation = AnimationProgram::compile(&model);
    let sample = animation.sample(AnimationSample::default()).unwrap();
    let sample = sample.at_times(Default::default(), Default::default(), Default::default());
    let mut programme = ProgrammedSprite::fixture();
    let node = Mat4::from_translation(Vec3::new(3., 4., 5.));
    let birth = programme
        .birth(&sample, node, Mat4::IDENTITY, [0.5; 12])
        .unwrap();
    assert_eq!(birth.origin, Vec3::new(3., 4., 5.));
    assert_eq!(birth.velocity, Vec3::Z * 2.);
    assert_eq!(birth.acceleration, Vec3::NEG_Z * 4.);
    assert_eq!(birth.appearance.sample(0.5).color, [0., 1., 0., 1.]);
    programme.model_space = true;
    programme.random_lifetime = true;
    programme.random_speed = true;
    let birth = programme
        .birth(&sample, node, Mat4::IDENTITY, [0.5; 12])
        .unwrap();
    assert_eq!(birth.origin, Vec3::ZERO);
    assert_eq!(birth.lifetime, 3.);
    assert_eq!(birth.velocity, Vec3::Z * 4.);
}

#[test]
fn channels_use_independent_normalized_midpoints() {
    let appearance = ProgrammedAppearance {
        size: [0., 2., 0.],
        rotation: [0., 1., 2.],
        color: [[1., 0., 0., 0.], [0., 1., 0., 1.], [0., 0., 1., 0.]],
        middle: [0.25, 0.5, 0.5, 0.75],
        smoothing: [Smoothing::Linear; 3],
        cells: [0, 3, 4, 7],
        cell_middle: 0.5,
    };
    let value = appearance.sample(0.25);
    assert_eq!(value.size, [2.; 2]);
    assert_eq!(value.rotation, 0.5);
    assert_eq!(value.color[0], 0.5);
    assert!((value.color[3] - 1. / 3.).abs() < 1e-6);
    assert_eq!(appearance.sample(1.).cell, 7);
}
#[test]
fn squirt_crossings_include_start_once_and_repeat_without_boundary_duplicates() {
    let mut occurrences = Vec::new();
    let interval = SquirtInterval {
        time: 1.,
        raw: 1.,
        previous: 0.,
        period: Some(1.),
        include_origin: true,
    };
    interval.add(&mut occurrences, 0., 3);
    interval.add(&mut occurrences, 0.5, 2);
    occurrences.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert_eq!(occurrences, [(0., 3), (0.5, 2), (1., 3)]);
    occurrences.clear();
    SquirtInterval {
        time: 2.,
        raw: 2.,
        previous: 1.,
        period: Some(1.),
        include_origin: false,
    }
    .add(&mut occurrences, 0., 3);
    assert_eq!(occurrences, [(2., 3)]);
    occurrences.clear();
    SquirtInterval {
        time: 2.,
        raw: 2.,
        previous: 2.,
        period: Some(1.),
        include_origin: false,
    }
    .add(&mut occurrences, 0., 3);
    assert!(occurrences.is_empty());
}
#[test]
fn squirt_occurrence_budget_is_shared_across_keys_and_ignores_zero_counts() {
    let interval = SquirtInterval {
        time: 10000.,
        raw: 10000.,
        previous: 0.,
        period: Some(1.),
        include_origin: true,
    };
    let mut occurrences = Vec::new();
    interval.add(&mut occurrences, 0., 0);
    assert!(occurrences.is_empty());
    interval.add(&mut occurrences, 0., 1);
    interval.add(&mut occurrences, 0.5, 7);
    assert_eq!(occurrences.len(), 4096);
    assert!(occurrences.iter().all(|entry| entry.1 == 1));
}
