//! Opt-in native WMO controls for the shared GPU capture example.
use std::array::from_fn;
use std::fs::{create_dir_all, write};
use wc3::model::conversion::ImportOptions;
use wc3::model::formats::wmo::{
    codec::Array, group::GroupChunk, records, root::RootChunk, Bundle, Classic, FileKey, Group,
    GroupKey, Root,
};
use wc3::model::io::wmo as semantic_wmo;

fn quad(shader: u32, layer_alpha: u8, scrolling: bool) -> Bundle<'static, Classic> {
    let bounds = [[-1., -0.01, -1.], [1., 0.01, 1.]];
    let mut root = Root::<Classic>::new();
    root.header_mut().bounds = bounds;
    root.header_mut().tail.flags = records::RootFlags::from(1);
    let mut names = Vec::new();
    let offsets = ["primary.tga", "pattern.tga", "tertiary.tga"].map(|name| {
        let offset = names.len() as u32;
        names.extend_from_slice(name.as_bytes());
        names.push(0);
        offset
    });
    root.insert_chunk(
        0,
        RootChunk::TextureNames(Array::from_records(&names).unwrap()),
    )
    .unwrap();
    root.insert_chunk(
        1,
        RootChunk::Materials(
            Array::from_records(&[records::Material::<Classic> {
                flags: records::MaterialFlags::from(7),
                shader,
                blend_mode: records::BlendMode::Alpha,
                texture0: offsets[0],
                texture1: offsets[1],
                texture2: offsets[2],
                diffuse_color: [255; 4],
                ..Default::default()
            }])
            .unwrap(),
        ),
    )
    .unwrap();
    root.insert_chunk(
        2,
        RootChunk::Groups(
            Array::from_records(&[records::GroupInfo {
                flags: records::GroupFlags::from(12),
                bounds,
                ..Default::default()
            }])
            .unwrap(),
        ),
    )
    .unwrap();
    if scrolling {
        root.insert_chunk(
            3,
            RootChunk::UvAnimations(
                Array::from_records(&[records::MaterialUvAnimation {
                    speed0: [0.125, 0.],
                    speed1: [0., 0.25],
                }])
                .unwrap(),
            ),
        )
        .unwrap();
    }
    root.synchronize_counts().unwrap();
    let mut group = Group::<Classic>::new();
    group.header_mut().bounds_minimum = bounds[0];
    group.header_mut().bounds_maximum = bounds[1];
    group.header_mut().flags = records::GroupFlags::from(12);
    group.header_mut().fog_indices = [255; 4];
    let chunks = group.chunks_mut();
    chunks.push(GroupChunk::Positions(
        Array::from_records(&[[-1., 0., -1.], [1., 0., -1.], [1., 0., 1.], [-1., 0., 1.]]).unwrap(),
    ));
    chunks.push(GroupChunk::Normals(
        Array::from_records(&[[0., 1., 0.]; 4]).unwrap(),
    ));
    for coordinates in [
        [[0., 1.], [1., 1.], [1., 0.], [0., 0.]],
        [[1., 1.], [1., 0.], [0., 0.], [0., 1.]],
        [[0.25, 0.25], [0.75, 0.25], [0.75, 0.75], [0.25, 0.75]],
    ] {
        chunks.push(GroupChunk::TextureCoordinates(
            Array::from_records(&coordinates).unwrap(),
        ));
    }
    chunks.push(GroupChunk::VertexColors(
        Array::from_records(&[[0, 0, 0, 255]; 4]).unwrap(),
    ));
    chunks.push(GroupChunk::VertexColors(
        Array::from_records(&[[0, 0, 0, layer_alpha]; 4]).unwrap(),
    ));
    chunks.push(GroupChunk::Indices(
        Array::from_records(&[0u16, 1, 2, 0, 2, 3]).unwrap(),
    ));
    chunks.push(GroupChunk::Triangles(
        Array::from_records(&[records::Triangle::default(); 2]).unwrap(),
    ));
    chunks.push(GroupChunk::Batches(
        Array::from_records(&[records::Batch::<Classic> {
            index_count: 6,
            maximum_vertex: 3,
            ..Default::default()
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
    bundle
}

fn texture(pixels: &[[u8; 4]; 16]) -> Vec<u8> {
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

fn write_fixture(directory: &str, name: &str, bundle: Bundle<'static, Classic>) {
    bundle.validate_complete().unwrap();
    semantic_wmo::import(&bundle, &ImportOptions::default()).unwrap();
    let encoded = bundle.encode().unwrap();
    write(format!("{directory}/{name}.wmo"), encoded.root).unwrap();
    let group = encoded.groups.into_values().next().unwrap();
    write(format!("{directory}/{name}_000.wmo"), group).unwrap();
    println!("{directory}/{name}.wmo");
}

fn lighting_quad(interior: bool) -> Bundle<'static, Classic> {
    let mut bundle = quad(0, 255, false);
    // Native colors are BGRA: red ambient dominates, with a smaller green term.
    bundle.root.header_mut().ambient_color = [16, 48, 112, 255];
    let flags = records::GroupFlags::from(if interior { 0x2004 } else { 12 });
    for chunk in bundle.root.chunks_mut() {
        match chunk {
            RootChunk::Materials(_) => {
                *chunk = RootChunk::Materials(
                    Array::from_records(&[records::Material::<Classic> {
                        flags: records::MaterialFlags::from(6),
                        blend_mode: records::BlendMode::Opaque,
                        diffuse_color: [255; 4],
                        ..Default::default()
                    }])
                    .unwrap(),
                );
            }
            RootChunk::Groups(rows) => {
                let mut info = rows.get(0).unwrap().unwrap();
                info.flags = flags;
                *rows = Array::from_records(&[info]).unwrap();
            }
            _ => {}
        }
    }
    let group = bundle.group_mut(GroupKey { index: 0, lod: 0 }).unwrap();
    group.header_mut().flags = flags;
    group.header_mut().interior_batch_count = u16::from(interior);
    group.header_mut().exterior_batch_count = u16::from(!interior);
    let mut first = true;
    for chunk in group.chunks_mut() {
        if let GroupChunk::VertexColors(rows) = chunk {
            if first {
                *rows = Array::from_records(&[[24, 36, 48, 0]; 4]).unwrap();
                first = false;
            }
        }
    }
    bundle
}

#[test]
#[ignore = "writes native WMO/TGA controls under /tmp for the existing capture example"]
fn write_native_world_capture_fixtures() {
    let directory = "/tmp/wc3-wmo-material-fixtures";
    create_dir_all(directory).unwrap();
    write(
        format!("{directory}/primary.tga"),
        texture(&[[102, 153, 204, 89]; 16]),
    )
    .unwrap();
    let pattern = from_fn(|i| match (i % 4 < 2, i / 4 < 2) {
        (true, true) => [240, 32, 16, 240],
        (false, true) => [16, 200, 64, 64],
        (true, false) => [32, 48, 240, 128],
        (false, false) => [220, 190, 32, 192],
    });
    write(format!("{directory}/pattern.tga"), texture(&pattern)).unwrap();
    let third_pattern = from_fn(|i| {
        if (i % 4 + i / 4) % 2 == 0 {
            [40, 220, 120, 192]
        } else {
            [230, 40, 160, 64]
        }
    });
    write(format!("{directory}/tertiary.tga"), texture(&third_pattern)).unwrap();
    for (name, shader, alpha, scroll) in [
        ("reflection-3", 3, 128, false),
        ("reflection-5", 5, 128, false),
        ("layer-6-zero", 6, 0, false),
        ("layer-6-one", 6, 255, false),
        ("third-uv-18", 18, 128, true),
        ("layer-13-zero", 13, 0, true),
        ("layer-13-one", 13, 255, true),
        ("layered-15", 15, 128, false),
        ("unsupported-24", 24, 128, false),
    ] {
        let bundle = quad(shader, alpha, scroll);
        write_fixture(directory, name, bundle);
    }
    for interior in [false, true] {
        write_fixture(
            directory,
            if interior {
                "lighting-interior"
            } else {
                "lighting-exterior"
            },
            lighting_quad(interior),
        );
    }
    for (name, blend_mode) in [
        ("blend-modulate-add", records::BlendMode::ModulateAdd),
        (
            "blend-inverse-alpha-add",
            records::BlendMode::InverseSourceAlphaAdd,
        ),
    ] {
        let mut bundle = quad(0, 128, false);
        for chunk in bundle.root.chunks_mut() {
            if let RootChunk::Materials(rows) = chunk {
                let mut material = rows.get(0).unwrap().unwrap();
                material.blend_mode = blend_mode;
                *rows = Array::from_records(&[material]).unwrap();
            }
        }
        write_fixture(directory, name, bundle);
    }
    let mut alpha_key = quad(13, 128, false);
    for chunk in alpha_key.root.chunks_mut() {
        if let RootChunk::Materials(rows) = chunk {
            let mut material = rows.get(0).unwrap().unwrap();
            material.blend_mode = records::BlendMode::AlphaKey;
            *rows = Array::from_records(&[material]).unwrap();
        }
    }
    write_fixture(directory, "layer-13-alpha-key", alpha_key);
}

#[path = "wmo_shadow_tests.rs"]
mod shadow;
