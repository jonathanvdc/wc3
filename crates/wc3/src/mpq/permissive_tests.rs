use std::io::Cursor;

use super::crypto::{crypt, file_key, hash};
use super::format::DELETED;
use super::{Archive, ArchiveWriter, Error, FileFlags, ReadMode, ReadOptions};

fn options() -> ReadOptions {
    ReadOptions {
        mode: ReadMode::Permissive,
        ..ReadOptions::default()
    }
}

fn fixture(flags: u32, payload: &[u8], decoded: u32) -> Vec<u8> {
    let mut bytes = vec![0; 32];
    bytes[..4].copy_from_slice(b"MPQ\x1a");
    bytes[4..8].copy_from_slice(&32u32.to_le_bytes());
    bytes[14..16].copy_from_slice(&3u16.to_le_bytes());
    bytes.extend_from_slice(payload);
    let hash_at = bytes.len() as u32;
    let mut hashes = vec![255; 64];
    let slot = (hash(b"data", 0) & 3) as usize * 16;
    hashes[slot..slot + 4].copy_from_slice(&hash(b"data", 1).to_le_bytes());
    hashes[slot + 4..slot + 8].copy_from_slice(&hash(b"data", 2).to_le_bytes());
    hashes[slot + 8..slot + 16].fill(0);
    crypt(&mut hashes, hash(b"(hash table)", 3), false);
    bytes.extend_from_slice(&hashes);
    let block_at = bytes.len() as u32;
    let mut block = Vec::new();
    for word in [32, payload.len() as u32, decoded, FileFlags::EXISTS | flags] {
        block.extend_from_slice(&word.to_le_bytes());
    }
    crypt(&mut block, hash(b"(block table)", 3), false);
    bytes.extend_from_slice(&block);
    let len = bytes.len() as u32;
    for (at, word) in [(8, len), (16, hash_at), (20, block_at), (24, 4), (28, 1)] {
        bytes[at..at + 4].copy_from_slice(&word.to_le_bytes());
    }
    bytes
}

fn mutate_table(bytes: &mut [u8], header_at: usize, edit: impl FnOnce(&mut [u8])) {
    let at = u32::from_le_bytes(bytes[header_at..header_at + 4].try_into().unwrap()) as usize;
    let size = if header_at == 16 { 64 } else { 16 };
    let name: &[u8] = if header_at == 16 {
        b"(hash table)"
    } else {
        b"(block table)"
    };
    let table = &mut bytes[at..at + size];
    crypt(table, hash(name, 3), true);
    edit(table);
    crypt(table, hash(name, 3), false);
}

#[test]
fn clean_archives_remain_editable() {
    let bytes = fixture(0, b"payload", 7);
    let mut archive = Archive::with_options(Cursor::new(bytes), options()).unwrap();
    assert_eq!(archive.read_file("data").unwrap(), b"payload");
    assert!(archive.diagnostics().is_empty());
    ArchiveWriter::from_archive(Cursor::new(Vec::new()), &mut archive).unwrap();
}

#[test]
fn malformed_header_fields_and_table_count_bits() {
    for (at, replacement) in [
        (4, 999u32.to_le_bytes().to_vec()),
        (12, 99u16.to_le_bytes().to_vec()),
        (8, 1u32.to_le_bytes().to_vec()),
        (14, 0xab03u16.to_le_bytes().to_vec()),
        (24, 0xf0000004u32.to_le_bytes().to_vec()),
        (28, 999u32.to_le_bytes().to_vec()),
    ] {
        let mut bytes = fixture(0, b"payload", 7);
        bytes[at..at + replacement.len()].copy_from_slice(&replacement);
        assert!(Archive::open(Cursor::new(&bytes)).is_err());
        let mut archive = Archive::with_options(Cursor::new(bytes), options()).unwrap();
        assert_eq!(archive.read_file("data").unwrap(), b"payload");
        assert!(!archive.diagnostics().is_empty());
        assert!(ArchiveWriter::from_archive(Cursor::new(Vec::new()), &mut archive).is_err());
    }
}

#[test]
fn reserved_byte_and_block_index_bits() {
    let mut bytes = fixture(0, b"payload", 7);
    mutate_table(&mut bytes, 16, |table| {
        let slot = (hash(b"data", 0) & 3) as usize * 16;
        table[slot + 11] = 0xaa;
        table[slot + 12..slot + 16].copy_from_slice(&0xf0000000u32.to_le_bytes());
    });
    assert!(Archive::open(Cursor::new(&bytes)).is_err());
    let mut archive = Archive::with_options(Cursor::new(bytes), options()).unwrap();
    assert_eq!(archive.read_file("data").unwrap(), b"payload");
}

#[test]
fn invalid_unrelated_hash_and_payload_ranges() {
    let mut bytes = fixture(0, b"payload", 7);
    mutate_table(&mut bytes, 16, |table| {
        let slot = ((hash(b"data", 0) + 1) & 3) as usize * 16;
        table[slot + 12..slot + 16].copy_from_slice(&999u32.to_le_bytes());
    });
    let mut archive = Archive::with_options(Cursor::new(bytes), options()).unwrap();
    assert_eq!(archive.read_file("data").unwrap(), b"payload");
    assert!(archive
        .index()
        .hashes
        .iter()
        .any(|h| h.block_index == DELETED));
    let mut bytes = fixture(0, b"payload", 7);
    mutate_table(&mut bytes, 20, |table| {
        table[..4].copy_from_slice(&999u32.to_le_bytes())
    });
    let mut archive = Archive::with_options(Cursor::new(bytes), options()).unwrap();
    assert!(archive.open_file("data").is_err());
    assert!(archive.encoded_file(0).is_err());
}

#[test]
fn fake_headers_and_bogus_userdata_do_not_hide_real_header() {
    for magic in [b"MPQ\x1a", b"MPQ\x1b"] {
        let mut bytes = vec![0; 512];
        bytes[..4].copy_from_slice(magic);
        bytes.extend_from_slice(&fixture(0, b"payload", 7));
        let mut archive = Archive::with_options(Cursor::new(bytes), options()).unwrap();
        assert_eq!(archive.archive_offset(), 512);
        assert_eq!(archive.read_file("data").unwrap(), b"payload");
        assert!(!archive.diagnostics().is_empty());
    }
}

#[test]
fn valid_unaligned_userdata_wrapper() {
    let mut bytes = vec![0; 37];
    bytes[..4].copy_from_slice(b"MPQ\x1b");
    bytes[8..12].copy_from_slice(&37u32.to_le_bytes());
    bytes.extend_from_slice(&fixture(0, b"payload", 7));
    let mut archive = Archive::with_options(Cursor::new(bytes), options()).unwrap();
    assert_eq!(archive.read_file("data").unwrap(), b"payload");
}

#[test]
fn sectors_before_offset_table_and_tables_before_header() {
    let mut bytes = fixture(FileFlags::COMPRESS, &[0; 16], 7);
    bytes[32..39].copy_from_slice(b"payload");
    bytes[40..44].copy_from_slice(&0xfffffff8u32.to_le_bytes());
    bytes[44..48].copy_from_slice(&0xffffffffu32.to_le_bytes());
    mutate_table(&mut bytes, 20, |table| {
        table[..4].copy_from_slice(&40u32.to_le_bytes())
    });
    let mut archive = Archive::with_options(Cursor::new(bytes), options()).unwrap();
    assert_eq!(archive.read_file("data").unwrap(), b"payload");

    let original = fixture(0, b"payload", 7);
    let hash_at = u32::from_le_bytes(original[16..20].try_into().unwrap()) as usize;
    let mut bytes = vec![0; 544 + 7];
    bytes[..80].copy_from_slice(&original[hash_at..]);
    bytes[512..544].copy_from_slice(&original[..32]);
    bytes[544..].copy_from_slice(b"payload");
    bytes[512 + 8..512 + 12].copy_from_slice(&39u32.to_le_bytes());
    bytes[528..532].copy_from_slice(&0xfffffe00u32.to_le_bytes());
    bytes[532..536].copy_from_slice(&0xfffffe40u32.to_le_bytes());
    let mut archive = Archive::with_options(Cursor::new(bytes), options()).unwrap();
    assert_eq!(archive.read_file("data").unwrap(), b"payload");
}

#[test]
fn extra_offset_bytes_and_invalid_checksums() {
    for checksums in [false, true] {
        let mut payload = Vec::new();
        let first = if checksums { 12u32 } else { 16 };
        payload.extend_from_slice(&first.to_le_bytes());
        payload.extend_from_slice(&(first + 7).to_le_bytes());
        payload.resize(first as usize, 0);
        payload.extend_from_slice(b"payload");
        let flags = FileFlags::COMPRESS | if checksums { FileFlags::SECTOR_CRC } else { 0 };
        let mut archive =
            Archive::with_options(Cursor::new(fixture(flags, &payload, 7)), options()).unwrap();
        assert_eq!(archive.read_file("data").unwrap(), b"payload");
        assert!(!archive.diagnostics().is_empty());
    }
}

#[test]
fn encryption_recovery_from_sector_table_and_known_content() {
    let key = 0x12345678u32;
    let mut payload = Vec::new();
    payload.extend_from_slice(&8u32.to_le_bytes());
    payload.extend_from_slice(&15u32.to_le_bytes());
    crypt(&mut payload, key.wrapping_sub(1), false);
    let mut data = b"payload".to_vec();
    crypt(&mut data, key, false);
    payload.extend_from_slice(&data);
    let bytes = fixture(FileFlags::COMPRESS | FileFlags::ENCRYPTED, &payload, 7);
    let shared = Archive::with_options(Cursor::new(bytes.clone()), options())
        .unwrap()
        .into_shared();
    let initial = shared.diagnostics().len();
    let mut first = shared.open_file_by_index(0).unwrap();
    let mut second = shared.open_file("data").unwrap();
    assert!(!first.diagnostics().is_empty());
    assert!(!second.diagnostics().is_empty());
    assert_eq!(shared.diagnostics().len(), initial);
    let mut first_bytes = Vec::new();
    let mut second_bytes = Vec::new();
    use std::io::Read as _;
    first.read_to_end(&mut first_bytes).unwrap();
    second.read_to_end(&mut second_bytes).unwrap();
    assert_eq!(first_bytes, b"payload");
    assert_eq!(second_bytes, first_bytes);
    for named in [false, true] {
        let mut archive = Archive::with_options(Cursor::new(&bytes), options()).unwrap();
        let mut reader = if named {
            archive.open_file("data").unwrap()
        } else {
            archive.open_file_by_index(0).unwrap()
        };
        let mut result = Vec::new();
        use std::io::Read as _;
        reader.read_to_end(&mut result).unwrap();
        assert_eq!(result, b"payload");
    }
    for plain in [
        b"RIFF\x04\0\0\0WAVE".as_slice(),
        b"<?xml version=\"1\"?>",
        b"MZ\x90\0\x03\0\0\0contents",
    ] {
        let mut encrypted = plain.to_vec();
        crypt(&mut encrypted, key, false);
        let mut archive = Archive::with_options(
            Cursor::new(fixture(
                FileFlags::ENCRYPTED,
                &encrypted,
                plain.len() as u32,
            )),
            options(),
        )
        .unwrap();
        assert_eq!(archive.read_file("data").unwrap(), plain);
    }
    assert_ne!(file_key(b"data", 32, 7, false), key);
}

#[test]
fn permissive_limits_and_checksums_remain_enforced() {
    let mut bytes = fixture(0, b"payload", 7);
    bytes[12..14].copy_from_slice(&99u16.to_le_bytes());
    let limits = ReadOptions {
        max_table_entries: 2,
        ..options()
    };
    assert!(matches!(
        Archive::with_options(Cursor::new(bytes), limits),
        Err(Error::LimitExceeded(_))
    ));
    let mut payload = Vec::new();
    for word in [12u32, 19, 23] {
        payload.extend_from_slice(&word.to_le_bytes());
    }
    payload.extend_from_slice(b"payload");
    payload.extend_from_slice(&123u32.to_le_bytes());
    let mut archive = Archive::with_options(
        Cursor::new(fixture(
            FileFlags::COMPRESS | FileFlags::SECTOR_CRC,
            &payload,
            7,
        )),
        options(),
    )
    .unwrap();
    assert!(matches!(
        archive.read_file("data"),
        Err(Error::ChecksumMismatch(0))
    ));
}

#[test]
fn truncated_hash_table_keeps_logical_slots_and_probe_chains() {
    let original = fixture(0, b"payload", 7);
    let hash_at = u32::from_le_bytes(original[16..20].try_into().unwrap()) as usize;
    let block_at = hash_at + 64;
    let mut table = original[hash_at..block_at].to_vec();
    crypt(&mut table, hash(b"(hash table)", 3), true);
    // Keep a valid entry at the first slot; unavailable tail slots are tombstones.
    let slot = (hash(b"data", 0) & 3) as usize;
    let entry = table[slot * 16..slot * 16 + 16].to_vec();
    table[..16].copy_from_slice(&entry);
    crypt(&mut table, hash(b"(hash table)", 3), false);
    let mut bytes = original[..hash_at].to_vec();
    bytes.extend_from_slice(&original[block_at..]);
    let new_hash_at = bytes.len() as u32;
    bytes.extend_from_slice(&table[..16]);
    bytes[16..20].copy_from_slice(&new_hash_at.to_le_bytes());
    bytes[20..24].copy_from_slice(&(hash_at as u32).to_le_bytes());
    let len = bytes.len() as u32;
    bytes[8..12].copy_from_slice(&len.to_le_bytes());
    let mut archive = Archive::with_options(Cursor::new(bytes), options()).unwrap();
    assert_eq!(archive.index().hashes.len(), 4);
    assert_eq!(archive.read_file("data").unwrap(), b"payload");
}

#[test]
fn sentinel_hash_table_offset_and_fake_extended_header() {
    let original = fixture(0, b"payload", 7);
    let hash_at = u32::from_le_bytes(original[16..20].try_into().unwrap()) as usize;
    let mut bytes = original[..32].to_vec();
    bytes.extend_from_slice(&original[hash_at..]);
    bytes.extend_from_slice(b"payload");
    bytes[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
    bytes[20..24].copy_from_slice(&96u32.to_le_bytes());
    mutate_table(&mut bytes, 20, |table| {
        table[..4].copy_from_slice(&112u32.to_le_bytes())
    });
    let mut archive = Archive::with_options(Cursor::new(bytes), options()).unwrap();
    assert_eq!(archive.read_file("data").unwrap(), b"payload");

    let mut bytes = original;
    bytes[4..8].copy_from_slice(&44u32.to_le_bytes());
    bytes[12..14].copy_from_slice(&1u16.to_le_bytes());
    let mut archive = Archive::with_options(Cursor::new(bytes), options()).unwrap();
    assert_eq!(archive.read_file("data").unwrap(), b"payload");
}

#[test]
fn payload_before_header_is_readable_but_cannot_be_encoded_copied() {
    let original = fixture(0, b"payload", 7);
    let hash_at = u32::from_le_bytes(original[16..20].try_into().unwrap()) as usize;
    let mut bytes = vec![0; 544];
    bytes[..7].copy_from_slice(b"payload");
    bytes[64..144].copy_from_slice(&original[hash_at..]);
    let mut block = bytes[128..144].to_vec();
    crypt(&mut block, hash(b"(block table)", 3), true);
    block[..4].copy_from_slice(&0xfffffe00u32.to_le_bytes());
    crypt(&mut block, hash(b"(block table)", 3), false);
    bytes[128..144].copy_from_slice(&block);
    bytes[512..544].copy_from_slice(&original[..32]);
    bytes[520..524].copy_from_slice(&32u32.to_le_bytes());
    bytes[528..532].copy_from_slice(&0xfffffe40u32.to_le_bytes());
    bytes[532..536].copy_from_slice(&0xfffffe80u32.to_le_bytes());
    let mut archive = Archive::with_options(Cursor::new(bytes), options()).unwrap();
    assert_eq!(archive.read_file("data").unwrap(), b"payload");
    assert!(ArchiveWriter::from_archive(Cursor::new(Vec::new()), &mut archive).is_err());
}
