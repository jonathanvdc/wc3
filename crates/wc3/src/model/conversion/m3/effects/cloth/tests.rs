use super::*;
use crate::model::conversion::{ExportOptions, ImportOptions};
use std::borrow::Cow;
fn geometry() -> wem::Document<'static> {
    sem::build(&sem::Model {
        profile: wr::ProfileId::SC2,
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
fn every_cloth_revision_rebuilds_from_edited_wem_solver_fields() {
    macro_rules! check {
        ($v:ty) => {{
            let mut source = native::Document::<native::Md34>::new();
            let section = source
                .append(&[nr::ClothPhysics::<$v> {
                    density: 3.,
                    tracking: 0.75,
                    gravity: 1.,
                    ..Default::default()
                }])
                .unwrap();
            let root = source
                .append(&[nr::ModelRecord::<nr::V29> {
                    cloth_physics: Ref::from_section(1, section.index()),
                    ..Default::default()
                }])
                .unwrap();
            source.set_root(Ref::<nr::ModelRecord<nr::V29>>::from_section(
                1,
                root.index(),
            ));
            let source = native::Model::from_document(source).unwrap();
            let document = attach(
                &source,
                geometry(),
                &mut Context::import(&ImportOptions::default()),
            )
            .unwrap();
            let root = document.root::<wr::DocumentRecord<wr::V1>>().unwrap();
            let mut raw = document.into_raw();
            let model = raw
                .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())
                .unwrap()
                .get(0)
                .unwrap()
                .unwrap();
            let id = raw.section_id(model.physics.cloths.index).unwrap();
            let mut rows = raw
                .array::<wr::Cloth<wr::V2>>(id)
                .unwrap()
                .iter()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert_eq!(rows[0].density, 3.);
            rows[0].density = 5.;
            rows[0].sc2.value.as_mut().unwrap().tracking = 0.5;
            raw.replace(id, &rows).unwrap();
            let document = wem::Document::from_raw(raw).unwrap();
            let model = sem::read(&document, 0, wr::ProfileId::SC2).unwrap();
            let mut output = native::Document::<native::Md34>::new();
            let reference = export(
                &model,
                &mut output,
                &[],
                &mut Context::export(&ExportOptions::default()),
            )
            .unwrap();
            let row = output
                .resolve::<nr::ClothPhysics<$v>>(reference.cast())
                .unwrap()
                .get(0)
                .unwrap()
                .unwrap();
            assert_eq!(row.density, 5.);
            assert_eq!(row.tracking, 0.5);
        }};
    }
    check!(nr::V0);
    check!(nr::V1);
    check!(nr::V2);
    check!(nr::V3);
    check!(nr::V4);
}
#[test]
fn stale_cloth_particle_arrays_are_rejected_after_topology_edits() {
    let mut source = native::Document::<native::Md34>::new();
    let particles = source.append(&[nr::Byte { value: 1 }; 3]).unwrap();
    let cloth = source
        .append(&[nr::ClothPhysics::<nr::V4> {
            sim_enabled: Ref::from_section(3, particles.index()),
            ..Default::default()
        }])
        .unwrap();
    let root = source
        .append(&[nr::ModelRecord::<nr::V29> {
            cloth_physics: Ref::from_section(1, cloth.index()),
            ..Default::default()
        }])
        .unwrap();
    source.set_root(Ref::<nr::ModelRecord<nr::V29>>::from_section(
        1,
        root.index(),
    ));
    let source = native::Model::from_document(source).unwrap();
    let document = attach(
        &source,
        geometry(),
        &mut Context::import(&ImportOptions::default()),
    )
    .unwrap();
    let mut model = sem::read(&document, 0, wr::ProfileId::SC2).unwrap();
    model.meshes[0].positions.to_mut().push([2.; 3]);
    let mut output = native::Document::<native::Md34>::new();
    let error = export(
        &model,
        &mut output,
        &[],
        &mut Context::export(&ExportOptions::default()),
    )
    .unwrap_err();
    assert_eq!(error.path, "cloth.sim_enabled");
}
