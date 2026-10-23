use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::{Assets, Mesh};
use bevy_wc3::compile_model;
use wc3::model::{
    conversion::ImportOptions,
    formats::mdlx::{mdl::Read as _, Document, V1100},
    io,
    profiles::Universal,
};

#[test]
fn preparation_shares_identical_uv_meshes_and_preserves_distinct_tangent_sources() {
    let mut source =
        Document::<V1100>::decode_mdl(include_str!("fixtures/hd_capture.mdl")).unwrap();
    let first = source.geosets().next().unwrap().clone();
    source.set_geosets([first]);
    let animation = source.geoset_animations().next().unwrap().clone();
    source.set_geoset_animations([animation]);
    let geoset = source.geosets_mut().next().unwrap();
    let uv = geoset.uv_sets()[0].clone();
    let mut coordinate = uv.get(0).unwrap();
    coordinate[0] += 0.000001;
    geoset.set_uv_sets(vec![uv.clone(), uv.clone(), uv]);
    geoset.set_uv(2, 0, coordinate).unwrap();
    let material = source.materials_mut().next().unwrap();
    let layer = material.layers[0].clone();
    material.layers = (0..3)
        .map(|coordinate| {
            let mut layer = layer.clone();
            layer.coordinate_id = coordinate;
            layer
        })
        .collect();
    let mut meshes = Assets::<Mesh>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let model = io::mdlx::import_owned(&source, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let compiled = compile_model(&model, |_| None).unwrap();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    assert_eq!(
        meshes.len(),
        2,
        "equal UV sets must share geometry; near-equal sets must remain distinct"
    );
    drop(prepared);

    let geoset = source.geosets_mut().next().unwrap();
    geoset.set_tangents(Some(
        (0..geoset.vertices().len())
            .map(|_| [1.0, 0.0, 0.0, 1.0])
            .collect(),
    ));
    let mut meshes = Assets::<Mesh>::default();
    let model = io::mdlx::import_owned(&source, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let compiled = compile_model(&model, |_| None).unwrap();
    let _prepared = compiled.prepare(&mut meshes, &mut binds);
    assert_eq!(
        meshes.len(),
        3,
        "authored UV0 tangents must not be replaced by generated tangents"
    );
}

#[test]
fn authored_default_unit_occlusion_requests_uv1_only_when_stream_and_role_exist() {
    for has_uv1 in [false, true] {
        for has_orm in [false, true] {
            let mut source =
                Document::<V1100>::decode_mdl(include_str!("fixtures/hd_capture.mdl")).unwrap();
            for geoset in source.geosets_mut() {
                let uv = geoset.uv_sets()[0].clone();
                geoset.set_uv_sets(if has_uv1 {
                    vec![uv.clone(), uv]
                } else {
                    vec![uv]
                });
            }
            let layer = &mut source.materials_mut().next().unwrap().layers[0];
            layer.shading_flags.set_ambient_occlusion(true);
            if !has_orm {
                let slots: Vec<_> = layer
                    .texture_slots()
                    .iter()
                    .filter(|slot| slot.texture_type != 2)
                    .cloned()
                    .collect();
                layer.set_texture_slots(&slots);
            }
            let model = io::mdlx::import_owned(&source, &ImportOptions::default())
                .unwrap()
                .value
                .widen::<Universal>();
            let compiled = compile_model(&model, |_| None)
                .expect("missing optional AO UV1/ORM must not fail geometry compilation");
            let mut meshes = Assets::<Mesh>::default();
            let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
            let _prepared = compiled.prepare(&mut meshes, &mut binds);
            assert!(!meshes.is_empty());
            for (_, mesh) in meshes.iter() {
                assert_eq!(
                    mesh.contains_attribute(Mesh::ATTRIBUTE_UV_1),
                    has_uv1 && has_orm
                );
            }
        }
    }
}
