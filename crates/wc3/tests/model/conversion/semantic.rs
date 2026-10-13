use wc3::model::{
    conversion::{Asset, ExportOptions},
    wem::{self, records as r, semantic},
};

#[test]
fn selecting_a_look_resolves_bindings_and_editing_keeps_other_looks() {
    let model = semantic::Model {
        profile: r::ProfileId::Wc3Classic,
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
    profile.looks.looks = wem::Ref::from_section(2, looks.index());
    let indices = raw
        .append(&[r::U32 { value: 0 }, r::U32 { value: 1 }])
        .unwrap();
    let binding = raw
        .append(&[r::SlotBinding {
            by_look: wem::Ref::from_section(2, indices.index()),
        }])
        .unwrap();
    profile.slot_bindings = wem::Ref::from_section(1, binding.index());
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
        profile: r::ProfileId::Sc2,
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
    let output = semantic::read(&document, 0, r::ProfileId::Sc2).unwrap();
    assert_eq!(output.clips[0].tracks, tracks);
    assert_eq!(output.clips[1].tracks, tracks);
    let raw = document.raw();
    let (_, section) = raw.sections().find(|(_, s)| s.tag() == *b"ACHN").unwrap();
    assert_eq!(section.count(), 3);
}

#[test]
fn common_enum_flags_and_clip_looping_survive_wem_edits() {
    let mut flags = r::MaterialFlags::from_bits_retain(0x8000_0000);
    flags.set_unlit(true);
    let model = semantic::Model {
        profile: r::ProfileId::Wc3Classic,
        materials: vec![semantic::Material {
            blend: r::BlendMode::Unknown(0xffff),
            flags,
            ..Default::default()
        }],
        clips: vec![semantic::Clip {
            name: "once".into(),
            looping: false,
            flags: r::ClipFlags::from_bits_retain(0x4000_0000),
            ..Default::default()
        }],
        ..Default::default()
    };
    let asset = Asset::new(semantic::build(&model).unwrap());
    let mut edited = asset.model(&ExportOptions::default()).unwrap();
    assert_eq!(edited.materials[0].blend, model.materials[0].blend);
    assert_eq!(edited.materials[0].flags, flags);
    assert!(!edited.clips[0].looping);
    assert_eq!(edited.clips[0].flags, model.clips[0].flags);
    edited.clips[0].name = "renamed".into();
    let rebuilt = Asset::new(semantic::build(&edited).unwrap());
    let saved = rebuilt.model(&ExportOptions::default()).unwrap();
    assert_eq!(saved.materials[0].blend, model.materials[0].blend);
    assert_eq!(saved.materials[0].flags, flags);
    assert!(!saved.clips[0].looping);
    assert_eq!(saved.clips[0].flags, model.clips[0].flags);
}

#[test]
fn renaming_native_values_preserves_reference_flags_and_noncanonical_looping() {
    let model = semantic::Model {
        name: "Old".into(),
        clips: vec![semantic::Clip {
            name: "Animation".into(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut raw = semantic::build(&model).unwrap().into_raw();
    let mesh_id = raw.sections().find(|(_, s)| s.tag() == *b"MODL").unwrap().0;
    let mut graph = raw
        .array::<r::ModelRecord<r::V9>>(mesh_id)
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    graph.name.flags = wem::ReferenceFlags(0x8000_0001);
    raw.replace(mesh_id, &[graph]).unwrap();
    let clips_id = raw.sections().find(|(_, s)| s.tag() == *b"CLIP").unwrap().0;
    let mut clip = raw
        .array::<r::Clip<r::V7>>(clips_id)
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    clip.looping = wem::Bool(17);
    clip.name.flags = wem::ReferenceFlags(0x41);
    raw.replace(clips_id, &[clip]).unwrap();
    let doc = wem::Document::from_raw(raw).unwrap();
    let mut common = semantic::read(&doc, 0, r::ProfileId::Generic).unwrap();
    common.name = "New".into();
    common.clips[0].name = "Renamed animation".into();
    let rebuilt = semantic::build(&common).unwrap();
    let root = rebuilt.root::<r::DocumentRecord<r::V1>>().unwrap();
    let graph = rebuilt
        .raw()
        .resolve::<r::ModelRecord<r::V9>>(root.models.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(graph.name.flags.bits(), 0x8000_0001);
    let clip = rebuilt
        .raw()
        .resolve::<r::Clip<r::V7>>(root.clips.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(clip.name.flags.bits(), 0x41);
    assert_eq!(clip.looping.0, 17);
}
