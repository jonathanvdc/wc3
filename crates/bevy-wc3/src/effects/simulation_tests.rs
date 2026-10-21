use super::*;

#[test]
fn fractional_births_survive_partial_steps_and_integer_boundaries() {
    for rounding in [
        EmissionRounding::Model,
        EmissionRounding::Quad,
        EmissionRounding::Ribbon,
    ] {
        let mut phase = EmissionPhase::default();
        let first = phase.continuous(10.0, 0.06, 16, rounding);
        assert_eq!(first.count, 0.0);
        let second = phase.continuous(10.0, 0.04, 16, rounding);
        assert_eq!(second.count, 1.0);
        assert!(second.age(0).abs() < 1e-15);
        let third = phase.continuous(10.0, 0.25, 16, rounding);
        assert_eq!(third.count, 2.0);
        assert!((third.age(0) - 0.15).abs() < 1e-15);
        assert!((third.age(1) - 0.05).abs() < 1e-15);
    }
}

#[test]
fn capacity_limits_preserve_format_specific_emission_phase() {
    let mut model = EmissionPhase::default();
    let mut quad = EmissionPhase::default();
    let mut ribbon = EmissionPhase::default();
    assert_eq!(
        model
            .continuous(10.0, 1.0, 2, EmissionRounding::Model)
            .count,
        2.0
    );
    assert_eq!(
        quad.continuous(10.0, 1.0, 2, EmissionRounding::Quad).count,
        2.0
    );
    let trail = ribbon.continuous(10.0, 1.0, 2, EmissionRounding::Ribbon);
    assert_eq!(trail.count, 10.0);
    assert_eq!(trail.skip(trail.first_birth(0.0), 1.0, 0.5, 2), 8.0);
    assert_eq!(
        model
            .continuous(10.0, 0.05, 2, EmissionRounding::Model)
            .count,
        0.0
    );
    assert_eq!(
        quad.continuous(10.0, 0.05, 2, EmissionRounding::Quad).count,
        1.0
    );
}

#[test]
fn live_metadata_and_records_stay_aligned_across_wrap_and_snapshots() {
    let mut live = LiveRecords::default();
    for i in 0..4u32 {
        live.push(i, i + 10, 4);
    }
    let snapshot = live.records.clone();
    live.retire(|metadata| *metadata < 12);
    live.push(4, 14, 4);
    live.push(5, 15, 4);
    assert_eq!(
        live.metadata().iter().copied().collect::<Vec<_>>(),
        [12, 13, 14, 15]
    );
    assert_eq!(
        live.records.iter().copied().collect::<Vec<_>>(),
        [2, 3, 4, 5]
    );
    assert_eq!(snapshot.iter().copied().collect::<Vec<_>>(), [0, 1, 2, 3]);
    live.retire(|_| true);
    assert_eq!(live.len(), 0);
    assert!(live.records.is_empty());
}

#[test]
fn authored_counts_include_discarded_model_births_without_retaining_backlog() {
    let mut phase = EmissionPhase::default();
    let first = phase.continuous(10., 1.05, 2, EmissionRounding::Model);
    assert_eq!((first.count, first.authored_count), (2., 10.));
    let next = phase.continuous(10., 0.05, 2, EmissionRounding::Model);
    assert_eq!((next.count, next.authored_count), (1., 1.));
    assert_eq!(
        phase.continuous(10., 0., 2, EmissionRounding::Model).count,
        0.
    );
    let huge = phase.continuous(1e12, 1., 0, EmissionRounding::Model);
    assert_eq!((huge.count, huge.authored_count), (0., 1e12));
    let invalid = phase.continuous(f64::INFINITY, 1., 2, EmissionRounding::Model);
    assert_eq!((invalid.count, invalid.authored_count), (0., 0.));
}
#[test]
fn occurrence_zero_preserves_the_original_first_random_packet() {
    for object in [0, 1, 10, u32::MAX] {
        let mut old = EmitterRng::new(object);
        let mut packet = EmitterRng::at_occurrence(object, 0);
        assert_eq!(
            [packet.random(), packet.signed(), packet.random()],
            [old.random(), old.signed(), old.random()]
        );
    }
}
