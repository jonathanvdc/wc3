//! Group validation separates local geometry from references into the owning root.
use wc3::model::wmo::{self, Bundle, Classic, Group, Root, Validate as _};
use wmo::codec::Array;
use wmo::group::GroupChunk;
use wmo::records::{Batch, BspNode, Material, Triangle};
use wmo::root::RootChunk;

fn triangle_group() -> Group<'static, Classic> {
    let mut group = Group::new();
    group.chunks_mut().extend([
        GroupChunk::Positions(
            Array::from_records(&[[0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]).unwrap(),
        ),
        GroupChunk::Indices(Array::from_records(&[0u16, 1, 2]).unwrap()),
        GroupChunk::Triangles(
            Array::from_records(&[Triangle {
                flags: 0x20,
                material: 0,
            }])
            .unwrap(),
        ),
        GroupChunk::Batches(
            Array::from_records(&[Batch::<Classic> {
                index_count: 3,
                maximum_vertex: 2,
                ..Default::default()
            }])
            .unwrap(),
        ),
    ]);
    group
}
#[test]
fn repeated_uv_layers_validate_but_index_and_batch_ranges_are_bounded() {
    let mut group = triangle_group();
    for _ in 0..4 {
        group.chunks_mut().push(GroupChunk::TextureCoordinates(
            Array::from_records(&[[0f32, 0.0]; 3]).unwrap(),
        ));
    }
    group.validate().unwrap();
    let GroupChunk::Indices(indices) = &mut group.chunks_mut()[1] else {
        panic!()
    };
    indices.set(2, &3).unwrap();
    assert!(group.validate().is_err());
    let GroupChunk::Indices(indices) = &mut group.chunks_mut()[1] else {
        panic!()
    };
    indices.set(2, &2).unwrap();
    let GroupChunk::Batches(batches) = &mut group.chunks_mut()[3] else {
        panic!()
    };
    let mut batch = batches.get(0).unwrap().unwrap();
    batch.index_count = 4;
    batches.set(0, &batch).unwrap();
    assert!(group.validate().is_err());
}
#[test]
fn collision_tree_cycles_and_face_windows_are_rejected() {
    let mut group = triangle_group();
    group
        .chunks_mut()
        .push(GroupChunk::BspFaces(Array::from_records(&[0u16]).unwrap()));
    group.chunks_mut().push(GroupChunk::BspNodes(
        Array::from_records(&[BspNode {
            negative_child: -1,
            positive_child: -1,
            face_count: 1,
            ..Default::default()
        }])
        .unwrap(),
    ));
    group.validate().unwrap();
    let GroupChunk::BspNodes(nodes) = group.chunks_mut().last_mut().unwrap() else {
        panic!()
    };
    let mut node = nodes.get(0).unwrap().unwrap();
    node.positive_child = 0;
    nodes.set(0, &node).unwrap();
    assert!(group.validate().is_err());
    let GroupChunk::BspNodes(nodes) = group.chunks_mut().last_mut().unwrap() else {
        panic!()
    };
    node.positive_child = -1;
    node.face_count = 2;
    nodes.set(0, &node).unwrap();
    assert!(group.validate().is_err());
}
#[test]
fn root_material_and_placement_references_are_checked_separately() {
    let group = triangle_group();
    let mut root = Root::<Classic>::new();
    let bundle = Bundle::from_root(root.clone());
    assert!(bundle.validate_group_references(&group).is_err());
    root.insert_chunk(
        0,
        RootChunk::Materials(Array::from_records(&[Material::<Classic>::default()]).unwrap()),
    )
    .unwrap();
    let bundle = Bundle::from_root(root);
    bundle.validate_group_references(&group).unwrap();
    let mut group = group;
    group.chunks_mut().push(GroupChunk::DoodadReferences(
        Array::from_records(&[0u16]).unwrap(),
    ));
    assert!(bundle.validate_group_references(&group).is_err());
}
