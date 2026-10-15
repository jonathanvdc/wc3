//! Semantic failures from independently assembled root payload bytes.
use wc3::model::formats::wmo::root::RootChunk;
use wc3::model::formats::wmo::{Classic, Root, Validate};

fn insert(root: &mut Root<'static, Classic>, tag: [u8; 4], bytes: Vec<u8>) {
    // Decoded records are detached through the public ownership contract.
    let chunk = RootChunk::decode(tag, &bytes, 0).unwrap().into_owned();
    let index = root.chunks().count();
    root.insert_chunk(index, chunk).unwrap();
}

#[test]
fn portal_vertex_spans_and_neighbor_groups_are_bounded() {
    let mut root = Root::new();
    root.header_mut().portal_count = 1;
    let mut portal = vec![0; 20];
    portal[..2].copy_from_slice(&2u16.to_le_bytes());
    portal[2..4].copy_from_slice(&2u16.to_le_bytes());
    insert(&mut root, *b"MOPT", portal);
    insert(&mut root, *b"MOPV", vec![0; 36]);
    assert_eq!(root.validate().unwrap_err().field, "portal vertex span");
    root.remove_chunks(*b"MOPT");
    let mut portal = vec![0; 20];
    portal[2..4].copy_from_slice(&3u16.to_le_bytes());
    insert(&mut root, *b"MOPT", portal);
    insert(&mut root, *b"MOPR", vec![0; 8]);
    assert_eq!(root.validate().unwrap_err().field, "portal group index");
}

#[test]
fn group_name_offsets_require_a_bounded_terminator() {
    let mut root = Root::new();
    root.header_mut().group_count = 1;
    insert(&mut root, *b"MOGI", vec![0; 32]);
    insert(&mut root, *b"MOGN", b"unterminated".to_vec());
    assert_eq!(root.validate().unwrap_err().field, "group name offset");
    root.remove_chunks(*b"MOGN");
    insert(&mut root, *b"MOGN", b"room\0".to_vec());
    root.validate().unwrap();
}

#[test]
fn doodad_set_ranges_and_model_indices_are_checked() {
    let mut root = Root::new();
    root.header_mut().doodad_set_count = 1;
    let mut set = vec![0; 32];
    set[24..28].copy_from_slice(&2u32.to_le_bytes());
    insert(&mut root, *b"MODS", set);
    let mut placement = vec![0; 40];
    placement[32..36].copy_from_slice(&1f32.to_le_bytes());
    insert(&mut root, *b"MODD", placement);
    insert(&mut root, *b"MODI", 7u32.to_le_bytes().to_vec());
    assert_eq!(root.validate().unwrap_err().field, "doodad set span");
    root.remove_chunks(*b"MODS");
    root.header_mut().doodad_set_count = 0;
    root.remove_chunks(*b"MODD");
    let mut placement = vec![0; 40];
    placement[..4].copy_from_slice(&1u32.to_le_bytes());
    placement[32..36].copy_from_slice(&1f32.to_le_bytes());
    insert(&mut root, *b"MODD", placement);
    assert_eq!(root.validate().unwrap_err().field, "doodad model index");
}

#[test]
fn historical_material_tail_words_are_not_treated_as_texture_offsets() {
    let mut root = Root::new();
    root.header_mut().texture_count = 99;
    root.header_mut().light_count = 77;
    root.header_mut().doodad_name_count = 55;
    root.header_mut().doodad_def_count = 33;
    let mut material = vec![0; 64];
    material[40..44].copy_from_slice(&0xffff_ffffu32.to_le_bytes());
    insert(&mut root, *b"MOMT", material);
    insert(&mut root, *b"MOTX", b"texture.blp\0".to_vec());
    root.validate().unwrap();
    root.remove_chunks(*b"MOTX");
    insert(&mut root, *b"MOTX", b"texture.blp".to_vec());
    assert_eq!(
        root.validate().unwrap_err().field,
        "material texture offset"
    );
}

#[test]
fn repeated_known_tables_reject_ambiguity_but_unknown_chunks_remain_repeatable() {
    let mut root = Root::new();
    insert(&mut root, *b"MOGI", vec![]);
    insert(&mut root, *b"MOGI", vec![]);
    assert_eq!(root.validate().unwrap_err().field, "duplicate root table");
    root.remove_chunks(*b"MOGI");
    insert(&mut root, *b"ZZZZ", vec![1]);
    insert(&mut root, *b"ZZZZ", vec![2]);
    root.validate().unwrap();
}
