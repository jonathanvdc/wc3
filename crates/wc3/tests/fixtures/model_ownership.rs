use m2::companions::SkinProfile;
use m2::records::{
    Batch, BlendMode, Bone, Material, ParticleEmitter, SkinSection, Texture, TextureSource, Vertex,
};
use wc3::model::common::ZeroVec;
use wc3::model::formats::{m2, wmo};
use wmo::{codec::Array, group::GroupChunk, records, root::RootChunk, Classic, Group, Root};

pub fn m2_bytes() -> Vec<u8> {
    let mut document = m2::Document::<m2::Flat<m2::V256>>::new();
    let core = document.core_mut();
    core.name = ZeroVec::alloc_from_slice(b"owned triangle\0");
    core.bones = vec![Bone::new()];
    core.vertices = ZeroVec::alloc_from_slice(&[
        Vertex {
            position: [0., 0., 0.],
            normal: [0., 0., 1.],
            texture_coordinates: [[0., 0.]; 2],
            bone_indices: [0; 4],
            bone_weights: [255, 0, 0, 0],
        },
        Vertex {
            position: [1., 0., 0.],
            normal: [0., 0., 1.],
            texture_coordinates: [[1., 0.]; 2],
            bone_indices: [0; 4],
            bone_weights: [255, 0, 0, 0],
        },
        Vertex {
            position: [0., 1., 0.],
            normal: [0., 0., 1.],
            texture_coordinates: [[0., 1.]; 2],
            bone_indices: [0; 4],
            bone_weights: [255, 0, 0, 0],
        },
    ]);
    core.textures = vec![Texture {
        kind: TextureSource::File,
        filename: ZeroVec::alloc_from_slice(b"textures/checker.blp\0"),
        ..Default::default()
    }];
    core.materials = ZeroVec::alloc_from_slice(&[Material {
        blending_mode: BlendMode::Alpha,
        ..Default::default()
    }]);
    core.texture_combos = ZeroVec::alloc_from_slice(&[0]);
    core.texture_weight_combos = ZeroVec::alloc_from_slice(&[u16::MAX]);
    core.texture_transform_combos = ZeroVec::alloc_from_slice(&[u16::MAX]);
    core.texture_transform_bone_map = ZeroVec::alloc_from_slice(&[0]);
    core.bone_combos = ZeroVec::alloc_from_slice(&[0]);
    core.skins = vec![SkinProfile {
        vertices: ZeroVec::alloc_from_slice(&[0, 1, 2]),
        indices: ZeroVec::alloc_from_slice(&[0, 1, 2]),
        bones: ZeroVec::alloc_from_slice(&[[0; 4]; 3]),
        sections: vec![SkinSection {
            vertex_count: 3,
            index_count: 3,
            bone_count: 1,
            id: 17,
            ..Default::default()
        }],
        batches: vec![Batch {
            color_index: -1,
            texture_count: 1,
            shader_id: 3,
            ..Default::default()
        }],
        ..Default::default()
    }];
    core.particles = vec![ParticleEmitter::new(Default::default())];
    document.encode_m2().unwrap()
}

pub fn wmo_bytes() -> (Vec<u8>, Vec<u8>) {
    let mut root = Root::<Classic>::new();
    root.header_mut().bounds = [[0.; 3], [1.; 3]];
    root.insert_chunk(
        0,
        RootChunk::TextureNames(Array::from_records(b"tiles/floor.blp\0").unwrap()),
    )
    .unwrap();
    root.insert_chunk(
        1,
        RootChunk::Materials(
            Array::from_records(&[records::Material::<Classic>::default()]).unwrap(),
        ),
    )
    .unwrap();
    root.insert_chunk(
        2,
        RootChunk::GroupNames(Array::from_records(b"Hall\0Stone chamber\0").unwrap()),
    )
    .unwrap();
    root.insert_chunk(
        3,
        RootChunk::Groups(
            Array::from_records(&[records::GroupInfo {
                name_offset: 0,
                bounds: [[0.; 3], [1.; 3]],
                ..Default::default()
            }])
            .unwrap(),
        ),
    )
    .unwrap();
    root.synchronize_counts().unwrap();
    let mut group = Group::<Classic>::new();
    group.header_mut().descriptive_name_offset = 5;
    group.header_mut().bounds_maximum = [1.; 3];
    group.header_mut().fog_indices = [255; 4];
    group.chunks_mut().extend([
        GroupChunk::Positions(Array::from_records(&[[0.; 3], [1., 0., 0.], [0., 1., 0.]]).unwrap()),
        GroupChunk::Normals(Array::from_records(&[[0., 0., 1.]; 3]).unwrap()),
        GroupChunk::TextureCoordinates(
            Array::from_records(&[[0.; 2], [1., 0.], [0., 1.]]).unwrap(),
        ),
        GroupChunk::Indices(Array::from_records(&[0u16, 1, 2]).unwrap()),
        GroupChunk::Triangles(
            Array::from_records(&[records::Triangle {
                flags: Default::default(),
                material: 0,
            }])
            .unwrap(),
        ),
    ]);
    (root.encode().unwrap(), group.encode().unwrap())
}
