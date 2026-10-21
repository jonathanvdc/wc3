use super::super::birth::BirthPlan;
use super::*;
fn sample(rate: f32) -> SquirtSample {
    SquirtSample::new(rate, true).unwrap()
}
fn count(state: &mut SquirtState, rate: f32) -> Option<BirthPlan> {
    state
        .observe(sample(rate), SquirtUpdate::Advance)
        .map(|burst| burst.plan(0., 0., 8192).unwrap())
}
#[test]
fn startup_and_sampled_threshold_edges_are_not_keyed_or_continuous() {
    let mut state = SquirtState::default();
    assert_eq!(count(&mut state, 12.).unwrap().requested, 12);
    for rate in [12., 25., 0.021, 100.] {
        assert!(count(&mut state, rate).is_none());
    }
    assert!(count(&mut state, 0.02).is_none());
    assert_eq!(count(&mut state, 3.9).unwrap().requested, 3);
    // No observation of the low part means an intervening pulse is missed.
    assert!(count(&mut state, 3.9).is_none());
}
#[test]
fn empty_bursts_have_no_residual_or_later_positive_ramp_retry() {
    let mut state = SquirtState::default();
    assert_eq!(count(&mut state, 0.3).unwrap().requested, 0);
    assert!(count(&mut state, 0.9).is_none());
    for _ in 0..10 {
        assert!(count(&mut state, 0.).is_none());
        assert_eq!(count(&mut state, 0.3).unwrap().requested, 0);
    }
}
#[test]
fn holds_never_consume_startup_or_update_edge_history() {
    let mut state = SquirtState::default();
    for _ in 0..120 {
        assert!(state.observe(sample(12.), SquirtUpdate::Hold).is_none());
    }
    assert_eq!(count(&mut state, 12.).unwrap().requested, 12);
    assert!(state.observe(sample(0.), SquirtUpdate::Hold).is_none());
    assert!(count(&mut state, 12.).is_none());
}
#[test]
fn seek_cancels_pending_and_rebases_without_an_edge_while_reset_rearms() {
    let mut state = SquirtState::default();
    assert!(state.observe(sample(20.), SquirtUpdate::Seek).is_none());
    assert!(count(&mut state, 20.).is_none());
    state.reset();
    assert_eq!(count(&mut state, 20.).unwrap().requested, 20);
    assert!(count(&mut state, 0.).is_none());
    assert!(state.observe(sample(20.), SquirtUpdate::Seek).is_none());
    assert!(count(&mut state, 20.).is_none());
    assert!(count(&mut state, 0.).is_none());
    assert_eq!(count(&mut state, 20.).unwrap().requested, 20);
}
#[test]
fn activation_is_a_strong_gate_and_enabling_supplies_a_real_sampled_edge() {
    let mut state = SquirtState::default();
    assert!(state
        .observe(
            SquirtSample::new(100., false).unwrap(),
            SquirtUpdate::Advance
        )
        .is_none());
    assert_eq!(count(&mut state, 4.).unwrap().requested, 4);
    assert!(count(&mut state, -2.).is_none());
    assert_eq!(count(&mut state, 4.).unwrap().requested, 4);
}
#[test]
fn count_is_additive_jitter_truncated_and_capacity_omissions_do_not_queue() {
    let mut state = SquirtState::default();
    let burst = state.observe(sample(10.), SquirtUpdate::Advance).unwrap();
    assert_eq!(
        burst.plan(4., -0.5, 3).unwrap(),
        BirthPlan {
            requested: 8,
            accepted: 3,
            omitted: 5
        }
    );
    assert!(count(&mut state, 10.).is_none());
    assert!(count(&mut state, 0.).is_none());
    assert_eq!(count(&mut state, 2.9).unwrap().requested, 2);
}
#[test]
fn enabled_zero_rate_can_jitter_but_negative_result_never_creates_a_count() {
    let mut state = SquirtState::default();
    assert_eq!(
        state
            .observe(sample(0.), SquirtUpdate::Advance)
            .unwrap()
            .plan(4., 0.5, 9)
            .unwrap()
            .requested,
        2
    );
    state.reset();
    assert_eq!(
        state
            .observe(sample(1.), SquirtUpdate::Advance)
            .unwrap()
            .plan(4., -0.5, 9)
            .unwrap()
            .requested,
        0
    );
}
#[test]
fn invalid_numeric_controls_fail_explicitly_without_a_residual() {
    assert!(matches!(
        SquirtSample::new(f32::NAN, true),
        Err(SquirtError::NonFiniteInput)
    ));
    assert!(matches!(
        SquirtSample::new(f32::INFINITY, true),
        Err(SquirtError::NonFiniteInput)
    ));
    for (variation, jitter, error) in [
        (0., 1., SquirtError::InvalidJitter),
        (f32::INFINITY, 0., SquirtError::NonFiniteInput),
    ] {
        let mut state = SquirtState::default();
        assert_eq!(
            state
                .observe(sample(10.), SquirtUpdate::Advance)
                .unwrap()
                .plan(variation, jitter, 9),
            Err(error)
        );
        assert!(count(&mut state, 10.).is_none());
    }
    let mut overflowing = SquirtState::default();
    assert_eq!(
        overflowing
            .observe(sample(f32::MAX), SquirtUpdate::Advance)
            .unwrap()
            .plan(f32::MAX, 0.5, 9),
        Err(SquirtError::NonFiniteInput)
    );
    let mut state = SquirtState::default();
    assert_eq!(
        state
            .observe(sample(1e30), SquirtUpdate::Advance)
            .unwrap()
            .plan(0., 0., 9),
        Err(SquirtError::CountOverflow)
    );
}

#[test]
fn burst_packet_identity_advances_for_empty_or_omitted_requests_but_not_holds() {
    let mut state = SquirtState::default();
    let first = state.observe(sample(0.3), SquirtUpdate::Advance).unwrap();
    assert_eq!(first.ordinal, 0);
    assert_eq!(first.plan(0., 0., 0).unwrap().requested, 0);
    assert!(state.observe(sample(0.), SquirtUpdate::Hold).is_none());
    assert!(count(&mut state, 0.).is_none());
    let second = state.observe(sample(3.), SquirtUpdate::Advance).unwrap();
    assert_eq!(second.ordinal, 1);
    assert_eq!(second.plan(0., 0., 0).unwrap().omitted, 3);
    state.reset();
    assert_eq!(
        state
            .observe(sample(3.), SquirtUpdate::Advance)
            .unwrap()
            .ordinal,
        0
    );
}
