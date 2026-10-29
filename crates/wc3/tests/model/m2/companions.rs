//! Independent M2 companion layouts, external stream contexts, and declared references.

mod chunks {
    use wc3::model::common::IntoZeroVec;
    use wc3::model::formats::m2::chunks::*;
    use wc3::model::formats::m2::records::ModelFlags;
    use wc3::model::formats::m2::V274;

    #[test]
    fn exp2_uses_descriptor_header_and_relocates_particle_keys() {
        // Independent layout: 16-byte EXP2 header, one 28-byte extension,
        // and two u16 arrays at non-contiguous offsets.
        let mut data = vec![0; 84];
        for (offset, value) in [(0, 1u32), (4, 16), (28, 1), (32, 64), (36, 1), (40, 80)] {
            data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        data[16..20].copy_from_slice(&2.0f32.to_le_bytes());
        data[20..24].copy_from_slice(&3.0f32.to_le_bytes());
        data[24..28].copy_from_slice(&4.0f32.to_le_bytes());
        data[64..66].copy_from_slice(&123u16.to_le_bytes());
        data[80..82].copy_from_slice(&456u16.to_le_bytes());
        let context = ChunkContext::<V274>::new(ModelFlags::default(), Some(0), [].into());
        let mut chunk = Chunk::read(*b"EXP2", &data, &context).unwrap();
        let Chunk::ParticleExtensions { entries, .. } = &mut chunk else {
            panic!()
        };
        assert_eq!(entries[0].alpha_cutoff.timestamps.get(0), Some(123));
        assert_eq!(entries[0].alpha_cutoff.values.get(0), Some(456));
        entries[0].alpha_cutoff.timestamps = [123, 999].into_zero_vec();
        entries[0].alpha_cutoff.values = [456, 777].into_zero_vec();
        let bytes = chunk.payload(ModelFlags::default()).unwrap();
        assert_eq!(Chunk::read(*b"EXP2", &bytes, &context).unwrap(), chunk);
    }

    #[test]
    fn parent_blacklist_uses_offset_array_not_flat_u16s() {
        let data = [
            2u32.to_le_bytes(),
            16u32.to_le_bytes(),
            0u32.to_le_bytes(),
            0u32.to_le_bytes(),
            0x00170005u32.to_le_bytes(),
        ]
        .concat();
        let c = ChunkContext::<V274>::new(ModelFlags::default(), Some(0), [].into());
        let chunk = Chunk::read(*b"PABC", &data, &c).unwrap();
        let Chunk::ParentAnimationBlacklist(values) = &chunk else {
            panic!()
        };
        assert_eq!(values.iter().collect::<Vec<_>>(), vec![5, 23]);
        let bytes = chunk.payload(ModelFlags::default()).unwrap();
        assert_eq!(Chunk::read(*b"PABC", &bytes, &c).unwrap(), chunk);
    }
}

mod skeleton {
    use wc3::model::formats::m2::codec::Array;
    use wc3::model::formats::m2::codec::ArrayRef;
    use wc3::model::formats::m2::companions::{Skeleton, SkeletonChunk};
    use wc3::model::formats::m2::records::ModelFlags;
    use wc3::model::formats::m2::V274;

    fn word(data: &mut [u8], offset: usize, value: u32) {
        data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn chunk(tag: [u8; 4], data: &[u8]) -> Vec<u8> {
        let mut bytes = tag.to_vec();
        bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(data);
        bytes
    }
    fn attachment_external() -> Vec<u8> {
        let mut bytes = vec![0; 88];
        for (offset, value) in [
            (0, 1),
            (4, 16),
            (40, 1),
            (44, 64),
            (48, 1),
            (52, 80),
            (64, 1),
            (68, 0xf000_0000),
            (80, 1),
            (84, 0xf000_0004),
        ] {
            word(&mut bytes, offset, value);
        }
        bytes[38..40].copy_from_slice(&u16::MAX.to_le_bytes());
        chunk(*b"SKA1", &bytes)
    }
    #[test]
    fn inherited_sequences_preserve_external_attachment_keys() {
        let bytes = attachment_external();
        let skeleton =
            Skeleton::<V274>::read_with_sequences(&bytes, ModelFlags::default(), &[true]).unwrap();
        let SkeletonChunk::Attachments { attachments, .. } = &skeleton.chunks[0] else {
            panic!()
        };
        assert_eq!(
            attachments[0].animate.base.timeline.timestamps,
            vec![Array::External(ArrayRef {
                count: 1,
                offset: 0xf000_0000
            })]
        );
        assert!(
            Skeleton::<V274>::read_with_sequences(&bytes, ModelFlags::default(), &[false]).is_err()
        );
    }
    #[test]
    fn local_sequences_override_inherited_external_flags() {
        let mut bytes = attachment_external();
        let mut sequences = vec![0; 96];
        word(&mut sequences, 8, 1);
        word(&mut sequences, 12, 32);
        word(&mut sequences, 44, 0x130);
        bytes.extend(chunk(*b"SKS1", &sequences));
        assert!(
            Skeleton::<V274>::read_with_sequences(&bytes, ModelFlags::default(), &[true]).is_err()
        );
    }
    #[test]
    fn last_local_sequence_chunk_supplies_context_without_losing_earlier_records() {
        let mut bytes = attachment_external();
        let mut inline = vec![0; 96];
        word(&mut inline, 8, 1);
        word(&mut inline, 12, 32);
        word(&mut inline, 44, 0x130);
        bytes.extend(chunk(*b"SKS1", &inline));
        let mut external = inline.clone();
        word(&mut external, 44, 0);
        bytes.extend(chunk(*b"SKS1", &external));
        let skeleton = Skeleton::<V274>::read(&bytes, ModelFlags::default()).unwrap();
        for (index, expected) in [(1, false), (2, true)] {
            let SkeletonChunk::Sequences { sequences, .. } = &skeleton.chunks[index] else {
                panic!("local sequences")
            };
            assert_eq!(sequences[0].flags.external_animation(), expected);
        }
        let written = skeleton.write().unwrap();
        assert_eq!(
            Skeleton::<V274>::read(&written, ModelFlags::default()).unwrap(),
            skeleton
        );

        // An empty last SKS1 still supplies local context and overrides inheritance.
        bytes.extend(chunk(*b"SKS1", &[0; 32]));
        assert!(
            Skeleton::<V274>::read_with_sequences(&bytes, ModelFlags::default(), &[true]).is_err()
        );
    }
}

mod resources {
    use wc3::model::common::IntoZeroVec;
    use wc3::model::formats::m2::chunks::Chunk;
    use wc3::model::formats::m2::records::Texture;
    use wc3::model::formats::m2::records::{TextureFlags, TextureSource};
    use wc3::model::formats::m2::resources::*;
    use wc3::model::formats::m2::Document;
    use wc3::model::formats::m2::{Chunked, Flat, V264, V274};

    #[test]
    fn sibling_names_ignore_dots_in_directories_and_preserve_native_bytes() {
        assert_eq!(
            FileKey::Path("models.v1\\Wolf.M2".into()).sibling("00.skin"),
            FileKey::Path("models.v1\\Wolf00.skin".into())
        );
        assert_eq!(
            FileKey::Path("models.v1/Wolf".into()).sibling(".phys"),
            FileKey::Path("models.v1/Wolf.phys".into())
        );
        assert_eq!(
            FileKey::RawPath(b"models/\xff.m2".to_vec()).sibling("00.skin"),
            FileKey::RawPath(b"models/\xff00.skin".to_vec())
        );
        assert_eq!(
            FileKey::FileId(12).sibling(".phys"),
            FileKey::Sibling {
                owner: Box::new(FileKey::FileId(12)),
                suffix: ".phys".into()
            }
        );
    }
    #[test]
    fn declared_skin_and_texture_references_preserve_names() {
        let mut model = Document::<Flat<V264>>::new();
        model.core_mut().skins = 2;
        model.core_mut().textures = vec![
            Texture {
                kind: TextureSource::File,
                flags: TextureFlags::default(),
                filename: b"Textures\\fur.blp\0".as_slice().into_zero_vec(),
            },
            Texture {
                kind: TextureSource::File,
                flags: TextureFlags::default(),
                filename: [0xff, 0].into_zero_vec(),
            },
            Texture {
                kind: TextureSource::Skin,
                flags: TextureFlags::default(),
                filename: b"replaceable.blp\0".as_slice().into_zero_vec(),
            },
        ];
        let owner = FileKey::Path("Creature/Wolf.m2".into());
        let deps = model.dependencies(&owner);
        assert_eq!(deps.len(), 4);
        assert_eq!(deps[0].key, FileKey::Path("Creature/Wolf00.skin".into()));
        assert_eq!(deps[1].key, FileKey::Path("Creature/Wolf01.skin".into()));
        assert_eq!(deps[2].key, FileKey::Path("Textures\\fur.blp".into()));
        assert_eq!(deps[3].key, FileKey::RawPath(vec![0xff]));
    }
    #[test]
    fn modern_skin_ids_split_base_views_from_lods() {
        let mut model = Document::<Chunked<V274>>::new();
        model.core_mut().skins = 1;
        model
            .insert_extension(1, Chunk::SkinIds([7, 8, 0, 9].into_zero_vec()))
            .unwrap();
        let deps = model.dependencies(&FileKey::FileId(1));
        assert_eq!(
            deps.iter().map(|d| d.key.clone()).collect::<Vec<_>>(),
            vec![FileKey::FileId(7), FileKey::FileId(8), FileKey::FileId(9)]
        );
        assert_eq!(
            deps[0].kind,
            DependencyKind::Skin {
                index: 0,
                lod: false
            }
        );
        assert_eq!(
            deps[1].kind,
            DependencyKind::Skin {
                index: 0,
                lod: true
            }
        );
        assert_eq!(
            deps[2].kind,
            DependencyKind::Skin {
                index: 2,
                lod: true
            }
        );
    }
}
