//! Public bundle checks using independently assembled animation streams and
//! explicit document identities, including aliases and inherited skeletons.
use std::borrow::Cow;
use wc3::model::formats::m2::records::{Interpolation, ModelFlags, SequenceFlags};

use wc3::model::formats::m2::{
    chunks::{AnimationFileId, Chunk},
    codec::{Array, ArrayRef},
    companions::{AnimationFile, AnimationStream, Skeleton, SkeletonChunk},
    records::{Bone, Sequence, TimelineStorage, TrackValues, ValuesLeaf, ValuesStorage},
    Bundle, Chunked, DependencyKind, Document, File, FileKey, Flat, ModelLayout, ResolvedValues,
    ZeroVec, V264, V272,
};

fn sequence<V: ModelLayout<Timing = u32>>(
    id: u16,
    variation: u16,
    flags: u32,
    alias: u16,
) -> Sequence<V> {
    Sequence {
        id,
        variation_index: variation,
        timing: 1000,
        flags: SequenceFlags::from_bits_retain(flags),
        variation_next: -1,
        alias_next: alias,
        ..Sequence::default()
    }
}
fn external_bone<V: ModelLayout>() -> Bone<'static, V> {
    let mut bone = Bone::<V>::new();
    bone.translation.base.interpolation = Interpolation::Linear;
    bone.translation
        .base
        .timeline
        .replace(
            0,
            Array::External(ArrayRef {
                count: 1,
                offset: 0,
            }),
        )
        .unwrap();
    bone.translation
        .values
        .replace(
            0,
            ValuesLeaf::Values(Array::External(ArrayRef {
                count: 1,
                offset: 16,
            })),
        )
        .unwrap();
    bone
}
fn animation_bytes() -> Vec<u8> {
    let mut bytes = vec![0; 44];
    bytes[0..4].copy_from_slice(&100_u32.to_le_bytes());
    bytes[4..8].copy_from_slice(&200_u32.to_le_bytes());
    for (index, value) in [11.0_f32, 12.0, 13.0].iter().enumerate() {
        bytes[16 + index * 4..20 + index * 4].copy_from_slice(&value.to_le_bytes());
    }
    for (index, value) in [21.0_f32, 22.0, 23.0].iter().enumerate() {
        bytes[32 + index * 4..36 + index * 4].copy_from_slice(&value.to_le_bytes());
    }
    bytes
}
fn path() -> FileKey {
    FileKey::Path("models/unit.m2".into())
}
fn raw_model() -> Document<'static, Flat<V264>> {
    let mut model = Document::new();
    model
        .core_mut()
        .sequences
        .push(sequence::<V264>(7, 0, 0, 0));
    model.core_mut().bones.push(external_bone::<V264>());
    model
}
fn root_animation_key() -> FileKey {
    FileKey::Path("models/unit0007-00.anim".into())
}
fn raw_bundle() -> Bundle<'static, Flat<V264>> {
    let mut bundle = Bundle::from_model(raw_model()).with_key(path());
    bundle.insert(
        root_animation_key(),
        File::Animation(AnimationFile::Raw(Cow::Owned(animation_bytes()))),
    );
    bundle
}
fn parent_chunk(id: u32) -> SkeletonChunk<'static, V272> {
    SkeletonChunk::Parent {
        reserved0: [0; 8],
        file_id: id,
        reserved1: [0; 4],
    }
}
fn skeleton(chunks: Vec<SkeletonChunk<'static, V272>>) -> Skeleton<'static, V272> {
    Skeleton {
        flags: ModelFlags::default(),
        chunks,
    }
}
fn skeleton_model() -> Document<'static, Chunked<V272>> {
    let mut model = Document::new();
    model.insert_extension(1, Chunk::SkeletonId(10)).unwrap();
    model
}

#[test]
fn complete_decode_rejects_missing_animation_documents() {
    let bytes = raw_model().encode_m2().unwrap();
    let result = Bundle::<Flat<V264>>::decode(&bytes, path(), |_| Ok(None));
    assert_eq!(result.unwrap_err().field, "missing supporting document");
}

#[test]
fn complete_bundle_rejects_external_bounds_and_mismatched_key_counts() {
    let mut bundle = raw_bundle();
    bundle.validate_complete().unwrap();
    bundle.model.core_mut().bones[0].translation.values =
        TrackValues::Values(vec![Array::External(ArrayRef {
            count: 1,
            offset: 1000,
        })]);
    assert!(bundle.validate_complete().is_err());
    bundle.model.core_mut().bones[0].translation.values =
        TrackValues::Values(vec![Array::External(ArrayRef {
            count: 2,
            offset: 16,
        })]);
    assert!(bundle.validate_complete().is_err());
}

#[test]
fn aliases_select_terminal_identity_but_requested_leaf_offsets() {
    let mut model = raw_model();
    model.core_mut().sequences = vec![
        sequence::<V264>(5, 3, 0x40, 1),
        sequence::<V264>(7, 0, 0, 0),
    ];
    let track = &mut model.core_mut().bones[0].translation;
    track
        .base
        .timeline
        .timestamps
        .push(Array::External(ArrayRef {
            count: 1,
            offset: 4,
        }));
    let TrackValues::Values(values) = &mut track.values else {
        panic!("ordinary values expected")
    };
    values.push(Array::External(ArrayRef {
        count: 1,
        offset: 32,
    }));
    let dependencies = model.dependencies(&path());
    let animations: Vec<_> = dependencies
        .iter()
        .filter(|dependency| matches!(dependency.kind, DependencyKind::Animation { .. }))
        .collect();
    assert_eq!(animations.len(), 1);
    assert_eq!(animations[0].key, root_animation_key());
    let mut bundle = Bundle::from_model(model).with_key(path());
    bundle.insert(
        root_animation_key(),
        File::Animation(AnimationFile::Raw(Cow::Owned(animation_bytes()))),
    );
    bundle.validate_complete().unwrap();
    let alias = bundle.bone_translation(0, 0).unwrap();
    let terminal = bundle.bone_translation(1, 0).unwrap();
    assert_eq!(alias.timestamps.get(0), Some(100));
    assert_eq!(terminal.timestamps.get(0), Some(200));
    let ResolvedValues::Values(alias) = alias.values else {
        panic!("ordinary values expected")
    };
    let ResolvedValues::Values(terminal) = terminal.values else {
        panic!("ordinary values expected")
    };
    assert_eq!(alias.get(0), Some([11.0, 12.0, 13.0]));
    assert_eq!(terminal.get(0), Some([21.0, 22.0, 23.0]));
}

#[test]
fn wrong_chunked_animation_stream_is_rejected_for_core_bones() {
    let mut model = Document::<Chunked<V272>>::new();
    model.core_mut().flags.set_chunked_animations(true);
    model
        .core_mut()
        .sequences
        .push(sequence::<V272>(7, 0, 0, 0));
    model.core_mut().bones.push(external_bone::<V272>());
    model
        .insert_extension(
            1,
            Chunk::AnimationIds(vec![AnimationFileId {
                animation_id: 7,
                variation_id: 0,
                file_id: 99,
            }]),
        )
        .unwrap();
    let mut bundle = Bundle::from_model(model).with_key(path());
    bundle.insert(
        FileKey::FileId(99),
        File::Animation(AnimationFile::Chunked(vec![AnimationStream {
            tag: *b"AFSB",
            data: Cow::Owned(animation_bytes()),
        }])),
    );
    assert!(bundle.validate_complete().is_err());
}

#[test]
fn skeleton_parent_cycles_terminate_without_repeated_resolver_calls() {
    let root = skeleton_model().encode_m2().unwrap();
    let child = skeleton(vec![parent_chunk(11)]).write().unwrap();
    let parent = skeleton(vec![parent_chunk(10)]).write().unwrap();
    let mut calls = 0;
    let result = Bundle::<Chunked<V272>>::decode(&root, path(), |dependency| {
        calls += 1;
        assert!(
            calls <= 3,
            "a parent cycle must not repeatedly resolve files"
        );
        Ok(match dependency.key {
            FileKey::FileId(10) => Some(child.as_slice()),
            FileKey::FileId(11) => Some(parent.as_slice()),
            _ => None,
        })
    });
    assert_eq!(result.unwrap_err().field, "skeleton parent cycle");
}

#[test]
fn child_skeleton_inherits_parent_sequences_and_animation_ids() {
    let root = skeleton_model().encode_m2().unwrap();
    let child = skeleton(vec![
        parent_chunk(11),
        SkeletonChunk::Bones {
            bones: vec![external_bone::<V272>()],
            lookup: ZeroVec::new(),
        },
    ])
    .write()
    .unwrap();
    let parent = skeleton(vec![
        SkeletonChunk::Sequences {
            global_loops: ZeroVec::new(),
            sequences: vec![sequence::<V272>(7, 0, 0, 0)],
            lookup: ZeroVec::new(),
            reserved: [0; 8],
        },
        SkeletonChunk::AnimationIds(vec![AnimationFileId {
            animation_id: 7,
            variation_id: 0,
            file_id: 99,
        }]),
    ])
    .write()
    .unwrap();
    let animation = AnimationFile::Chunked(vec![AnimationStream {
        tag: *b"AFSB",
        data: Cow::Owned(animation_bytes()),
    }])
    .write()
    .unwrap();
    let mut bundle = Bundle::<Chunked<V272>>::decode(&root, path(), |dependency| {
        Ok(match dependency.key {
            FileKey::FileId(10) => Some(child.as_slice()),
            FileKey::FileId(11) => Some(parent.as_slice()),
            FileKey::FileId(99) => Some(animation.as_slice()),
            _ => None,
        })
    })
    .unwrap();
    let resolved = bundle.bone_translation(0, 0).unwrap();
    assert_eq!(resolved.timestamps.get(0), Some(100));
    let ResolvedValues::Values(values) = resolved.values else {
        panic!("ordinary values expected")
    };
    assert_eq!(values.get(0), Some([11.0, 12.0, 13.0]));
    bundle
        .set_bone_translation(
            0,
            0,
            [300],
            ResolvedValues::Values(ZeroVec::alloc_from_slice(&[[41.0, 42.0, 43.0]])),
        )
        .unwrap();
    let encoded = bundle.encode().unwrap();
    let decoded = Bundle::<Chunked<V272>>::decode(&encoded.model, path(), |dependency| {
        Ok(encoded.files.get(&dependency.key).map(Vec::as_slice))
    })
    .unwrap();
    let resolved = decoded.bone_translation(0, 0).unwrap();
    assert_eq!(resolved.timestamps.get(0), Some(300));
    let ResolvedValues::Values(values) = resolved.values else {
        panic!("ordinary values expected")
    };
    assert_eq!(values.get(0), Some([41.0, 42.0, 43.0]));
}

#[test]
fn bone_translation_edit_rejects_nonexistent_sequences_before_mutation() {
    let mut bundle = raw_bundle();
    let before = bundle.clone();
    assert!(bundle
        .set_bone_translation(
            8,
            0,
            [0],
            ResolvedValues::Values(ZeroVec::alloc_from_slice(&[[1.0, 2.0, 3.0]]))
        )
        .is_err());
    assert_eq!(bundle, before);
}

#[test]
fn skeleton_track_key_count_mismatch_is_not_a_complete_bundle() {
    let mut bone = external_bone::<V272>();
    bone.translation.values = TrackValues::Values(vec![Array::External(ArrayRef {
        count: 2,
        offset: 16,
    })]);
    let skel = skeleton(vec![
        SkeletonChunk::Bones {
            bones: vec![bone],
            lookup: ZeroVec::new(),
        },
        SkeletonChunk::Sequences {
            global_loops: ZeroVec::new(),
            sequences: vec![sequence::<V272>(7, 0, 0, 0)],
            lookup: ZeroVec::new(),
            reserved: [0; 8],
        },
        SkeletonChunk::AnimationIds(vec![AnimationFileId {
            animation_id: 7,
            variation_id: 0,
            file_id: 99,
        }]),
    ]);
    let mut bundle = Bundle::from_model(skeleton_model()).with_key(path());
    bundle.insert(FileKey::FileId(10), skel.into());
    bundle.insert(
        FileKey::FileId(99),
        File::Animation(AnimationFile::Chunked(vec![AnimationStream {
            tag: *b"AFSB",
            data: Cow::Owned(animation_bytes()),
        }])),
    );
    assert!(bundle.validate_complete().is_err());
}

#[test]
fn inconsistent_existing_track_rejects_edits_without_partial_mutation() {
    let mut bundle = raw_bundle();
    bundle.model.core_mut().bones[0].translation.values =
        TrackValues::Splines(vec![Array::External(ArrayRef {
            count: 1,
            offset: 16,
        })]);
    let before = bundle.clone();
    assert!(bundle
        .set_bone_translation(
            0,
            0,
            [500],
            ResolvedValues::Values(ZeroVec::alloc_from_slice(&[[1.0, 2.0, 3.0]]))
        )
        .is_err());
    assert_eq!(bundle, before);
}

#[test]
fn skeleton_alias_cycles_are_rejected_even_without_animation_leaves() {
    let skel = skeleton(vec![SkeletonChunk::Sequences {
        global_loops: ZeroVec::new(),
        sequences: vec![sequence::<V272>(7, 0, 0x140, 0)],
        lookup: ZeroVec::new(),
        reserved: [0; 8],
    }]);
    let mut bundle = Bundle::from_model(skeleton_model()).with_key(path());
    bundle.insert(FileKey::FileId(10), skel.into());
    assert!(bundle.validate_complete().is_err());
}

#[test]
fn duplicate_core_payloads_cannot_silently_skip_external_relocation() {
    let mut first = Document::<Flat<V272>>::new();
    first
        .core_mut()
        .sequences
        .push(sequence::<V272>(7, 0, 0, 0));
    first.core_mut().bones.push(external_bone::<V272>());
    let mut second = first.clone();
    let track = &mut second.core_mut().bones[0].translation;
    track.base.timeline.timestamps[0] = Array::External(ArrayRef {
        count: 1,
        offset: 4,
    });
    track.values = TrackValues::Values(vec![Array::External(ArrayRef {
        count: 1,
        offset: 32,
    })]);
    let mut bytes = Vec::new();
    for core in [first.encode_m2().unwrap(), second.encode_m2().unwrap()] {
        bytes.extend_from_slice(b"MD21");
        bytes.extend_from_slice(&(core.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&core);
    }
    let animation = animation_bytes();
    let result =
        Bundle::<Chunked<V272>>::decode(&bytes, path(), |_| Ok(Some(animation.as_slice())));
    assert!(
        result.is_err(),
        "duplicate cores need explicit validation or complete independent relocation"
    );
}

#[test]
fn raw_animation_framing_is_rejected_for_skeleton_owned_native_streams() {
    let skel = skeleton(vec![
        SkeletonChunk::Bones {
            bones: vec![external_bone::<V272>()],
            lookup: ZeroVec::new(),
        },
        SkeletonChunk::Sequences {
            global_loops: ZeroVec::new(),
            sequences: vec![sequence::<V272>(7, 0, 0, 0)],
            lookup: ZeroVec::new(),
            reserved: [0; 8],
        },
        SkeletonChunk::AnimationIds(vec![AnimationFileId {
            animation_id: 7,
            variation_id: 0,
            file_id: 99,
        }]),
    ]);
    let mut bundle = Bundle::from_model(skeleton_model()).with_key(path());
    bundle.insert(FileKey::FileId(10), skel.into());
    bundle.insert(
        FileKey::FileId(99),
        File::Animation(AnimationFile::Raw(Cow::Owned(animation_bytes()))),
    );
    assert!(
        bundle.validate_complete().is_err(),
        "skeleton-owned animation streams must retain chunk framing"
    );
}

#[test]
fn zero_parent_skeleton_id_keeps_inherited_sequence_context() {
    let mut model = skeleton_model();
    model.core_mut().flags.set_chunked_animations(true);
    model
        .core_mut()
        .sequences
        .push(sequence::<V272>(7, 0, 0, 0));
    model
        .insert_extension(
            2,
            Chunk::AnimationIds(vec![AnimationFileId {
                animation_id: 7,
                variation_id: 0,
                file_id: 99,
            }]),
        )
        .unwrap();
    let root = model.encode_m2().unwrap();
    let child = skeleton(vec![
        parent_chunk(0),
        SkeletonChunk::Bones {
            bones: vec![external_bone::<V272>()],
            lookup: ZeroVec::new(),
        },
        SkeletonChunk::AnimationIds(vec![AnimationFileId {
            animation_id: 7,
            variation_id: 0,
            file_id: 99,
        }]),
    ])
    .write()
    .unwrap();
    let animation = AnimationFile::Chunked(vec![AnimationStream {
        tag: *b"AFSB",
        data: Cow::Owned(animation_bytes()),
    }])
    .write()
    .unwrap();
    let bundle = Bundle::<Chunked<V272>>::decode(&root, path(), |dependency| {
        assert_ne!(dependency.key, FileKey::FileId(0));
        Ok(match dependency.key {
            FileKey::FileId(10) => Some(child.as_slice()),
            FileKey::FileId(99) => Some(animation.as_slice()),
            _ => None,
        })
    })
    .unwrap();
    let resolved = bundle.bone_translation(0, 0).unwrap();
    assert_eq!(resolved.timestamps.get(0), Some(100));
    let ResolvedValues::Values(values) = resolved.values else {
        panic!("ordinary values expected")
    };
    assert_eq!(values.get(0), Some([11.0, 12.0, 13.0]));
}

#[test]
fn nested_model_skin_uses_its_native_owner_layout() {
    use wc3::model::formats::m2::{
        companions::{LayoutKind, Skin, SkinProfile},
        records::ParticleEmitter,
        V274,
    };
    let mut root = Document::<Chunked<V274>>::new();
    root.core_mut()
        .particles
        .push(ParticleEmitter::new(ModelFlags::default()));
    root.insert_extension(
        1,
        Chunk::GeometryParticleIds(ZeroVec::alloc_from_slice(&[33])),
    )
    .unwrap();
    let mut child = Document::<Chunked<V264>>::new();
    child.core_mut().skins = 1;
    child
        .insert_extension(1, Chunk::SkinIds(ZeroVec::alloc_from_slice(&[66])))
        .unwrap();
    let skin = Skin::<V264> {
        profile: SkinProfile::default(),
    }
    .write()
    .unwrap();
    assert_eq!(skin.len(), 48);
    let root_bytes = root.encode_m2().unwrap();
    let child_bytes = child.encode_m2().unwrap();
    let bundle = Bundle::<Chunked<V274>>::decode(&root_bytes, FileKey::Root, |dependency| {
        Ok(match dependency.key {
            FileKey::FileId(33) => Some(child_bytes.as_slice()),
            FileKey::FileId(66) => Some(skin.as_slice()),
            _ => None,
        })
    })
    .unwrap();
    let Some(File::Skin(native)) = bundle.file(&FileKey::FileId(66)) else {
        panic!("native skin")
    };
    assert_eq!(native.layout(), LayoutKind::V264);
    assert!(native.as_typed::<V264>().is_some());
    assert!(native.as_typed::<V274>().is_none());
    let encoded = bundle.encode().unwrap();
    assert_eq!(encoded.files[&FileKey::FileId(66)].len(), 48);
    let decoded = Bundle::<Chunked<V274>>::decode(&encoded.model, FileKey::Root, |dependency| {
        Ok(encoded.files.get(&dependency.key).map(Vec::as_slice))
    })
    .unwrap();
    assert_eq!(decoded, bundle);
}

#[test]
fn cached_skin_cannot_be_reused_by_an_owner_with_a_different_layout() {
    use wc3::model::formats::m2::{
        companions::{Skin, SkinProfile},
        V274,
    };
    let mut root = Document::<Chunked<V274>>::new();
    root.core_mut().skins = 1;
    root.insert_extension(1, Chunk::SkinIds(ZeroVec::alloc_from_slice(&[66])))
        .unwrap();
    let mut bundle = Bundle::from_model(root);
    bundle.insert(
        FileKey::FileId(66),
        Skin::<V264> {
            profile: SkinProfile::default(),
        }
        .into(),
    );
    assert_eq!(
        bundle.validate_complete().unwrap_err().field,
        "supporting document owner layout"
    );
}
