use super::*;
fn limits() -> ModelParticleLimits {
    ModelParticleLimits {
        actors_per_tree: 2,
        birth_work_per_frame: 4,
        emission_steps_per_frame: 3,
    }
}
#[test]
fn pending_ready_and_dropped_leases_share_one_resident_limit() {
    let budget = ActorBudget::new(Entity::PLACEHOLDER, limits());
    let mut first = budget.reserve(FrameCount(7)).unwrap();
    let second = budget.reserve(FrameCount(7)).unwrap();
    assert_eq!(
        budget.reserve(FrameCount(7)).unwrap_err(),
        ActorBudgetError::Actors
    );
    first.ready();
    first.ready();
    let stats = budget.stats(FrameCount(7));
    assert_eq!((stats.actors, stats.pending, stats.ready()), (2, 1, 1));
    drop(second);
    let third = budget.reserve(FrameCount(8)).unwrap();
    assert_eq!(budget.stats(FrameCount(8)).actors, 2);
    drop((first, third));
    assert_eq!(
        (
            budget.stats(FrameCount(8)).actors,
            budget.stats(FrameCount(8)).pending
        ),
        (0, 0)
    );
}
#[test]
fn actual_frame_quota_is_independent_of_clock_and_ready_pose_work() {
    let budget = ActorBudget::new(Entity::PLACEHOLDER, limits());
    let mut lease = budget.reserve(FrameCount(9)).unwrap();
    lease.ready();
    for _ in 0..3 {
        assert!(budget.step(FrameCount(9)));
    }
    assert!(!budget.step(FrameCount(9)));
    assert_eq!(budget.stats(FrameCount(9)).emission_steps, 3);
    assert!(
        lease.admit_ready(FrameCount(9)),
        "initial factory shares birth work"
    );
    assert_eq!(budget.stats(FrameCount(9)).birth_work, 1);
    // Changing an authored clock never appears in this API. Only a real new
    // FrameCount resets work; an already-ready pose requires no admission.
    assert!(budget.step(FrameCount(10)));
    assert!(lease.admit_ready(FrameCount(10)));
    assert_eq!(budget.stats(FrameCount(10)).birth_work, 1);
}
#[test]
fn limiting_or_deferred_work_cannot_leak_or_double_count_requests() {
    let mut config = limits();
    config.birth_work_per_frame = 1;
    let budget = ActorBudget::new(Entity::PLACEHOLDER, config);
    let lease = budget.reserve(FrameCount(0)).unwrap();
    assert_eq!(
        budget.reserve(FrameCount(0)).unwrap_err(),
        ActorBudgetError::BirthWork
    );
    budget.omit_work(99);
    assert_eq!(budget.stats(FrameCount(0)).omitted_actor_requests(), 100);
    budget.set_limits(ModelParticleLimits {
        actors_per_tree: 0,
        ..config
    });
    assert_eq!(
        budget.stats(FrameCount(1)).actors,
        1,
        "lowering limits preserves accepted actors"
    );
    assert_eq!(
        budget.reserve(FrameCount(1)).unwrap_err(),
        ActorBudgetError::Actors
    );
    budget.skip_seconds(2.);
    budget.skip_seconds(3.);
    assert_eq!(budget.stats(FrameCount(1)).skipped_emission_seconds, 5.);
    drop(lease);
    assert_eq!(budget.stats(FrameCount(1)).actors, 0);
}
