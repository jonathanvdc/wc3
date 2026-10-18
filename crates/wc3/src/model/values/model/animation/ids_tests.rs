use super::{AnimationId, CATALOG};
use std::collections::BTreeSet;

#[test]
fn catalog_has_unique_ids_and_names() {
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for &(id, name) in CATALOG {
        assert!(ids.insert(id), "duplicate ID: {id:?}");
        assert!(names.insert(name), "duplicate name: {name}");
    }
}

#[test]
fn representative_native_names_and_ids_are_stable() {
    const WALK_NAME: Option<&str> = AnimationId::WALK.name();
    assert_eq!(WALK_NAME, Some("Walk"));
    assert_eq!(AnimationId::STAND.get(), 0);
    assert_eq!(AnimationId::ATTACK_1H.get(), 17);
    assert_eq!(AnimationId::FLY_STAND.get(), 229);
    assert_eq!(AnimationId::CINEMATIC_01.get(), 1452);
    assert_eq!(AnimationId::FLY_DRAC_FLY_BREATH_LAND_FINISH.get(), 1577);
    assert_eq!(
        AnimationId::FLY_DRAC_FLY_BREATH_LAND_FINISH.name(),
        Some("FlyDracFlyBreathLandFinish")
    );
}

#[test]
fn uncatalogued_values_keep_their_numeric_identity() {
    for value in [1578, 32000, u16::MAX] {
        let id = AnimationId::from(value);
        assert_eq!(id.name(), None);
        assert_eq!(u16::from(id), value);
    }
}
