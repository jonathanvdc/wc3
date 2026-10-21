use super::*;

fn state() -> State {
    State {
        entity: Entity::PLACEHOLDER,
        children: Vec::new(),
        phase: Default::default(),
        ordinal: 0,
        scratch: Vec::new(),
        previous_node: Mat4::IDENTITY,
        omitted: 0,
        skipped_seconds: 0.,
    }
}
fn packet(ordinal: u64) -> [f32; 3] {
    let mut rng = EmitterRng::at_occurrence(17, ordinal);
    [rng.random(), rng.signed(), rng.random()]
}
#[test]
fn budgets_and_invalid_occurrences_do_not_shift_later_model_packets() {
    let mut whole = state();
    let mut limited = state();
    let (_, whole_first) = whole.schedule(20., 0.2, 1024);
    let (_, limited_first) = limited.schedule(20., 0.2, 1);
    assert_eq!(packet(whole_first), packet(limited_first));
    assert_eq!((limited.ordinal, limited.omitted), (4, 3));
    let (_, whole_next) = whole.schedule(20., 0.1, 1024);
    let (_, limited_next) = limited.schedule(20., 0.1, 1024);
    assert_eq!(whole_next, 4);
    assert_eq!(packet(whole_next), packet(limited_next));
    // The production loop validates pose/lifetime/resource only after this
    // reservation. Discarding all candidates must still consume their identities.
    let _invalid = limited.schedule(20., 0.1, 1024);
    let (_, following) = limited.schedule(20., 0.05, 1024);
    assert_eq!(following, 8);
}
#[test]
fn split_updates_and_reset_preserve_source_occurrence_identity() {
    let mut whole = state();
    let (all, first) = whole.schedule(30., 0.1, 1024);
    let expected: Vec<_> = (0..all.count as u64).map(|i| packet(first + i)).collect();
    let mut split = state();
    let mut observed = Vec::new();
    for dt in [0.03, 0.04, 0.03] {
        let (schedule, first) = split.schedule(30., dt, 1024);
        observed.extend((0..schedule.count as u64).map(|i| packet(first + i)));
    }
    assert_eq!(observed, expected);
    split.reset_emission();
    let (_, first) = split.schedule(30., 0.1, 1024);
    assert_eq!(first, 0);
    assert_eq!(packet(first), expected[0]);
}
#[test]
fn large_omitted_schedules_advance_once_and_budget_totals_saturate() {
    let mut state = state();
    let (schedule, first) = state.schedule(1e12, 1., 0);
    assert_eq!((schedule.count, first), (0., 0));
    assert_eq!(
        (state.ordinal, state.omitted),
        (1_000_000_000_000, 1_000_000_000_000)
    );
    state.omitted = u64::MAX - 1;
    let (_, first) = state.schedule(4., 1., 1);
    assert_eq!(first, 1_000_000_000_000);
    assert_eq!(state.omitted, u64::MAX);
}
