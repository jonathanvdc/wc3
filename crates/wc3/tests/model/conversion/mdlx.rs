use wc3::model::common::IntoZeroVec;
#[path = "mdlx/graph.rs"]
mod graph;
use std::borrow::Cow;
use wc3::model::{
    conversion::{self, ExportOptions, ImportOptions, UnknownPolicy},
    mdlx::{self, mdl::Read as _},
    semantic::{profiles::mdlx as profile, Animated, Channel, Clock},
};
fn source() -> mdlx::Model<'static, mdlx::V800> {
    mdlx::Model::decode_mdl(include_str!("../../fixtures/mdl/quad_model.mdl")).unwrap()
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
fn complete_native_surface_is_an_editable_logical_payload() {
    let source = source();
    let mut imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let (id, material) = graph.materials().next().unwrap();
    let mut material = material.surface.clone();
    material.layers[0].opacity.default = 0.25;
    imported
        .value
        .edit(|builder| builder.set_material_surface(id, material))
        .unwrap();
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
    let mut imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let (id, mut feature) = graph
        .features()
        .find(|(_, feature)| matches!(feature.payload, profile::Node::Particle2(_)))
        .map(|(id, feature)| (id, feature.payload.clone()))
        .unwrap();
    let profile::Node::Particle2(emitter) = &mut feature else {
        unreachable!()
    };
    emitter.emission_rate = Animated::constant(17.);
    imported
        .value
        .edit(|builder| builder.set_feature(id, feature))
        .unwrap();
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
    assert!(conversion::mdlx::import(&source, &ImportOptions::default()).is_err());
    let mut imported = conversion::mdlx::import(
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
    let (id, mesh) = graph.meshes().next().unwrap();
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
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
    let mut imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    let graph = &imported.value;
    let Channel::Vector3(mut track) = graph.shared_channels()[0].clone() else {
        panic!("translation curve missing")
    };
    assert!(matches!(track.clock, Clock::Shared(_)));
    assert!(track.curve.values().is_owned());
    let mut values = track.curve.values().to_vec();
    values[1] = [3.; 3];
    track.curve = wc3::model::semantic::Curve::hermite(
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
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
    let imported = conversion::mdlx::import(&source, &ImportOptions::default()).unwrap();
    assert!(!imported.value.clips().next().unwrap().1.is_looping());
    let output = conversion::mdlx::plan::<mdlx::V800>(&imported.value, &ExportOptions::default())
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
