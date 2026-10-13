use std::borrow::Cow;
use wc3::model::conversion::{ExportOptions, ImportOptions};
use wc3::model::{
    m3::{self as native, records as nr, Ref},
    wem::{self, records as wr, semantic as sem},
};
fn geometry() -> wem::Document<'static> {
    sem::build(&sem::Model {
        profile: wr::ProfileId::Sc2,
        meshes: vec![sem::Mesh {
            name: Cow::Borrowed("LOD 0 region 0"),
            positions: Cow::Owned(vec![[0.; 3], [1., 0., 0.], [0., 1., 0.]]),
            triangles: Cow::Owned(vec![[0, 1, 2]]),
            material_slot: 0,
            ..Default::default()
        }],
        materials: vec![sem::Material::default()],
        ..Default::default()
    })
    .unwrap()
}
#[test]
fn public_adapter_rebuilds_edited_cloth_density() {
    use wc3::model::conversion::{self, ApproximationPolicy, Asset};
    let asset = Asset::new(geometry());
    let options = ExportOptions {
        approximation: ApproximationPolicy::Allow { tolerance: 0.01 },
        ..Default::default()
    };
    let native = conversion::m3::plan::<native::StarCraft2>(&asset, &options)
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root_id = native.model.root_id();
    let mut doc = native.model.into_document();
    let mut root = doc
        .resolve::<nr::ModelRecord<nr::V29>>(Ref::from_section(1, root_id.index()))
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let enabled = doc.append(&[nr::Byte { value: 1 }; 3]).unwrap();
    let cloth = doc
        .append(&[nr::ClothPhysics::<nr::V4> {
            density: 3.,
            sim_enabled: Ref::from_section(3, enabled.index()),
            active: nr::AnimationValue {
                initial_value: 1,
                ..Default::default()
            },
            ..Default::default()
        }])
        .unwrap();
    root.cloth_physics = Ref::from_section(1, cloth.index());
    doc.replace(root_id, &[root]).unwrap();
    let source = native::Bundle::from_model(native::Model::from_document(doc).unwrap());
    let imported = conversion::m3::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let root = imported
        .document
        .root::<wr::DocumentRecord<wr::V1>>()
        .unwrap();
    let mut raw = imported.document.into_raw();
    let graph = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let cloth_id = raw.section_id(graph.physics.cloths.index).unwrap();
    let mut rows = raw
        .array::<wr::Cloth<wr::V2>>(cloth_id)
        .unwrap()
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    rows[0].density = 7.;
    raw.replace(cloth_id, &rows).unwrap();
    let asset = Asset::new(wem::Document::from_raw(raw).unwrap());
    let result = conversion::m3::plan::<native::StarCraft2>(&asset, &options)
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let root = result.model.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let cloth = result
        .model
        .document()
        .resolve::<nr::ClothPhysics<nr::V4>>(root.cloth_physics.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(cloth.density, 7.);
}
