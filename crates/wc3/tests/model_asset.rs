//! Public checked, owned-model loading shared by archive and async asset callers.
use glam::Vec3;
use std::borrow::Cow;
use std::error::Error as _;
use std::io;
use wc3::model::common::FileKey;
use wc3::model::conversion::{Conversion, ExportOptions, ImportOptions, IssueKind, UnknownPolicy};
use wc3::model::formats::m2::{
    chunks::{AnimationFileId, Chunk},
    codec::{Array as M2Array, ArrayRef},
    companions::{AnimationFile, AnimationStream, Skeleton, SkeletonChunk, Skin, SkinProfile},
    records::{
        Bone, Interpolation, ModelFlags, ParticleEmitter, Sequence, SequenceFlags, Texture,
        TextureSource, TimelineStorage as _, ValuesLeaf, ValuesStorage as _,
    },
    Chunked, Dialect as M2Dialect, Document, Flat, ModelLayout, ZeroVec, V256, V257, V260, V263,
    V264, V265, V272, V273, V274,
};
use wc3::model::formats::mdlx::{
    self,
    chunks::{ModelChunk, RawChunk, UnknownChunk},
    mdl::Read as _,
    mdx::Write as _,
};
use wc3::model::formats::wmo::{
    self, codec::Array as WmoArray, group::GroupChunk, records, root::RootChunk, Classic, Dialect,
    Group, Root,
};
use wc3::model::io::asset::{
    load, CompanionKind, DecodeError, Error, Format, LoadError, Progress, Reader, Request,
};
use wc3::model::io::wmo as semantic_wmo;
use wc3::model::profiles::Wmo;
use wc3::model::{AnimationProgram, AnimationSample, Model, Time};

fn ready(reader: &mut Reader) -> Conversion<Model<'static>> {
    match reader.poll().unwrap() {
        Progress::Ready(asset) => *asset,
        Progress::Pending(request) => panic!("unexpected companion {request:?}"),
    }
}

fn pending(reader: &mut Reader) -> Request {
    match reader.poll().unwrap() {
        Progress::Pending(request) => request,
        Progress::Ready(_) => panic!("expected a required companion"),
    }
}

fn reader_error(reader: &mut Reader) -> Error {
    match reader.poll() {
        Err(error) => error,
        Ok(_) => panic!("invalid native bytes were accepted"),
    }
}

#[test]
fn mdlx_header_selects_binary_independently_of_the_owner_extension() {
    let text = include_bytes!("fixtures/mdl/quad_model.mdl");
    let binary = include_bytes!("fixtures/mdl/quad_model.mdx");
    let mut text_reader = Reader::new(
        Format::Mdlx,
        FileKey::Path("text.mdx".into()),
        text.to_vec(),
        ImportOptions::default(),
    );
    let mut binary_reader = Reader::new(
        Format::Mdlx,
        FileKey::Path("binary.mdl".into()),
        binary.to_vec(),
        ImportOptions::default(),
    );
    let text_asset = ready(&mut text_reader);
    let binary_asset = ready(&mut binary_reader);
    drop((text_reader, binary_reader));

    let text_model = text_asset;
    let binary_model = binary_asset;
    text_model.value.validate().unwrap();
    binary_model.value.validate().unwrap();
    assert_eq!(text_model.report, binary_model.report);
    assert_eq!(
        text_model.value.nodes().len(),
        binary_model.value.nodes().len()
    );
    assert!(text_model
        .value
        .nodes()
        .values()
        .any(|node| node.name == "Bone_Root"));
    assert_eq!(text_model.value.meshes().len(), 1);
    assert_eq!(text_model.value.clips().len(), 1);
    assert_eq!(
        text_model
            .value
            .nodes()
            .values()
            .map(|node| &node.name)
            .collect::<Vec<_>>(),
        binary_model
            .value
            .nodes()
            .values()
            .map(|node| &node.name)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        text_model
            .value
            .meshes()
            .values()
            .next()
            .unwrap()
            .positions()
            .iter()
            .collect::<Vec<_>>(),
        binary_model
            .value
            .meshes()
            .values()
            .next()
            .unwrap()
            .positions()
            .iter()
            .collect::<Vec<_>>()
    );
    assert_eq!(
        text_model
            .value
            .dependencies()
            .images()
            .values()
            .collect::<Vec<_>>(),
        binary_model
            .value
            .dependencies()
            .images()
            .values()
            .collect::<Vec<_>>()
    );
}

#[test]
fn native_decode_errors_keep_the_root_identity_and_original_source() {
    for (case, (format, bytes)) in [
        (Format::Mdlx, vec![0xff]),
        (Format::Mdlx, b"Version {".to_vec()),
        (Format::Mdlx, b"MDLXVERS\x04".to_vec()),
        (Format::M2, b"MD20".to_vec()),
        (Format::Wmo(wmo::Profile::Classic), b"REVM".to_vec()),
    ]
    .into_iter()
    .enumerate()
    {
        let key = FileKey::FileId(71);
        let error = reader_error(&mut Reader::new(
            format,
            key.clone(),
            bytes,
            ImportOptions::default(),
        ));
        assert!(matches!(&error, Error::Decode { key: actual, .. } if actual == &key));
        let source = error.source().expect("typed native decode error");
        assert!(
            source.source().is_some(),
            "native decoder cause must remain inspectable"
        );
        assert!(error.to_string().contains("71"));
        assert!(matches!(
            (case, error),
            (
                0,
                Error::Decode {
                    source: DecodeError::Utf8(_),
                    ..
                }
            ) | (
                1,
                Error::Decode {
                    source: DecodeError::Mdl(_),
                    ..
                }
            ) | (
                2,
                Error::Decode {
                    source: DecodeError::Mdx(_),
                    ..
                }
            ) | (
                3,
                Error::Decode {
                    source: DecodeError::M2(_),
                    ..
                }
            ) | (
                4,
                Error::Decode {
                    source: DecodeError::Wmo(_),
                    ..
                }
            )
        ));
    }
}

#[test]
fn reader_finishes_only_after_the_fixed_import_policy_accepts_the_model() {
    let mut source =
        mdlx::Document::<mdlx::V800>::decode_mdl(include_str!("fixtures/mdl/quad_model.mdl"))
            .unwrap();
    source.chunks.push(ModelChunk::Unknown(
        UnknownChunk::new(RawChunk::new(*b"TEST", Cow::Owned(vec![1, 2, 3]))).unwrap(),
    ));
    let key = FileKey::Path("strict.mdx".into());
    let bytes = source.encode_mdx().unwrap();
    let mut strict = Reader::new(
        Format::Mdlx,
        key.clone(),
        bytes.clone(),
        ImportOptions::default(),
    );
    let error = reader_error(&mut strict);
    assert!(matches!(&error, Error::Import { key: actual, .. } if actual == &key));
    assert!(error.source().is_some());
    let mut accepted = Reader::new(
        Format::Mdlx,
        key.clone(),
        bytes.clone(),
        ImportOptions {
            unknown: UnknownPolicy::Drop,
            ..Default::default()
        },
    );
    let accepted = ready(&mut accepted);
    assert_eq!(accepted.value.meshes().len(), 1);
    assert!(accepted
        .report
        .issues
        .iter()
        .any(|issue| issue.path == "chunks" && issue.kind == IssueKind::Dropped));
    let error = load(
        Format::Mdlx,
        key.clone(),
        bytes,
        ImportOptions::default(),
        |_| {
            panic!("MDLX has no required native companions");
            #[allow(unreachable_code)]
            Ok::<_, io::Error>(Vec::new())
        },
    )
    .unwrap_err();
    assert!(matches!(error, LoadError::Native(Error::Import { key: actual, .. }) if actual == key));
}

fn animation_sequence<V: ModelLayout<Timing = u32>>() -> Sequence<V> {
    Sequence {
        id: 7,
        timing: 1000,
        flags: SequenceFlags::default(),
        variation_next: -1,
        ..Default::default()
    }
}

fn external_bone() -> Bone<'static, V272> {
    let mut bone = Bone::<V272>::new();
    bone.translation.base.interpolation = Interpolation::Linear;
    bone.translation
        .base
        .timeline
        .replace(
            0,
            M2Array::External(ArrayRef {
                count: 1,
                offset: 0,
            }),
        )
        .unwrap();
    bone.translation
        .values
        .replace(
            0,
            ValuesLeaf::Values(M2Array::External(ArrayRef {
                count: 1,
                offset: 16,
            })),
        )
        .unwrap();
    bone
}

fn skeleton(chunks: Vec<SkeletonChunk<'static, V272>>) -> Vec<u8> {
    Skeleton {
        flags: ModelFlags::default(),
        chunks,
    }
    .write()
    .unwrap()
}

fn parent(id: u32) -> SkeletonChunk<'static, V272> {
    SkeletonChunk::Parent {
        reserved0: [0; 8],
        file_id: id,
        reserved1: [0; 4],
    }
}

fn skeleton_root() -> Vec<u8> {
    let mut root = Document::<Chunked<V272>>::new();
    root.insert_extension(1, Chunk::SkeletonId(10)).unwrap();
    root.encode_m2().unwrap()
}

#[test]
fn recursive_m2_skeleton_requests_retain_the_declaring_owner_and_animate() {
    let mut reader = Reader::new(
        Format::M2,
        FileKey::FileId(1),
        skeleton_root(),
        ImportOptions::default(),
    );
    let child = pending(&mut reader);
    assert_eq!(
        child,
        Request {
            owner: FileKey::FileId(1),
            key: FileKey::FileId(10),
            kind: CompanionKind::Skeleton,
        }
    );
    assert_eq!(
        pending(&mut reader),
        child,
        "pending reads are stable until bytes arrive"
    );
    reader
        .provide(
            &child,
            skeleton(vec![
                parent(11),
                SkeletonChunk::Bones {
                    bones: vec![external_bone()],
                    lookup: ZeroVec::new(),
                },
            ]),
        )
        .unwrap();
    let parent = pending(&mut reader);
    assert_eq!(parent.owner, FileKey::FileId(10));
    assert_eq!(parent.key, FileKey::FileId(11));
    assert_eq!(parent.kind, CompanionKind::Skeleton);
    reader
        .provide(
            &parent,
            skeleton(vec![
                SkeletonChunk::Sequences {
                    global_loops: ZeroVec::new(),
                    sequences: vec![animation_sequence::<V272>()],
                    lookup: ZeroVec::new(),
                    reserved: [0; 8],
                },
                SkeletonChunk::AnimationIds(vec![AnimationFileId {
                    animation_id: 7,
                    variation_id: 0,
                    file_id: 99,
                }]),
            ]),
        )
        .unwrap();
    let animation = pending(&mut reader);
    assert_eq!(animation.owner, FileKey::FileId(11));
    assert_eq!(animation.key, FileKey::FileId(99));
    assert_eq!(
        animation.kind,
        CompanionKind::Animation {
            animation_id: 7,
            variation_id: 0
        }
    );
    let mut stream = vec![0; 28];
    stream[..4].copy_from_slice(&100_u32.to_le_bytes());
    for (index, component) in [11_f32, 12., 13.].into_iter().enumerate() {
        stream[16 + index * 4..20 + index * 4].copy_from_slice(&component.to_le_bytes());
    }
    reader
        .provide(
            &animation,
            AnimationFile::Chunked(vec![AnimationStream {
                tag: *b"AFSB",
                data: Cow::Owned(stream),
            }])
            .write()
            .unwrap(),
        )
        .unwrap();
    let asset = ready(&mut reader);
    let imported = asset;
    drop(reader);
    assert_eq!(imported.value.clips().len(), 1);
    let clip = imported.value.clips().iter().next().unwrap().0;
    let program = AnimationProgram::compile(&imported.value);
    let sample = program
        .sample(AnimationSample {
            clip: Some(clip),
            clip_time: Time::seconds(0.1).unwrap(),
            ..Default::default()
        })
        .unwrap();
    let node = imported.value.nodes().values().next().unwrap();
    assert_eq!(
        sample.animated(&node.transform.translation),
        Vec3::new(11., 12., 13.)
    );
}

#[test]
fn m2_parent_cycles_terminate_after_each_unique_file_is_supplied_once() {
    let mut reader = Reader::new(
        Format::M2,
        FileKey::FileId(1),
        skeleton_root(),
        ImportOptions::default(),
    );
    let child = pending(&mut reader);
    reader.provide(&child, skeleton(vec![parent(11)])).unwrap();
    let ancestor = pending(&mut reader);
    reader
        .provide(&ancestor, skeleton(vec![parent(10)]))
        .unwrap();
    let error = reader_error(&mut reader);
    assert!(matches!(error, Error::Decode { .. }));
    assert!(error.to_string().contains("skeleton parent cycle"));
}

#[test]
fn a_companion_response_must_match_the_outstanding_request() {
    let mut reader = Reader::new(
        Format::M2,
        FileKey::FileId(1),
        skeleton_root(),
        ImportOptions::default(),
    );
    let request = pending(&mut reader);
    let mut wrong = request.clone();
    wrong.owner = FileKey::FileId(123);
    let error = reader.provide(&wrong, Vec::new()).unwrap_err();
    assert!(
        matches!(&error, Error::UnexpectedCompanion { expected: Some(expected), supplied } if expected.as_ref() == &request && supplied.as_ref() == &wrong)
    );
    assert_eq!(pending(&mut reader), request);
    reader
        .provide(
            &request,
            skeleton(vec![SkeletonChunk::Bones {
                bones: Vec::new(),
                lookup: ZeroVec::new(),
            }]),
        )
        .unwrap();
    let _asset = ready(&mut reader);
    assert!(matches!(reader.poll(), Err(Error::Finished)));
    assert!(reader.provide(&request, Vec::new()).is_err());
}

#[test]
fn m2_legacy_and_modern_roots_share_the_same_public_loading_contract() {
    fn check<D: M2Dialect>() {
        let bytes = Document::<D>::new().encode_m2().unwrap();
        let mut reader = Reader::new(Format::M2, FileKey::Root, bytes, ImportOptions::default());
        let asset = ready(&mut reader);
        assert!(asset.value.nodes().is_empty());
    }
    fn both<V: ModelLayout>() {
        check::<Flat<V>>();
        check::<Chunked<V>>();
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
fn malformed_required_m2_bytes_remain_a_codec_error_after_supplying_the_request() {
    let root = FileKey::RawPath(b"models/\xff.m2".to_vec());
    let mut reader = Reader::new(
        Format::M2,
        root.clone(),
        skeleton_root(),
        ImportOptions::default(),
    );
    let request = pending(&mut reader);
    assert_eq!(request.owner, root);
    reader
        .provide(&request, b"not a skeleton".to_vec())
        .unwrap();
    let error = reader_error(&mut reader);
    assert!(matches!(&error, Error::Decode { key, source: DecodeError::M2(_) } if key == &root));
    assert!(error.source().unwrap().source().is_some());
}

#[test]
fn nested_m2_model_companions_keep_their_own_layout_and_skip_texture_bytes() {
    let mut root = Document::<Chunked<V256>>::new();
    root.core_mut().bones = vec![Bone::new()];
    root.core_mut().textures = vec![Texture {
        kind: TextureSource::File,
        filename: ZeroVec::alloc_from_slice(b"not-required.blp\0"),
        ..Default::default()
    }];
    root.core_mut().particles = vec![ParticleEmitter::new(ModelFlags::default())];
    root.insert_extension(
        1,
        Chunk::GeometryParticleIds(ZeroVec::alloc_from_slice(&[42])),
    )
    .unwrap();
    root.insert_extension(
        2,
        Chunk::RecursiveParticleIds(ZeroVec::alloc_from_slice(&[42])),
    )
    .unwrap();
    let mut reader = Reader::new(
        Format::M2,
        FileKey::Root,
        root.encode_m2().unwrap(),
        ImportOptions::default(),
    );
    let child = pending(&mut reader);
    assert_eq!(
        child,
        Request {
            owner: FileKey::Root,
            key: FileKey::FileId(42),
            kind: CompanionKind::Model,
        }
    );
    reader.provide(&child, skeleton_root()).unwrap();
    let skeleton_request = pending(&mut reader);
    assert_eq!(skeleton_request.owner, FileKey::FileId(42));
    assert_eq!(skeleton_request.key, FileKey::FileId(10));
    assert_eq!(skeleton_request.kind, CompanionKind::Skeleton);
    reader
        .provide(&skeleton_request, skeleton(Vec::new()))
        .unwrap();
    let asset = ready(&mut reader);
    assert_eq!(asset.value.features().len(), 1);
}

#[test]
fn cached_m2_bytes_cannot_satisfy_incompatible_native_roles() {
    let mut root = Document::<Chunked<V272>>::new();
    root.insert_extension(1, Chunk::SkeletonId(10)).unwrap();
    root.insert_extension(2, Chunk::BoneIds(ZeroVec::alloc_from_slice(&[10])))
        .unwrap();
    let mut reader = Reader::new(
        Format::M2,
        FileKey::Root,
        root.encode_m2().unwrap(),
        ImportOptions::default(),
    );
    let request = pending(&mut reader);
    assert_eq!(request.key, FileKey::FileId(10));
    reader.provide(&request, skeleton(Vec::new())).unwrap();
    let error = reader_error(&mut reader);
    assert!(matches!(
        &error,
        Error::Decode {
            source: DecodeError::M2(_),
            ..
        }
    ));
    assert!(error.to_string().contains("supporting document kind"));
}

fn wmo_fixture<D: Dialect>() -> (Vec<u8>, Vec<u8>) {
    let mut root = Root::<D>::new();
    root.header_mut().wmo_id = 55;
    root.header_mut().bounds = [[0.; 3], [1.; 3]];
    root.insert_chunk(
        0,
        RootChunk::Materials(WmoArray::from_records(&[records::Material::<D>::default()]).unwrap()),
    )
    .unwrap();
    root.insert_chunk(
        1,
        RootChunk::Groups(
            WmoArray::from_records(&[records::GroupInfo {
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
    group.chunks_mut().extend([
        GroupChunk::Positions(
            WmoArray::from_records(&[[0.; 3], [1., 0., 0.], [0., 1., 0.]]).unwrap(),
        ),
        GroupChunk::Normals(WmoArray::from_records(&[[0., 0., 1.]; 3]).unwrap()),
        GroupChunk::TextureCoordinates(
            WmoArray::from_records(&[[0.; 2], [1., 0.], [0., 1.]]).unwrap(),
        ),
        GroupChunk::Indices(WmoArray::from_records(&[0u16, 1, 2]).unwrap()),
        GroupChunk::Triangles(
            WmoArray::from_records(&[records::Triangle {
                flags: Default::default(),
                material: 0,
            }])
            .unwrap(),
        ),
    ]);
    (root.encode().unwrap(), group.encode().unwrap())
}

fn many_wmo_groups(count: usize) -> (Vec<u8>, Vec<(FileKey, Vec<u8>)>) {
    let (root_bytes, group_bytes) = wmo_fixture::<Classic>();
    let mut root = Root::<Classic>::decode(&root_bytes).unwrap().into_owned();
    let maximum = (count.saturating_sub(1) * 3 + 1) as f32;
    root.header_mut().bounds = [[0.; 3], [maximum, 1., 1.]];
    for chunk in root.chunks_mut() {
        if let RootChunk::Groups(values) = chunk {
            *values = WmoArray::from_records(
                &(0..count)
                    .map(|index| records::GroupInfo {
                        bounds: [
                            [index as f32 * 3., 0., 0.],
                            [index as f32 * 3. + 1., 1., 1.],
                        ],
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        }
    }
    // Physical identities intentionally sort opposite to their declaration order.
    let ids: Vec<_> = (0..count)
        .map(|index| 10_000 + (count - index) as u32)
        .collect();
    let chunk_count = root.chunks().count();
    root.insert_chunk(
        chunk_count,
        RootChunk::GroupFileIds(WmoArray::from_records(&ids).unwrap()),
    )
    .unwrap();
    root.synchronize_counts().unwrap();
    let groups = ids
        .into_iter()
        .enumerate()
        .map(|(index, id)| {
            let x = index as f32 * 3.;
            let mut group = Group::<Classic>::decode(&group_bytes).unwrap().into_owned();
            group.header_mut().bounds_minimum = [x, 0., 0.];
            group.header_mut().bounds_maximum = [x + 1., 1., 1.];
            for chunk in group.chunks_mut() {
                if let GroupChunk::Positions(values) = chunk {
                    *values = WmoArray::from_records(&[[x, 0., 0.], [x + 1., 0., 0.], [x, 1., 0.]])
                        .unwrap();
                }
            }
            (FileKey::FileId(id), group.encode().unwrap())
        })
        .collect();
    (root.encode().unwrap(), groups)
}

#[test]
fn many_wmo_groups_keep_declaration_order_and_native_semantics_during_incremental_loading() {
    let (root, groups) = many_wmo_groups(48);
    let root_key = FileKey::RawPath(b"World\\\xff-multigroup.wmo".to_vec());
    let native = wmo::Bundle::<Classic>::decode(&root, root_key.clone(), |dependency| {
        Ok(groups
            .iter()
            .find(|(key, _)| key == &dependency.key)
            .map(|(_, bytes)| bytes.as_slice()))
    })
    .unwrap();
    let (expected_report, expected) = {
        let imported = semantic_wmo::import(&native, &ImportOptions::default()).unwrap();
        let expected = semantic_wmo::plan::<Classic>(&imported.value, &ExportOptions::default())
            .unwrap()
            .execute()
            .value
            .encode()
            .unwrap();
        (imported.report, expected)
    };
    let mut reader = Reader::new(
        Format::Wmo(wmo::Profile::Classic),
        root_key.clone(),
        root.clone(),
        ImportOptions::default(),
    );
    let mut previous = None;
    for (index, (key, bytes)) in groups.iter().enumerate() {
        let request = pending(&mut reader);
        assert_eq!(request.owner, root_key);
        assert_eq!(&request.key, key);
        assert_eq!(request.kind, CompanionKind::WmoGroup { index, lod: 0 });
        for _ in 0..4 {
            assert_eq!(pending(&mut reader), request);
        }
        let mut wrong = request.clone();
        if let Some((key, _)) = groups.get(index + 1) {
            wrong.key = key.clone();
            wrong.kind = CompanionKind::WmoGroup {
                index: index + 1,
                lod: 0,
            };
        } else {
            wrong.owner = FileKey::Root;
        }
        assert!(matches!(
            reader.provide(&wrong, Vec::new()),
            Err(Error::UnexpectedCompanion { expected: Some(expected), supplied })
                if *expected == request && *supplied == wrong
        ));
        if let Some(stale) = &previous {
            assert!(matches!(
                reader.provide(stale, Vec::new()),
                Err(Error::UnexpectedCompanion { expected: Some(expected), .. })
                    if *expected == request
            ));
        }
        assert_eq!(pending(&mut reader), request);
        reader.provide(&request, bytes.clone()).unwrap();
        previous = Some(request);
    }
    let converted = ready(&mut reader);
    assert_eq!(converted.report, expected_report);
    assert!(matches!(reader.poll(), Err(Error::Finished)));
    drop(reader);
    drop(native);
    drop((groups, root));
    assert_eq!(converted.value.meshes().len(), 48);
    assert_eq!(converted.value.instances().len(), 48);
    let positions: Vec<_> = converted
        .value
        .meshes()
        .values()
        .map(|mesh| mesh.positions().first().unwrap().x)
        .collect();
    assert_eq!(
        positions,
        (0..48).map(|index| index as f32 * 3.).collect::<Vec<_>>()
    );
    let typed = converted.value.try_narrow::<Wmo>().unwrap();
    let actual = semantic_wmo::plan::<Classic>(&typed, &ExportOptions::default())
        .unwrap()
        .execute()
        .value
        .encode()
        .unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn malformed_wmo_group_is_checked_after_the_complete_companion_plan() {
    let (root, groups) = many_wmo_groups(12);
    let mut reader = Reader::new(
        Format::Wmo(wmo::Profile::Classic),
        FileKey::FileId(700),
        root,
        ImportOptions::default(),
    );
    let bad_key = groups[5].0.clone();
    for (index, (key, bytes)) in groups.into_iter().enumerate() {
        let request = pending(&mut reader);
        assert_eq!(request.key, key);
        assert_eq!(request.kind, CompanionKind::WmoGroup { index, lod: 0 });
        reader
            .provide(
                &request,
                if index == 5 {
                    b"invalid group".to_vec()
                } else {
                    bytes
                },
            )
            .unwrap();
    }
    for _ in 0..2 {
        assert!(matches!(
            reader_error(&mut reader),
            Error::Decode { key, source: DecodeError::Wmo(_) } if key == bad_key
        ));
    }
    let unrelated = Request {
        owner: FileKey::FileId(700),
        key: bad_key,
        kind: CompanionKind::WmoGroup { index: 5, lod: 0 },
    };
    assert!(matches!(
        reader.provide(&unrelated, Vec::new()),
        Err(Error::UnexpectedCompanion { expected: None, .. })
    ));
}

#[test]
fn wmo_lod_rows_skip_missing_and_aliased_files_without_losing_group_roles() {
    let (root, groups) = many_wmo_groups(6);
    let mut root = Root::<Classic>::decode(&root).unwrap().into_owned();
    let base_ids: Vec<_> = groups
        .iter()
        .map(|(key, _)| match key {
            FileKey::FileId(id) => *id,
            _ => unreachable!(),
        })
        .collect();
    let extra_ids = [0, base_ids[0], 9000, base_ids[2], 0, 9001];
    let ids: Vec<_> = base_ids.iter().copied().chain(extra_ids).collect();
    for chunk in root.chunks_mut() {
        if let RootChunk::GroupFileIds(values) = chunk {
            *values = WmoArray::from_records(&ids).unwrap();
        }
    }
    let mut reader = Reader::new(
        Format::Wmo(wmo::Profile::Classic),
        FileKey::Root,
        root.encode().unwrap(),
        ImportOptions::default(),
    );
    for (index, (key, bytes)) in groups.iter().enumerate() {
        let request = pending(&mut reader);
        assert_eq!(&request.key, key);
        assert_eq!(request.kind, CompanionKind::WmoGroup { index, lod: 0 });
        reader.provide(&request, bytes.clone()).unwrap();
    }
    for (id, index) in [(9000, 2), (9001, 5)] {
        let request = pending(&mut reader);
        assert_eq!(request.key, FileKey::FileId(id));
        assert_eq!(request.kind, CompanionKind::WmoGroup { index, lod: 1 });
        assert_eq!(pending(&mut reader), request);
        reader.provide(&request, groups[index].1.clone()).unwrap();
    }
    let model = ready(&mut reader).value.try_narrow::<Wmo>().unwrap();
    drop((reader, root, groups));
    assert_eq!(model.meshes().len(), 10);
    let mut actual: Vec<_> = model
        .instances()
        .values()
        .map(|instance| (instance.lod.unwrap(), instance.properties.file_id.unwrap()))
        .collect();
    let mut expected: Vec<_> = base_ids
        .into_iter()
        .map(|id| (0, id))
        .chain(
            extra_ids
                .into_iter()
                .filter(|id| *id != 0)
                .map(|id| (1, id)),
        )
        .collect();
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected);
    model.validate().unwrap();
}

#[test]
#[ignore = "explicit incremental WMO dependency-discovery measurement"]
fn wmo_dependency_discovery_scaling() {
    use std::hint::black_box;
    use std::time::Instant;
    for count in [128, 512, 2048] {
        let (root, groups) = many_wmo_groups(count);
        let mut elapsed = Vec::new();
        for _ in 0..5 {
            let mut reader = Reader::new(
                Format::Wmo(wmo::Profile::Classic),
                FileKey::FileId(700),
                root.clone(),
                ImportOptions::default(),
            );
            let start = Instant::now();
            for (index, (key, bytes)) in groups.iter().enumerate() {
                let request = pending(&mut reader);
                assert_eq!(&request.key, key);
                assert_eq!(request.kind, CompanionKind::WmoGroup { index, lod: 0 });
                reader.provide(&request, bytes.clone()).unwrap();
            }
            elapsed.push(start.elapsed());
            // Final group decoding and semantic import are outside discovery.
            black_box(reader);
        }
        elapsed.sort();
        eprintln!(
            "WMO dependency discovery: groups={count}, root_bytes={}, median_ns={}, ns_per_group={:.1}",
            root.len(),
            elapsed[2].as_nanos(),
            elapsed[2].as_nanos() as f64 / count as f64,
        );
    }
}

#[test]
fn wmo_profile_is_explicit_and_required_group_identity_is_preserved() {
    let (root, group) = wmo_fixture::<Classic>();
    let root_key = FileKey::Path("World/house.wmo".into());
    let mut reader = Reader::new(
        Format::Wmo(wmo::Profile::Classic),
        root_key.clone(),
        root,
        ImportOptions::default(),
    );
    let request = pending(&mut reader);
    assert_eq!(request.owner, root_key);
    assert_eq!(request.key, FileKey::Path("World/house_000.wmo".into()));
    assert_eq!(request.kind, CompanionKind::WmoGroup { index: 0, lod: 0 });
    reader.provide(&request, group).unwrap();
    let asset = ready(&mut reader);
    let imported = asset;
    drop(reader);
    assert_eq!(imported.value.meshes().len(), 1);
    assert_eq!(
        imported
            .value
            .meshes()
            .values()
            .next()
            .unwrap()
            .positions()
            .len(),
        3
    );
}

#[test]
fn every_modern_wmo_profile_loads_and_imports_its_own_group_layout() {
    fn check<D: Dialect>(profile: wmo::Profile) {
        let (root, group) = wmo_fixture::<D>();
        let mut reader = Reader::new(
            Format::Wmo(profile),
            FileKey::Root,
            root,
            ImportOptions::default(),
        );
        let request = pending(&mut reader);
        reader.provide(&request, group).unwrap();
        let asset = ready(&mut reader);
        assert_eq!(asset.value.meshes().len(), 1);
    }
    check::<wmo::Classic>(wmo::Profile::Classic);
    check::<wmo::Legion>(wmo::Profile::Legion);
    check::<wmo::Bfa>(wmo::Profile::Bfa);
    check::<wmo::ShadowlandsPre92>(wmo::Profile::ShadowlandsPre92);
    check::<wmo::Shadowlands>(wmo::Profile::Shadowlands);
    check::<wmo::Dragonflight>(wmo::Profile::Dragonflight);
    check::<wmo::WarWithin>(wmo::Profile::WarWithin);
}

#[test]
fn a_malformed_wmo_companion_reports_its_own_identity() {
    let (root, _) = wmo_fixture::<Classic>();
    let mut reader = Reader::new(
        Format::Wmo(wmo::Profile::Classic),
        FileKey::Root,
        root,
        ImportOptions::default(),
    );
    let request = pending(&mut reader);
    reader.provide(&request, b"invalid group".to_vec()).unwrap();
    let error = reader_error(&mut reader);
    assert!(matches!(&error, Error::Decode { key, .. } if key == &request.key));
    assert!(error.source().is_some());
}

#[test]
fn wmo_root_siblings_preserve_raw_and_unassigned_native_identities() {
    for owner in [FileKey::Root, FileKey::RawPath(b"World\\\xff.wmo".to_vec())] {
        let (root, group) = wmo_fixture::<Classic>();
        let mut reader = Reader::new(
            Format::Wmo(wmo::Profile::Classic),
            owner.clone(),
            root,
            ImportOptions::default(),
        );
        let request = pending(&mut reader);
        assert_eq!(request.owner, owner);
        assert_eq!(request.key, owner.sibling("_000.wmo"));
        reader.provide(&request, group).unwrap();
        assert_eq!(ready(&mut reader).value.meshes().len(), 1);
    }
}

#[test]
fn wmo_group_aliases_reuse_the_supplied_file_without_another_request() {
    let mut root = Root::<Classic>::new();
    root.header_mut().group_count = 2;
    root.insert_chunk(
        0,
        RootChunk::Groups(
            WmoArray::from_records(&[records::GroupInfo::default(), records::GroupInfo::default()])
                .unwrap(),
        ),
    )
    .unwrap();
    root.insert_chunk(
        1,
        RootChunk::GroupFileIds(WmoArray::from_records(&[100_u32, 100]).unwrap()),
    )
    .unwrap();
    let mut reader = Reader::new(
        Format::Wmo(wmo::Profile::Classic),
        FileKey::Root,
        root.encode().unwrap(),
        ImportOptions::default(),
    );
    let request = pending(&mut reader);
    assert_eq!(request.key, FileKey::FileId(100));
    reader
        .provide(&request, Group::<Classic>::new().encode().unwrap())
        .unwrap();
    let converted = ready(&mut reader);
    assert_eq!(converted.value.meshes().len(), 2);
    assert_eq!(converted.value.instances().len(), 2);
}

#[test]
fn historical_wmo_profiles_fail_explicitly_instead_of_guessing_a_modern_layout() {
    for profile in [wmo::Profile::Alpha14, wmo::Profile::Legacy16] {
        let mut reader = Reader::new(
            Format::Wmo(profile),
            FileKey::Root,
            Vec::new(),
            ImportOptions::default(),
        );
        assert!(
            matches!(reader.poll(), Err(Error::UnsupportedProfile { profile: actual }) if actual == profile)
        );
    }
}

#[test]
fn synchronous_loading_drives_recursive_requests_without_app_owned_dependency_scanning() {
    let mut requests = Vec::new();
    let asset = load(
        Format::M2,
        FileKey::FileId(1),
        skeleton_root(),
        ImportOptions::default(),
        |request| {
            requests.push(request.clone());
            match request.key {
                FileKey::FileId(10) => Ok(skeleton(vec![
                    parent(11),
                    SkeletonChunk::Bones {
                        bones: vec![Bone::new()],
                        lookup: ZeroVec::new(),
                    },
                ])),
                FileKey::FileId(11) => Ok(skeleton(Vec::new())),
                _ => Err(io::Error::from(io::ErrorKind::NotFound)),
            }
        },
    )
    .unwrap();
    assert_eq!(
        requests,
        [
            Request {
                owner: FileKey::FileId(1),
                key: FileKey::FileId(10),
                kind: CompanionKind::Skeleton,
            },
            Request {
                owner: FileKey::FileId(10),
                key: FileKey::FileId(11),
                kind: CompanionKind::Skeleton,
            },
        ]
    );
    assert_eq!(asset.value.nodes().len(), 1);
    asset.value.validate().unwrap();
}

#[test]
fn synchronous_source_errors_retain_the_request_and_distinguish_missing_from_unreadable() {
    for kind in [io::ErrorKind::NotFound, io::ErrorKind::PermissionDenied] {
        let error = load(
            Format::M2,
            FileKey::FileId(1),
            skeleton_root(),
            ImportOptions::default(),
            |_| Err(io::Error::from(kind)),
        )
        .unwrap_err();
        let source = error.source().unwrap().downcast_ref::<io::Error>().unwrap();
        assert_eq!(source.kind(), kind);
        let LoadError::Companion { request, source } = error else {
            panic!("source failure was reclassified as invalid native data");
        };
        assert_eq!(request.owner, FileKey::FileId(1));
        assert_eq!(request.key, FileKey::FileId(10));
        assert_eq!(request.kind, CompanionKind::Skeleton);
        assert_eq!(source.kind(), kind);
    }
}

#[test]
fn synchronous_native_failures_do_not_invoke_the_storage_resolver() {
    let result = load(
        Format::M2,
        FileKey::FileId(1),
        b"MD20".to_vec(),
        ImportOptions::default(),
        |_| {
            panic!("a malformed root cannot request companions");
            #[allow(unreachable_code)]
            Ok::<_, io::Error>(Vec::new())
        },
    );
    assert!(matches!(
        result,
        Err(LoadError::Native(Error::Decode {
            source: DecodeError::M2(_),
            ..
        }))
    ));
}

#[test]
fn import_failure_follows_required_companions_and_preserves_fixed_policy_on_retry() {
    let mut root = Document::<Chunked<V274>>::new();
    root.core_mut().skins = 1;
    root.insert_extension(1, Chunk::SkinIds(ZeroVec::alloc_from_slice(&[42])))
        .unwrap();
    root.insert_extension(
        2,
        Chunk::Unknown {
            tag: *b"TEST",
            data: Cow::Owned(vec![1, 2, 3]),
        },
    )
    .unwrap();
    let key = FileKey::RawPath(b"models/\xff.m2".to_vec());
    let mut reader = Reader::new(
        Format::M2,
        key.clone(),
        root.encode_m2().unwrap(),
        ImportOptions::default(),
    );
    let request = pending(&mut reader);
    assert_eq!(request.owner, key);
    assert_eq!(request.key, FileKey::FileId(42));
    let skin = Skin::<V274> {
        profile: SkinProfile::default(),
    }
    .write()
    .unwrap();
    reader.provide(&request, skin).unwrap();
    for _ in 0..2 {
        let error = reader_error(&mut reader);
        assert!(matches!(&error, Error::Import { key: actual, .. } if actual == &key));
        let source = error
            .source()
            .unwrap()
            .downcast_ref::<wc3::model::conversion::Error>()
            .unwrap();
        assert!(source.to_string().contains("m2.chunk"));
    }
    let error = reader.provide(&request, Vec::new()).unwrap_err();
    assert!(
        matches!(error, Error::UnexpectedCompanion { expected: None, supplied } if *supplied == request)
    );
    assert!(matches!(reader.poll(), Err(Error::Import { .. })));
}
