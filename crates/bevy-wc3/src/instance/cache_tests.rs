use super::*;
use crate::assets::Wc3Model;

fn asset(valid: bool) -> Wc3ModelAsset {
    let mut text = include_str!("../../tests/fixtures/quad_model.mdl").to_owned();
    if !valid {
        text = text
            .replace("Groups 1 1", "Groups 1 9")
            .replace("Matrices { 0 }", "Matrices { 0, 0, 0, 0, 0, 0, 0, 0, 0 }");
    }
    Wc3ModelAsset {
        source: Wc3Model::decode_mdl(&text).unwrap(),
        textures: default(),
        models: default(),
    }
}

#[test]
fn source_events_invalidate_preparation_and_allow_failed_models_to_retry() {
    let mut app = App::new();
    app.init_resource::<PreparedModelCache>();
    app.add_message::<AssetEvent<Wc3ModelAsset>>();
    app.add_systems(Update, invalidate_prepared_models);
    let mut sources = Assets::default();
    let good = sources.add(asset(true));
    let bad = sources.add(asset(false));
    let mut meshes = Assets::default();
    let mut bindposes = Assets::default();
    {
        let mut cache = app.world_mut().resource_mut::<PreparedModelCache>();
        assert!(cache
            .get_or_prepare(
                good.id(),
                sources.get(&good).unwrap(),
                &mut meshes,
                &mut bindposes
            )
            .unwrap()
            .is_some());
        assert!(cache
            .get_or_prepare(
                bad.id(),
                sources.get(&bad).unwrap(),
                &mut meshes,
                &mut bindposes
            )
            .is_err());
        assert!(cache
            .get_or_prepare(
                bad.id(),
                sources.get(&bad).unwrap(),
                &mut meshes,
                &mut bindposes
            )
            .unwrap()
            .is_none());
    }
    *sources.get_mut(&bad).unwrap() = asset(true);
    app.world_mut()
        .write_message(AssetEvent::Modified { id: good.id() });
    app.world_mut()
        .write_message(AssetEvent::Modified { id: bad.id() });
    app.update();
    let mut cache = app.world_mut().resource_mut::<PreparedModelCache>();
    assert!(cache.prepared.is_empty());
    assert!(cache.failed.is_empty());
    assert!(cache
        .get_or_prepare(
            bad.id(),
            sources.get(&bad).unwrap(),
            &mut meshes,
            &mut bindposes
        )
        .unwrap()
        .is_some());
}

#[test]
fn removal_and_reinsertion_invalidate_but_dependency_completion_preserves_cache() {
    let mut app = App::new();
    app.init_resource::<PreparedModelCache>();
    app.add_message::<AssetEvent<Wc3ModelAsset>>();
    app.add_systems(Update, invalidate_prepared_models);
    let mut sources = Assets::default();
    let handle = sources.add(asset(true));
    let mut meshes = Assets::default();
    let mut bindposes = Assets::default();
    for event in [
        AssetEvent::LoadedWithDependencies { id: handle.id() },
        AssetEvent::Unused { id: handle.id() },
        AssetEvent::Removed { id: handle.id() },
        AssetEvent::Added { id: handle.id() },
    ] {
        app.world_mut()
            .resource_mut::<PreparedModelCache>()
            .get_or_prepare(
                handle.id(),
                sources.get(&handle).unwrap(),
                &mut meshes,
                &mut bindposes,
            )
            .unwrap();
        app.world_mut().write_message(event);
        app.update();
        let expected = usize::from(matches!(
            event,
            AssetEvent::LoadedWithDependencies { .. } | AssetEvent::Unused { .. }
        ));
        assert_eq!(app.world().resource::<PreparedModelCache>().len(), expected);
    }
    app.world_mut()
        .resource_mut::<PreparedModelCache>()
        .get_or_prepare(
            handle.id(),
            sources.get(&handle).unwrap(),
            &mut meshes,
            &mut bindposes,
        )
        .unwrap();
    assert_eq!(app.world().resource::<PreparedModelCache>().len(), 1);
    sources.remove(handle.id());
    app.world_mut()
        .resource_mut::<PreparedModelCache>()
        .retain_live(&sources);
    assert_eq!(app.world().resource::<PreparedModelCache>().len(), 0);
}
