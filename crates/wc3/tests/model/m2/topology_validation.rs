//! Public semantic checks against an independent bounded-walk graph oracle.
use wc3::model::formats::m2::companions::{Skeleton, SkeletonChunk};
use wc3::model::formats::m2::records::{Bone, ModelFlags, Sequence, SequenceFlags};
use wc3::model::formats::m2::{Core, ZeroVec, V274};

fn bones(parents: &[i16]) -> Vec<Bone<'static, V274>> {
    parents
        .iter()
        .map(|&parent| {
            let mut bone = Bone::new();
            bone.parent_bone_id = parent;
            bone
        })
        .collect()
}

fn skeleton(parents: &[i16]) -> Skeleton<'static, V274> {
    Skeleton {
        flags: ModelFlags::default(),
        chunks: vec![SkeletonChunk::Bones {
            bones: bones(parents),
            lookup: ZeroVec::new(),
        }],
    }
}

// One literal V274 SKB1, with no tracks or key-bone lookup. Neither the native
// record encoder nor its computed layout participates in constructing this input.
fn bone_fixture(parent: i16) -> Vec<u8> {
    let mut bytes = vec![0; 112];
    bytes[..4].copy_from_slice(b"SKB1");
    bytes[4..8].copy_from_slice(&104u32.to_le_bytes());
    bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
    bytes[12..16].copy_from_slice(&16u32.to_le_bytes());
    bytes[24..28].copy_from_slice(&(-1i32).to_le_bytes());
    bytes[32..34].copy_from_slice(&parent.to_le_bytes());
    for offset in [42, 62, 82] {
        bytes[offset..offset + 2].copy_from_slice(&(-1i16).to_le_bytes());
    }
    bytes
}

#[test]
fn skeleton_decoding_preserves_semantically_invalid_bone_records() {
    for (parent, field) in [
        (-2, "bone parent sentinel"),
        (1, "bone parent index"),
        (0, "bone parent cycle"),
    ] {
        let bytes = bone_fixture(parent);
        let parsed = Skeleton::<V274>::read(&bytes, ModelFlags::default()).unwrap();
        assert_eq!(parsed.write().unwrap(), bytes);
        let error = parsed.validate().unwrap_err();
        assert_eq!((error.offset, error.field), (0, field));
    }
    let bytes = bone_fixture(-1);
    let parsed = Skeleton::<V274>::read(&bytes, ModelFlags::default()).unwrap();
    parsed.validate().unwrap();
    for length in 1..bytes.len() {
        assert!(
            Skeleton::<V274>::read(&bytes[..length], ModelFlags::default()).is_err(),
            "prefix {length}"
        );
    }
    for (offset, value) in [(8, u32::MAX), (12, u32::MAX), (8, 2)] {
        let mut invalid = bytes.clone();
        invalid[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(Skeleton::<V274>::read(&invalid, ModelFlags::default()).is_err());
    }
}

fn valid_parents(parents: &[i16]) -> bool {
    for start in 0..parents.len() {
        let mut current = start;
        let mut terminated = false;
        for _ in 0..=parents.len() {
            match parents.get(current).copied() {
                Some(-1) => {
                    terminated = true;
                    break;
                }
                Some(parent) if parent >= 0 => current = parent as usize,
                _ => return false,
            }
        }
        if !terminated {
            return false;
        }
    }
    true
}

#[test]
fn inline_and_external_bone_graphs_match_exhaustive_walk_oracle() {
    for count in 0..=4u32 {
        let radix = count + 3;
        for mut encoding in 0..radix.pow(count) {
            let parents: Vec<_> = (0..count)
                .map(|_| {
                    let parent = (encoding % radix) as i16 - 2;
                    encoding /= radix;
                    parent
                })
                .collect();
            let expected = valid_parents(&parents);
            assert_eq!(
                skeleton(&parents).validate().is_ok(),
                expected,
                "external {parents:?}"
            );
            let mut core = Core::<V274>::new();
            core.bones = bones(&parents);
            assert_eq!(core.validate().is_ok(), expected, "inline {parents:?}");
        }
    }
}

fn sequences(links: &[Option<u16>]) -> Vec<Sequence<V274>> {
    links
        .iter()
        .map(|&link| Sequence {
            flags: SequenceFlags::from_bits_retain(if link.is_some() { 0x40 } else { 0 }),
            alias_next: link.unwrap_or(u16::MAX),
            variation_next: -1,
            ..Sequence::default()
        })
        .collect()
}

fn valid_aliases(links: &[Option<u16>]) -> bool {
    for start in 0..links.len() {
        let mut current = start;
        let mut terminated = false;
        for _ in 0..=links.len() {
            match links.get(current) {
                Some(None) => {
                    terminated = true;
                    break;
                }
                Some(Some(next)) => current = usize::from(*next),
                None => return false,
            }
        }
        if !terminated {
            return false;
        }
    }
    true
}

#[test]
fn inline_and_external_alias_graphs_match_exhaustive_walk_oracle() {
    for count in 0..=4u32 {
        let radix = count + 2;
        for mut encoding in 0..radix.pow(count) {
            let links: Vec<_> = (0..count)
                .map(|_| {
                    let digit = encoding % radix;
                    encoding /= radix;
                    if digit == 0 {
                        None
                    } else {
                        Some((digit - 1) as u16)
                    }
                })
                .collect();
            let expected = valid_aliases(&links);
            let mut core = Core::<V274>::new();
            core.sequences = sequences(&links);
            assert_eq!(core.validate().is_ok(), expected, "inline {links:?}");
            let external = Skeleton::<V274> {
                flags: ModelFlags::default(),
                chunks: vec![SkeletonChunk::Sequences {
                    global_loops: ZeroVec::new(),
                    sequences: sequences(&links),
                    lookup: ZeroVec::new(),
                    reserved: [0; 8],
                }],
            };
            assert_eq!(external.validate().is_ok(), expected, "external {links:?}");
        }
    }
}

#[test]
fn skeleton_lookup_tables_validate_only_live_indices() {
    for lookup in [0, u16::MAX, 1] {
        let mut bytes = bone_fixture(-1);
        bytes[4..8].copy_from_slice(&106u32.to_le_bytes());
        bytes[16..20].copy_from_slice(&1u32.to_le_bytes());
        bytes[20..24].copy_from_slice(&104u32.to_le_bytes());
        bytes.extend_from_slice(&lookup.to_le_bytes());
        let parsed = Skeleton::<V274>::read(&bytes, ModelFlags::default()).unwrap();
        let SkeletonChunk::Bones {
            lookup: borrowed, ..
        } = &parsed.chunks[0]
        else {
            panic!("bone chunk")
        };
        assert_eq!(borrowed.as_bytes().as_ptr(), bytes[112..].as_ptr());
        let encoded = parsed.write().unwrap();
        let roundtrip = Skeleton::<V274>::read(&encoded, ModelFlags::default()).unwrap();
        assert_eq!(roundtrip, parsed);
        if lookup == 1 {
            assert_eq!(parsed.validate().unwrap_err().field, "lookup index");
        } else {
            parsed.validate().unwrap();
        }
    }
    for missing in [false, true] {
        let lookup = ZeroVec::alloc_from_slice(&[if missing { u16::MAX } else { 0 }]);
        for chunk in [
            SkeletonChunk::<V274>::Bones {
                bones: vec![],
                lookup: lookup.clone(),
            },
            SkeletonChunk::<V274>::Attachments {
                attachments: vec![],
                lookup: lookup.clone(),
            },
            SkeletonChunk::<V274>::Sequences {
                global_loops: ZeroVec::new(),
                sequences: vec![],
                lookup: lookup.clone(),
                reserved: [0; 8],
            },
        ] {
            let parsed = Skeleton {
                flags: ModelFlags::default(),
                chunks: vec![chunk],
            };
            assert_eq!(parsed.validate().is_ok(), missing);
        }
        let mut core = Core::<V274>::new();
        core.key_bone_lookup = lookup;
        assert_eq!(core.validate().is_ok(), missing);
    }
}

#[test]
fn alias_errors_distinguish_cycles_and_missing_targets() {
    for (links, field) in [
        (vec![Some(0)], "animation alias cycle"),
        (vec![Some(1)], "animation alias index"),
        (vec![None, Some(2), Some(1)], "animation alias cycle"),
    ] {
        let mut core = Core::<V274>::new();
        core.sequences = sequences(&links);
        assert_eq!(core.validate().unwrap_err().field, field);
        let external = Skeleton::<V274> {
            flags: ModelFlags::default(),
            chunks: vec![SkeletonChunk::Sequences {
                global_loops: ZeroVec::new(),
                sequences: sequences(&links),
                lookup: ZeroVec::new(),
                reserved: [0; 8],
            }],
        };
        assert_eq!(external.validate().unwrap_err().field, field);
    }
}

#[test]
fn parent_skeleton_supplies_missing_tables_without_replacing_local_ones() {
    let parent = SkeletonChunk::Parent {
        reserved0: [0; 8],
        file_id: 17,
        reserved1: [0; 4],
    };
    let mut inherited = Skeleton::<V274> {
        flags: ModelFlags::default(),
        chunks: vec![parent],
    };
    inherited.validate().unwrap();
    let bytes = inherited.write().unwrap();
    Skeleton::<V274>::read(&bytes, ModelFlags::default())
        .unwrap()
        .validate()
        .unwrap();
    inherited.chunks.push(SkeletonChunk::Bones {
        bones: bones(&[1]),
        lookup: ZeroVec::new(),
    });
    assert_eq!(inherited.validate().unwrap_err().field, "bone parent index");
}

#[test]
fn long_shared_chains_validate_without_recursion() {
    let count = 8192;
    let mut parents: Vec<_> = (0..count).map(|index| (index + 1) as i16).collect();
    parents[count - 1] = -1;
    let mut core = Core::<V274>::new();
    core.bones = bones(&parents);
    core.validate().unwrap();
    skeleton(&parents).validate().unwrap();
    parents[count - 1] = 0;
    assert_eq!(
        skeleton(&parents).validate().unwrap_err().field,
        "bone parent cycle"
    );

    let mut links: Vec<_> = (0..count).map(|index| Some((index + 1) as u16)).collect();
    links[count - 1] = None;
    core.bones.clear();
    core.sequences = sequences(&links);
    core.validate().unwrap();
    links[count - 1] = Some(0);
    core.sequences = sequences(&links);
    assert_eq!(core.validate().unwrap_err().field, "animation alias cycle");
}
