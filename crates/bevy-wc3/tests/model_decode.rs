use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::{Assets, Mesh};
use bevy_wc3::compile_model;
use wc3::model::{NodeFeature, ParticleDetails, ParticleEmitter};

#[path = "../examples/support/model.rs"]
mod semantic;

#[test]
fn classic_source_compiles_after_encoded_storage_is_released() {
    let bytes = include_bytes!("fixtures/quad_model.mdx").to_vec();
    let model = semantic::decode(&bytes).unwrap();
    drop(bytes);
    assert_eq!(model.meshes().count(), 1);
    let compiled = compile_model(&model, |_| None).unwrap();
    drop(model);
    let mut meshes = Assets::<Mesh>::default();
    let prepared = compiled.prepare(
        &mut meshes,
        &mut Assets::<SkinnedMeshInverseBindposes>::default(),
    );
    assert!(!meshes.is_empty());
    assert!(prepared.rest_bounds().is_some());
}

#[test]
fn mdl_and_mdx_compile_equivalent_geometry_and_clocks() {
    let text = include_str!("fixtures/quad_model.mdl");
    let binary = include_bytes!("fixtures/quad_model.mdx");
    let models = [
        semantic::decode(text.as_bytes()).unwrap(),
        semantic::decode(binary).unwrap(),
        semantic::decode(format!("\u{feff}{text}").as_bytes()).unwrap(),
    ];
    let mut results = Vec::new();
    for model in models {
        let compiled = compile_model(&model, |_| None).unwrap();
        let mut meshes = Assets::<Mesh>::default();
        let prepared = compiled.prepare(
            &mut meshes,
            &mut Assets::<SkinnedMeshInverseBindposes>::default(),
        );
        let vertices: usize = meshes.iter().map(|(_, mesh)| mesh.count_vertices()).sum();
        results.push((
            compiled
                .clips()
                .iter()
                .map(|(_, name)| name.clone())
                .collect::<Vec<_>>(),
            prepared
                .rest_bounds()
                .map(|bounds| (bounds.center, bounds.half_extents)),
            meshes.len(),
            vertices,
        ));
    }
    assert_eq!(results[0], results[1]);
    assert_eq!(results[0], results[2]);
}

#[test]
fn malformed_sources_fail_before_compilation() {
    assert!(semantic::decode(b"Version { FormatVersion").is_err());
    assert!(semantic::decode(&[0xff]).is_err());
    // Native parsing alone permits this fixture's absent materials; semantic import rejects it.
    assert!(semantic::decode(include_bytes!("fixtures/attachment_parent.mdl")).is_err());
}

#[test]
fn particle_fixture_preserves_distinct_native_emitter_definitions() {
    let model = semantic::decode(include_bytes!("fixtures/particle_capture.mdl")).unwrap();
    let emitters: Vec<_> = model
        .features()
        .filter_map(|(id, feature)| match &feature.payload {
            NodeFeature::Particle(ParticleEmitter::Simulated(value))
                if matches!(value.details, ParticleDetails::Atlas(_)) =>
            {
                Some(id)
            }
            _ => None,
        })
        .collect();
    assert_eq!(emitters.len(), 2);
    let compiled = compile_model(&model, |_| None).unwrap();
    let prepared = compiled.prepare(
        &mut Assets::<Mesh>::default(),
        &mut Assets::<SkinnedMeshInverseBindposes>::default(),
    );
    assert_eq!(
        prepared
            .features()
            .iter()
            .filter(|(feature, _)| emitters.contains(&feature.id))
            .count(),
        2
    );
}

#[test]
fn semantic_compilation_retains_feature_identities_across_fixture_families() {
    for text in [
        include_bytes!("fixtures/cameras.mdl").as_slice(),
        include_bytes!("fixtures/events.mdl").as_slice(),
        include_bytes!("fixtures/light_capture.mdl").as_slice(),
        include_bytes!("fixtures/ribbon_capture.mdl").as_slice(),
        include_bytes!("fixtures/prem_capture.mdl").as_slice(),
        include_bytes!("fixtures/hd_capture.mdl").as_slice(),
    ] {
        let model = semantic::decode(text).unwrap();
        let expected: Vec<_> = model
            .features()
            .map(|(id, feature)| (id, feature.node))
            .collect();
        let compiled = compile_model(&model, |_| None).unwrap();
        let prepared = compiled.prepare(
            &mut Assets::<Mesh>::default(),
            &mut Assets::<SkinnedMeshInverseBindposes>::default(),
        );
        for (id, node) in expected {
            assert!(prepared
                .features()
                .iter()
                .any(|(feature, _)| feature.id == id && feature.node == node));
        }
    }
}
