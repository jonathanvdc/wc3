use super::*;
fn query(store: &Store, coordinate: Vec3, radius: f32) -> Result<Option<Vec3>> {
    let mut checks = usize::MAX;
    store
        .closest(coordinate, radius, &mut checks)
        .map(|result| match result {
            Search::Found(payload) => Some(payload),
            Search::Missing => None,
            Search::Omitted => panic!("unbounded test query exhausted checks"),
        })
}
#[test]
fn keys_and_payloads_are_separate_and_writes_become_visible_next_tick() {
    let mut store = Store::new(4, 0.75).unwrap();
    let key = Vec3::splat(3.);
    let payload = Vec3::new(100., -20., 60.);
    assert!(store.insert(key, payload));
    assert_eq!(query(&store, key, 0.).unwrap(), None);
    store.begin_tick();
    assert_eq!(query(&store, key, 0.).unwrap(), Some(payload));
    assert_eq!(query(&store, payload, 0.).unwrap(), None);
    assert!(store.insert(key, Vec3::Y));
    assert_eq!(query(&store, key, 0.).unwrap(), Some(payload));
    store.begin_tick();
    assert_eq!(query(&store, key, 0.).unwrap(), Some(Vec3::Y));
    store.begin_tick();
    assert_eq!(
        query(&store, key, 0.).unwrap(),
        None,
        "missing rebuild removes dead rows"
    );
}
#[test]
fn inclusive_radius_and_original_insertion_ties_survive_cell_sorting() {
    let mut store = Store::new(4, 0.75).unwrap();
    assert!(store.insert(Vec3::X, Vec3::new(1., 2., 3.)));
    assert!(store.insert(-Vec3::X, Vec3::new(4., 5., 6.)));
    store.begin_tick();
    assert_eq!(
        query(&store, Vec3::ZERO, 1.).unwrap(),
        Some(Vec3::new(1., 2., 3.))
    );
    assert_eq!(query(&store, Vec3::ZERO, 0.999).unwrap(), None);
    assert_eq!(
        query(&store, Vec3::ZERO, f32::MAX).unwrap(),
        Some(Vec3::new(1., 2., 3.))
    );
}
#[test]
fn capacity_omissions_preserve_old_reads_and_capacity_recycles_without_backlog() {
    let mut store = Store::new(1, 0.75).unwrap();
    assert!(store.insert(Vec3::ZERO, Vec3::X));
    assert!(!store.insert(Vec3::ZERO, Vec3::Y));
    store.begin_tick();
    assert!(store.insert(Vec3::ZERO, Vec3::Z));
    assert_eq!(query(&store, Vec3::ZERO, 0.).unwrap(), Some(Vec3::X));
    store.begin_tick();
    assert_eq!(query(&store, Vec3::ZERO, 0.).unwrap(), Some(Vec3::Z));
    store.clear();
    assert_eq!(query(&store, Vec3::ZERO, 0.).unwrap(), None);
    store.begin_tick();
    assert_eq!(query(&store, Vec3::ZERO, 0.).unwrap(), None);
    assert!(store.insert(Vec3::ZERO, Vec3::Y));
}
#[test]
fn unavailability_and_invalid_numeric_inputs_do_not_contaminate_other_rows() {
    let mut store = Store::new(4, 0.75).unwrap();
    assert!(!store.insert(Vec3::INFINITY, Vec3::ZERO));
    assert!(!store.insert(Vec3::ZERO, Vec3::NAN));
    assert!(!store.insert(Vec3::splat(f32::MAX), Vec3::ZERO));
    assert!(store.insert(Vec3::ZERO, Vec3::ONE));
    store.begin_tick();
    assert_eq!(query(&store, Vec3::ZERO, 0.).unwrap(), Some(Vec3::ONE));
    assert_eq!(query(&store, Vec3::INFINITY, 1.).unwrap(), None);
    assert!(query(&store, Vec3::ZERO, -1.).is_err());
    assert!(query(&store, Vec3::ZERO, f32::NAN).is_err());
    assert!(Store::new(4, 0.).is_err());
    assert!(Store::storage_bytes(usize::MAX).is_none());
}
#[test]
fn repeated_ticks_and_queries_reuse_both_preallocated_snapshots() {
    let mut store = Store::new(64, 0.75).unwrap();
    let mut pointers = [store.read.as_ptr(), store.write.as_ptr()];
    pointers.sort();
    for tick in 0..100 {
        store.begin_tick();
        for index in 0..64 {
            let key = Vec3::splat(index as f32);
            assert!(store.insert(key, Vec3::splat(tick as f32)));
            let _ = query(&store, key, 0.0001).unwrap();
        }
        let mut current = [store.read.as_ptr(), store.write.as_ptr()];
        current.sort();
        assert_eq!(current, pointers);
        assert_eq!(store.read.capacity(), 64);
        assert_eq!(store.write.capacity(), 64);
    }
}

#[test]
fn shared_row_budget_omits_incomplete_searches_and_zero_distance_finishes_early() {
    let mut store = Store::new(4, 1.).unwrap();
    assert!(store.insert(Vec3::new(0.4, 0., 0.), Vec3::X));
    assert!(store.insert(Vec3::new(0.2, 0., 0.), Vec3::Y));
    assert!(store.insert(Vec3::ZERO, Vec3::Z));
    store.begin_tick();
    let mut checks = 1;
    assert_eq!(
        store.closest(Vec3::ZERO, 1., &mut checks).unwrap(),
        Search::Omitted,
        "a partially examined nearest candidate is not an exact answer"
    );
    assert_eq!(checks, 0);
    assert_eq!(
        store.closest(Vec3::ZERO, 1., &mut checks).unwrap(),
        Search::Omitted,
        "a second query cannot reset the shared allowance"
    );
    store.clear();
    assert!(store.insert(Vec3::ZERO, Vec3::X));
    assert!(store.insert(Vec3::ZERO, Vec3::Y));
    assert!(store.insert(Vec3::new(0.1, 0., 0.), Vec3::Z));
    store.begin_tick();
    let mut checks = 1;
    assert_eq!(
        store.closest(Vec3::ZERO, 1., &mut checks).unwrap(),
        Search::Found(Vec3::X)
    );
    assert_eq!(
        checks, 0,
        "the first exact hit proves the deterministic answer"
    );
    assert_eq!(
        store.closest(Vec3::splat(20.), 0., &mut checks).unwrap(),
        Search::Missing,
        "an empty cell slab examines no row"
    );
}
