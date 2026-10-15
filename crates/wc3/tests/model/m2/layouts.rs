//! Independent wire-layout checks. Fixture bytes are constructed from documented
//! offsets and literal records, without using the production encoder.
use wc3::model::formats::m2::codec::{Array, ArrayRef};
use wc3::model::formats::m2::companions::{
    AnimationFile, BoneFile, PhysicsChunk, PhysicsFile, Skeleton, SkeletonChunk, Skin,
};
use wc3::model::formats::m2::records::ModelFlags;
use wc3::model::formats::m2::records::Validate as _;
use wc3::model::formats::m2::records::{BlendMode, PhysicsJointKind, PhysicsShapeKind};
use wc3::model::formats::m2::{
    Chunked, Dialect, Document, Flat, IntoZeroVec, ModelLayout, V256, V257, V260, V263, V264, V265,
    V272, V273, V274,
};

fn word(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
fn frame(tag: [u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut result = tag.to_vec();
    result.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    result.extend_from_slice(payload);
    result
}
fn physics_frame(mut tag: [u8; 4], payload: &[u8]) -> Vec<u8> {
    tag.reverse();
    frame(tag, payload)
}

fn core_fixture(version: u32) -> Vec<u8> {
    // Modern header is 0x130 bytes. Legacy adds playable lookup, inline skin
    // descriptor (replacing a word), and texture flipbooks: 20 additional bytes.
    let header = if version <= 257 {
        340
    } else if version < 264 {
        324
    } else {
        304
    };
    let mut bytes = vec![0; header + 11];
    bytes[..4].copy_from_slice(b"MD20");
    word(&mut bytes, 4, version);
    word(&mut bytes, 8, 5);
    word(&mut bytes, 12, (header + 1) as u32);
    word(&mut bytes, 20, 1);
    word(&mut bytes, 24, (header + 7) as u32);
    // Empty arrays do not dereference their arbitrary offset words.
    word(&mut bytes, 40, u32::MAX);
    bytes[header + 1..header + 6].copy_from_slice(b"unit\0");
    word(&mut bytes, header + 7, 1234);
    bytes
}
fn dialect_fixture<D: Dialect>() -> Vec<u8> {
    let core = core_fixture(D::VERSION);
    if Document::<D>::new().is_chunked() {
        frame(*b"MD21", &core)
    } else {
        core
    }
}
fn check_dialect<D: Dialect>() {
    let bytes = dialect_fixture::<D>();
    let model = Document::<D>::decode_m2(&bytes).unwrap();
    assert_eq!(model.version(), D::VERSION);
    assert_eq!(model.is_chunked(), Document::<D>::new().is_chunked());
    assert_eq!(model.core().name.as_bytes(), b"unit\0");
    assert_eq!(model.core().global_sequences.get(0), Some(1234));
    let origin = if Document::<D>::new().is_chunked() {
        8
    } else {
        0
    };
    let header = if D::VERSION <= 257 {
        340
    } else if D::VERSION < 264 {
        324
    } else {
        304
    };
    assert_eq!(
        model.core().name.as_bytes().as_ptr(),
        bytes[origin + header + 1..].as_ptr()
    );
    assert_eq!(
        model.core().global_sequences.as_bytes().as_ptr(),
        bytes[origin + header + 7..].as_ptr()
    );
    assert!(model.core().embedded_skins().is_empty());
    assert_eq!(model.core().external_skin_count(), 0);
    let written = model.encode_m2().unwrap();
    let decoded = Document::<D>::decode_m2(&written).unwrap();
    assert_eq!(decoded.core(), model.core());
}

#[test]
fn every_supported_layout_reads_both_envelopes_from_independent_offsets() {
    fn both<V: ModelLayout>() {
        check_dialect::<Flat<V>>();
        check_dialect::<Chunked<V>>();
    }
    both::<V256>();
    both::<V257>();
    both::<V260>();
    both::<V263>();
    both::<V264>();
    both::<V265>();
    both::<V272>();
    both::<V273>();
    both::<V274>();
}

#[test]
fn resizing_arrays_relocates_payloads_without_overwriting_other_fields() {
    let bytes = dialect_fixture::<Chunked<V274>>();
    let mut model = Document::<Chunked<V274>>::decode_m2(&bytes).unwrap();
    let name_pointer = model.core().name.as_bytes().as_ptr();
    model.core_mut().global_sequences = [11, 22, 33, 44].into_zero_vec();
    assert_eq!(model.core().name.as_bytes().as_ptr(), name_pointer);
    let written = model.encode_m2().unwrap();
    let decoded = Document::<Chunked<V274>>::decode_m2(&written).unwrap();
    assert_eq!(decoded.core().name.as_bytes(), b"unit\0");
    assert_eq!(
        decoded.core().global_sequences.iter().collect::<Vec<_>>(),
        [11, 22, 33, 44]
    );
}

#[test]
fn malformed_offsets_counts_versions_and_chunk_lengths_are_rejected() {
    let mut bytes = core_fixture(274);
    word(&mut bytes, 24, u32::MAX);
    assert!(Document::<Flat<V274>>::decode_m2(&bytes).is_err());
    let mut bytes = core_fixture(274);
    word(&mut bytes, 20, u32::MAX);
    assert!(Document::<Flat<V274>>::decode_m2(&bytes).is_err());
    let mut bytes = core_fixture(274);
    word(&mut bytes, 44, u32::MAX); // Bone count, before any allocation.
    assert!(Document::<Flat<V274>>::decode_m2(&bytes).is_err());
    assert!(Document::<Flat<V273>>::decode_m2(&core_fixture(274)).is_err());
    let mut bytes = dialect_fixture::<Chunked<V274>>();
    word(&mut bytes, 4, u32::MAX);
    assert!(Document::<Chunked<V274>>::decode_m2(&bytes).is_err());
    let mut bytes = dialect_fixture::<Chunked<V274>>();
    bytes.extend_from_slice(b"short");
    assert!(Document::<Chunked<V274>>::decode_m2(&bytes).is_err());
}

#[test]
fn external_array_descriptors_resolve_only_against_the_supplied_stream() {
    let array = Array::<u32>::External(ArrayRef {
        count: 1,
        offset: 1,
    });
    let bytes = [0, 42, 0, 0, 0];
    let values = array.resolve(&bytes).unwrap();
    assert_eq!(values.get(0), Some(42));
    assert_eq!(values.as_bytes().as_ptr(), bytes[1..].as_ptr());
    assert!(array.resolve(&bytes[..4]).is_err());
}

#[test]
fn animation_streams_require_explicit_framing_and_keep_unknown_streams() {
    let raw = [1, 2, 3, 4, 5];
    let animation = AnimationFile::read(&raw, false).unwrap();
    assert_eq!(animation.stream(*b"AFM2").unwrap(), raw);
    assert_eq!(animation.write().unwrap(), raw);
    let mut chunked = frame(*b"AFM2", &raw);
    chunked.extend(frame(*b"XXXX", &[9, 8]));
    let animation = AnimationFile::read(&chunked, true).unwrap();
    assert_eq!(animation.stream(*b"XXXX").unwrap(), [9, 8]);
    assert_eq!(animation.write().unwrap(), chunked);
    assert!(AnimationFile::read(&chunked[..chunked.len() - 1], true).is_err());
}

#[test]
fn external_skins_use_document_offsets_and_four_bone_bytes_per_vertex() {
    fn check<V: wc3::model::formats::m2::companions::SupportsExternalSkins>() {
        let version = V::VERSION;
        let header = if version < 265 { 48 } else { 64 };
        let mut bytes = vec![0; header + 4];
        bytes[..4].copy_from_slice(b"SKIN");
        word(&mut bytes, 20, 1); // third descriptor: bone tuple count
        word(&mut bytes, 24, header as u32);
        bytes[header..].copy_from_slice(&[1, 2, 3, 4]);
        let skin = Skin::<V>::read(&bytes).unwrap();
        assert_eq!(skin.profile.bones.get(0), Some([1, 2, 3, 4]));
        assert_eq!(
            skin.profile.bones.as_bytes().as_ptr(),
            bytes[header..].as_ptr()
        );
        let written = skin.write().unwrap();
        assert_eq!(Skin::<V>::read(&written).unwrap().profile, skin.profile);
        assert!(Skin::<V>::read(&bytes[..bytes.len() - 1]).is_err());
    }
    check::<V264>();
    check::<V265>();
    check::<V272>();
    check::<V274>();
    assert!(Skin::<V264>::read(b"SKIN").is_err());
}

#[test]
fn bone_file_matrices_and_indices_keep_their_native_wire_values() {
    let mut bytes = 1u32.to_le_bytes().to_vec();
    bytes.extend(frame(*b"BIDA", &7u16.to_le_bytes()));
    let mut matrix = vec![0; 64];
    for axis in 0..4 {
        word(&mut matrix, (axis * 4 + axis) * 4, 1f32.to_bits());
    }
    bytes.extend(frame(*b"BOMT", &matrix));
    let bone = BoneFile::read(&bytes).unwrap();
    assert_eq!(bone.indices.get(0), Some(7));
    assert_eq!(bone.matrices[0].values[15], 1.0);
    assert_eq!(bone.write().unwrap(), bytes);
    bytes.pop();
    assert!(BoneFile::read(&bytes).is_err());
}

#[test]
fn skeleton_offsets_are_relative_to_each_chunk_payload() {
    let mut header = vec![0; 21];
    word(&mut header, 0, 17);
    word(&mut header, 4, 5);
    word(&mut header, 8, 16);
    header[12..16].copy_from_slice(&[1, 2, 3, 4]);
    header[16..].copy_from_slice(b"rig\0\0");
    let mut bytes = frame(*b"XXXX", &[1]);
    bytes.extend(frame(*b"SKL1", &header));
    let skeleton = Skeleton::<V274>::read(&bytes, ModelFlags::default()).unwrap();
    let SkeletonChunk::Header {
        flags,
        name,
        reserved,
    } = &skeleton.chunks[1]
    else {
        panic!("skeleton header")
    };
    assert_eq!(flags.bits(), 17);
    assert_eq!(name.as_bytes(), b"rig\0\0");
    assert_eq!(*reserved, [1, 2, 3, 4]);
    let written = skeleton.write().unwrap();
    assert_eq!(
        Skeleton::<V274>::read(&written, ModelFlags::default()).unwrap(),
        skeleton
    );
}

#[test]
fn physics_versions_select_their_actual_body_records() {
    for version in 0u16..=6 {
        let (tag, size) = match version {
            0 | 1 => (*b"BODY", 28),
            2 => (*b"BDY2", 32),
            3 => (*b"BDY3", 44),
            _ => (*b"BDY4", 48),
        };
        let mut body = vec![0; size];
        body[..2].copy_from_slice(&1u16.to_le_bytes());
        word(&mut body, 4, 2.5f32.to_bits());
        if version < 3 {
            word(&mut body, 16, 9);
        } else {
            body[2..4].copy_from_slice(&9u16.to_le_bytes());
        }
        let mut bytes = physics_frame(*b"PHYS", &version.to_le_bytes());
        bytes.extend(physics_frame(tag, &body));
        let physics = PhysicsFile::read(&bytes).unwrap();
        match &physics.chunks[1] {
            PhysicsChunk::Bodies(values) => {
                assert_eq!(values[0].bone_index, 9);
                assert_eq!(values[0].position[0], 2.5);
            }
            PhysicsChunk::Bodies2(values) => {
                assert_eq!(values[0].body.bone_index, 9);
                assert_eq!(values[0].body.position[0], 2.5);
            }
            PhysicsChunk::Bodies3(values) => {
                assert_eq!(values[0].bone_index, 9);
                assert_eq!(values[0].position[0], 2.5);
            }
            PhysicsChunk::Bodies4(values) => {
                assert_eq!(values[0].bone_index, 9);
                assert_eq!(values[0].position[0], 2.5);
            }
            _ => panic!("version-selected body"),
        }
        assert_eq!(physics.write().unwrap(), bytes);
        let inline = physics.write_inline().unwrap();
        assert!(inline.len().is_multiple_of(16));
        assert_eq!(
            PhysicsFile::read_inline(&inline).unwrap().chunks,
            physics.chunks
        );
    }
}

#[test]
fn physics_rejects_wrong_version_records_and_duplicate_headers() {
    let mut bytes = physics_frame(*b"PHYS", &0u16.to_le_bytes());
    bytes.extend(physics_frame(*b"BDY4", &[0; 48]));
    assert!(PhysicsFile::read(&bytes).is_err());
    let mut bytes = physics_frame(*b"PHYS", &6u16.to_le_bytes());
    bytes.extend(physics_frame(*b"WLJ2", &[]));
    assert!(PhysicsFile::read(&bytes).is_err());
    let mut bytes = physics_frame(*b"PHYS", &6u16.to_le_bytes());
    bytes.extend(physics_frame(*b"PHYS", &6u16.to_le_bytes()));
    assert!(PhysicsFile::read(&bytes).is_err());
}

#[test]
fn native_sequence_timing_and_blend_words_follow_independent_layouts() {
    fn check<V: ModelLayout>() {
        let mut bytes = core_fixture(V::VERSION);
        let offset = bytes.len();
        let legacy = V::VERSION < 264;
        bytes.resize(offset + if legacy { 68 } else { 64 }, 0);
        word(&mut bytes, 28, 1);
        word(&mut bytes, 32, offset as u32);
        bytes[offset..offset + 2].copy_from_slice(&7u16.to_le_bytes());
        word(&mut bytes, offset + 4, if legacy { 100 } else { 900 });
        if legacy {
            word(&mut bytes, offset + 8, 1000);
        }
        let shift = if legacy { 4 } else { 0 };
        word(&mut bytes, offset + 12 + shift, 0x20); // Locally stored sequence.
        bytes[offset + 16 + shift..offset + 18 + shift].copy_from_slice(&(-4i16).to_le_bytes());
        if V::VERSION < 272 {
            word(&mut bytes, offset + 28 + shift, 350);
        } else {
            bytes[offset + 28..offset + 30].copy_from_slice(&150u16.to_le_bytes());
            bytes[offset + 30..offset + 32].copy_from_slice(&275u16.to_le_bytes());
        }
        let model = Document::<Flat<V>>::decode_m2(&bytes).unwrap();
        let sequence = &model.core().sequences[0];
        assert_eq!(sequence.id, 7);
        assert_eq!(sequence.frequency, -4);
        let written = model.encode_m2().unwrap();
        let relocated = u32::from_le_bytes(written[32..36].try_into().unwrap()) as usize;
        assert_eq!(
            &written[relocated..relocated + if legacy { 68 } else { 64 }],
            &bytes[offset..]
        );
        assert_eq!(
            Document::<Flat<V>>::decode_m2(&written)
                .unwrap()
                .core()
                .sequences,
            model.core().sequences
        );
    }
    check::<V256>();
    check::<V257>();
    check::<V260>();
    check::<V263>();
    check::<V264>();
    check::<V265>();
    check::<V272>();
    check::<V273>();
    check::<V274>();
}

#[test]
fn polytope_topology_has_native_signed_edges_and_borrowed_geometry() {
    // PLYT: one 80-byte header, one vertex, one plane, one face edge index,
    // and one four-byte edge. Arrays begin after the entire header table.
    let mut payload = vec![0; 117];
    word(&mut payload, 0, 1);
    word(&mut payload, 4, 1); // vertex count
    word(&mut payload, 20, 1); // face count
    word(&mut payload, 44, 1); // edge count
    word(&mut payload, 84, 3f32.to_bits());
    word(&mut payload, 96, 1f32.to_bits());
    payload[113..].copy_from_slice(&[255, 0, 0, 0]);
    let mut bytes = physics_frame(*b"PHYS", &6u16.to_le_bytes());
    bytes.extend(physics_frame(*b"PLYT", &payload));
    let physics = PhysicsFile::read(&bytes).unwrap();
    let PhysicsChunk::Polytopes(values) = &physics.chunks[1] else {
        panic!("polytopes")
    };
    assert_eq!(values[0].vertices.get(0), Some([3.0, 0.0, 0.0]));
    assert_eq!(values[0].edges[0].twin_offset, -1);
    assert_eq!(
        values[0].vertices.as_bytes().as_ptr(),
        bytes[10 + 8 + 84..].as_ptr()
    );
    assert_eq!(physics.write().unwrap(), bytes);
    let mut edited = physics.into_owned();
    let PhysicsChunk::Polytopes(values) = &mut edited.chunks[1] else {
        panic!("polytopes")
    };
    values[0].vertices = [[3.0, 0.0, 0.0], [4.0, 5.0, 6.0]].into_zero_vec();
    let written = edited.write().unwrap();
    let decoded = PhysicsFile::read(&written).unwrap();
    let PhysicsChunk::Polytopes(values) = &decoded.chunks[1] else {
        panic!("polytopes")
    };
    assert_eq!(values[0].header.vertex_count, 2);
    assert_eq!(values[0].vertices.get(1), Some([4.0, 5.0, 6.0]));
    bytes.pop();
    assert!(PhysicsFile::read(&bytes).is_err());
}

#[test]
fn physics_writer_checks_edited_layouts_and_opaque_tags() {
    use std::borrow::Cow;
    let mut bytes = physics_frame(*b"PHYS", &6u16.to_le_bytes());
    bytes.extend(physics_frame(*b"BDY4", &[0; 48]));
    let mut physics = PhysicsFile::read(&bytes).unwrap();
    physics.chunks[0] = PhysicsChunk::Header(0);
    assert!(physics.write().is_err());
    physics.chunks.truncate(1);
    physics.chunks.push(PhysicsChunk::Unknown {
        tag: *b"PHYS",
        data: Cow::Borrowed(&[0, 0]),
    });
    assert!(physics.write().is_err());
    physics.chunks[1] = PhysicsChunk::Unknown {
        tag: *b"TEST",
        data: Cow::Borrowed(&[1, 2, 3]),
    };
    let written = physics.write().unwrap();
    assert_eq!(PhysicsFile::read(&written).unwrap().chunks, physics.chunks);
}

#[test]
fn classic_header_tails_and_version257_opaque_address_space_survive_edits() {
    use wc3::model::formats::m2::DynamicDocument;
    let mut bytes = core_fixture(256);
    for (index, value) in bytes[0x144..0x154].iter_mut().enumerate() {
        *value = index as u8 + 1;
    }
    let model = Document::<Flat<V256>>::decode_m2(&bytes).unwrap();
    let written = model.encode_m2().unwrap();
    assert_eq!(&written[0x144..0x154], &bytes[0x144..0x154]);
    assert!(Document::<Flat<V256>>::decode_m2(&bytes[..0x153]).is_err());

    let mut bytes = core_fixture(257);
    let opaque_offset = bytes.len();
    let opaque_payload = [7, 6, 5, 4, 3, 2, 1, 99, 88];
    bytes.extend_from_slice(&opaque_payload);
    word(&mut bytes, 0x144, 2); // Count deliberately conveys no invented stride.
    word(&mut bytes, 0x148, opaque_offset as u32);
    bytes[0x14c..0x154].copy_from_slice(&[1, 3, 5, 7, 9, 11, 13, 15]);
    let DynamicDocument::Flat257(mut model) = DynamicDocument::decode_m2(&bytes).unwrap() else {
        panic!("classic layout")
    };
    assert_eq!(model.version(), 257);
    let source = model
        .core()
        .classic_header_tail
        .array
        .address_space
        .as_ref()
        .unwrap();
    assert_eq!(source.as_ptr(), bytes.as_ptr());
    model.core_mut().name = b"changed\0".as_slice().into_zero_vec();
    model.core_mut().global_sequences = [123, 456, 789].into_zero_vec();
    let written = model.encode_m2().unwrap();
    assert_eq!(
        &written[opaque_offset..opaque_offset + opaque_payload.len()],
        opaque_payload
    );
    assert_eq!(&written[0x144..0x154], &bytes[0x144..0x154]);
    let decoded = Document::<Flat<V257>>::decode_m2(&written).unwrap();
    assert_eq!(decoded.core().name.as_bytes(), b"changed\0");
    assert_eq!(
        decoded.core().global_sequences.iter().collect::<Vec<_>>(),
        [123, 456, 789]
    );
}

#[test]
fn classic_unnamed_lookup_uses_the_documented_header_offset() {
    let mut bytes = core_fixture(256);
    let offset = bytes.len();
    bytes.extend_from_slice(&71u16.to_le_bytes());
    word(&mut bytes, 0xac, 1);
    word(&mut bytes, 0xb0, offset as u32);
    let model = Document::<Flat<V256>>::decode_m2(&bytes).unwrap();
    let opaque = &model.core().classic_lookup;
    assert_eq!(opaque.reference.count, 1);
    assert_eq!(
        &opaque.address_space.as_ref().unwrap()[offset..offset + 2],
        &71u16.to_le_bytes()
    );
    let written = model.encode_m2().unwrap();
    assert_eq!(
        u32::from_le_bytes(written[0xac..0xb0].try_into().unwrap()),
        1
    );
    assert_eq!(&written[offset..offset + 2], &71u16.to_le_bytes());
    let mut malformed = bytes;
    word(&mut malformed, 0xb0, u32::MAX);
    assert!(Document::<Flat<V256>>::decode_m2(&malformed).is_err());
}

fn external_bone_fixture(version: u32, address_base: usize) -> Vec<u8> {
    // Modern bone: 16-byte metadata, three 20-byte tracks, 12-byte pivot.
    // Translation points at two local outer descriptors; their leaves belong
    // to the .anim address space and intentionally use unaligned offsets.
    assert!(version >= 264);
    let mut bytes = vec![0; 104];
    word(&mut bytes, 0, u32::MAX);
    bytes[8..10].copy_from_slice(&(-1i16).to_le_bytes());
    for offset in [16, 36, 56] {
        bytes[offset + 2..offset + 4].copy_from_slice(&u16::MAX.to_le_bytes());
    }
    bytes[16..18].copy_from_slice(&1u16.to_le_bytes());
    word(&mut bytes, 20, 1);
    word(&mut bytes, 24, (address_base + 88) as u32);
    word(&mut bytes, 28, 1);
    word(&mut bytes, 32, (address_base + 96) as u32);
    word(&mut bytes, 88, 1);
    word(&mut bytes, 92, 1);
    word(&mut bytes, 96, 1);
    word(&mut bytes, 100, 9);
    bytes
}
fn external_animation_fixture() -> Vec<u8> {
    let mut bytes = vec![0; 21];
    word(&mut bytes, 1, 750);
    for (index, value) in [2f32, 3.0, 4.0].into_iter().enumerate() {
        word(&mut bytes, 9 + index * 4, value.to_bits());
    }
    bytes
}
fn external_core_fixture(bones: bool, chunked_animation: bool) -> Vec<u8> {
    let mut bytes = core_fixture(274);
    let sequence_offset = bytes.len();
    bytes.resize(sequence_offset + 64, 0);
    word(&mut bytes, 28, 1);
    word(&mut bytes, 32, sequence_offset as u32);
    bytes[sequence_offset..sequence_offset + 2].copy_from_slice(&7u16.to_le_bytes());
    word(&mut bytes, sequence_offset + 4, 1000);
    if chunked_animation {
        word(&mut bytes, 16, 0x200000);
    }
    if bones {
        let bone_offset = bytes.len();
        word(&mut bytes, 44, 1);
        word(&mut bytes, 48, bone_offset as u32);
        bytes.extend(external_bone_fixture(274, bone_offset));
    }
    bytes
}

#[test]
fn bundles_borrow_resolved_animation_keys_and_encode_relocated_external_leaves() {
    use wc3::model::formats::m2::records::TrackValues;
    use wc3::model::formats::m2::resources::FileKey;
    use wc3::model::formats::m2::{Bundle, ResolvedValues};
    let model_bytes = external_core_fixture(true, false);
    let animation_bytes = external_animation_fixture();
    let model_key = FileKey::Path("units/test.m2".into());
    let animation_key = FileKey::Path("units/test0007-00.anim".into());
    let mut bundle = Bundle::<Flat<V274>>::decode(&model_bytes, model_key.clone(), |dependency| {
        assert_eq!(dependency.key, animation_key);
        Ok(Some(animation_bytes.as_slice()))
    })
    .unwrap();
    let track = bundle.bone_translation(0, 0).unwrap();
    assert_eq!(track.timestamps.get(0), Some(750));
    assert_eq!(
        track.timestamps.as_bytes().as_ptr(),
        animation_bytes[1..].as_ptr()
    );
    let ResolvedValues::Values(values) = track.values else {
        panic!("plain values")
    };
    assert_eq!(values.get(0), Some([2.0, 3.0, 4.0]));
    assert_eq!(values.as_bytes().as_ptr(), animation_bytes[9..].as_ptr());
    let translation = &mut bundle.model.core_mut().bones[0].translation;
    translation.base.timeline.timestamps[0] = Array::Inline([750, 900].into_zero_vec());
    translation.values = TrackValues::Values(vec![Array::Inline(
        [[2.0, 3.0, 4.0], [5.0, 6.0, 7.0]].into_zero_vec(),
    )]);
    let encoded = bundle.encode().unwrap();
    let decoded = Bundle::<Flat<V274>>::decode(&encoded.model, model_key, |dependency| {
        Ok(encoded.files.get(&dependency.key).map(Vec::as_slice))
    })
    .unwrap();
    let track = decoded.bone_translation(0, 0).unwrap();
    assert_eq!(track.timestamps.get(0), Some(750));
    let ResolvedValues::Values(values) = track.values else {
        panic!("plain values")
    };
    assert_eq!(values.get(0), Some([2.0, 3.0, 4.0]));
    assert!(Bundle::<Flat<V274>>::decode(&model_bytes, FileKey::Root, |_| Ok(None)).is_err());
    assert_eq!(values.get(1), Some([5.0, 6.0, 7.0]));
    let owned = bundle.into_owned();
    assert_eq!(
        owned.bone_translation(0, 0).unwrap().timestamps.get(0),
        Some(750)
    );
}

#[test]
fn bundle_skeleton_tracks_use_the_skeletons_afsb_stream_and_file_identity() {
    use wc3::model::formats::m2::records::TrackValues;
    use wc3::model::formats::m2::resources::FileKey;
    use wc3::model::formats::m2::{Bundle, File, ResolvedValues};
    let core = external_core_fixture(false, true);
    let mut model_bytes = frame(*b"MD21", &core);
    model_bytes.extend(frame(*b"SKID", &17u32.to_le_bytes()));
    let mut bone_payload = vec![0; 16];
    word(&mut bone_payload, 0, 1);
    word(&mut bone_payload, 4, 16);
    bone_payload.extend(external_bone_fixture(274, 16));
    let mut skeleton = frame(*b"SKB1", &bone_payload);
    let mut afid = vec![0; 8];
    afid[..2].copy_from_slice(&7u16.to_le_bytes());
    word(&mut afid, 4, 33);
    skeleton.extend(frame(*b"AFID", &afid));
    let animation = frame(*b"AFSB", &external_animation_fixture());
    let root_animation = frame(*b"AFM2", &[]);
    let root_key = FileKey::Path("character/test.m2".into());
    let mut bundle =
        Bundle::<Chunked<V274>>::decode(&model_bytes, root_key.clone(), |dependency| {
            Ok(match &dependency.key {
                FileKey::FileId(17) => Some(skeleton.as_slice()),
                FileKey::FileId(33) => Some(animation.as_slice()),
                FileKey::Path(path) if path == "character/test0007-00.anim" => {
                    Some(root_animation.as_slice())
                }
                _ => None,
            })
        })
        .unwrap();
    let (owner, bones) = bundle.bones().unwrap();
    assert_eq!(*owner, FileKey::FileId(17));
    assert_eq!(bones.len(), 1);
    let track = bundle.bone_translation(0, 0).unwrap();
    assert_eq!(track.timestamps.get(0), Some(750));
    let ResolvedValues::Values(values) = track.values else {
        panic!("plain skeleton keys")
    };
    assert_eq!(values.get(0), Some([2.0, 3.0, 4.0]));
    let Some(File::Skeleton(skeleton)) = bundle.file_mut(&FileKey::FileId(17)) else {
        panic!("loaded skeleton")
    };
    let skeleton = skeleton.as_typed_mut::<V274>().unwrap();
    let SkeletonChunk::Bones { bones, .. } = &mut skeleton.chunks[0] else {
        panic!("skeleton bones")
    };
    bones[0].translation.base.timeline.timestamps[0] = Array::Inline([750, 800].into_zero_vec());
    bones[0].translation.values = TrackValues::Values(vec![Array::Inline(
        [[2.0, 3.0, 4.0], [8.0, 9.0, 10.0]].into_zero_vec(),
    )]);
    let encoded = bundle.encode().unwrap();
    let decoded = Bundle::<Chunked<V274>>::decode(&encoded.model, root_key, |dependency| {
        Ok(encoded.files.get(&dependency.key).map(Vec::as_slice))
    })
    .unwrap();
    assert_eq!(
        decoded.bone_translation(0, 0).unwrap().timestamps.get(0),
        Some(750)
    );
}

#[test]
fn physics_validation_checks_known_reference_domains_without_reinterpreting_unknown_kinds() {
    let mut body = vec![0; 48];
    body[2..4].copy_from_slice(&9u16.to_le_bytes());
    word(&mut body, 20, 1);
    let mut shape = vec![0; 32];
    shape[..2].copy_from_slice(&1u16.to_le_bytes()); // Capsule, not sphere.
    let mut joint = vec![0; 16];
    joint[12..14].copy_from_slice(&2u16.to_le_bytes()); // File enum 2 is weld.
    let mut bytes = physics_frame(*b"PHYS", &6u16.to_le_bytes());
    for (tag, payload) in [
        (*b"BDY4", body.as_slice()),
        (*b"SHP2", shape.as_slice()),
        (*b"CAPS", &[0; 28]),
        (*b"WLJ3", &[0; 116]),
        (*b"JOIN", joint.as_slice()),
    ] {
        bytes.extend(physics_frame(tag, payload));
    }
    let physics = PhysicsFile::read(&bytes).unwrap();
    physics.validate().unwrap();
    physics.validate_with_bone_count(10).unwrap();
    assert!(physics.validate_with_bone_count(9).is_err());
    let mut edited = physics.clone();
    let PhysicsChunk::Shapes2(shapes) = &mut edited.chunks[2] else {
        panic!("shape table")
    };
    shapes[0].shape.shape_index = 1;
    assert!(edited.validate().is_err());
    let mut edited = physics.clone();
    let PhysicsChunk::Bodies4(bodies) = &mut edited.chunks[1] else {
        panic!("body table")
    };
    bodies[0].shape_index = 1;
    assert!(edited.validate().is_err());
    let mut edited = physics.clone();
    let PhysicsChunk::Joints(joints) = &mut edited.chunks[5] else {
        panic!("joint bindings")
    };
    joints[0].joint_id = 1;
    assert!(edited.validate().is_err());
    let mut edited = physics.clone();
    let PhysicsChunk::Joints(joints) = &mut edited.chunks[5] else {
        panic!("joint bindings")
    };
    joints[0].body_b = 1;
    assert!(edited.validate().is_err());
    let mut edited = physics;
    let PhysicsChunk::Joints(joints) = &mut edited.chunks[5] else {
        panic!("joint bindings")
    };
    joints[0].joint_type = PhysicsJointKind::Unknown(66);
    joints[0].joint_id = -1;
    let PhysicsChunk::Shapes2(shapes) = &mut edited.chunks[2] else {
        panic!("shape table")
    };
    shapes[0].shape.shape_type = PhysicsShapeKind::Unknown(55);
    shapes[0].shape.shape_index = -9;
    edited.validate().unwrap();
}

#[test]
fn late_tbc_header_keeps_its_pre_wrath_lookup_and_embedded_skin_layout() {
    use wc3::model::formats::m2::DynamicDocument;
    let mut bytes = core_fixture(263);
    let lookup = bytes.len();
    bytes.resize(lookup + 34, 0);
    bytes[lookup + 2..lookup + 4].copy_from_slice(&u16::MAX.to_le_bytes());
    word(&mut bytes, lookup + 12, 1);
    word(&mut bytes, lookup + 16, (lookup + 28) as u32);
    word(&mut bytes, lookup + 20, 1);
    word(&mut bytes, lookup + 24, (lookup + 32) as u32);
    word(&mut bytes, lookup + 28, 300);
    bytes[lookup + 32..lookup + 34].copy_from_slice(&81u16.to_le_bytes());
    word(&mut bytes, 0x6c, 1);
    word(&mut bytes, 0x70, lookup as u32);
    let material = bytes.len();
    bytes.extend_from_slice(&[0x13, 0, 4, 0]);
    word(&mut bytes, 0x84, 1);
    word(&mut bytes, 0x88, material as u32);
    let skin = bytes.len();
    bytes.resize(skin + 44, 0);
    word(&mut bytes, 0x4c, 1);
    word(&mut bytes, 0x50, skin as u32);
    let DynamicDocument::Flat263(model) = DynamicDocument::decode_m2(&bytes).unwrap() else {
        panic!("TBC layout")
    };
    assert_eq!(model.version(), 263);
    let frames = &model.core().texture_flipbooks[0].frames;
    let wc3::model::formats::m2::records::LegacyValues::Values(values) = &frames.values else {
        panic!("plain flipbook frames")
    };
    assert_eq!(values.get(0), Some(81));
    frames.validate().unwrap();
    assert_eq!(model.core().materials.get(0).unwrap().flags.bits(), 0x13);
    assert_eq!(
        model.core().materials.get(0).unwrap().blending_mode,
        BlendMode::Add
    );
    let skins = &model.core().skins;
    assert_eq!(skins.len(), 1);
    let written = model.encode_m2().unwrap();
    assert_eq!(
        Document::<Flat<V263>>::decode_m2(&written).unwrap().core(),
        model.core()
    );
}

#[test]
fn version272_profiles_distinguish_single_and_split_blend_words() {
    use wc3::model::formats::m2::records::SplitBlendTime;
    use wc3::model::formats::m2::{DynamicDocument, V272PreSplit};
    let mut bytes = core_fixture(272);
    let offset = bytes.len();
    bytes.resize(offset + 64, 0);
    word(&mut bytes, 28, 1);
    word(&mut bytes, 32, offset as u32);
    word(&mut bytes, offset + 4, 1000);
    word(&mut bytes, offset + 12, 0x20);
    word(&mut bytes, offset + 28, 350);
    let mut older = Document::<Flat<V272PreSplit>>::decode_m2(&bytes).unwrap();
    assert_eq!(older.core().sequences[0].timing, 1000);
    assert_eq!(older.core().sequences[0].blend_time, 350);
    let newer = Document::<Flat<V272>>::decode_m2(&bytes).unwrap();
    assert_eq!(
        newer.core().sequences[0].blend_time,
        SplitBlendTime {
            incoming: 350,
            outgoing: 0
        }
    );
    older.core_mut().sequences[0].blend_time = 700;
    let written = older.encode_m2().unwrap();
    assert_eq!(
        Document::<Flat<V272PreSplit>>::decode_m2(&written)
            .unwrap()
            .core()
            .sequences[0]
            .blend_time,
        700
    );
    fn edit<D: Dialect<Layout = V272PreSplit>>(model: &mut Document<'_, D>) {
        assert_eq!(model.core().sequences[0].blend_time, 350);
        model.core_mut().sequences[0].blend_time = 123456;
    }
    for input in [&bytes, &frame(*b"MD21", &bytes)] {
        let mut dynamic = DynamicDocument::decode_m2_with_layout::<V272PreSplit>(input).unwrap();
        match &mut dynamic {
            DynamicDocument::Flat272PreSplit(model) => edit(model),
            DynamicDocument::Chunked272PreSplit(model) => edit(model),
            _ => panic!("selected blend profile"),
        }
        let encoded = dynamic.encode_m2().unwrap();
        let decoded = DynamicDocument::decode_m2_with_layout::<V272PreSplit>(&encoded).unwrap();
        match decoded {
            DynamicDocument::Flat272PreSplit(model) => {
                assert_eq!(model.core().sequences[0].blend_time, 123456)
            }
            DynamicDocument::Chunked272PreSplit(model) => {
                assert_eq!(model.core().sequences[0].blend_time, 123456)
            }
            _ => panic!("selected blend profile"),
        }
        match DynamicDocument::decode_m2(input).unwrap() {
            DynamicDocument::Flat272(model) => assert_eq!(
                model.core().sequences[0].blend_time,
                SplitBlendTime {
                    incoming: 350,
                    outgoing: 0
                }
            ),
            DynamicDocument::Chunked272(model) => assert_eq!(
                model.core().sequences[0].blend_time,
                SplitBlendTime {
                    incoming: 350,
                    outgoing: 0
                }
            ),
            _ => panic!("default blend profile"),
        }
    }
    assert!(DynamicDocument::decode_m2_with_layout::<V272PreSplit>(&core_fixture(274)).is_err());
    assert!(DynamicDocument::decode_m2_with_layout::<V272>(&core_fixture(264)).is_err());
    let mut payload = vec![0; 32];
    word(&mut payload, 8, 1);
    word(&mut payload, 12, 32);
    payload.extend_from_slice(&bytes[offset..offset + 64]);
    let skeleton_bytes = frame(*b"SKS1", &payload);
    let skeleton =
        Skeleton::<V272PreSplit>::read_with_sequences(&skeleton_bytes, ModelFlags::default(), &[])
            .unwrap();
    let SkeletonChunk::Sequences { sequences, .. } = &skeleton.chunks[0] else {
        panic!("skeleton sequences")
    };
    assert_eq!(sequences[0].blend_time, 350);
    let written = skeleton.write().unwrap();
    assert_eq!(
        Skeleton::<V272PreSplit>::read_with_sequences(&written, ModelFlags::default(), &[])
            .unwrap(),
        skeleton
    );
}

#[test]
fn opaque_classic_arrays_ignore_empty_offsets_but_require_backing_for_nonempty_data() {
    let mut bytes = core_fixture(257);
    word(&mut bytes, 0x148, u32::MAX);
    let model = Document::<Flat<V257>>::decode_m2(&bytes).unwrap();
    let encoded = model.encode_m2().unwrap();
    assert_eq!(
        u32::from_le_bytes(encoded[0x148..0x14c].try_into().unwrap()),
        u32::MAX
    );
    word(&mut bytes, 0x144, 1);
    let end = bytes.len() as u32;
    word(&mut bytes, 0x148, end);
    assert!(Document::<Flat<V257>>::decode_m2(&bytes).is_err());
}
