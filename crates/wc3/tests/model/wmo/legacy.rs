use wc3::model::wmo::{
    self,
    codec::Array,
    legacy::{
        File14, File16, GroupInfo14, Header14, Light14, Material14, Root14, Root16, RootChunk14,
    },
    root::RootChunk,
};
fn frame(output: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
    output.extend(tag.iter().rev());
    output.extend((data.len() as u32).to_le_bytes());
    output.extend(data);
}
fn alpha() -> Vec<u8> {
    let mut data = Vec::new();
    let mut header = vec![0; 64];
    header[0..4].copy_from_slice(&2u32.to_le_bytes());
    header[4..8].copy_from_slice(&1u32.to_le_bytes());
    header[12..16].copy_from_slice(&1u32.to_le_bytes());
    header[36..].fill(0xab);
    frame(&mut data, b"MOHD", &header);
    frame(&mut data, b"MOTX", b"a.blp\0b.blp\0");
    let mut material = vec![0; 44];
    material[24..28].copy_from_slice(&6u32.to_le_bytes());
    frame(&mut data, b"MOMT", &material);
    let group_info_position = data.len() + 8;
    let mut info = vec![0; 40];
    info[4..8].copy_from_slice(&136u32.to_le_bytes());
    info[36..40].copy_from_slice(&u32::MAX.to_le_bytes());
    frame(&mut data, b"MOGI", &info);
    frame(&mut data, b"MOLT", &[0; 32]);
    frame(&mut data, b"XTRA", &[1, 2, 3, 4]);
    let group_offset = 12 + 8 + data.len();
    data[group_info_position..group_info_position + 4]
        .copy_from_slice(&(group_offset as u32).to_le_bytes());
    let mut bytes = Vec::new();
    frame(&mut bytes, b"MVER", &14u32.to_le_bytes());
    frame(&mut bytes, b"MOMO", &data);
    frame(&mut bytes, b"MOGP", &[0; 128]);
    bytes
}
#[test]
fn alpha_records_use_verified_distinct_widths() {
    use wmo::RecordLayout as _;
    assert_eq!(Header14::size(), 64);
    assert_eq!(Material14::size(), 44);
    assert_eq!(GroupInfo14::size(), 40);
    assert_eq!(Light14::size(), 32);
    let bytes = alpha();
    let root = Root14::decode(&bytes).unwrap();
    assert_eq!(root.header().reserved, [0xab; 28]);
    assert_eq!(root.group_count(), 1);
    root.validate().unwrap();
    assert_eq!(root.encode().unwrap(), bytes);
    let material = root
        .chunks()
        .find_map(|c| match c {
            RootChunk14::Materials(a) => Some(a),
            _ => None,
        })
        .unwrap();
    assert_eq!(material.len(), 1);
    assert_eq!(material.get(0).unwrap().unwrap().texture1, 6);
    assert_eq!(material.as_bytes().as_ptr(), bytes[120..].as_ptr());
    assert_eq!(root.dependencies(&wmo::FileKey::Root).unwrap().len(), 2);
    assert!(matches!(File14::decode(&bytes).unwrap(), File14::Root(_)));
}
#[test]
fn alpha_resize_relocates_absolute_group_references() {
    let bytes = alpha();
    let mut root = Root14::decode(&bytes).unwrap();
    let old = root
        .chunks()
        .find_map(|c| match c {
            RootChunk14::Groups(a) => Some(a.get(0).unwrap().unwrap().group_offset),
            _ => None,
        })
        .unwrap();
    root.push_chunk(RootChunk14::Common(RootChunk::Unknown {
        tag: *b"NEW!",
        data: vec![9; 17].into(),
    }))
    .unwrap();
    let output = root.encode().unwrap();
    let decoded = Root14::decode(&output).unwrap();
    let info = decoded
        .chunks()
        .find_map(|c| match c {
            RootChunk14::Groups(a) => Some(a.get(0).unwrap().unwrap()),
            _ => None,
        })
        .unwrap();
    assert_eq!(info.group_offset, old + 25);
    assert_eq!(info.group_size, 136);
    decoded.validate().unwrap();
    let mut malformed = bytes.clone();
    malformed[172..176].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(Root14::decode(&malformed).is_err());
    assert!(Root16::decode(&bytes).is_err());
}
#[test]
fn early_beta_roots_keep_version16_and_bound_external_group_counts() {
    let mut bytes = Vec::new();
    frame(&mut bytes, b"MVER", &16u32.to_le_bytes());
    frame(&mut bytes, b"MOHD", &[0; 64]);
    frame(&mut bytes, b"XTRA", &[9, 8, 7]);
    let root = Root16::decode(&bytes).unwrap();
    assert_eq!(root.encode().unwrap(), bytes);
    root.validate().unwrap();
    assert!(matches!(File16::decode(&bytes).unwrap(), File16::Root(_)));
    let mut missing = bytes.clone();
    missing[24..28].copy_from_slice(&u32::MAX.to_le_bytes());
    let root = Root16::decode(&missing).unwrap();
    assert!(root.dependencies(&wmo::FileKey::Root).is_err());
    let mut root = Root16::new();
    root.push_chunk(RootChunk::Unknown {
        tag: *b"MOHD",
        data: vec![0; 64].into(),
    })
    .unwrap_err();
    root.push_chunk(RootChunk::Groups(Array::decode(&[]).unwrap()))
        .unwrap();
}
#[test]
fn alpha_shared_variants_cannot_encode_foreign_widths() {
    let chunk = RootChunk14::Common(RootChunk::Materials(Array::decode(&[]).unwrap()));
    assert!(chunk.encode().is_err());
}

#[test]
fn malformed_alpha_fixed_widths_report_the_source_origin() {
    let error = RootChunk14::decode(*b"MOMT", &[0; 43], 91).unwrap_err();
    assert_eq!(error.offset, 91);
    let bytes = alpha();
    assert!(Root14::decode(&bytes[..bytes.len() - 1]).is_err());
    assert!(matches!(
        wmo::DynamicFile::decode(&bytes, wmo::Profile::Alpha14).unwrap(),
        wmo::DynamicFile::Alpha14(_)
    ));
}

#[test]
fn beta_root_writing_updates_counts_after_resizing_group_info() {
    let mut root = Root16::new();
    let mut info = [0u8; 32];
    info[28..32].copy_from_slice(&u32::MAX.to_le_bytes());
    root.push_chunk(RootChunk::Groups(Array::decode(&info).unwrap()))
        .unwrap();
    let bytes = root.encode().unwrap();
    let decoded = Root16::decode(&bytes).unwrap();
    assert_eq!(decoded.header().group_count, 1);
    decoded.validate().unwrap();
}
