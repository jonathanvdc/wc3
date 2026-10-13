use std::borrow::Cow;
use wc3::model::wem::{self, records as r, semantic as s};

fn fixture() -> wem::Document<'static> {
    let model = s::Model {
        materials: vec![s::Material::default(), s::Material::default()],
        meshes: vec![s::Mesh {
            positions: Cow::Owned(vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]),
            triangles: Cow::Owned(vec![[0, 1, 2], [0, 2, 3]]),
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut raw = s::build(&model).unwrap().into_raw();
    let id = raw
        .sections()
        .find(|(_, section)| section.tag() == *b"MESH")
        .unwrap()
        .0;
    let mut mesh = raw
        .array::<r::Mesh<r::V2>>(id)
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let section = raw
        .resolve::<r::MeshSection>(mesh.sections.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let sections = raw
        .append(&[
            section.clone(),
            r::MeshSection {
                material_slot: 1,
                ..section
            },
        ])
        .unwrap();
    mesh.sections = wem::Ref::new(2, sections.index(), 0);
    let mut layers = raw
        .resolve::<r::AttrLayer>(mesh.attributes.layers.cast())
        .unwrap()
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    for (name, domain, type_, data) in [
        (
            "section",
            r::Domain::FACE,
            r::AttrType::U32,
            vec![0u32, 1]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect::<Vec<_>>(),
        ),
        (
            "uv0",
            r::Domain::HALFEDGE,
            r::AttrType::F32X2,
            vec![[0., 0.], [1., 0.], [1., 1.], [0.25, 0.], [1., 1.], [0., 1.]]
                .into_iter()
                .flatten()
                .flat_map(f32::to_le_bytes)
                .collect(),
        ),
        (
            "normal",
            r::Domain::FACE,
            r::AttrType::F32X3,
            vec![[0., 0., 1.], [0., 1., 0.]]
                .into_iter()
                .flatten()
                .flat_map(f32::to_le_bytes)
                .collect(),
        ),
    ] {
        let name = raw
            .append(
                &name
                    .bytes()
                    .map(|value| r::Character { value })
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        let data = raw
            .append(
                &data
                    .into_iter()
                    .map(|value| r::Byte { value })
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        layers.push(r::AttrLayer {
            name: wem::Ref::new(raw.section(name).unwrap().count(), name.index(), 0),
            domain,
            type_,
            data: wem::Ref::new(raw.section(data).unwrap().count(), data.index(), 0),
            ..Default::default()
        });
    }
    let layers_id = raw.append(&layers).unwrap();
    mesh.attributes.layers = wem::Ref::new(layers.len() as u32, layers_id.index(), 0);
    mesh.attributes.domain_counts = [4, 6, 0, 2, 1];
    raw.replace(id, &[mesh]).unwrap();
    wem::Document::from_raw(raw).unwrap()
}
#[test]
fn seams_and_material_draws_remain_editable_in_original_domains() {
    let doc = fixture();
    let mut model = s::read(&doc, 0, r::ProfileId::GENERIC).unwrap();
    assert_eq!(model.meshes.len(), 2);
    assert_eq!(model.meshes[1].material_slot, 1);
    assert_eq!(model.meshes[0].normals[0], [0., 0., 1.]);
    assert_eq!(model.meshes[1].normals[0], [0., 1., 0.]);
    assert_eq!(model.meshes[1].uv_sets[0][0], [0.25, 0.]);
    model.meshes[1].uv_sets[0].to_mut()[0] = [0.5, 0.];
    let rebuilt = s::build(&model).unwrap();
    let root = rebuilt.root::<r::DocumentRecord<r::V1>>().unwrap();
    let graph = rebuilt
        .raw()
        .resolve::<r::ModelRecord<r::V9>>(root.models.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(graph.meshes.entries, 1);
    let reread = s::read(&rebuilt, 0, r::ProfileId::GENERIC).unwrap();
    assert_eq!(reread.meshes[1].uv_sets[0][0], [0.5, 0.]);
}
#[test]
fn contradictory_shared_vertex_edits_are_rejected() {
    let doc = fixture();
    let mut model = s::read(&doc, 0, r::ProfileId::GENERIC).unwrap();
    model.meshes[0].positions.to_mut()[0][0] = 7.;
    assert!(s::build(&model).is_err());
}
#[test]
fn projected_topology_edits_are_rejected() {
    let doc = fixture();
    let mut model = s::read(&doc, 0, r::ProfileId::GENERIC).unwrap();
    model.meshes[0].triangles.to_mut()[0] = [0, 0, 0];
    assert!(s::build(&model).is_err());
}

#[test]
fn mdlx_export_uses_each_material_draw_and_preserves_uv_seams() {
    use wc3::model::{
        conversion::{self, Asset, ExportOptions},
        mdlx,
    };
    let asset = Asset::new(fixture());
    let target = conversion::mdlx::plan::<mdlx::V800>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let geosets = target.geosets().collect::<Vec<_>>();
    assert_eq!(geosets.len(), 2);
    assert_eq!(geosets[1].material_id, 1);
    assert_eq!(geosets[1].uv_sets()[0].get(0).unwrap(), [0.25, 0.]);
}
