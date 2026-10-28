use wc3::model::common::PackedArray;
use wc3::model::common::ZeroVec;
use wc3::model::conversion::{ExportOptions, ImportOptions};
use wc3::model::formats::wmo::codec::Array;
use wc3::model::formats::wmo::group::GroupChunk;
use wc3::model::formats::wmo::legacy::{
    Batch14, Batch16, Bundle16, Group14, Group16, GroupInfo14, HeaderBatchRange, LegacyGroupChunk,
    LegacyTriangle, Material14, Root14, Root16, RootChunk14,
};
use wc3::model::formats::wmo::records::{CullingBox, GroupInfo, Material};
use wc3::model::formats::wmo::root::RootChunk;
use wc3::model::formats::wmo::{Classic, FileKey};
use wc3::model::io;
use wc3::model::Draw;
use wc3::model::IntoPackedArray;

fn triangle<B>() -> Vec<LegacyGroupChunk<'static, B>> {
    vec![
        LegacyGroupChunk::Common(GroupChunk::Positions(
            Array::from_records(&[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]).unwrap(),
        )),
        LegacyGroupChunk::Common(GroupChunk::Normals(
            Array::from_records(&[[0., 0., 1.]; 3]).unwrap(),
        )),
        LegacyGroupChunk::Common(GroupChunk::Indices(
            Array::from_records(&[0u16, 1, 2]).unwrap(),
        )),
        LegacyGroupChunk::Triangles(
            Array::from_records(&[LegacyTriangle {
                material: 0,
                lightmap: 17,
                reserved: 31,
                ..Default::default()
            }])
            .unwrap(),
        ),
    ]
}
fn alpha() -> Root14<'static> {
    let mut root = Root14::new();
    root.header_mut().wmo_id = 91;
    root.push_chunk(RootChunk14::Materials(
        Array::from_records(&[Material14 {
            diffuse_color: [1, 2, 3, 4],
            ground_type: 9,
            ..Default::default()
        }])
        .unwrap(),
    ))
    .unwrap();
    root.push_chunk(RootChunk14::Common(RootChunk::TextureNames(
        Array::from_records(&[0u8]).unwrap(),
    )))
    .unwrap();
    let mut group = Group14::new();
    group.header.bounds_maximum = [1., 1., 0.];
    group.header.fog_indices = [u8::MAX; 4];
    group.header.unique_id = -7;
    group.header.reserved_tail = [1, 2, 3, 4, 5, 6, 7, 8];
    group.header.interior_batches[0] = HeaderBatchRange {
        vertex_start: 0,
        vertex_count: 3,
        batch_start: 0,
        batch_count: 1,
    };
    group.chunks = triangle();
    group.chunks.push(LegacyGroupChunk::Batches(
        Array::from_records(&[Batch14 {
            bounds: CullingBox {
                minimum: [0; 3],
                maximum: [1, 1, 0],
            },
            start_index: 0,
            index_count: 3,
            minimum_vertex: 0,
            maximum_vertex: 2,
            material: 0,
            lightmap: 12,
            reserved: 29,
            ..Default::default()
        }])
        .unwrap(),
    ));
    root.push_group(
        group,
        GroupInfo14 {
            bounds: [[0.; 3], [1., 1., 0.]],
            ..Default::default()
        },
    )
    .unwrap();
    root.synchronize_counts().unwrap();
    root.validate().unwrap();
    root
}
fn beta() -> Bundle16<'static> {
    let mut root = Root16::new();
    root.header_mut().wmo_id = 92;
    root.header_mut().group_count = 1;
    root.push_chunk(RootChunk::TextureNames(
        Array::from_records(&[0u8]).unwrap(),
    ))
    .unwrap();
    root.push_chunk(RootChunk::Materials(
        Array::from_records(&[Material::<Classic>::default()]).unwrap(),
    ))
    .unwrap();
    root.push_chunk(RootChunk::Groups(
        Array::from_records(&[GroupInfo {
            bounds: [[0.; 3], [1., 1., 0.]],
            ..Default::default()
        }])
        .unwrap(),
    ))
    .unwrap();
    let mut group = Group16::new();
    group.header.bounds_maximum = [1., 1., 0.];
    group.header.fog_indices = [u8::MAX; 4];
    group.header.interior_batch_count = 1;
    group.header.unique_id = -11;
    group.header.reserved_tail = 0x12345678;
    group.header.reserved_batch_count = 88;
    group.chunks = triangle();
    group.chunks.push(LegacyGroupChunk::Batches(
        Array::from_records(&[Batch16 {
            bounds: CullingBox {
                minimum: [0; 3],
                maximum: [1, 1, 0],
            },
            start_index: 0,
            index_count: 3,
            minimum_vertex: 0,
            maximum_vertex: 2,
            material: 0,
            extension: [1, 2, 3, 4, 5, 6, 7, 8],
            ..Default::default()
        }])
        .unwrap(),
    ));
    root.synchronize_counts().unwrap();
    let mut bundle = Bundle16::from_root(root);
    bundle.insert_group(0, FileKey::Root.sibling("_000.wmo"), group);
    bundle.validate_complete().unwrap();
    bundle
}
#[test]
fn alpha_direct_geometry_and_lighting_controls_survive_edits() {
    let source = alpha();
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
    assert!(!mesh.positions().is_owned());
    let mut positions = mesh.positions().to_vec();
    positions[0][0] = 0.5;
    mesh.geometry = mesh
        .geometry
        .with_positions((positions).into_packed_array())
        .unwrap();
    asset.edit(|builder| builder.set_mesh(id, mesh)).unwrap();
    let output = io::wmo::export::<io::wmo::V14>(&asset, &ExportOptions::default())
        .unwrap()
        .value;
    output.validate().unwrap();
    let group = output.group(0).unwrap();
    assert_eq!(group.header.unique_id, -7);
    assert_eq!(group.header.reserved_tail, [1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(group.header.interior_batches[0].vertex_count, 3);
    let batch = group
        .chunks
        .iter()
        .find_map(|chunk| match chunk {
            LegacyGroupChunk::Batches(rows) => Some(rows.get(0).unwrap().unwrap()),
            _ => None,
        })
        .unwrap();
    assert_eq!(batch.lightmap, 12);
    assert_eq!(batch.reserved, 29);
    let face = group
        .chunks
        .iter()
        .find_map(|chunk| match chunk {
            LegacyGroupChunk::Triangles(rows) => Some(rows.get(0).unwrap().unwrap()),
            _ => None,
        })
        .unwrap();
    assert_eq!(face.lightmap, 17);
    assert_eq!(face.reserved, 31);
    let position = group
        .chunks
        .iter()
        .find_map(|chunk| match chunk {
            LegacyGroupChunk::Common(GroupChunk::Positions(rows)) => {
                Some(rows.get(0).unwrap().unwrap())
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(position, [0.5, 0., 0.]);
}
#[test]
fn beta_native_header_and_draw_controls_survive_semantic_roundtrip() {
    let source = beta();
    let asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let output = io::wmo::export::<io::wmo::V16>(&asset, &ExportOptions::default())
        .unwrap()
        .value;
    output.validate_complete().unwrap();
    let group = output.group(0).unwrap();
    assert_eq!(group.header.unique_id, -11);
    assert_eq!(group.header.reserved_batch_count, 88);
    assert_eq!(group.header.reserved_tail, 0x12345678);
    let batch = group
        .chunks
        .iter()
        .find_map(|chunk| match chunk {
            LegacyGroupChunk::Batches(rows) => Some(rows.get(0).unwrap().unwrap()),
            _ => None,
        })
        .unwrap();
    assert_eq!(batch.extension, [1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn historical_multiple_placements_relocate_lighting_windows() {
    let source = alpha();
    let mut asset = io::wmo::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    asset
        .edit(|builder| {
            let (system, world) = builder.model().systems().iter().next().unwrap();
            let mut world = world.clone();
            let first = world.groups[0].instances[0];
            let mut placement = builder.model().instance(first).unwrap().clone();
            let mut mesh = builder.model().mesh(placement.mesh).unwrap().clone();
            let positions = mesh
                .positions()
                .iter()
                .map(|point| [point[0] + 2., point[1], point[2]])
                .collect::<ZeroVec<_>>();
            mesh.geometry = mesh
                .geometry
                .with_positions(PackedArray::from_storage(positions))?;
            placement.mesh = builder.add_mesh(mesh)?;
            let materials = placement
                .draws
                .iter()
                .map(|draw| builder.model().material_for_draw(first, draw.id(), None))
                .collect::<Result<Vec<_>, _>>()?;
            placement.draws = placement
                .draws
                .iter()
                .map(|draw| Draw::new(draw.subset, draw.properties.clone()))
                .collect();
            let second = builder.add_instance_with_materials(placement, materials)?;
            world.groups[0].instances.push(second);
            builder.set_system(system, world)?;
            Ok(())
        })
        .unwrap();
    let encoded = io::wmo::export::<io::wmo::V14>(&asset, &ExportOptions::default())
        .unwrap()
        .value;
    assert_eq!(asset.instances().iter().count(), 2);
    assert_eq!(
        encoded.group(0).unwrap().header.interior_batches[0].vertex_count,
        6
    );
    assert_eq!(
        encoded.group(0).unwrap().header.interior_batches[0].batch_count,
        2
    );
    encoded.validate().unwrap();
}
