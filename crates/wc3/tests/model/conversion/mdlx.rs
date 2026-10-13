use std::borrow::Cow;
use wc3::model::{
    conversion::{self, Asset, ExportOptions, ImportOptions, UnknownPolicy},
    mdlx::{self, mdl::Read as _},
    wem::{self, records as r, semantic},
};

fn source() -> mdlx::Model<'static, mdlx::V800> {
    mdlx::Model::decode_mdl(include_str!("../../fixtures/mdl/quad_model.mdl")).unwrap()
}
#[test]
fn untouched_round_trip_retains_native_model() {
    let source = source();
    let imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let model = imported.value.model(&ExportOptions::default()).unwrap();
    assert_eq!(model.meshes[0].positions.len(), 4);
    assert_eq!(model.meshes[0].triangles.as_ref(), [[0, 1, 2], [0, 2, 3]]);
    assert_eq!(model.nodes[0].name, "Bone_Root");
    assert_eq!(model.meshes[0].influences[0][0].node, 0);
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap();
    assert_eq!(
        source.geosets().next().unwrap().vertices(),
        output.value.geosets().next().unwrap().vertices()
    );
    assert_eq!(
        source.sequences().next().unwrap().interval,
        output.value.sequences().next().unwrap().interval
    );
}
#[test]
fn editable_geometry_exports_without_source_archive() {
    let imported = conversion::mdlx::import(&source(), &ImportOptions::default()).unwrap();
    let mut model = imported.value.model(&ExportOptions::default()).unwrap();
    model.meshes[0].positions.to_mut()[0] = [1., 2., 3.];
    let fresh = Asset::new(semantic::build(&model).unwrap());
    let output = conversion::mdlx::plan::<mdlx::V800>(&fresh, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output.geosets().next().unwrap().vertices().get(0),
        Some([1., 2., 3.])
    );
    assert_eq!(
        output.textures().next().unwrap().path.text(),
        "Textures\\Quad.blp"
    );
}
#[test]
fn spline_and_global_clock_are_editable() {
    let mut source = source();
    let track = mdlx::animation::Track::hermite(
        [
            mdlx::animation::TangentKeyframe {
                frame: 0,
                value: [0.; 3],
                in_tangent: [0.; 3],
                out_tangent: [1.; 3],
            },
            mdlx::animation::TangentKeyframe {
                frame: 1000,
                value: [2.; 3],
                in_tangent: [1.; 3],
                out_tangent: [2.; 3],
            },
        ],
        Some(0),
    )
    .unwrap();
    source.bones_mut().next().unwrap().node.translation = Some(track);
    source.set_global_sequences(&[2000]);
    let imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let mut model = imported.value.model(&ExportOptions::default()).unwrap();
    assert_eq!(model.clips[0].tracks[0].global_duration, Some(2.));
    model.clips[0].tracks[0].keys[1].value = vec![3.; 3];
    let fresh = Asset::new(semantic::build(&model).unwrap());
    let output = conversion::mdlx::plan::<mdlx::V800>(&fresh, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let track = output
        .bones()
        .next()
        .unwrap()
        .node
        .translation
        .as_ref()
        .unwrap();
    assert_eq!(track.hermite_keys().unwrap().get(1).unwrap().value, [3.; 3]);
    assert_eq!(output.global_sequences(), vec![2000]);
}
#[test]
fn native_material_is_active_wem_record() {
    let imported = conversion::mdlx::import(&source(), &ImportOptions::default()).unwrap();
    let mut raw = imported.value.document.into_raw();
    let id = raw
        .sections()
        .find(|(_, section)| section.tag() == *b"NMDX")
        .map(|(id, _)| id)
        .unwrap();
    let mut material = raw
        .array::<r::MdxMaterial>(id)
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    material.layers.0[0].alpha = 0.25;
    raw.replace(id, &[material]).unwrap();
    let asset = Asset::new(wem::Document::from_raw(raw).unwrap());
    let output = conversion::mdlx::plan::<mdlx::V800>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output.materials().next().unwrap().layers[0].alpha.value(),
        Some(&0.25)
    );
}
#[test]
fn native_emitter_fields_round_trip_after_raw_edit() {
    let mut source = source();
    let node = mdlx::scene::Node::new("Emitter", 1).unwrap();
    let mut emitter = mdlx::emitters::ParticleEmitter2::new(node);
    emitter.emission_rate = mdlx::animation::Animatable::Static(10.);
    source.set_particle_emitters2([emitter]);
    source.set_pivot_points(&[[0.; 3], [0.; 3]]);
    let mut imported = conversion::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let root = imported
        .document
        .root::<r::DocumentRecord<r::V1>>()
        .unwrap();
    let raw = imported.document.raw();
    let model = raw
        .resolve::<r::ModelRecord<r::V9>>(root.models.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let node_id = raw.section_id(model.nodes.nodes.index).unwrap();
    let mut nodes = raw
        .array::<r::Node<r::V19>>(node_id)
        .unwrap()
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    if let r::NodePayload::Wc3ParticleEmitter2(payload) = &mut nodes[1].payload {
        payload.emission_rate = 17.;
    } else {
        panic!("native emitter payload missing");
    }
    let mut raw = imported.document.into_raw();
    raw.replace(node_id, &nodes).unwrap();
    imported.document = wem::Document::from_raw(raw).unwrap();
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output
            .particle_emitters2()
            .next()
            .unwrap()
            .emission_rate
            .value(),
        Some(&17.)
    );
}
#[test]
fn unknown_chunks_require_policy_and_stay_inactive_after_edits() {
    let mut source = source();
    source.chunks.push(mdlx::chunks::ModelChunk::Unknown(
        mdlx::chunks::UnknownChunk::new(mdlx::chunks::RawChunk::new(
            *b"TEST",
            Cow::Owned(vec![1, 2, 3]),
        ))
        .unwrap(),
    ));
    assert!(conversion::mdlx::import(&source, &ImportOptions::default()).is_err());
    let options = ImportOptions {
        unknown: UnknownPolicy::Archive,
        ..Default::default()
    };
    let mut asset = conversion::mdlx::import(&source, &options).unwrap().value;
    let mut model = asset.model(&ExportOptions::default()).unwrap();
    model.name = Cow::Borrowed("Changed");
    asset.document = semantic::build(&model).unwrap();
    let options = ExportOptions {
        unknown: UnknownPolicy::Archive,
        ..Default::default()
    };
    let output = conversion::mdlx::plan::<mdlx::V800>(&asset, &options)
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert!(output
        .chunks
        .iter()
        .all(|chunk| !matches!(chunk, mdlx::chunks::ModelChunk::Unknown(_))));
}
#[test]
fn native_model_sequence_and_geoset_metadata_rebuilds_without_source_bytes() {
    let mut source = source();
    let mut info = source.model_info().unwrap();
    info.blend_time = 123;
    info.animation_file_name
        .set_text("Units\\Pose.mdx")
        .unwrap();
    source.set_model_info(&info);
    let sequence = source.sequences_mut().next().unwrap();
    sequence.move_speed = 7.;
    sequence.rarity = 0.125;
    sequence.sync_point = 42;
    source
        .geosets_mut()
        .next()
        .unwrap()
        .set_raw_unselectable(0xa5);
    let asset = conversion::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert!(asset.opaque.is_empty());
    let output = conversion::mdlx::plan::<mdlx::V800>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.model_info().unwrap().blend_time, 123);
    assert_eq!(
        output.model_info().unwrap().animation_file_name.text(),
        "Units\\Pose.mdx"
    );
    let sequence = output.sequences().next().unwrap();
    assert_eq!(sequence.move_speed, 7.);
    assert_eq!(sequence.rarity, 0.125);
    assert_eq!(sequence.sync_point, 42);
    assert_eq!(output.geosets().next().unwrap().raw_unselectable(), 0xa5);
}
#[test]
fn emitter_animation_is_active_and_not_dependent_on_source_bytes() {
    let mut source = source();
    let mut emitter =
        mdlx::emitters::ParticleEmitter2::new(mdlx::scene::Node::new("Emitter", 1).unwrap());
    emitter.emission_rate = mdlx::animation::Animatable::Both {
        value: 5.,
        track: mdlx::animation::Track::linear(
            [
                mdlx::animation::ValueKeyframe {
                    frame: 0,
                    value: 3.,
                },
                mdlx::animation::ValueKeyframe {
                    frame: 1000,
                    value: 10.,
                },
            ],
            None,
        )
        .unwrap(),
    };
    source.set_particle_emitters2([emitter]);
    source.set_pivot_points(&[[0.; 3], [0.; 3]]);
    let asset = conversion::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let output = conversion::mdlx::plan::<mdlx::V800>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let emitter = output.particle_emitters2().next().unwrap();
    assert_eq!(emitter.emission_rate.value(), Some(&5.));
    assert_eq!(
        emitter.emission_rate.track().unwrap().evaluate(500.),
        Some(6.5)
    );
}
#[test]
fn older_native_shader_string_uses_selected_dialect_layout() {
    let mut source = mdlx::Model::<mdlx::V1000>::new();
    source.set_textures([mdlx::materials::Texture::new("Shader.blp").unwrap()]);
    let mut material = mdlx::materials::Material::new();
    material.try_set_shader("Shader_HD_DefaultUnit").unwrap();
    material.layers.push(mdlx::materials::Layer::new());
    source.set_materials([material]);
    let asset = conversion::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let output = conversion::mdlx::plan::<mdlx::V1000>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output.materials().next().unwrap().try_shader().unwrap(),
        "Shader_HD_DefaultUnit"
    );
    assert!(conversion::mdlx::plan::<mdlx::V1800>(&asset, &ExportOptions::default()).is_err());
}
#[test]
fn empty_native_tables_do_not_replace_reserved_root() {
    let source = mdlx::Model::<mdlx::V800>::new();
    let asset = conversion::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let output = conversion::mdlx::plan::<mdlx::V800>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.nodes().len(), 0);
    assert_eq!(output.materials().count(), 0);
}
#[test]
fn integer_texture_selectors_remain_exact_above_f32_precision() {
    let mut source = source();
    let mut node = mdlx::scene::Node::new("Ribbon", 1).unwrap();
    node.parent_id = 0;
    let mut ribbon = mdlx::emitters::RibbonEmitter::new(node);
    ribbon.rows = 1;
    ribbon.columns = 1;
    ribbon.texture_slot.set_track(
        mdlx::animation::Track::step(
            [mdlx::animation::ValueKeyframe {
                frame: 0,
                value: 16_777_217,
            }],
            None,
        )
        .unwrap(),
    );
    source.set_ribbon_emitters([ribbon]);
    source.set_pivot_points(&[[0.; 3], [0.; 3]]);
    let asset = conversion::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let semantic = asset.model(&ExportOptions::default()).unwrap();
    let track = semantic
        .clips
        .iter()
        .flat_map(|clip| &clip.integer_tracks)
        .find_map(|track| match track {
            semantic::IntegerTrack::Unsigned(track) => Some(track),
            _ => None,
        })
        .unwrap();
    assert_eq!(track.keys[0].value, [16_777_217]);
    let root = asset.document.root::<r::DocumentRecord<r::V1>>().unwrap();
    let record = asset
        .document
        .raw()
        .resolve::<r::ModelRecord<r::V9>>(root.models.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let nodes = asset
        .document
        .raw()
        .resolve::<r::Node<r::V19>>(record.nodes.nodes.cast())
        .unwrap()
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(
        matches!(nodes[1].payload, r::NodePayload::Wc3RibbonEmitter(_)),
        "{:?}",
        nodes[1].payload
    );
    let output = conversion::mdlx::plan::<mdlx::V800>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output
            .ribbon_emitters()
            .next()
            .unwrap()
            .texture_slot
            .track()
            .unwrap()
            .step_keys()
            .unwrap()
            .get(0)
            .unwrap()
            .value,
        16_777_217
    );
}

#[test]
fn ambient_lights_and_sequence_looping_keep_their_semantic_meanings() {
    use mdlx::scene::{Light, LightType, Node};
    let mut source = source();
    source.set_lights(vec![Light::new(
        Node::new("Ambient", 9).unwrap(),
        LightType::Ambient,
    )]);
    let mut sequences = source.sequences().cloned().collect::<Vec<_>>();
    sequences[0].flags.set_non_looping(true);
    source.set_sequences(sequences);
    let imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let common = imported.value.model(&ExportOptions::default()).unwrap();
    assert!(!common.clips[0].looping);
    assert!(common.clips[0].flags.is_empty());
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output.lights().next().unwrap().light_type,
        LightType::Ambient
    );
    assert!(output.sequences().next().unwrap().flags.non_looping());
}

#[test]
fn unknown_native_blend_codes_do_not_become_known_common_operations() {
    for code in [7, 8, 9] {
        let mut source = source();
        source.materials_mut().next().unwrap().layers[0].filter_mode =
            mdlx::materials::LayerFilterMode::Unknown(code);
        let imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
        let mut model = imported.value.model(&ExportOptions::default()).unwrap();
        model.materials[0].flags.set_unlit(true);
        let edited = Asset::new(semantic::build(&model).unwrap());
        assert_eq!(model.materials[0].blend, r::BlendMode::Opaque);
        let output = conversion::mdlx::plan::<mdlx::V800>(&edited, &ExportOptions::default())
            .unwrap()
            .execute()
            .unwrap()
            .value;
        assert_eq!(
            output.materials().next().unwrap().layers[0].filter_mode,
            mdlx::materials::LayerFilterMode::Unknown(code)
        );
    }
}
