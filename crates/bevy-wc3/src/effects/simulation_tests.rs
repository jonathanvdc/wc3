use super::*;

#[test]
fn seeks_report_discontinuities_without_rewinding_simulation_time() {
    let mut animation = Wc3Animation {
        sequence: 0,
        elapsed_ms: 0.0,
        speed: 1.0,
        playing: true,
        sequences: Vec::new(),
        global_sequences: Vec::new(),
        event_playback: Default::default(),
        pose_playback: Default::default(),
    };
    let mut clock = SimulationClock::default();
    assert!(clock.observe(&animation));
    clock.advance(0.5);
    animation.elapsed_ms = 500.0;
    assert!(!clock.observe(&animation));
    animation.elapsed_ms = 100.0;
    assert!(clock.observe(&animation));
    assert_eq!(clock.time, 0.5);
    animation.sequence = 1;
    assert!(clock.observe(&animation));
    for dt in [0.0, -1.0, f64::INFINITY, f64::NAN] {
        clock.advance(dt);
    }
    assert_eq!(clock.time, 0.5);
}

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
