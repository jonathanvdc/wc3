use glam::Vec3;
use wc3::model::common::PackedArray;
use wc3::model::common::{IntoZeroVec, ZeroVec};
use wc3::model::io;
use wc3::model::IntoPackedArray;
use wc3::model::{
    conversion::{self, ExportOptions, ImportOptions},
    formats::wmo::{self, Bundle, Dialect, FileKey, Group, GroupKey, Root},
    WorldImageResource,
};
use wmo::{codec::Array, group::GroupChunk, records as r, root::RootChunk};
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
    use wc3::model::profiles::{Universal, Wmo};
    let source = fixture::<D>();
    let asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>()
        .try_narrow::<Wmo>()
        .unwrap();
    let output = io::wmo::export::<D>(&asset, &ExportOptions::default())
        .unwrap()
        .value;
    output.validate_complete().unwrap();
    assert_eq!(output.root.header().wmo_id, 55);
}
#[test]
fn modern() {
    roundtrip::<wmo::Classic>();
    roundtrip::<wmo::Legion>();
    roundtrip::<wmo::Bfa>();
    roundtrip::<wmo::ShadowlandsPre92>();
    roundtrip::<wmo::Shadowlands>();
    roundtrip::<wmo::Dragonflight>();
    roundtrip::<wmo::WarWithin>();
}
#[test]
fn region_lights_are_independent_editable_features() {
    let mut source = fixture::<wmo::Classic>();
    let group = source.group_mut(GroupKey { index: 0, lod: 0 }).unwrap();
    group.chunks_mut().push(GroupChunk::PointLights(
        Array::from_records(&[r::PointLight {
            color_bgra: [30, 20, 10, 40],
            intensity: 2.,
            position: [1., 2., 3.],
            reserved_header: 42,
            attenuation_end: 10.,
            ..Default::default()
        }])
        .unwrap(),
    ));
    group.chunks_mut().push(GroupChunk::PointLightAnimations(
        Array::from_records(&[r::PointLightAnimation {
            light_index: 0,
            intensity: 3.,
            position: [2., 3., 4.],
            texture_file_id: 1234,
            flicker_mode: r::FlickerMode::NoiseStep,
            reserved: [1, 2, 3, 4],
            ..Default::default()
        }])
        .unwrap(),
    ));
    group.chunks_mut().push(GroupChunk::PointLightSets(
        Array::from_records(&[r::LightSetRange {
            offset: 0,
            count: 1,
        }])
        .unwrap(),
    ));
    let mut asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let model = &asset;
    let (id, mut feature) = model
        .features()
        .iter()
        .find(|(_, feature)| {
            matches!(
                feature.payload,
                wc3::model::profiles::wmo::Node::PointLight(_)
            )
        })
        .map(|(id, value)| (id, value.payload.clone()))
        .unwrap();
    if let wc3::model::profiles::wmo::Node::PointLight(light) = &mut feature {
        light.intensity = 7.;
    }
    asset
        .edit(|builder| builder.set_feature(id, feature))
        .unwrap();
    let output = io::wmo::export::<wmo::Classic>(&asset, &ExportOptions::default())
        .unwrap()
        .value;
    let group = output.group(GroupKey { index: 0, lod: 0 }).unwrap();
    let light = group
        .chunks()
        .iter()
        .find_map(|chunk| {
            if let GroupChunk::PointLights(rows) = chunk {
                Some(rows.get(0).unwrap().unwrap())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(light.intensity, 7.);
    assert_eq!(light.reserved_header, 42);
    assert_eq!(light.color_bgra, [30, 20, 10, 40]);
    let program = group
        .chunks()
        .iter()
        .find_map(|chunk| {
            if let GroupChunk::PointLightAnimations(rows) = chunk {
                Some(rows.get(0).unwrap().unwrap())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(program.texture_file_id, 1234);
    assert_eq!(program.reserved, [1, 2, 3, 4]);
    assert_eq!(program.flicker_mode, r::FlickerMode::NoiseStep);
    output.validate_complete().unwrap();
}

#[test]
fn edited_geometry_rebuilds_native_companions() {
    let source = fixture::<wmo::Classic>();
    let mut asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let model = &asset;
    let (id, mut mesh) = model
        .meshes()
        .iter()
        .next()
        .map(|(id, mesh)| (id, mesh.clone()))
        .unwrap();
    let mut positions = mesh.positions().to_vec();
    positions[0] = Vec3::from_array([0.25, 0.5, 0.75]);
    mesh.geometry = mesh
        .geometry
        .with_positions((positions).into_packed_array())
        .unwrap();
    asset.edit(|builder| builder.set_mesh(id, mesh)).unwrap();
    let output = io::wmo::export::<wmo::Classic>(&asset, &ExportOptions::default())
        .unwrap()
        .value;
    let group = output.group(GroupKey { index: 0, lod: 0 }).unwrap();
    let positions = group
        .chunks()
        .iter()
        .find_map(|chunk| {
            if let GroupChunk::Positions(rows) = chunk {
                Some(rows)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(positions.get(0).unwrap().unwrap(), [0.25, 0.5, 0.75]);
    assert_eq!(
        asset
            .image_resource(
                asset.materials().iter().next().unwrap().1.surface.inputs()[0]
                    .image
                    .unwrap()
            )
            .unwrap(),
        &WorldImageResource::FileDataId(123)
    );
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
    let asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let model = &asset;
    let instance = model.instances().iter().next().unwrap().1;
    assert_eq!(instance.draws.len(), 1);
    assert!(model
        .mesh(instance.mesh)
        .unwrap()
        .geometry
        .triangle_properties()
        .get(0)
        .unwrap()
        .material
        .is_none());
    let output = io::wmo::export::<wmo::Classic>(&asset, &ExportOptions::default())
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
fn liquid_grid_fields_are_editable_without_source_packets() {
    use wc3::model::profiles::wmo::LiquidVertex as SurfaceVertex;
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
    let mut asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (id, mut instance) = asset
        .instances()
        .iter()
        .next()
        .map(|(id, value)| (id, value.clone()))
        .unwrap();
    let liquid = &mut instance.properties.liquids[0];
    let mut vertices = liquid.vertices.to_vec();
    let SurfaceVertex::Unresolved { height, .. } = &mut vertices[0] else {
        panic!("unresolved liquid interpretation changed")
    };
    *height = 6.;
    liquid.vertices = PackedArray::from_storage((vertices).into_zero_vec());
    asset
        .edit(|builder| builder.set_instance(id, instance))
        .unwrap();
    let output = io::wmo::export::<wmo::Classic>(&asset, &ExportOptions::default())
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
fn unknown_native_blends_keep_their_family_namespace_during_edits() {
    use wc3::model::profiles::wmo::Blend;
    for code in [8, 9] {
        let mut source = fixture::<wmo::Classic>();
        for chunk in source.root.chunks_mut() {
            if let RootChunk::Materials(rows) = chunk {
                let mut values = rows.iter().collect::<Result<Vec<_>, _>>().unwrap();
                values[0].blend_mode = r::BlendMode::Unknown(code);
                *rows = Array::from_records(&values).unwrap();
            }
        }
        let mut asset = io::wmo::import(&source, &ImportOptions::default())
            .unwrap()
            .value;
        let (id, mut material) = asset
            .materials()
            .iter()
            .next()
            .map(|(id, value)| (id, value.clone()))
            .unwrap();
        assert_eq!(
            material.surface.declaration().state.blend,
            Blend::Unknown(code)
        );
        material.name = "Renamed".into();
        asset
            .edit(|builder| builder.set_material(id, material))
            .unwrap();
        let output = io::wmo::export::<wmo::Classic>(&asset, &ExportOptions::default())
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

#[path = "wmo/historical.rs"]
mod historical;

#[test]
fn unknown_payloads_are_rejected_or_dropped_before_rebuilding_files() {
    use std::borrow::Cow;
    use wc3::model::conversion::UnknownPolicy;
    let mut source = fixture::<wmo::Classic>();
    source
        .root
        .insert_chunk(
            source.root.chunks().count(),
            RootChunk::Unknown {
                tag: *b"XTRA",
                data: Cow::Borrowed(&[7, 8, 9]),
            },
        )
        .unwrap();
    assert!(io::wmo::import(&source, &ImportOptions::default()).is_err());
    let asset = io::wmo::import(
        &source,
        &ImportOptions {
            unknown: UnknownPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap()
    .value;

    let output = io::wmo::export::<wmo::Classic>(&asset, &ExportOptions::default())
        .unwrap()
        .value;
    assert!(output.root.chunks().all(|chunk| chunk.tag() != *b"XTRA"));
}

#[test]
fn portals_are_scoped_authoring_relationships_and_rebuild_native_windows() {
    let mut source = fixture::<wmo::Classic>();
    source.root.header_mut().portal_count = 1;
    for chunk in [
        RootChunk::PortalVertices(
            Array::from_records(&[[0., 0., 0.], [0., 1., 0.], [0., 0., 1.]]).unwrap(),
        ),
        RootChunk::Portals(
            Array::from_records(&[r::Portal {
                start_vertex: 0,
                vertex_count: 3,
                plane: [1., 0., 0., 0.],
            }])
            .unwrap(),
        ),
        RootChunk::PortalRefs(
            Array::from_records(&[r::PortalRef {
                portal_index: 0,
                group_index: 0,
                side: 1,
                flags: Default::default(),
            }])
            .unwrap(),
        ),
    ] {
        source
            .root
            .insert_chunk(source.root.chunks().count(), chunk)
            .unwrap();
    }
    source
        .group_mut(GroupKey { index: 0, lod: 0 })
        .unwrap()
        .header_mut()
        .portal_count = 1;
    source.validate_complete().unwrap();
    let asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let world = asset.systems().iter().next().unwrap().1;
    assert_eq!(world.portals.len(), 1);
    let region = &asset.instances().iter().next().unwrap().1.properties;
    assert_eq!(region.portals[0].portal.index, 0);
    assert_eq!(region.portals[0].neighbor.index, 0);
    let output = io::wmo::export::<wmo::Classic>(&asset, &ExportOptions::default())
        .unwrap()
        .value;
    output.validate_complete().unwrap();
    assert_eq!(output.root.header().portal_count, 1);
    assert_eq!(
        output
            .group(GroupKey { index: 0, lod: 0 })
            .unwrap()
            .header()
            .portal_count,
        1
    );
}

#[test]
fn strip_connectivity_and_tangent_pools_are_independent_editable_geometry() {
    use std::borrow::Cow;
    use wc3::model::profiles::{Universal, Wmo};
    let mut source = fixture::<wmo::Classic>();
    let group = source.group_mut(GroupKey { index: 0, lod: 0 }).unwrap();
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
            material: 0,
            ..Default::default()
        }])
        .unwrap(),
    ));
    group.chunks_mut().push(GroupChunk::StripIndices(
        Array::from_records(&[0u16, 1, 2]).unwrap(),
    ));
    group.chunks_mut().push(GroupChunk::BatchOverrides(
        Array::from_records(&[r::BatchOverride {
            start_index: 0,
            index_count: 3,
            padding: 19,
        }])
        .unwrap(),
    ));
    let mut tangent_bytes = 0u16.to_le_bytes().to_vec();
    tangent_bytes.extend_from_slice(Array::from_records(&[[0., 1., 0., 1.]]).unwrap().as_bytes());
    group
        .chunks_mut()
        .push(GroupChunk::TangentsUnresolved(Cow::Owned(tangent_bytes)));
    let asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>();
    let id = asset.meshes().iter().next().unwrap().0;
    let mut invalid = asset.clone();
    assert!(invalid
        .edit(|builder| {
            let mut mesh = builder.model().mesh(id).unwrap().clone();
            let data = &mut mesh.geometry;
            data.connectivity =
                wc3::model::ConnectivitySets::new(data.connectivity.primary.clone(), Vec::new());
            builder.set_mesh(id, mesh)
        })
        .is_err());
    let mut asset = asset.try_narrow::<Wmo>().unwrap();
    let (id, mut mesh) = asset
        .meshes()
        .iter()
        .next()
        .map(|(id, mesh)| (id, mesh.clone()))
        .unwrap();
    assert!(!mesh.geometry.tangent_vectors().is_owned());
    assert_eq!(mesh.geometry.strip_indices().to_vec(), vec![0, 1, 2]);
    mesh.geometry = mesh
        .geometry
        .with_tangent_vectors(PackedArray::from_storage(
            (vec![[1., 0., 0., -1.]]).into_zero_vec(),
        ))
        .unwrap();
    asset.edit(|builder| builder.set_mesh(id, mesh)).unwrap();
    let output = io::wmo::export::<wmo::Classic>(&asset, &ExportOptions::default())
        .unwrap()
        .value;
    output.validate_complete().unwrap();
    let group = output.group(GroupKey { index: 0, lod: 0 }).unwrap();
    let window = group
        .chunks()
        .iter()
        .find_map(|chunk| match chunk {
            GroupChunk::BatchOverrides(rows) => Some(rows.get(0).unwrap().unwrap()),
            _ => None,
        })
        .unwrap();
    assert_eq!(window.padding, 19);
    assert_eq!(window.index_count, 3);
    let tangents = group
        .chunks()
        .iter()
        .find_map(|chunk| match chunk {
            GroupChunk::Tangents(rows) => Some(rows),
            _ => None,
        })
        .unwrap();
    assert_eq!(tangents.first_tangent.get(0).unwrap().unwrap(), 0);
    assert_eq!(
        tangents.tangents.get(0).unwrap().unwrap(),
        [1., 0., 0., -1.]
    );
}

#[test]
fn ambient_memberships_remap_scoped_volumes_across_native_table_order() {
    use wc3::model::profiles::wmo::Volume;
    let mut source = fixture::<wmo::Legion>();
    for chunk in [
        RootChunk::AmbientBoxes(
            Array::from_records(&[r::AmbientBox {
                colors: [[1, 2, 3, 4]; 3],
                ..Default::default()
            }])
            .unwrap(),
        ),
        RootChunk::GlobalAmbients(
            Array::from_records(&[r::AmbientVolume {
                end: 5.,
                ..Default::default()
            }])
            .unwrap(),
        ),
        RootChunk::AmbientVolumes(
            Array::from_records(&[r::AmbientVolume {
                position: [2., 3., 4.],
                start: 1.,
                end: 2.,
                colors: [[5, 6, 7, 8]; 3],
                ..Default::default()
            }])
            .unwrap(),
        ),
    ] {
        source
            .root
            .insert_chunk(source.root.chunks().count(), chunk)
            .unwrap();
    }
    let group = source.group_mut(GroupKey { index: 0, lod: 0 }).unwrap();
    group.chunks_mut().push(GroupChunk::AmbientVolumeReferences(
        Array::from_records(&[0u16]).unwrap(),
    ));
    group.chunks_mut().push(GroupChunk::AmbientBoxReferences(
        Array::from_records(&[0u16]).unwrap(),
    ));
    source.validate_complete().unwrap();
    let asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let model = &asset;
    let region = &model.instances().iter().next().unwrap().1.properties;
    assert_eq!(region.ambients.len(), 2);
    let world = model.system(region.ambients[0].system).unwrap();
    assert!(matches!(
        world.ambient_volumes[region.ambients[0].index as usize].volume,
        Volume::Sphere { .. }
    ));
    assert!(matches!(
        world.ambient_volumes[region.ambients[1].index as usize].volume,
        Volume::Planes { .. }
    ));
    let output = io::wmo::export::<wmo::Legion>(&asset, &ExportOptions::default())
        .unwrap()
        .value;
    output.validate_complete().unwrap();
    let group = output.group(GroupKey { index: 0, lod: 0 }).unwrap();
    for references in group.chunks().iter().filter_map(|chunk| match chunk {
        GroupChunk::AmbientVolumeReferences(rows) | GroupChunk::AmbientBoxReferences(rows) => {
            Some(rows)
        }
        _ => None,
    }) {
        assert_eq!(references.len(), 1);
        assert_eq!(references.get(0).unwrap().unwrap(), 0);
    }
}

#[test]
fn compatible_same_detail_geometry_joins_without_changing_material_look_or_source() {
    use wc3::model::{Bounds, Draw};
    let mut source = fixture::<wmo::Classic>();
    let group = source.group_mut(GroupKey { index: 0, lod: 0 }).unwrap();
    group.header_mut().exterior_batch_count = 1;
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
            material: 0,
            ..Default::default()
        }])
        .unwrap(),
    ));
    let mut asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let model = &asset;
    let (original_id, original) = model
        .instances()
        .iter()
        .next()
        .map(|(id, instance)| (id, instance.clone()))
        .unwrap();
    let mut second_mesh = model.mesh(original.mesh).unwrap().clone();
    let positions = second_mesh
        .positions()
        .iter()
        .map(|position| [position[0] + 2., position[1], position[2]])
        .collect::<ZeroVec<_>>();
    second_mesh.geometry = second_mesh
        .geometry
        .with_positions(PackedArray::from_storage(positions))
        .unwrap();
    second_mesh.bounds = Bounds {
        minimum: Vec3::from_array([2., 0., 0.]),
        maximum: Vec3::from_array([3., 1., 1.]),
        sphere_radius: 0.75f32.sqrt(),
    };
    let mut second = original.clone();
    second.draws = second
        .draws
        .iter()
        .map(|draw| Draw::new(draw.subset, draw.properties.clone()))
        .collect();
    let (world_id, mut world) = model
        .systems()
        .iter()
        .next()
        .map(|(id, world)| (id, world.clone()))
        .unwrap();
    let material = model.materials().iter().next().unwrap().1.clone();
    asset
        .edit(|builder| {
            let material = builder.add_material(material)?;
            let mesh = builder.add_mesh(second_mesh)?;
            second.mesh = mesh;
            let second = builder.add_instance_with_materials(second, vec![material])?;
            world.groups[0].instances.push(second);
            builder.set_system(world_id, world)?;
            Ok(())
        })
        .unwrap();
    let result = io::wmo::export::<wmo::Classic>(&asset, &ExportOptions::default()).unwrap();
    assert!(result
        .report
        .issues
        .iter()
        .any(|issue| issue.kind == conversion::IssueKind::Normalized));
    assert_eq!(asset.instances().iter().count(), 2);
    assert!(asset.instance(original_id).is_some());
    let group = result.value.group(GroupKey { index: 0, lod: 0 }).unwrap();
    let positions = group
        .chunks()
        .iter()
        .find_map(|chunk| match chunk {
            GroupChunk::Positions(rows) => Some(rows),
            _ => None,
        })
        .unwrap();
    assert_eq!(positions.len(), 6);
    assert_eq!(positions.get(3).unwrap().unwrap(), [2., 0., 0.]);
    let batches = group
        .chunks()
        .iter()
        .find_map(|chunk| match chunk {
            GroupChunk::Batches(rows) => Some(rows),
            _ => None,
        })
        .unwrap();
    assert_eq!(batches.len(), 2);
    assert_eq!(batches.get(0).unwrap().unwrap().material_index(), 0);
    assert_eq!(batches.get(1).unwrap().unwrap().material_index(), 1);
    result.value.validate_complete().unwrap();
}

#[test]
fn implicitly_filled_region_keeps_its_liquid_category_without_a_surface_grid() {
    let mut source = fixture::<wmo::Classic>();
    let key = GroupKey { index: 0, lod: 0 };
    let group = source.group_mut(key).unwrap();
    group.header_mut().flags.set_has_liquid(true);
    group.header_mut().liquid_type = 37;
    let asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (_, instance) = asset.instances().iter().next().unwrap();
    assert_eq!(instance.properties.implicit_liquid, Some(37));
    assert!(instance.properties.liquids.is_empty());
    let output = io::wmo::export::<wmo::Classic>(&asset, &ExportOptions::default())
        .unwrap()
        .value;
    let group = output.group(key).unwrap();
    assert!(group.header().flags.has_liquid());
    assert_eq!(group.header().liquid_type, 37);
    assert!(!group
        .chunks()
        .iter()
        .any(|chunk| matches!(chunk, GroupChunk::Liquid(_))));
}

#[test]
fn region_selection_groups_are_rejected_before_native_export() {
    let source = fixture::<wmo::Legion>();
    let mut asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let instance = asset.instances().iter().next().unwrap().0;
    let result = asset.edit(|builder| {
        let mut region = builder.model().instance(instance).unwrap().clone();
        region.selection_group = 7;
        builder.set_instance(instance, region)
    });
    assert!(result.is_err());
    assert_eq!(asset.instance(instance).unwrap().selection_group, 0);
}

#[test]
fn semantic_world_controls_drop_unclassified_bits() {
    let mut source = fixture::<wmo::Legion>();
    source.root.header_mut().tail.flags = r::RootFlags(0x8000);
    let asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let output = io::wmo::export::<wmo::Legion>(&asset, &ExportOptions::default())
        .unwrap()
        .value;
    assert_eq!(output.root.header().flags().bits(), 0);
}

#[test]
fn modern_world_output_requires_explicit_bounds_or_loss_policy() {
    let source = fixture::<wmo::Legion>();
    let mut asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    asset
        .edit(|builder| {
            builder.metadata_mut().bounds = None;
            Ok(())
        })
        .unwrap();
    assert!(io::wmo::export::<wmo::Legion>(&asset, &ExportOptions::default()).is_err());
    let conversion = io::wmo::export::<wmo::Legion>(
        &asset,
        &ExportOptions {
            loss_policy: conversion::LossPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(conversion
        .report
        .issues
        .iter()
        .any(|issue| issue.path == "world.bounds" && issue.kind == conversion::IssueKind::Dropped));
}

#[test]
fn world_radius_is_derived_from_each_authored_box() {
    let source = fixture::<wmo::Legion>();
    let mut asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let original = asset.bounds().unwrap();
    assert!(asset
        .edit(|builder| {
            let mut bounds = original;
            bounds.sphere_radius += 1.;
            builder.metadata_mut().bounds = Some(bounds);
            Ok(())
        })
        .is_err());
    assert_eq!(asset.bounds(), Some(original));
    let (id, mesh) = asset.meshes().iter().next().unwrap();
    let original = mesh.bounds;
    assert!(asset
        .edit(|builder| {
            let mut mesh = builder.model().mesh(id).unwrap().clone();
            mesh.bounds.sphere_radius += 1.;
            builder.set_mesh(id, mesh)
        })
        .is_err());
    assert_eq!(asset.mesh(id).unwrap().bounds, original);
}
