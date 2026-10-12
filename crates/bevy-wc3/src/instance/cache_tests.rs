use super::*;
use crate::assets::Wc3Model;

fn asset() -> Wc3ModelAsset {
    let source = Wc3Model::decode_mdl(include_str!("../../tests/fixtures/quad_model.mdl")).unwrap();
    Wc3ModelAsset::from_source(&source, default(), default()).unwrap()
}

#[test]
fn source_events_invalidate_preparation() {
    let mut app = App::new();
    app.init_resource::<PreparedModelCache>();
    app.add_message::<AssetEvent<Wc3ModelAsset>>();
    app.add_systems(Update, invalidate_prepared_models);
    let mut sources = Assets::default();
    let good = sources.add(asset());
    let other = sources.add(asset());
    let mut meshes = Assets::default();
    let mut bindposes = Assets::default();
    {
        let mut cache = app.world_mut().resource_mut::<PreparedModelCache>();
        for handle in [&good, &other] {
            cache.get_or_prepare(
                handle.id(),
                sources.get(handle).unwrap(),
                &mut meshes,
                &mut bindposes,
            );
        }
        assert_eq!(cache.len(), 2);
    }
    *sources.get_mut(&other).unwrap() = asset();
    app.world_mut()
        .write_message(AssetEvent::Modified { id: good.id() });
    app.world_mut()
        .write_message(AssetEvent::Modified { id: other.id() });
    app.update();
    let mut cache = app.world_mut().resource_mut::<PreparedModelCache>();
    assert!(cache.prepared.is_empty());
    cache.get_or_prepare(
        other.id(),
        sources.get(&other).unwrap(),
        &mut meshes,
        &mut bindposes,
    );
    assert_eq!(cache.len(), 1);
}

#[test]
fn removal_and_reinsertion_invalidate_but_dependency_completion_preserves_cache() {
    let mut app = App::new();
    app.init_resource::<PreparedModelCache>();
    app.add_message::<AssetEvent<Wc3ModelAsset>>();
    app.add_systems(Update, invalidate_prepared_models);
    let mut sources = Assets::default();
    let handle = sources.add(asset());
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
            );
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
        );
    assert_eq!(app.world().resource::<PreparedModelCache>().len(), 1);
    sources.remove(handle.id());
    app.world_mut()
        .resource_mut::<PreparedModelCache>()
        .retain_live(&sources);
    assert_eq!(app.world().resource::<PreparedModelCache>().len(), 0);
}
