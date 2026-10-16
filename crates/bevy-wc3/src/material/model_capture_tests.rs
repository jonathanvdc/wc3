//! Opt-in native fixtures for the existing GPU capture example.
use std::fs::{create_dir_all, write};
use wc3::model::common::ZeroVec;
use wc3::model::formats::m2::{
    self,
    companions::SkinProfile,
    records::{
        Batch, BlendMode, Bone, Extent, Material, MaterialFlags, SkinSection, Texture,
        TextureFlags, TextureSource, Vertex,
    },
};

fn quad(selector: u16, count: u16, patterned: bool) -> m2::Bundle<'static, m2::Flat<m2::V256>> {
    let mut document = m2::Document::<m2::Flat<m2::V256>>::new();
    let core = document.core_mut();
    core.name = ZeroVec::alloc_from_slice(b"material controls\0");
    core.bones = vec![Bone::new()];
    let vertices = [
        ([-1., 0., -1.], [0., 1.], [1., 0.]),
        ([1., 0., -1.], [1., 1.], [0., 0.]),
        ([1., 0., 1.], [1., 0.], [0., 1.]),
        ([-1., 0., 1.], [0., 0.], [1., 1.]),
    ]
    .map(|(position, uv0, uv1)| Vertex {
        position,
        normal: [0., 1., 0.],
        texture_coordinates: [uv0, uv1],
        bone_indices: [0; 4],
        bone_weights: [255, 0, 0, 0],
    });
    core.vertices = ZeroVec::alloc_from_slice(&vertices);
    core.bounding = Extent {
        minimum: [-1., -0.01, -1.],
        maximum: [1., 0.01, 1.],
        radius: 1.5,
    };
    core.textures = [
        "primary.tga",
        "secondary.tga",
        if patterned {
            "pattern.tga"
        } else {
            "tertiary.tga"
        },
        "pattern.tga",
    ]
    .into_iter()
    .map(|path| Texture {
        filename: ZeroVec::alloc_from_slice(format!("{path}\0").as_bytes()),
        kind: TextureSource::File,
        flags: TextureFlags::default(),
    })
    .collect();
    core.materials = ZeroVec::alloc_from_slice(&[Material {
        flags: MaterialFlags::from_bits_retain(7),
        blending_mode: BlendMode::Alpha,
    }]);
    core.texture_combos = ZeroVec::alloc_from_slice(&[0, 1, 2, 3]);
    core.texture_weight_combos = ZeroVec::alloc_from_slice(&[u16::MAX; 4]);
    core.texture_transform_combos = ZeroVec::alloc_from_slice(&[u16::MAX; 4]);
    core.texture_transform_bone_map = ZeroVec::alloc_from_slice(&[0, 0, 1, 1]);
    core.skins = vec![SkinProfile {
        vertices: ZeroVec::alloc_from_slice(&[0, 1, 2, 3]),
        indices: ZeroVec::alloc_from_slice(&[0, 1, 2, 0, 2, 3]),
        bones: ZeroVec::alloc_from_slice(&[[0; 4]; 4]),
        sections: vec![SkinSection {
            vertex_count: 4,
            index_count: 6,
            ..Default::default()
        }],
        batches: vec![Batch {
            color_index: -1,
            texture_count: count,
            shader_id: 0x8000 | selector,
            ..Default::default()
        }],
        ..Default::default()
    }];
    m2::Bundle::from_model(document)
}

fn texture(pixels: &[[u8; 4]]) -> Vec<u8> {
    let mut bytes = vec![0; 18];
    bytes[2] = 2;
    bytes[12..14].copy_from_slice(&4u16.to_le_bytes());
    bytes[14..16].copy_from_slice(&4u16.to_le_bytes());
    bytes[16] = 32;
    bytes[17] = 0x28;
    for &[r, g, b, a] in pixels {
        bytes.extend_from_slice(&[b, g, r, a]);
    }
    bytes
}

#[test]
#[ignore = "writes native material M2/TGA fixtures under /tmp for the existing capture example"]
fn write_native_material_capture_fixtures() {
    let directory = "/tmp/wc3-m2-material-fixtures";
    create_dir_all(directory).unwrap();
    for (name, rgba) in [
        ("primary", [102, 153, 204, 179]),
        ("secondary", [204, 77, 128, 102]),
        ("tertiary", [51, 230, 102, 153]),
    ] {
        write(format!("{directory}/{name}.tga"), texture(&[rgba; 16])).unwrap();
    }
    let pattern: Vec<_> = (0..16)
        .map(|i| match (i % 4 < 2, i / 4 < 2) {
            (true, true) => [240, 32, 16, 240],
            (false, true) => [16, 200, 64, 64],
            (true, false) => [32, 48, 240, 128],
            (false, false) => [220, 190, 32, 192],
        })
        .collect();
    write(format!("{directory}/pattern.tga"), texture(&pattern)).unwrap();
    for (selector, count) in [
        (11, 2),
        (18, 3),
        (19, 1),
        (20, 3),
        (21, 2),
        (22, 4),
        (24, 3),
        (25, 1),
        (26, 3),
        (27, 2),
        (28, 3),
        (32, 1),
        (33, 2),
        (34, 1),
        (35, 2),
    ] {
        let bundle = quad(selector, count, false);
        bundle.validate_complete().unwrap();
        let path = format!("{directory}/shader-{selector}.m2");
        write(&path, bundle.encode().unwrap().model).unwrap();
        println!("{path}");
    }
    let mut masked = quad(22, 4, false);
    masked.model.core_mut().materials = ZeroVec::alloc_from_slice(&[Material {
        flags: MaterialFlags::from_bits_retain(7),
        blending_mode: BlendMode::AlphaKey,
    }]);
    write(
        format!("{directory}/shader-22-mask.m2"),
        masked.encode().unwrap().model,
    )
    .unwrap();
    let mut primary_route = quad(17, 3, true);
    primary_route.model.core_mut().texture_transform_bone_map =
        ZeroVec::alloc_from_slice(&[0, 0, 0]);
    write(
        format!("{directory}/third-route-primary.m2"),
        primary_route.encode().unwrap().model,
    )
    .unwrap();
    let bundle = quad(24, 3, true);
    bundle.validate_complete().unwrap();
    write(
        format!("{directory}/third-route.m2"),
        bundle.encode().unwrap().model,
    )
    .unwrap();
    println!("{directory}/third-route.m2");
}
