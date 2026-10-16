//! Explicit vertex declarations are supplied by the caller, never guessed from a game revision.
use wc3::model::conversion::{ExportOptions, ImportOptions};
use wc3::model::formats::diablo::d4::{self, records as native};
use wc3::model::io::diablo::d4 as convert;
use wc3::model::{GeometrySchema as _, InfluenceRows};
use zerovec::ZeroVec;

fn layout() -> convert::VertexLayout {
    convert::VertexLayout {
        native_format: 71,
        stride: 20,
        elements: vec![
            convert::VertexElement {
                native_semantic: 101,
                native_format: 201,
                offset: 0,
                semantic: convert::ElementSemantic::Position,
                encoding: convert::ElementEncoding::F32x3,
            },
            convert::VertexElement {
                native_semantic: 102,
                native_format: 202,
                offset: 12,
                semantic: convert::ElementSemantic::TexCoord(0),
                encoding: convert::ElementEncoding::F32x2,
            },
        ],
    }
}
fn source() -> d4::Appearance<'static> {
    let mut vertices = Vec::new();
    for row in [
        [0., 0., 0., 0., 0.],
        [1., 0., 0., 1., 0.],
        [0., 1., 0., 0., 1.],
    ] {
        for value in row {
            vertices.extend(f32::to_le_bytes(value));
        }
    }
    let buffer = native::GeoChunkVertexBuffer {
        vb_format: 71,
        vert_stride: 20,
        vertex_elems: vec![
            native::VertexElem {
                semantic: 101,
                format: 201,
                offset: 0,
            },
            native::VertexElem {
                semantic: 102,
                format: 202,
                offset: 12,
            },
        ],
        chunk_vertices: ZeroVec::alloc_from_slice(&vertices),
        ..Default::default()
    };
    let sub = native::SubObject {
        material_index: 0,
        vert_buffer_index: 0,
        index_buffer_index: 0,
        base_lod_sub_object_index: -1,
        sub_object_hash: 0x1234,
        fixed_point_pos_scale: [1.; 3],
        segments: vec![native::SubObjectSegment {
            vert_count: 3,
            index_count: 3,
            ..Default::default()
        }],
        aabb_bounds: native::Bounds {
            wp: [0.5, 0.5, 0.],
            ext: [0.5, 0.5, 0.],
        },
        ..Default::default()
    };
    let chunk = native::GeoChunk {
        aabb_bounds: sub.aabb_bounds.clone(),
        lods: vec![native::GeoSet {
            sub_objects: vec![sub],
            lod_distance: 100.,
            ..Default::default()
        }],
        chunk_vertex_buffers: vec![buffer],
        chunk_index_buffers: vec![native::GeoChunkIndexBuffer {
            pdw_chunk_indices: ZeroVec::alloc_from_slice(&[0, 1, 2]),
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut doc = d4::Appearance::default();
    doc.root.sno_id = 321;
    doc.root.structure.aabb_bounds = chunk.aabb_bounds.clone();
    doc.root.structure.bounds = native::Sphere {
        center: [0.25, 0.25, 0.],
        wd_radius: 2.,
    };
    doc.root.structure.chunks.push(chunk);
    doc.root.lod_distance_multiplier = 1.;
    doc.root.deformation_scale = 1.;
    doc.root
        .appearance_materials
        .push(native::AppearanceMaterial {
            material_hash: 99,
            persona: 3,
            persona_material: 88,
            so_as: vec![native::SubObjectAppearance {
                material: 66,
                ..Default::default()
            }],
            ..Default::default()
        });
    doc.root
        .appearance_looks
        .push(native::AppearanceLook { look_name: 55 });
    doc
}
#[test]
fn declared_geometry_material_slots_and_independent_sphere_round_trip() {
    let source = source();
    let layouts = [convert::BufferLayout {
        chunk: 0,
        buffer: 0,
        layout: layout(),
    }];
    let imported =
        convert::import_with_layouts(&source, &layouts, &ImportOptions::default()).unwrap();
    assert!(imported.report.issues.is_empty());
    let options = ExportOptions::default();
    let model = &imported.value;
    let (_, mesh) = model.meshes().iter().next().unwrap();
    assert_eq!(mesh.positions().get(1), Some([1., 0., 0.]));
    assert_eq!(mesh.texture_coordinates()[0].get(2), Some([0., 1.]));
    assert!(matches!(mesh.geometry.influences(), InfluenceRows::None));
    assert_eq!(model.metadata().structure_sphere_center, [0.25, 0.25, 0.]);
    assert_eq!(model.metadata().structure_sphere_radius, 2.);
    let (_, slot) = model.appearance().slots().next().unwrap();
    assert_eq!(
        (slot.hash, slot.persona_material, slot.persona),
        (99, 88, 3)
    );
    assert_eq!(slot.variants().len(), 1);
    assert_eq!(model.appearance().alternatives().next().unwrap().1.hash, 55);
    let plan = convert::plan(&imported.value, &layout(), &options).unwrap();
    let native = plan.execute().unwrap().value;
    assert_eq!(
        native.root.structure.chunks[0].lods[0].sub_objects[0].segments[0].vert_count,
        3
    );
    assert_eq!(native.root.appearance_materials[0].so_as[0].material, 66);
    let encoded = native.write_files().unwrap();
    let decoded = d4::Appearance::read_with_payload(&encoded.root, &encoded.payload).unwrap();
    let second =
        convert::import_with_layouts(&decoded, &layouts, &ImportOptions::default()).unwrap();
    let model = &second.value;
    assert_eq!(
        model.meshes().iter().next().unwrap().1.positions().get(1),
        Some([1., 0., 0.])
    );
}
#[test]
fn unmapped_and_duplicate_native_vertex_declarations_fail_explicitly() {
    let mut source = source();
    assert!(convert::import(&source, &ImportOptions::default()).is_err());
    let layout = layout();
    let buffer = &mut source.root.structure.chunks[0].chunk_vertex_buffers[0];
    buffer.vertex_elems[1] = buffer.vertex_elems[0].clone();
    assert!(layout.validate_buffer(buffer).is_err());
}
#[test]
fn invalid_layouts_do_not_reinterpret_bytes() {
    let mut bad = layout();
    bad.elements[1].offset = 8;
    assert!(bad.validate().is_err());
    let mut bad = layout();
    bad.elements[0].encoding = convert::ElementEncoding::F32x2;
    assert!(bad.validate().is_err());
    let mut bad = layout();
    bad.elements[1].semantic = convert::ElementSemantic::TexCoord(2);
    assert!(bad.validate().is_err());
}

#[test]
fn segment_palettes_map_actual_bones_and_rebuild_inverse_skinning_bindings() {
    let mut source = source();
    let identity = native::PRSTransform {
        q: native::BcQuat {
            w: 1.,
            ..Default::default()
        },
        scale: [1.; 3],
        ..Default::default()
    };
    let bone = |hash, parent_index| native::BoneStructure {
        hash,
        parent_index,
        transform: identity.clone(),
        transform_inv: identity.clone(),
        transform_parent_rel: identity.clone(),
        transform_skinning_inv: identity.clone(),
        particle_system: -1,
        ..Default::default()
    };
    source.root.structure.bone_data.push(native::BoneData {
        base_bone_count: 2,
        bone_structure: vec![bone(10, u16::MAX), bone(20, 0)],
        hardpoints: vec![native::Hardpoint {
            info: native::HardpointInfo {
                hash: 30,
                hash_flag: 3,
            },
            bone_index: 1,
            ignore_parent_orientation: 1,
            transform: native::PRTransform {
                q: identity.q.clone(),
                ..Default::default()
            },
        }],
        ..Default::default()
    });
    let mut declared = layout();
    declared.stride = 40;
    declared.elements.extend([
        convert::VertexElement {
            native_semantic: 103,
            native_format: 203,
            offset: 20,
            semantic: convert::ElementSemantic::JointIndices,
            encoding: convert::ElementEncoding::Uint8x4,
        },
        convert::VertexElement {
            native_semantic: 104,
            native_format: 204,
            offset: 24,
            semantic: convert::ElementSemantic::JointWeights,
            encoding: convert::ElementEncoding::F32x4,
        },
    ]);
    let chunk = &mut source.root.structure.chunks[0];
    let buffer = &mut chunk.chunk_vertex_buffers[0];
    let old = buffer.chunk_vertices.as_bytes().to_vec();
    let mut bytes = Vec::new();
    for i in 0..3 {
        bytes.extend_from_slice(&old[i * 20..i * 20 + 20]);
        bytes.extend([0, 1, 0, 0]);
        for weight in [0.25f32, 0.75, 0., 0.] {
            bytes.extend(weight.to_le_bytes());
        }
    }
    buffer.vert_stride = 40;
    buffer.chunk_vertices = ZeroVec::alloc_from_slice(&bytes);
    buffer.vertex_elems = declared
        .elements
        .iter()
        .map(|e| native::VertexElem {
            semantic: e.native_semantic,
            format: e.native_format,
            offset: e.offset as i32,
        })
        .collect();
    chunk.lods[0].sub_objects[0].segments[0].bone_ids = ZeroVec::alloc_from_slice(&[1, 0]);
    let layouts = [convert::BufferLayout {
        chunk: 0,
        buffer: 0,
        layout: declared.clone(),
    }];
    let imported =
        convert::import_with_layouts(&source, &layouts, &ImportOptions::default()).unwrap();
    let options = ExportOptions::default();
    let model = &imported.value;
    let row = model
        .meshes()
        .iter()
        .next()
        .unwrap()
        .1
        .geometry
        .data()
        .influences
        .rows()
        .joint_weights(0)
        .unwrap();
    assert_eq!((row[0].joint, row[0].weight), (1, 0.25));
    assert_eq!((row[1].joint, row[1].weight), (0, 0.75));
    let native = convert::plan(&imported.value, &declared, &options)
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(
        native.root.structure.bone_data[0].bone_structure[1].parent_index,
        0
    );
    assert_eq!(
        native.root.structure.bone_data[0].hardpoints[0]
            .info
            .hash_flag,
        3
    );
    assert_eq!(
        native.root.structure.chunks[0].lods[0].sub_objects[0].segments[0]
            .bone_ids
            .iter()
            .collect::<Vec<_>>(),
        [1, 0]
    );
    assert_eq!(
        &native.root.structure.chunks[0].chunk_vertex_buffers[0]
            .chunk_vertices
            .as_bytes()[20..24],
        &[0, 1, 0, 0]
    );
}

#[test]
fn unsupported_hardpoint_sentinel_is_not_collapsed_into_an_absent_parent() {
    let mut source = source();
    source.root.structure.bone_data.push(native::BoneData {
        hardpoints: vec![native::Hardpoint {
            bone_index: -2,
            transform: native::PRTransform {
                q: native::BcQuat {
                    w: 1.,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }],
        ..Default::default()
    });
    let layouts = [convert::BufferLayout {
        chunk: 0,
        buffer: 0,
        layout: layout(),
    }];
    assert!(convert::import_with_layouts(&source, &layouts, &ImportOptions::default()).is_err());
}
