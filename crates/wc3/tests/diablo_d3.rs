//! Independently constructed SNO byte images exercise native address spaces.
use wc3::model::common::{Borrowed, IntoOwned};
use wc3::model::formats::diablo::d3::{
    self,
    records::{FatVertex, MaterialTextureEntry, SubObject},
    Appearance, V260,
};
use zerovec::ZeroVec;

fn i32_at(bytes: &mut [u8], at: usize, value: i32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
fn u32_at(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
fn f32_at(bytes: &mut [u8], at: usize, value: f32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
fn descriptor(bytes: &mut [u8], at: usize, offset: i32, length: i32) {
    i32_at(bytes, at, offset);
    i32_at(bytes, at + 4, length);
}
fn envelope(version: u32, payload: Vec<u8>) -> Vec<u8> {
    let mut bytes = vec![0; 16];
    u32_at(&mut bytes, 0, 0xDEADBEEF);
    u32_at(&mut bytes, 4, version);
    bytes.extend(payload);
    bytes
}
fn appearance_image() -> Vec<u8> {
    // 536-byte v260 root, 312-byte bone, 400-byte subobject,
    // three 44-byte fat vertices, three 24-byte influences, and three u16 indices.
    let mut p = vec![0; 1458];
    i32_at(&mut p, 0, 123);
    i32_at(&mut p, 20, 1);
    descriptor(&mut p, 24, 536, 312);
    p[536..540].copy_from_slice(b"Root");
    i32_at(&mut p, 536 + 64, -1);
    for t in [108, 140, 172, 204, 236] {
        f32_at(&mut p, 536 + t + 12, 1.0);
        f32_at(&mut p, 536 + t + 28, 1.0);
    }
    i32_at(&mut p, 152, 1);
    descriptor(&mut p, 156, 848, 400);
    i32_at(&mut p, 848 + 4, 3);
    descriptor(&mut p, 848 + 8, 1248, 132);
    descriptor(&mut p, 848 + 24, 1380, 72);
    i32_at(&mut p, 848 + 40, 3);
    descriptor(&mut p, 848 + 44, 1452, 6);
    p[848 + 92..848 + 96].copy_from_slice(b"Body");
    for (i, xyz) in [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]
        .into_iter()
        .enumerate()
    {
        for (j, x) in xyz.into_iter().enumerate() {
            f32_at(&mut p, 1248 + i * 44 + j * 4, x);
        }
        u32_at(&mut p, 1248 + i * 44 + 12, 0x00ff8080);
        u32_at(&mut p, 1248 + i * 44 + 16, 0x44332211);
        u32_at(&mut p, 1248 + i * 44 + 24, 0x80008000);
        f32_at(&mut p, 1380 + i * 24 + 4, 1.0);
    }
    p[1452..].copy_from_slice(&[0, 0, 1, 0, 2, 0]);
    envelope(260, p)
}
#[test]
fn independent_appearance_decodes_borrowed_geometry_and_relocates_edits() {
    let bytes = appearance_image();
    let mut doc = Appearance::<V260>::read(&bytes).unwrap();
    assert_eq!(doc.root.sno_id, 123);
    assert_eq!(doc.root.bones[0].parent_index, -1);
    let sub = &doc.root.geo_set0.sub_objects[0];
    assert_eq!(sub.vertices.len(), 3);
    assert!(!sub.vertices.is_owned());
    let v = sub.vertices.get(1).unwrap();
    assert_eq!(v.position, [1., 0., 0.]);
    assert_eq!(v.rgba(), [0x11, 0x22, 0x33, 0x44]);
    assert_eq!(v.uv0(), [0., 0.]);
    assert_eq!(sub.indices.iter().collect::<Vec<_>>(), [0, 1, 2]);
    assert_eq!(v.normal_vector()[2], 1.0);
    doc.root.geo_set0.sub_objects[0].vertices = ZeroVec::alloc_from_slice(&[v; 4]);
    let encoded = doc.write().unwrap();
    let reread = Appearance::<V260>::read(&encoded).unwrap();
    assert_eq!(reread.root.geo_set0.sub_objects[0].vertex_count, 4);
    assert_eq!(reread.root.geo_set0.sub_objects[0].vertices.len(), 4);
    assert_eq!(reread.root.bones[0].transform0.scale, 1.0);
    let detached = doc.into_owned();
    drop(bytes);
    assert_eq!(detached.root.sno_id, 123);
    let view = detached.as_borrowed();
    assert!(!view.root.geo_set0.sub_objects[0].vertices.is_owned());
}
#[test]
fn selected_revision_and_descriptor_failures_are_explicit() {
    let mut bytes = appearance_image();
    u32_at(&mut bytes, 4, 306);
    assert!(matches!(
        Appearance::<V260>::read(&bytes),
        Err(d3::ReadError::Revision {
            expected: 260,
            actual: 306
        })
    ));
    let mut bytes = appearance_image();
    i32_at(&mut bytes, 16 + 24, -1);
    assert!(matches!(
        Appearance::<V260>::read(&bytes),
        Err(d3::ReadError::Descriptor { .. })
    ));
    let mut bytes = appearance_image();
    i32_at(&mut bytes, 16 + 24 + 4, 311);
    assert!(matches!(
        Appearance::<V260>::read(&bytes),
        Err(d3::ReadError::Descriptor { .. })
    ));
    let mut bytes = appearance_image();
    i32_at(&mut bytes, 16 + 20, 2);
    assert!(matches!(
        Appearance::<V260>::read(&bytes),
        Err(d3::ReadError::Count { .. })
    ));
    let mut bytes = appearance_image();
    i32_at(&mut bytes, 16 + 848, 0x8000);
    assert!(matches!(
        Appearance::<V260>::read(&bytes),
        Err(d3::ReadError::VertexFormat(0x8000))
    ));
}
#[test]
fn standalone_material_counted_tags_and_texture_modes_round_trip() {
    let mut p = vec![0; 352];
    i32_at(&mut p, 0, 99);
    descriptor(&mut p, 16, 136, 16);
    i32_at(&mut p, 136, 1);
    i32_at(&mut p, 140, 2);
    u32_at(&mut p, 144, 0x10203040);
    u32_at(&mut p, 148, 0x7fc01234);
    i32_at(&mut p, 32, 77);
    descriptor(&mut p, 32 + 76, 152, 160);
    i32_at(&mut p, 152, 4);
    i32_at(&mut p, 152 + 8, 1234);
    i32_at(&mut p, 152 + 12, 6);
    i32_at(&mut p, 152 + 152, 3);
    let bytes = envelope(25, p);
    let doc = d3::Material::<d3::V25>::read(&bytes).unwrap();
    assert_eq!(doc.root.shader_params.len(), 1);
    assert_eq!(doc.root.shader_params.get(0).unwrap().value, 0x7fc01234);
    let texture = doc.root.material.textures.get(0).unwrap();
    assert_eq!(
        (
            texture.texture_type,
            texture.uv_transform_mode,
            texture.uv_flags
        ),
        (4, 6, 3)
    );
    let out = doc.write().unwrap();
    let reread = d3::Material::<d3::V25>::read(&out).unwrap();
    assert_eq!(reread.root, doc.root);
}
#[test]
fn every_supported_root_can_be_authored_and_read() {
    macro_rules! check {
        ($family:ty) => {{
            let doc = <$family>::default();
            let bytes = doc.write().unwrap();
            let parsed = <$family>::read(&bytes).unwrap();
            assert_eq!(parsed.root, doc.root);
        }};
    }
    check!(d3::Appearance);
    check!(d3::Actor);
    check!(d3::Animation);
    check!(d3::AnimationSet);
    check!(d3::Material);
    check!(d3::ShaderMapDocument);
    check!(d3::Shader);
    let _ = FatVertex::default();
    let _ = MaterialTextureEntry::default();
    let _ = SubObject::default();
}

#[test]
fn independent_animation_curves_borrow_and_relocate_with_their_own_revision() {
    let mut p = vec![0; 672];
    i32_at(&mut p, 0, 5);
    i32_at(&mut p, 20, 6);
    descriptor(&mut p, 24, 56, 408);
    i32_at(&mut p, 32, 1);
    p[60..64].copy_from_slice(b"Walk");
    for at in [72, 80, 84, 88] {
        f32_at(&mut p, 56 + at, 1.);
    }
    i32_at(&mut p, 56 + 120, 1);
    descriptor(&mut p, 56 + 124, 464, 64);
    p[464..468].copy_from_slice(b"Root");
    i32_at(&mut p, 56 + 144, 2);
    descriptor(&mut p, 56 + 148, 528, 24);
    descriptor(&mut p, 56 + 168, 584, 24);
    descriptor(&mut p, 56 + 184, 632, 24);
    i32_at(&mut p, 528, 2);
    descriptor(&mut p, 532, 552, 32);
    i32_at(&mut p, 568, 30);
    for (i, value) in [1., 2., 3.].into_iter().enumerate() {
        f32_at(&mut p, 572 + i * 4, value);
    }
    i32_at(&mut p, 584, 2);
    descriptor(&mut p, 588, 608, 24);
    p[618..620].copy_from_slice(&32767u16.to_le_bytes());
    i32_at(&mut p, 620, 30);
    p[628..630].copy_from_slice(&32767u16.to_le_bytes());
    i32_at(&mut p, 632, 2);
    descriptor(&mut p, 636, 656, 16);
    f32_at(&mut p, 660, 1.);
    i32_at(&mut p, 664, 30);
    f32_at(&mut p, 668, 2.);
    let bytes = envelope(118, p);
    let mut doc = d3::Animation::<d3::V118>::read(&bytes).unwrap();
    let perm = &doc.root.permutations[0];
    assert_eq!(perm.bone_count, 1);
    assert_eq!(perm.frame_count, 2);
    assert!(!perm.translation_curves[0].keys.is_owned());
    assert_eq!(
        perm.translation_curves[0].keys.get(1).unwrap().position,
        [1., 2., 3.]
    );
    assert_eq!(
        perm.rotation_curves[0]
            .keys
            .get(0)
            .unwrap()
            .rotation
            .components(),
        [0., 0., 0., 1.]
    );
    let mut keys = perm.translation_curves[0].keys.iter().collect::<Vec<_>>();
    keys.push(d3::records::TranslationKey {
        frame: 60,
        position: [4., 5., 6.],
    });
    doc.root.permutations[0].translation_curves[0].keys = ZeroVec::alloc_from_slice(&keys);
    let out = doc.write().unwrap();
    let decoded = d3::Animation::<d3::V118>::read(&out).unwrap();
    assert_eq!(
        decoded.root.permutations[0].translation_curves[0].key_count,
        3
    );
    assert_eq!(
        decoded.root.permutations[0].translation_curves[0]
            .keys
            .get(2)
            .unwrap()
            .position,
        [4., 5., 6.]
    );
}
