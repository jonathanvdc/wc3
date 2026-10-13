use std::borrow::Cow;
use wc3::model::{
    conversion::{self, ExportOptions, ImportOptions, UnknownPolicy},
    wem::{self, records as wr, semantic},
    wmo::{self, Bundle, Dialect, FileKey, Group, GroupKey, Root},
};
use wmo::{
    codec::Array,
    group::GroupChunk,
    records::{self as r},
    root::RootChunk,
};
fn fixture<D: Dialect>() -> Bundle<'static, D> {
    let mut root = Root::<D>::new();
    root.header_mut().wmo_id = 55;
    root.header_mut().bounds = [[0.; 3], [1.; 3]];
    root.insert_chunk(
        0,
        RootChunk::Materials(
            Array::from_records(&[r::Material::<D> {
                texture0: 123,
                diffuse_color: [10, 20, 30, 40],
                ..Default::default()
            }])
            .unwrap(),
        ),
    )
    .unwrap();
    root.insert_chunk(
        1,
        RootChunk::Groups(
            Array::from_records(&[r::GroupInfo {
                bounds: [[0.; 3], [1.; 3]],
                ..Default::default()
            }])
            .unwrap(),
        ),
    )
    .unwrap();
    root.synchronize_counts().unwrap();
    let mut group = Group::<D>::new();
    group.header_mut().bounds_maximum = [1.; 3];
    group.header_mut().fog_indices = [255; 4];
    group.chunks_mut().push(GroupChunk::Positions(
        Array::from_records(&[[0.; 3], [1., 0., 0.], [0., 1., 0.]]).unwrap(),
    ));
    group.chunks_mut().push(GroupChunk::Normals(
        Array::from_records(&[[0., 0., 1.]; 3]).unwrap(),
    ));
    group.chunks_mut().push(GroupChunk::TextureCoordinates(
        Array::from_records(&[[0.; 2], [1., 0.], [0., 1.]]).unwrap(),
    ));
    group.chunks_mut().push(GroupChunk::Indices(
        Array::from_records(&[0u16, 1, 2]).unwrap(),
    ));
    group.chunks_mut().push(GroupChunk::Triangles(
        Array::from_records(&[r::Triangle {
            flags: Default::default(),
            material: 0,
        }])
        .unwrap(),
    ));
    let mut bundle = Bundle::from_root(root);
    bundle.insert_group(
        GroupKey { index: 0, lod: 0 },
        FileKey::Sibling {
            owner: Box::new(FileKey::Root),
            suffix: "_000.wmo".into(),
        },
        group,
    );
    bundle.validate_complete().unwrap();
    bundle
}
fn roundtrip<D: Dialect>() {
    let source = fixture::<D>();
    let asset = conversion::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert!(asset.opaque.is_empty());
    let model = asset.model(&ExportOptions::default()).unwrap();
    assert_eq!(model.meshes[0].triangles.as_ref(), [[0, 1, 2]]);
    assert_eq!(model.textures[0].id, Some(123));
    let output = conversion::wmo::plan::<D>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.root.header().wmo_id, 55);
    assert_eq!(output.root.header().group_count, 1);
    output.validate_complete().unwrap();
    let material = output
        .root
        .chunks()
        .find_map(|chunk| {
            if let RootChunk::Materials(rows) = chunk {
                rows.get(0)
            } else {
                None
            }
        })
        .unwrap()
        .unwrap();
    assert_eq!(material.texture0, 123);
    assert_eq!(material.diffuse_color, [10, 20, 30, 40]);
}
#[test]
fn every_modern_dialect_round_trips_semantics() {
    roundtrip::<wmo::Classic>();
    roundtrip::<wmo::Legion>();
    roundtrip::<wmo::Bfa>();
    roundtrip::<wmo::ShadowlandsPre92>();
    roundtrip::<wmo::Shadowlands>();
    roundtrip::<wmo::Dragonflight>();
    roundtrip::<wmo::WarWithin>();
}
#[test]
fn edited_geometry_rebuilds_native_companions() {
    let asset = conversion::wmo::import(&fixture::<wmo::Classic>(), &ImportOptions::default())
        .unwrap()
        .value;
    let mut model = asset.model(&ExportOptions::default()).unwrap();
    model.meshes[0].positions.to_mut()[0] = [0.25, 0.5, 0.75];
    let asset = conversion::Asset::new(semantic::build(&model).unwrap());
    let output = conversion::wmo::plan::<wmo::Classic>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let group = output.group(GroupKey { index: 0, lod: 0 }).unwrap();
    let value = group
        .chunks()
        .iter()
        .find_map(|chunk| {
            if let GroupChunk::Positions(rows) = chunk {
                rows.get(0)
            } else {
                None
            }
        })
        .unwrap()
        .unwrap();
    assert_eq!(value, [0.25, 0.5, 0.75]);
}
#[test]
fn unrepresentable_portals_and_unknown_payloads_require_policies() {
    let mut source = fixture::<wmo::Classic>();
    source
        .root
        .insert_chunk(
            source.root.chunks().count(),
            RootChunk::PortalVertices(Array::from_records(&[[0.; 3]]).unwrap()),
        )
        .unwrap();
    assert!(conversion::wmo::import(&source, &ImportOptions::default()).is_err());
    let mut source = fixture::<wmo::Classic>();
    source
        .root
        .insert_chunk(
            source.root.chunks().count(),
            RootChunk::Unknown {
                tag: *b"TEST",
                data: Cow::Owned(vec![1, 2, 3]),
            },
        )
        .unwrap();
    assert!(conversion::wmo::import(&source, &ImportOptions::default()).is_err());
    let options = ImportOptions {
        unknown: UnknownPolicy::Archive,
        ..Default::default()
    };
    let asset = conversion::wmo::import(&source, &options).unwrap().value;
    assert_eq!(asset.opaque[0].bytes.as_ref(), [1, 2, 3]);
    let output = conversion::wmo::plan::<wmo::Classic>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert!(output
        .root
        .chunks()
        .all(|chunk| !matches!(chunk, RootChunk::Unknown { .. })));
}
#[test]
fn native_lights_are_editable_wem_payloads() {
    let mut source = fixture::<wmo::Classic>();
    source
        .root
        .insert_chunk(
            source.root.chunks().count(),
            RootChunk::Lights(
                Array::from_records(&[r::Light {
                    color: [10, 20, 30, 255],
                    intensity: 2.,
                    rotation: [0., 0., 0., 1.],
                    ..Default::default()
                }])
                .unwrap(),
            ),
        )
        .unwrap();
    source.root.synchronize_counts().unwrap();
    let asset = conversion::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let mut raw = asset.document.into_raw();
    let root = raw
        .resolve::<wr::DocumentRecord<wr::V1>>(wem::Ref::from_section(
            1,
            raw.root_reference().index,
        ))
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let model = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let id = raw.section_id(model.nodes.nodes.index).unwrap();
    let mut rows = raw
        .array::<wr::Node<wr::V19>>(id)
        .unwrap()
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    if let wr::NodePayload::Light(value) = &mut rows[0].payload {
        value.intensity = 7.;
    } else {
        panic!("native light missing");
    }
    raw.replace(id, &rows).unwrap();
    let asset = conversion::Asset::new(wem::Document::from_raw(raw).unwrap());
    let output = conversion::wmo::plan::<wmo::Classic>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let value = output
        .root
        .chunks()
        .find_map(|chunk| {
            if let RootChunk::Lights(rows) = chunk {
                rows.get(0)
            } else {
                None
            }
        })
        .unwrap()
        .unwrap();
    assert_eq!(value.intensity, 7.);
    assert_eq!(value.color, [10, 20, 30, 255]);
}
#[test]
fn historical_destinations_rebuild_native_values() {
    let mut source = fixture::<wmo::Classic>();
    for chunk in source.root.chunks_mut() {
        if let RootChunk::Materials(rows) = chunk {
            *rows = Array::from_records(&[r::Material::<wmo::Classic> {
                texture0: 1,
                ..Default::default()
            }])
            .unwrap();
        }
    }
    let count = source.root.chunks().count();
    source
        .root
        .insert_chunk(
            count,
            RootChunk::TextureNames(Array::from_records(b"\0Legacy.blp\0").unwrap()),
        )
        .unwrap();
    let asset = conversion::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let alpha = conversion::wmo::plan::<conversion::wmo::V14>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let imported = conversion::wmo::import(&alpha, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(
        imported.model(&ExportOptions::default()).unwrap().meshes[0]
            .triangles
            .as_ref(),
        [[0, 1, 2]]
    );
    let beta = conversion::wmo::plan::<conversion::wmo::V16>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let imported = conversion::wmo::import(&beta, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(
        imported.model(&ExportOptions::default()).unwrap().meshes[0]
            .triangles
            .as_ref(),
        [[0, 1, 2]]
    );
}
#[test]
fn collision_is_an_active_wem_triangle_mesh() {
    let mut source = fixture::<wmo::Classic>();
    let group = source.group_mut(GroupKey { index: 0, lod: 0 }).unwrap();
    group.chunks_mut().push(GroupChunk::BspNodes(
        Array::from_records(&[r::BspNode {
            flags: r::BspFlags(4),
            negative_child: -1,
            positive_child: -1,
            face_count: 1,
            ..Default::default()
        }])
        .unwrap(),
    ));
    group
        .chunks_mut()
        .push(GroupChunk::BspFaces(Array::from_records(&[0u16]).unwrap()));
    let asset = conversion::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let root = asset.document.root::<wr::DocumentRecord<wr::V1>>().unwrap();
    let model = asset
        .document
        .raw()
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(model.physics.bodies.entries, 1);
    let output = conversion::wmo::plan::<wmo::Classic>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert!(output
        .group(GroupKey { index: 0, lod: 0 })
        .unwrap()
        .chunks()
        .iter()
        .any(|chunk| matches!(chunk,GroupChunk::BspFaces(rows)if rows.len()==1)));
}
#[test]
fn liquid_grid_fields_rebuild_without_source_bytes() {
    use wmo::group::{Liquid, LiquidHeader, LiquidVertex, LiquidVertices};
    let mut source = fixture::<wmo::Classic>();
    source
        .group_mut(GroupKey { index: 0, lod: 0 })
        .unwrap()
        .chunks_mut()
        .push(GroupChunk::Liquid(Liquid {
            header: LiquidHeader {
                vertex_dimensions: [2, 2],
                tile_dimensions: [1, 1],
                corner: [1., 2., 3.],
                material: 0,
            },
            vertices: LiquidVertices::Unresolved(
                Array::from_records(
                    &[LiquidVertex {
                        data: [1, 2, 3, 4],
                        height: 4.,
                    }; 4],
                )
                .unwrap(),
            ),
            tiles: Array::from_records(&[0u8]).unwrap(),
        }));
    let asset = conversion::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let root = asset.document.root::<wr::DocumentRecord<wr::V1>>().unwrap();
    let mut raw = asset.document.into_raw();
    let graph = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let mesh = raw
        .resolve::<wr::Mesh<wr::V2>>(graph.meshes.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let section = raw
        .resolve::<wr::MeshSection>(mesh.sections.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let mut fields = raw
        .resolve::<wr::NativeEntry<wr::V2>>(section.native.entries.cast())
        .unwrap()
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let height = fields
        .iter()
        .position(|row| raw.string(row.name).unwrap() == Some("wmo.liquid.heights"))
        .unwrap();
    let characters = "40c00000,40800000,40800000,40800000"
        .bytes()
        .map(|value| wr::Character { value })
        .collect::<Vec<_>>();
    let text = raw.append(&characters).unwrap();
    fields[height].text = wem::Ref::from_section(characters.len() as u32, text.index());
    raw.replace(
        raw.section_id(section.native.entries.index).unwrap(),
        &fields,
    )
    .unwrap();
    let asset = conversion::Asset::new(wem::Document::from_raw(raw).unwrap());
    let output = conversion::wmo::plan::<wmo::Classic>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let liquid = output
        .group(GroupKey { index: 0, lod: 0 })
        .unwrap()
        .chunks()
        .iter()
        .find_map(|chunk| {
            if let GroupChunk::Liquid(value) = chunk {
                Some(value)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(liquid.header.corner, [1., 2., 3.]);
    let rows = Array::<LiquidVertex>::decode(liquid.vertices.as_bytes()).unwrap();
    assert_eq!(rows.get(0).unwrap().unwrap().height, 6.);
    assert_eq!(rows.get(0).unwrap().unwrap().data, [1, 2, 3, 4]);
}
#[test]
fn draw_batches_override_collision_face_material_and_keep_classification() {
    let mut source = fixture::<wmo::Classic>();
    let group = source.group_mut(GroupKey { index: 0, lod: 0 }).unwrap();
    for chunk in group.chunks_mut() {
        if let GroupChunk::Triangles(rows) = chunk {
            *rows = Array::from_records(&[r::Triangle {
                flags: Default::default(),
                material: 255,
            }])
            .unwrap();
        }
    }
    group.header_mut().interior_batch_count = 1;
    group.chunks_mut().push(GroupChunk::Batches(
        Array::from_records(&[r::Batch::<wmo::Classic> {
            prefix: r::CullingBox {
                minimum: [0; 3],
                maximum: [1, 1, 0],
            },
            start_index: 0,
            index_count: 3,
            minimum_vertex: 0,
            maximum_vertex: 2,
            flags: Default::default(),
            material: 0,
        }])
        .unwrap(),
    ));
    let asset = conversion::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(
        asset.model(&ExportOptions::default()).unwrap().meshes[0].material_slot,
        0
    );
    let output = conversion::wmo::plan::<wmo::Classic>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let group = output.group(GroupKey { index: 0, lod: 0 }).unwrap();
    assert_eq!(group.header().interior_batch_count, 1);
    assert_eq!(group.header().exterior_batch_count, 0);
    let face = group
        .chunks()
        .iter()
        .find_map(|chunk| {
            if let GroupChunk::Triangles(rows) = chunk {
                Some(rows.get(0).unwrap().unwrap())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(face.material, 255);
    let batch = group
        .chunks()
        .iter()
        .find_map(|chunk| {
            if let GroupChunk::Batches(rows) = chunk {
                Some(rows.get(0).unwrap().unwrap())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(batch.material, 0);
    assert_eq!(batch.prefix.maximum, [1, 1, 0]);
}
#[test]
fn historical_untextured_materials_have_an_empty_name_table() {
    let asset = conversion::Asset::new(
        semantic::build(&semantic::Model {
            profile: wr::ProfileId::Wow,
            materials: vec![semantic::Material::default()],
            ..Default::default()
        })
        .unwrap(),
    );
    let alpha = conversion::wmo::plan::<conversion::wmo::V14>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    alpha.validate().unwrap();
    let beta = conversion::wmo::plan::<conversion::wmo::V16>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    beta.validate_complete().unwrap();
}

#[test]
fn unknown_native_blends_keep_their_namespace_during_common_edits() {
    for code in [8, 9] {
        let mut source = fixture::<wmo::Classic>();
        for chunk in source.root.chunks_mut() {
            if let RootChunk::Materials(rows) = chunk {
                let mut records = rows.iter().collect::<Result<Vec<_>, _>>().unwrap();
                records[0].blend_mode = r::BlendMode::Unknown(code);
                *rows = Array::from_records(&records).unwrap();
            }
        }
        let imported = conversion::wmo::import(&source, &ImportOptions::default()).unwrap();
        let mut common = imported.value.model(&ExportOptions::default()).unwrap();
        assert_eq!(common.materials[0].blend, wr::BlendMode::Opaque);
        common.materials[0].name = "Renamed".into();
        let edited = conversion::Asset::new(wem::semantic::build(&common).unwrap());
        let output = conversion::wmo::plan::<wmo::Classic>(&edited, &ExportOptions::default())
            .unwrap()
            .execute()
            .unwrap()
            .value;
        let material = output
            .root
            .chunks()
            .find_map(|chunk| {
                if let RootChunk::Materials(rows) = chunk {
                    Some(rows.get(0).unwrap().unwrap())
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(material.blend_mode, r::BlendMode::Unknown(code));
    }
}
