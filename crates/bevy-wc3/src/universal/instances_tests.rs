use super::super::{compile_universal_model, UniversalPoseOverrides};
use super::*;
use wc3::model::profiles::Universal;
use wc3::model::{Model, Node, NodeId, Transform as ModelTransform};

fn fixture() -> (UniversalModelAsset, NodeId) {
    let mut builder = Model::builder("lifecycle");
    let node = builder.add_node(Node::named("node")).unwrap();
    let model = builder.finish().unwrap();
    (
        UniversalModelAsset {
            compiled: compile_universal_model(&model, |_| None).unwrap(),
            images: HashMap::new(),
            unresolved_images: Vec::new(),
        },
        node,
    )
}
fn app() -> App {
    let mut app = App::new();
    app.init_resource::<Assets<UniversalModelAsset>>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<SkinnedMeshInverseBindposes>>()
        .init_resource::<Assets<UniversalMaterial>>()
        .init_resource::<UniversalPreparedCache>()
        .add_message::<AssetEvent<UniversalModelAsset>>()
        .add_systems(
            Update,
            (invalidate_universal_models, spawn_loaded_universal_models).chain(),
        );
    app
}
#[test]
fn populated_source_retains_a_strong_asset_handle() {
    let mut app = app();
    let (asset, _) = fixture();
    let handle = app
        .world_mut()
        .resource_mut::<Assets<UniversalModelAsset>>()
        .add(asset);
    let id = handle.id();
    let root = app.world_mut().spawn(UniversalModelInstance(handle)).id();
    app.update();
    let source = app.world().get::<UniversalModelSource>(root).unwrap();
    assert_eq!(source.0.id(), id);
    assert!(source.0.is_strong());
}
#[test]
fn modified_assets_repopulate_and_preserve_playback_and_application_children() {
    let mut app = app();
    let (asset, node) = fixture();
    let handle = app
        .world_mut()
        .resource_mut::<Assets<UniversalModelAsset>>()
        .add(asset);
    let id = handle.id();
    let root = app
        .world_mut()
        .spawn((
            UniversalModelInstance(handle),
            UniversalPlayback {
                shared_time: 3.,
                paused: true,
                ..Default::default()
            },
        ))
        .id();
    let application_child = app.world_mut().spawn(ChildOf(root)).id();
    app.update();
    let old_node = app
        .world()
        .get::<UniversalInstance>(root)
        .unwrap()
        .node(node)
        .unwrap();
    let overrides = app
        .world()
        .get::<UniversalPoseOverrides>(root)
        .unwrap()
        .0
        .clone();
    app.world_mut()
        .write_message(AssetEvent::<UniversalModelAsset>::Modified { id });
    app.update();
    let new_node = app
        .world()
        .get::<UniversalInstance>(root)
        .unwrap()
        .node(node)
        .unwrap();
    assert_ne!(old_node, new_node);
    assert!(app.world().get_entity(old_node).is_err());
    assert!(app.world().get_entity(application_child).is_ok());
    assert_eq!(
        app.world()
            .get::<UniversalPlayback>(root)
            .unwrap()
            .shared_time,
        3.
    );
    assert_eq!(
        app.world().get::<UniversalPoseOverrides>(root).unwrap().0,
        overrides
    );
    assert_eq!(
        app.world()
            .get::<UniversalModelSource>(root)
            .unwrap()
            .0
            .id(),
        id
    );
}
#[test]
fn child_model_ancestry_rejects_cycles_but_keeps_unrelated_requests() {
    let mut app = app();
    let (asset, _) = fixture();
    let handle = app
        .world_mut()
        .resource_mut::<Assets<UniversalModelAsset>>()
        .add(asset);
    let root = app
        .world_mut()
        .spawn(UniversalModelInstance(handle.clone()))
        .id();
    app.update();
    let mount = app.world_mut().spawn(ChildOf(root)).id();
    let cycle = app
        .world_mut()
        .spawn((UniversalModelInstance(handle.clone()), ChildOf(mount)))
        .id();
    let unrelated = app.world_mut().spawn(UniversalModelInstance(handle)).id();
    app.update();
    assert!(app.world().get::<UniversalModelInstance>(cycle).is_none());
    assert!(app.world().get::<UniversalInstance>(cycle).is_none());
    assert!(
        matches!(app.world().get::<UniversalModelInstanceError>(cycle),Some(UniversalModelInstanceError::CyclicDependency {ancestor,..}) if *ancestor == root)
    );
    assert!(app.world().get::<UniversalInstance>(unrelated).is_some());
}

#[test]
fn parent_replacement_defers_requests_in_its_old_owned_hierarchy() {
    let mut app = app();
    let (asset, node) = fixture();
    let handle = app
        .world_mut()
        .resource_mut::<Assets<UniversalModelAsset>>()
        .add(asset);
    let root = app
        .world_mut()
        .spawn(UniversalModelInstance(handle.clone()))
        .id();
    app.update();
    let owned_node = app
        .world()
        .get::<UniversalInstance>(root)
        .unwrap()
        .node(node)
        .unwrap();
    let child = app
        .world_mut()
        .spawn((UniversalModelInstance(handle.clone()), ChildOf(owned_node)))
        .id();
    app.world_mut()
        .entity_mut(root)
        .insert(UniversalModelInstance(handle));
    app.update();
    assert!(app.world().get_entity(child).is_err());
    assert!(app.world().get::<UniversalInstance>(root).is_some());
}

#[test]
fn mounted_frame_correction_preserves_root_and_cached_basis_across_reload() {
    use super::super::placements::ModelPlacementSpace;
    let mut app = app();
    let (mut asset, node) = fixture();
    let child_basis =
        Transform::from_rotation(Quat::from_rotation_z(0.4)).with_scale(Vec3::splat(2.));
    asset.compiled.basis = child_basis;
    let handle = app
        .world_mut()
        .resource_mut::<Assets<UniversalModelAsset>>()
        .add(asset);
    let id = handle.id();
    let placement = ChildModelPlacement {
        space: ModelPlacementSpace::Physical,
        parent_basis: Transform::from_rotation(Quat::from_rotation_x(1.1))
            .with_scale(Vec3::splat(0.01)),
    };
    let root_transform = Transform::from_xyz(4., 5., 6.).with_rotation(Quat::from_rotation_y(0.3));
    let root = app
        .world_mut()
        .spawn((UniversalModelInstance(handle), placement, root_transform))
        .id();
    for reload in 0..3 {
        if reload > 0 {
            app.world_mut()
                .write_message(AssetEvent::<UniversalModelAsset>::Modified { id });
        }
        app.update();
        assert_eq!(*app.world().get::<Transform>(root).unwrap(), root_transform);
        let node = app
            .world()
            .get::<UniversalInstance>(root)
            .unwrap()
            .node(node)
            .unwrap();
        let basis = app.world().get::<ChildOf>(node).unwrap().parent();
        assert_eq!(
            *app.world().get::<Transform>(basis).unwrap(),
            placement.basis(child_basis)
        );
        assert_eq!(
            app.world().resource::<UniversalPreparedCache>().0[&id].basis,
            child_basis
        );
    }
}

#[test]
fn separately_decoded_replacement_rebinds_clip_and_pose_scoped_identities() {
    use wc3::model::conversion::ImportOptions;
    use wc3::model::io::mdlx;
    use wc3::model::Controller;
    let text = r#"
        Version { FormatVersion 800, } Model "rebind" {}
        Sequences 2 {
            Anim "Other" { Interval {0,1000}, }
            Anim "Selected" { Interval {2000,3000}, }
        }
        Helper "anchor" { ObjectId 0, }
        PivotPoints 1 { {0,0,0}, }
    "#;
    let old = mdlx::decode_mdl(text, &ImportOptions::default())
        .unwrap()
        .value
        .widen();
    let new = mdlx::decode_mdl(text, &ImportOptions::default())
        .unwrap()
        .value
        .widen();
    let old_clip = old
        .clips()
        .find(|(_, clip)| clip.name() == "Selected")
        .unwrap()
        .0;
    let new_clip = new
        .clips()
        .find(|(_, clip)| clip.name() == "Selected")
        .unwrap()
        .0;
    let old_node = old.nodes().next().unwrap().0;
    let new_node = new.nodes().next().unwrap().0;
    assert_ne!(old_clip, new_clip);
    assert_ne!(old_node, new_node);
    let mut app = app();
    let old_asset = UniversalModelAsset {
        compiled: compile_universal_model(&old, |_| None).unwrap(),
        images: HashMap::new(),
        unresolved_images: Vec::new(),
    };
    let new_asset = UniversalModelAsset {
        compiled: compile_universal_model(&new, |_| None).unwrap(),
        images: HashMap::new(),
        unresolved_images: Vec::new(),
    };
    let old_handle = app
        .world_mut()
        .resource_mut::<Assets<UniversalModelAsset>>()
        .add(old_asset);
    let new_handle = app
        .world_mut()
        .resource_mut::<Assets<UniversalModelAsset>>()
        .add(new_asset);
    let pose = ModelTransform {
        translation: [4., 5., 6.],
        ..Default::default()
    };
    let root = app
        .world_mut()
        .spawn((
            UniversalModelInstance(old_handle),
            UniversalPlayback {
                clip: Some(old_clip),
                clip_time: 0.42,
                timeline_time: 7.,
                shared_time: 9.,
                paused: true,
                speed: 0.3,
                ..Default::default()
            },
            UniversalPoseOverrides(HashMap::from([(old_node, pose)])),
        ))
        .id();
    let app_child = app.world_mut().spawn(ChildOf(root)).id();
    app.update();
    let foreign_controller = Model::<Universal>::builder("foreign controller")
        .add_controller(Controller::default())
        .unwrap();
    app.world_mut()
        .get_mut::<UniversalPlayback>(root)
        .unwrap()
        .controllers = UniversalControllerSelection::Explicit(vec![foreign_controller]);
    app.world_mut()
        .entity_mut(root)
        .insert(UniversalModelInstance(new_handle));
    app.update();
    let playback = app.world().get::<UniversalPlayback>(root).unwrap();
    assert_eq!(playback.clip, Some(new_clip));
    assert_eq!(
        [
            playback.clip_time,
            playback.timeline_time,
            playback.shared_time
        ],
        [0.42, 7., 9.]
    );
    assert!(playback.paused);
    assert_eq!(playback.speed, 0.3);
    assert_eq!(
        playback.controllers,
        UniversalControllerSelection::Explicit(vec![])
    );
    let overrides = &app.world().get::<UniversalPoseOverrides>(root).unwrap().0;
    assert_eq!(overrides, &HashMap::from([(new_node, pose)]));
    let report = app.world().get::<UniversalModelRebind>(root).unwrap();
    assert!(report.clip_rebound);
    assert!(!report.clip_reset);
    assert_eq!(report.poses_rebound, 1);
    assert_eq!(report.controllers_reset, 1);
    assert!(app.world().get_entity(app_child).is_ok());
    let model = app.world().get::<UniversalInstance>(root).unwrap().model();
    assert!(model
        .animation
        .sample(playback.selection(model).unwrap())
        .is_ok());
    assert!(!playback.event_traversal.include_origin);
    assert_eq!(playback.event_traversal.origin.shared, 9.);
}

#[test]
fn ambiguous_names_reset_selection_and_remove_pose_overrides() {
    use wc3::model::Clip;
    let prepare = |clips: &[&str], nodes: &[&str]| {
        let mut builder = Model::builder("ambiguous rebind");
        for name in clips {
            builder.add_clip(Clip::named((*name).to_owned())).unwrap();
        }
        for name in nodes {
            builder.add_node(Node::named((*name).to_owned())).unwrap();
        }
        let model = builder.finish().unwrap();
        compile_universal_model(&model, |_| None)
            .unwrap()
            .prepare(&mut Assets::default(), &mut Assets::default())
    };
    let old = prepare(&["Selected"], &["anchor"]);
    let new = prepare(&["Fallback", "Selected", "Selected"], &["anchor", "anchor"]);
    let mut playback = UniversalPlayback {
        clip: Some(old.clips[0].0),
        ..Default::default()
    };
    let overrides = UniversalPoseOverrides(HashMap::from([(
        old.nodes[0].id,
        ModelTransform::default(),
    )]));
    let (report, overrides) = rebind_state(Some(&old), &new, &mut playback, Some(&overrides));
    assert_eq!(playback.clip, Some(new.clips[0].0));
    assert!(report.clip_reset);
    assert!(!report.clip_rebound);
    assert_eq!(report.poses_reset, 1);
    assert!(overrides.unwrap().0.is_empty());
}

#[test]
fn failed_asset_and_dependency_requests_remain_observable_and_valid_replacement_recovers() {
    use super::super::UniversalModelLoader;
    use bevy::asset::{AssetApp, AssetPlugin};
    use std::{
        env::temp_dir,
        fs,
        thread::sleep,
        time::{Duration, Instant},
    };
    let directory = temp_dir().join(format!(
        "universal-instance-failures-{}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join("parent.mdl"),
        r#"
        Version { FormatVersion 800, } Model "dependency failure" {}
        Attachment "child" { ObjectId 0, Path "broken.mdl", }
        PivotPoints 1 { {0,0,0}, }
    "#,
    )
    .unwrap();
    fs::write(directory.join("broken.mdl"), "invalid native model").unwrap();
    let mut app = app();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: directory.to_string_lossy().into_owned(),
            ..Default::default()
        },
    ));
    app.init_asset::<UniversalModelAsset>();
    app.init_asset_loader::<UniversalModelLoader>();
    let missing = app
        .world()
        .resource::<AssetServer>()
        .load::<UniversalModelAsset>("missing.mdl");
    let parent = app
        .world()
        .resource::<AssetServer>()
        .load::<UniversalModelAsset>("parent.mdl");
    let missing_root = app.world_mut().spawn(UniversalModelInstance(missing)).id();
    let dependency_root = app.world_mut().spawn(UniversalModelInstance(parent)).id();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        app.update();
        if app
            .world()
            .get::<UniversalModelInstanceError>(missing_root)
            .is_some()
            && app
                .world()
                .get::<UniversalModelInstanceError>(dependency_root)
                .is_some()
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "asset failures were not surfaced"
        );
        sleep(Duration::from_millis(5));
    }
    assert!(matches!(
        app.world().get::<UniversalModelInstanceError>(missing_root),
        Some(UniversalModelInstanceError::FailedAsset { .. })
    ));
    assert!(matches!(
        app.world()
            .get::<UniversalModelInstanceError>(dependency_root),
        Some(UniversalModelInstanceError::FailedDependency { .. })
    ));
    assert!(app
        .world()
        .get::<UniversalModelInstance>(missing_root)
        .is_some());
    assert!(app
        .world()
        .get::<UniversalModelInstance>(dependency_root)
        .is_some());
    let (asset, _) = fixture();
    let replacement = app
        .world_mut()
        .resource_mut::<Assets<UniversalModelAsset>>()
        .add(asset);
    for root in [missing_root, dependency_root] {
        app.world_mut()
            .entity_mut(root)
            .insert(UniversalModelInstance(replacement.clone()));
    }
    app.update();
    for root in [missing_root, dependency_root] {
        assert!(app
            .world()
            .get::<UniversalModelInstanceError>(root)
            .is_none());
        assert!(app.world().get::<UniversalInstance>(root).is_some());
        assert!(app.world().get::<UniversalModelInstance>(root).is_none());
    }
    fs::remove_dir_all(directory).unwrap();
}
