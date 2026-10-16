use wc3::model::common::IntoZeroVec;
#[path = "mdlx/graph.rs"]
mod graph;
use std::borrow::Cow;
use wc3::model::{
    conversion::{self, ExportOptions, ImportOptions, UnknownPolicy},
    formats::mdlx::{self, mdl::Read as _, mdx::Write as _},
    io,
    profiles::mdlx as profile,
    Animated, Channel, Clock,
};
fn source() -> mdlx::Document<'static, mdlx::V800> {
    mdlx::Document::decode_mdl(include_str!("../../fixtures/mdl/quad_model.mdl")).unwrap()
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
    let asset = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;

    let output = io::mdlx::plan::<mdlx::V800>(&asset, &ExportOptions::default())
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
    assert!(output.geosets().next().unwrap().unselectable());
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
    let asset = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let output = io::mdlx::plan::<mdlx::V800>(&asset, &ExportOptions::default())
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
    let mut source = mdlx::Document::<mdlx::V1000>::new();
    source.set_textures([mdlx::materials::Texture::new("Shader.blp").unwrap()]);
    let mut material = mdlx::materials::Material::new();
    material.try_set_shader("Shader_HD_DefaultUnit").unwrap();
    material.layers.push(mdlx::materials::Layer::new());
    source.set_materials([material]);
    let asset = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let output = io::mdlx::plan::<mdlx::V1000>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output.materials().next().unwrap().try_shader().unwrap(),
        "Shader_HD_DefaultUnit"
    );
    assert!(io::mdlx::plan::<mdlx::V1800>(&asset, &ExportOptions::default()).is_err());
}

#[test]
fn empty_native_tables_do_not_replace_reserved_root() {
    let source = mdlx::Document::<mdlx::V800>::new();
    let asset = io::mdlx::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let output = io::mdlx::plan::<mdlx::V800>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.nodes().len(), 0);
    assert_eq!(output.materials().count(), 0);
}
#[test]
fn complete_native_surface_is_an_editable_logical_payload() {
    let source = source();
    let mut imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let (id, material) = graph.materials().iter().next().unwrap();
    let mut material = material.surface.clone();
    material.layers[0].opacity.default = 0.25;
    imported
        .value
        .edit(|builder| builder.set_material_surface(id, material))
        .unwrap();
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
fn complete_native_emitter_is_an_editable_logical_feature() {
    let mut source = source();
    let mut emitter =
        mdlx::emitters::ParticleEmitter2::new(mdlx::scene::Node::new("Emitter", 1).unwrap());
    emitter.emission_rate = mdlx::animation::Animatable::Static(10.);
    source.set_particle_emitters2([emitter]);
    source.set_pivot_points(&[[0.; 3], [0.; 3]]);
    let mut imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let (id, mut feature) = graph
        .features()
        .iter()
        .find(|(_, feature)| matches!(feature.payload, profile::Node::AtlasParticles(_)))
        .map(|(id, feature)| (id, feature.payload.clone()))
        .unwrap();
    let profile::Node::AtlasParticles(emitter) = &mut feature else {
        unreachable!()
    };
    emitter.emission.rate = Animated::constant(17.);
    imported
        .value
        .edit(|builder| builder.set_feature(id, feature))
        .unwrap();
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
fn unknown_blocks_are_rejected_or_dropped_before_authoring_edits() {
    let mut source = source();
    source.chunks.push(mdlx::chunks::ModelChunk::Unknown(
        mdlx::chunks::UnknownChunk::new(mdlx::chunks::RawChunk::new(
            *b"TEST",
            Cow::Owned(vec![1, 2, 3]),
        ))
        .unwrap(),
    ));
    assert!(io::mdlx::import(&source, &ImportOptions::default()).is_err());
    let mut imported = io::mdlx::import(
        &source,
        &ImportOptions {
            unknown: UnknownPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap();

    assert!(imported
        .report
        .issues
        .iter()
        .any(|issue| issue.path == "chunks" && issue.kind == conversion::IssueKind::Dropped));
    assert!(source
        .chunks
        .iter()
        .any(|chunk| matches!(chunk, mdlx::chunks::ModelChunk::Unknown(_))));
    let graph = &imported.value;
    let (id, mesh) = graph.meshes().iter().next().unwrap();
    let mut mesh = mesh.clone();
    mesh.geometry = mesh
        .geometry
        .clone()
        .with_positions(
            (mesh
                .positions()
                .iter()
                .map(|position| position.map(|value| value + 1.))
                .collect::<Vec<_>>())
            .into_zero_vec(),
        )
        .unwrap();
    imported
        .value
        .edit(|builder| builder.set_mesh(id, mesh))
        .unwrap();
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
fn spline_keys_materialize_once_and_remain_editable() {
    let mut source = source();
    source.bones_mut().next().unwrap().node.translation = Some(
        mdlx::animation::Track::hermite(
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
        .unwrap(),
    );
    source.set_global_sequences(&[2000]);
    let mut imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let Channel::Vector3(mut track) = graph.shared_channels()[0].clone() else {
        panic!("translation curve missing")
    };
    assert!(matches!(track.clock, Clock::Shared(_)));
    assert!(track.curve.values().is_owned());
    let mut values = track.curve.values().to_vec();
    values[1] = [3.; 3];
    track.curve = wc3::model::Curve::hermite(
        track.curve.times().to_vec(),
        (values).into_zero_vec(),
        track.curve.hermite_derivatives().unwrap().incoming.clone(),
        track.curve.hermite_derivatives().unwrap().outgoing.clone(),
    )
    .unwrap();
    imported
        .value
        .edit(|builder| builder.set_shared_channel(0, Channel::Vector3(track)))
        .unwrap();
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output
            .bones()
            .next()
            .unwrap()
            .node
            .translation
            .as_ref()
            .unwrap()
            .hermite_keys()
            .unwrap()
            .get(1)
            .unwrap()
            .value,
        [3.; 3]
    );
    assert_eq!(output.global_sequences(), [2000]);
}
#[test]
fn ambient_illumination_and_nonlooping_clips_preserve_behavior() {
    let mut source = source();
    source.set_lights([mdlx::scene::Light::new(
        mdlx::scene::Node::new("Ambient", 9).unwrap(),
        mdlx::scene::LightType::Ambient,
    )]);
    source
        .sequences_mut()
        .next()
        .unwrap()
        .flags
        .set_non_looping(true);
    let imported = io::mdlx::import(&source, &ImportOptions::default()).unwrap();
    assert!(!imported.value.clips().iter().next().unwrap().1.is_looping());
    let output = io::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        output.lights().next().unwrap().light_type,
        mdlx::scene::LightType::Ambient
    );
    assert!(output.sequences().next().unwrap().flags.non_looping());
}

#[test]
fn owned_import_survives_native_document_and_input_drop() {
    let imported = {
        let input = include_str!("../../fixtures/mdl/quad_model.mdl").to_owned();
        let document = mdlx::Document::<mdlx::V800>::decode_mdl(&input).unwrap();
        io::mdlx::import_owned(&document, &ImportOptions::default())
            .unwrap()
            .value
    };
    assert_eq!(imported.name(), "Quad");
    let positions = imported.meshes().iter().next().unwrap().1.positions();
    assert_eq!(positions.len(), 4);
    assert!(positions.is_owned());
    let output = io::mdlx::plan::<mdlx::V800>(&imported, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.geosets().next().unwrap().vertices().len(), 4);
    assert_eq!(
        output.textures().next().unwrap().path.text(),
        "Textures\\Quad.blp"
    );
}

#[test]
fn borrowed_import_retains_native_vertex_storage() {
    let document = source();
    let source_positions = document.geosets().next().unwrap().vertices();
    let imported = io::mdlx::import(&document, &ImportOptions::default())
        .unwrap()
        .value;
    let positions = imported.meshes().iter().next().unwrap().1.positions();
    assert!(!positions.is_owned());
    assert_eq!(
        positions.as_bytes().as_ptr(),
        source_positions.as_bytes().as_ptr()
    );
}

#[test]
fn direct_decoders_produce_independent_semantic_models() {
    let text = include_str!("../../fixtures/mdl/quad_model.mdl");
    let native = mdlx::Document::<mdlx::V800>::decode_mdl(text).unwrap();
    let bytes = native.encode_mdx().unwrap();
    let from_text = io::mdlx::decode_mdl(text, &ImportOptions::default())
        .unwrap()
        .value;
    let from_bytes = io::mdlx::decode_mdx(&bytes, &ImportOptions::default())
        .unwrap()
        .value;
    drop(bytes);
    drop(native);
    assert_eq!(from_text.name(), from_bytes.name());
    assert_eq!(
        from_text.meshes().iter().next().unwrap().1.positions(),
        from_bytes.meshes().iter().next().unwrap().1.positions()
    );
    assert!(from_text
        .meshes()
        .iter()
        .next()
        .unwrap()
        .1
        .positions()
        .is_owned());
    assert!(from_bytes
        .meshes()
        .iter()
        .next()
        .unwrap()
        .1
        .positions()
        .is_owned());
}

#[test]
fn semantic_export_restores_native_node_kinds_for_mdl() {
    use mdlx::mdl::Write as _;
    let text = format!(
        "{}\n{}\n{}\n{}",
        include_str!("../../fixtures/mdl/quad_model.mdl")
            .replace("FormatVersion 800", "FormatVersion 1100"),
        r#"
        Helper "Helper" { ObjectId 10, }
        Light "Light" { ObjectId 11, Ambient, }
        Attachment "Attachment" { ObjectId 12, }
        ParticleEmitter "Particle" { ObjectId 13, }
        RibbonEmitter "Ribbon" { ObjectId 14, Rows 1, Columns 1, MaterialID 0, }
        EventObject "Event" { ObjectId 15, EventTrack 1 { 10, } }
        CollisionShape "Collision" { ObjectId 16, Sphere, Vertices 1 { { 0.0, 0.0, 0.0 }, } BoundsRadius 1.0, }
        "#,
        include_str!("../../fixtures/mdl/smoke.mdl"),
        include_str!("../../fixtures/mdl/popcorn_fire.mdl"),
    );
    let source = mdlx::Document::<mdlx::V1100>::decode_mdl(&text).unwrap();
    let semantic = io::mdlx::import_owned(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let output = io::mdlx::plan::<mdlx::V1100>(&semantic, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let encoded = output.encode_mdl().unwrap();
    let decoded = mdlx::Document::<mdlx::V1100>::decode_mdl(&encoded).unwrap();
    assert_eq!(decoded.bones().count(), source.bones().count());
    assert!(decoded.helpers().count() >= source.helpers().count());
    assert_eq!(decoded.lights().count(), 1);
    assert_eq!(decoded.attachments().count(), 1);
    assert_eq!(decoded.particle_emitters().count(), 1);
    assert_eq!(decoded.particle_emitters2().count(), 1);
    assert_eq!(decoded.try_popcorn_emitters().unwrap().count(), 1);
    assert_eq!(decoded.ribbon_emitters().count(), 1);
    assert_eq!(decoded.event_objects().count(), 1);
    assert_eq!(decoded.collision_shapes().count(), 1);
}
