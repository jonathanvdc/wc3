use wc3::model::{
    conversion::{Asset, ExportOptions},
    wem::{self, records as r, semantic},
};

#[test]
fn selecting_a_look_resolves_bindings_and_editing_keeps_other_looks() {
    let model = semantic::Model {
        profile: r::ProfileId::WC3_CLASSIC,
        materials: vec![
            semantic::Material {
                name: "first".into(),
                ..Default::default()
            },
            semantic::Material {
                name: "second".into(),
                ..Default::default()
            },
        ],
        meshes: vec![semantic::Mesh::default()],
        ..Default::default()
    };
    let mut raw = semantic::build(&model).unwrap().into_raw();
    let profile_id = raw.sections().find(|(_, s)| s.tag() == *b"PROF").unwrap().0;
    let mut profile = raw
        .array::<r::ProfileMaterialSet>(profile_id)
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let looks = raw
        .append(&[r::Look::default(), r::Look::default()])
        .unwrap();
    profile.looks.looks = wem::Ref::new(2, looks.index(), 0);
    let indices = raw
        .append(&[r::U32 { value: 0 }, r::U32 { value: 1 }])
        .unwrap();
    let binding = raw
        .append(&[r::SlotBinding {
            by_look: wem::Ref::new(2, indices.index(), 0),
        }])
        .unwrap();
    profile.slot_bindings = wem::Ref::new(1, binding.index(), 0);
    raw.replace(profile_id, &[profile]).unwrap();
    let asset = Asset::new(wem::Document::from_raw(raw).unwrap());
    let selected = asset
        .model(&ExportOptions {
            look: 1,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(selected.meshes[0].material_slot, 1);
    let rebuilt = Asset::new(semantic::build(&selected).unwrap());
    assert_eq!(
        rebuilt.model(&ExportOptions::default()).unwrap().meshes[0].material_slot,
        0
    );
    assert_eq!(
        rebuilt
            .model(&ExportOptions {
                look: 1,
                ..Default::default()
            })
            .unwrap()
            .meshes[0]
            .material_slot,
        1
    );
    assert!(rebuilt
        .model(&ExportOptions {
            look: 2,
            ..Default::default()
        })
        .is_err());
}

#[test]
fn native_animation_ids_do_not_collide_with_common_channels() {
    let track = |target| semantic::Track {
        layer: Default::default(),
        target,
        interpolation: semantic::Interpolation::Linear,
        global_duration: None,
        keys: vec![semantic::Key {
            time: 0.,
            value: vec![1., 2., 3.],
            incoming: None,
            outgoing: None,
        }],
    };
    let tracks = vec![
        track(semantic::Target::NativeProperty { owner: 0, id: 0 }),
        track(semantic::Target::Translation(0)),
    ];
    let model = semantic::Model {
        profile: r::ProfileId::SC2,
        nodes: vec![semantic::Node::default()],
        clips: vec![
            semantic::Clip {
                duration: 1.,
                tracks: tracks.clone(),
                ..Default::default()
            },
            semantic::Clip {
                duration: 1.,
                tracks: tracks.clone(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let document = semantic::build(&model).unwrap();
    let output = semantic::read(&document, 0, r::ProfileId::SC2).unwrap();
    assert_eq!(output.clips[0].tracks, tracks);
    assert_eq!(output.clips[1].tracks, tracks);
    let raw = document.raw();
    let (_, section) = raw.sections().find(|(_, s)| s.tag() == *b"ACHN").unwrap();
    assert_eq!(section.count(), 3);
}
