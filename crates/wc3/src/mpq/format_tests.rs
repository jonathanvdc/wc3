use super::{BlockEntry, FileFlags, HashEntry, Header};

#[test]
fn classic_header_matches_fixed_wire_layout() {
    let bytes = [
        b'M', b'P', b'Q', 0x1a, 32, 0, 0, 0, 0, 1, 0, 0, 0, 0, 3, 0, 0x60, 0, 0, 0, 0xe0, 0, 0, 0,
        8, 0, 0, 0, 2, 0, 0, 0,
    ];
    let header = Header {
        archive_size: 256,
        sector_size_shift: 3,
        hash_table_offset: 96,
        block_table_offset: 224,
        hash_table_entries: 8,
        block_table_entries: 2,
    };
    assert_eq!(Header::decode(&bytes).unwrap(), header);
    assert_eq!(header.sector_size(), Some(4096));
    let mut encoded = Vec::new();
    header.write(&mut encoded).unwrap();
    assert_eq!(encoded, bytes);
}

#[test]
fn table_records_match_fixed_wire_layout() {
    let hash = HashEntry {
        name_hash_a: 0x04030201,
        name_hash_b: 0x08070605,
        locale: 0x0409,
        platform: 0,
        block_index: 42,
    };
    assert_eq!(
        hash.bytes(),
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 4, 0, 0, 42, 0, 0, 0]
    );
    let block = BlockEntry {
        offset: 32,
        stored_size: 100,
        file_size: 256,
        flags: FileFlags(FileFlags::EXISTS | FileFlags::ENCRYPTED | FileFlags::COMPRESS),
    };
    assert_eq!(
        block.bytes(),
        [32, 0, 0, 0, 100, 0, 0, 0, 0, 1, 0, 0, 0, 2, 1, 128]
    );
}
