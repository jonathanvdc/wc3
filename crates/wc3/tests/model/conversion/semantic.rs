use authored::animation::{
    AnyDeclaration, AnyTrack, ClipFlags, Declaration, NodeOperation, Target, Track,
};
use std::borrow::Cow;
use wc3::model::conversion::{self, Asset, ExportOptions, ImportOptions, IssueKind};
use wc3::model::semantic::profiles::wem::{
    self as authored, AppearanceLook, AppearanceSet, Family,
};
use wc3::model::semantic::profiles::Wem;
use wc3::model::semantic::{Clip, Controller, Curve, Material, Model, Node, Time, Values};
use wc3::model::wem::{records, Bool, RawDocument, ReferenceFlags};

fn encode<'view, 'data: 'view>(asset: &'view Asset<'data, Wem>) -> RawDocument<'view> {
    conversion::wem::plan(asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value
}
fn material(name: &str) -> Material<'_, Wem> {
    Material {
        name: Cow::Borrowed(name),
        surface: authored::Material {
            common: authored::materials::CommonSurface {
                blend: Default::default(),
                cull: Default::default(),
                depth_test: true,
                depth_write: true,
                depth_bias: 0.,
                alpha_test_threshold: 0.,
                priority: 0,
                flags: Default::default(),
                body: authored::materials::Body::Composite {
                    layers: Vec::new(),
                    diffuse: [1.; 4],
                    emissive: [0.; 4],
                    specular: [0.; 4],
                    specular_exponent: 0.,
                    environment: 0.,
                    simulate_roughness: false,
                },
                features: Vec::new(),
            },
            specialized: None,
            authority: authored::RepresentationAuthority::Unspecified,
        },
    }
}

#[test]
fn selecting_an_alternative_and_editing_preserves_other_bindings() {
    let mut builder = Model::<Wem>::builder("alternatives");
    let first = builder.add_material(material("first")).unwrap();
    let second = builder.add_material(material("second")).unwrap();
    let mut appearance = authored::Appearance::new();
    let slot = appearance.add_slot("surface");
    let mut set = AppearanceSet::new(Family::Common);
    set.materials = vec![first, second];
    let first_look = set.add_look(AppearanceLook::named("first look"));
    let second_look = set.add_look(AppearanceLook::named("second look"));
    set.set_default_look(Some(first_look)).unwrap();
    let set = appearance.add_set(set).unwrap();
    appearance.bind(set, first_look, slot, Some(first)).unwrap();
    appearance
        .bind(set, second_look, slot, Some(second))
        .unwrap();
    builder.set_appearance(appearance);
    let asset = Asset::new(builder.finish().unwrap());
    let source = encode(&asset);
    let mut imported = conversion::wem::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let model = &imported.models[0];
    let (set_id, set) = model.appearance().sets().next().unwrap();
    let alternatives = set.looks().map(|(id, _)| id).collect::<Vec<_>>();
    let slot = model.appearance().slots().next().unwrap().0;
    let second = set
        .look(alternatives[1])
        .unwrap()
        .bindings()
        .next()
        .unwrap()
        .1
        .unwrap();
    assert_eq!(model.material(second).unwrap().name, "second");
    assert_eq!(set.default_look(), Some(alternatives[0]));
    imported
        .edit_model(0, |builder| {
            let mut surface = builder.model().material(second).unwrap().clone();
            surface.name = "edited second".into();
            builder.set_material(second, surface)?;
            let mut appearance = builder.model().appearance().clone();
            appearance.bind(set_id, alternatives[1], slot, Some(second))?;
            builder.set_appearance(appearance);
            Ok(())
        })
        .unwrap();
    let saved = encode(&imported);
    let rebuilt = conversion::wem::import(&saved, &ImportOptions::default())
        .unwrap()
        .value;
    let model = &rebuilt.models[0];
    let (_, set) = model.appearance().sets().next().unwrap();
    let names = set
        .looks()
        .map(|(_, look)| {
            let material = look.bindings().next().unwrap().1.unwrap();
            model.material(material).unwrap().name.as_ref()
        })
        .collect::<Vec<_>>();
    assert_eq!(names, ["first", "edited second"]);
    assert_eq!(set.default_look(), Some(set.looks().next().unwrap().0));
}

#[test]
fn producer_property_selection_is_distinct_from_hierarchy_translation() {
    let mut builder = Model::<Wem>::builder("separate channel domains");
    let node = builder.add_node(Node::named("owner")).unwrap();
    let mut controls = authored::ModelControls::default();
    let emitter = controls.channels.add(Declaration {
        target: Target::Node {
            node,
            operation: NodeOperation::UnknownEmitter(0),
        },
        initial: Some([1., 2., 3.]),
    });
    let translation = controls.channels.add(Declaration {
        target: Target::Node {
            node,
            operation: NodeOperation::Translation,
        },
        initial: Some([4., 5., 6.]),
    });
    builder.set_controls(controls);
    let track = |channel| Track {
        channel,
        curve: Curve::linear(
            Values::owned(vec![Time::default()]),
            Values::owned(vec![[7., 8., 9.]]),
        )
        .unwrap(),
        auxiliary_tcb: Values::default(),
    };
    let program = builder
        .add_controller(Controller {
            controls: authored::ControllerControls {
                tracks: vec![
                    AnyTrack::Vector3(track(emitter)),
                    AnyTrack::Vector3(track(translation)),
                ],
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap();
    for name in ["first", "second"] {
        let mut clip = Clip::<Wem>::named(name);
        clip.controls.programs = vec![program];
        builder.add_clip(clip).unwrap();
    }
    let asset = Asset::new(builder.finish().unwrap());
    let source = encode(&asset);
    let imported = conversion::wem::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let model = &imported.models[0];
    let declarations = model.controls().channels.declarations().collect::<Vec<_>>();
    assert_eq!(declarations.len(), 2);
    assert!(declarations.iter().any(|declaration| matches!(
        declaration,
        AnyDeclaration::Vector3(
            _,
            Declaration {
                target: Target::Node {
                    operation: NodeOperation::UnknownEmitter(0),
                    ..
                },
                initial: Some(value)
            }
        ) if *value == [1., 2., 3.]
    )));
    assert!(declarations.iter().any(|declaration| matches!(
        declaration,
        AnyDeclaration::Vector3(
            _,
            Declaration {
                target: Target::Node {
                    operation: NodeOperation::Translation,
                    ..
                },
                initial: Some(value)
            }
        ) if *value == [4., 5., 6.]
    )));
    assert_eq!(
        model.nodes().next().unwrap().1.transform.translation,
        [0.; 3]
    );
    assert_eq!(model.controllers().len(), 1);
    let controller = model.controllers().next().unwrap();
    assert_eq!(controller.1.controls.tracks.len(), 2);
    assert!(controller.1.channels.is_empty());
    for (_, clip) in model.clips() {
        assert_eq!(clip.controls.programs, [controller.0]);
        assert!(clip.channels().is_empty());
    }
}

#[test]
fn common_enum_flags_and_clip_looping_survive_wem_edits() {
    let mut builder = Model::<Wem>::builder("flags");
    let mut surface = material("surface");
    surface.surface.common.blend = authored::materials::BlendMode::Unknown(0xffff);
    let mut flags = authored::materials::MaterialFlags::from_bits_retain(0x8000_0000);
    flags.set_unlit(true);
    surface.surface.common.flags = flags;
    let surface = builder.add_material(surface).unwrap();
    let mut appearance = authored::Appearance::new();
    let mut set = AppearanceSet::new(Family::Common);
    set.materials.push(surface);
    appearance.add_set(set).unwrap();
    builder.set_appearance(appearance);
    let mut clip = Clip::<Wem>::named("once").once();
    clip.controls.flags = ClipFlags::from_bits_retain(0x4000_0000);
    builder.add_clip(clip).unwrap();
    let asset = Asset::new(builder.finish().unwrap());
    let source = encode(&asset);
    let mut imported = conversion::wem::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let clip_id = imported.models[0].clips().next().unwrap().0;
    imported
        .edit_model(0, |builder| {
            let original = builder.model().clip(clip_id).unwrap();
            let mut clip = Clip::<Wem>::named("renamed")
                .with_duration(original.duration())
                .once();
            clip.controls = original.controls.clone();
            builder.set_clip(clip_id, clip)
        })
        .unwrap();
    let output = encode(&imported);
    let rebuilt = conversion::wem::import(&output, &ImportOptions::default())
        .unwrap()
        .value;
    let surface = rebuilt.models[0].materials().next().unwrap().1;
    assert_eq!(
        surface.surface.common.blend,
        authored::materials::BlendMode::Unknown(0xffff)
    );
    assert_eq!(surface.surface.common.flags, flags);
    let clip = rebuilt.models[0].clips().next().unwrap().1;
    assert_eq!(clip.name(), "renamed");
    assert!(!clip.is_looping());
    assert_eq!(clip.controls.flags.bits(), 0x4000_0000);
}

#[test]
fn noncanonical_looping_and_reference_qualifiers_have_explicit_diagnostics() {
    let mut builder = Model::<Wem>::builder("old");
    builder.add_clip(Clip::<Wem>::named("animation")).unwrap();
    let asset = Asset::new(builder.finish().unwrap());
    let mut source = encode(&asset);
    let model_section = source
        .sections()
        .find(|(_, s)| s.tag() == *b"MODL")
        .unwrap()
        .0;
    macro_rules! patch_model {
        ($dialect:ty) => {{
            let mut model = source
                .array::<records::ModelRecord<$dialect>>(model_section)
                .unwrap()
                .get(0)
                .unwrap()
                .unwrap();
            model.name.flags = ReferenceFlags::from_bits_retain(0x8000_0001);
            source.replace(model_section, &[model]).unwrap();
        }};
    }
    match source.section(model_section).unwrap().revision() {
        1 => patch_model!(records::V1),
        2 => patch_model!(records::V2),
        3 => patch_model!(records::V3),
        4 => patch_model!(records::V4),
        5 => patch_model!(records::V5),
        6 => patch_model!(records::V6),
        7 => patch_model!(records::V7),
        8 => patch_model!(records::V8),
        9 => patch_model!(records::V9),
        _ => panic!("unsupported output model revision"),
    }
    let clip_section = source
        .sections()
        .find(|(_, s)| s.tag() == *b"CLIP")
        .unwrap()
        .0;
    let revision = source.section(clip_section).unwrap().revision();
    macro_rules! patch_clip {
        ($dialect:ty) => {{
            let mut clip = source
                .array::<records::Clip<$dialect>>(clip_section)
                .unwrap()
                .get(0)
                .unwrap()
                .unwrap();
            clip.looping = Bool(17);
            clip.name.flags = ReferenceFlags::from_bits_retain(0x41);
            source.replace(clip_section, &[clip]).unwrap();
        }};
    }
    match revision {
        1 => patch_clip!(records::V1),
        2 => patch_clip!(records::V2),
        3 => patch_clip!(records::V3),
        4 => patch_clip!(records::V4),
        5 => patch_clip!(records::V5),
        6 => patch_clip!(records::V6),
        7 => patch_clip!(records::V7),
        _ => panic!("unsupported output clip revision"),
    }
    let imported = conversion::wem::import(&source, &ImportOptions::default()).unwrap();
    assert!(
        imported.report.issues.iter().any(|issue| {
            let description = issue.description.to_lowercase();
            issue.kind == IssueKind::Normalized
                && (description.contains("reference") || description.contains("qualifier"))
        }),
        "native reference qualifiers need explicit diagnostics"
    );
    assert!(
        imported.report.issues.iter().any(|issue| {
            let description = issue.description.to_lowercase();
            issue.kind == IssueKind::Normalized
                && (description.contains("bool") || description.contains("looping"))
        }),
        "noncanonical boolean encoding needs an explicit diagnostic"
    );
    assert!(imported.value.models[0]
        .clips()
        .next()
        .unwrap()
        .1
        .is_looping());
    let mut edited = imported.value;
    let clip_id = edited.models[0].clips().next().unwrap().0;
    edited
        .edit_model(0, |builder| {
            let original = builder.model().clip(clip_id).unwrap();
            let mut replacement =
                Clip::<Wem>::named("renamed animation").with_duration(original.duration());
            replacement.controls = original.controls.clone();
            builder.set_clip(clip_id, replacement)
        })
        .unwrap();
    let output = encode(&edited);
    let clip = output
        .sections()
        .find(|(_, s)| s.tag() == *b"CLIP")
        .unwrap()
        .0;
    macro_rules! looping {
        ($dialect:ty) => {
            output
                .array::<records::Clip<$dialect>>(clip)
                .unwrap()
                .get(0)
                .unwrap()
                .unwrap()
                .looping
        };
    }
    let looping = match output.section(clip).unwrap().revision() {
        1 => looping!(records::V1),
        2 => looping!(records::V2),
        3 => looping!(records::V3),
        4 => looping!(records::V4),
        5 => looping!(records::V5),
        6 => looping!(records::V6),
        7 => looping!(records::V7),
        _ => panic!("unsupported output clip revision"),
    };
    assert_eq!(looping, Bool::from(true));
}

#[test]
fn optional_model_conventions_survive_encoded_document_roundtrips() {
    for convention in [
        None,
        Some(authored::HierarchyConvention::PivotRelative),
        Some(authored::HierarchyConvention::ExplicitBind),
    ] {
        let mut builder = Model::<Wem>::builder("hierarchy convention");
        let mut controls = authored::ModelControls::default();
        controls.hierarchy_convention = convention;
        builder.set_controls(controls);
        builder.add_node(Node::named("root")).unwrap();
        let asset = Asset::new(builder.finish().unwrap());
        let document = encode(&asset);
        let bytes = document.write().unwrap();
        let decoded = RawDocument::read(&bytes).unwrap();
        let imported = conversion::wem::import(&decoded, &ImportOptions::default())
            .unwrap()
            .value;
        assert_eq!(
            imported.models[0].controls().hierarchy_convention,
            convention
        );
        let output = encode(&imported);
        let bytes = output.write().unwrap();
        let decoded = RawDocument::read(&bytes).unwrap();
        let restored = conversion::wem::import(&decoded, &ImportOptions::default())
            .unwrap()
            .value;
        assert_eq!(
            restored.models[0].controls().hierarchy_convention,
            convention
        );
        assert_eq!(restored.models[0].nodes().next().unwrap().1.name, "root");
    }
}
