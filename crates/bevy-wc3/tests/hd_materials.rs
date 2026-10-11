use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::{Assets, Mesh};
use bevy_wc3::{prepare_model, Wc3Model};
use wc3::model::ZeroVec;

#[test]
fn preparation_shares_identical_uv_meshes_and_preserves_distinct_tangent_sources() {
    let mut source = Wc3Model::decode_mdl(include_str!("fixtures/hd_capture.mdl")).unwrap();
    let mut geoset = source
        .model
        .geosets()
        .cloned()
        .collect::<Vec<_>>()
        .remove(0);
    let uv = geoset.uv_sets()[0].to_vec();
    let mut near = uv.clone();
    near[0][0] += 0.000001;
    geoset.set_uv_sets(
        ([uv.clone(), uv, near])
            .iter()
            .map(|set| ZeroVec::alloc_from_slice(set))
            .collect(),
    );
    source.model.set_geosets(&[geoset.clone()]);
    let mut material = source
        .model
        .materials()
        .cloned()
        .collect::<Vec<_>>()
        .remove(0);
    let layer = material.layers[0].clone();
    material.layers = (0..3)
        .map(|coordinate| {
            let mut layer = layer.clone();
            layer.coordinate_id = coordinate;
            layer
        })
        .collect();
    source.model.set_materials(&[material]);
    let mut meshes = Assets::<Mesh>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let prepared = prepare_model(&mut meshes, &mut binds, &source, |_| None).unwrap();
    assert_eq!(
        meshes.len(),
        2,
        "equal UV sets must share geometry; near-equal sets must remain distinct"
    );
    drop(prepared);

    let authored = vec![[1.0, 0.0, 0.0, 1.0]; geoset.vertices().len()];
    geoset.set_tangents(Some(ZeroVec::alloc_from_slice(&authored)));
    source.model.set_geosets(&[geoset]);
    let mut meshes = Assets::<Mesh>::default();
    let _prepared = prepare_model(&mut meshes, &mut binds, &source, |_| None).unwrap();
    assert_eq!(
        meshes.len(),
        3,
        "authored UV0 tangents must not be replaced by generated tangents"
    );
}
