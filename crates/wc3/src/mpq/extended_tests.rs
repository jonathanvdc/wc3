use std::io::Cursor;

use super::{encode_extended, read_extended, ExtendedIndex};
use crate::mpq::crypto::{crypt, hash, jenkins};
use crate::mpq::{Archive, BlockEntry, FileFlags, Header, ReadOptions};

#[test]
fn deleted_slots_keep_probe_chains_and_round_trip() {
    let names: Vec<_> = (0..100).map(|i| format!("name-{i}").into_bytes()).collect();
    let first = &names[0];
    let value = jenkins(first) | 1 << 63;
    let second = names
        .iter()
        .skip(1)
        .find(|n| (jenkins(n) | 1 << 63) % 4 == value % 4)
        .unwrap();
    let slot = (value % 4) as usize;
    let mut slots = vec![None; 4];
    slots[slot] = Some((0, u32::MAX));
    slots[(slot + 1) % 4] = Some((jenkins(second) | 1 << 63, 1));
    let index = ExtendedIndex {
        hash_bits: 64,
        slots,
    };
    assert_eq!(index.find(second), Some(1));
    assert_eq!(index.find(first), None);
    let blocks = vec![
        BlockEntry {
            offset: 68,
            stored_size: 0,
            file_size: 0,
            flags: FileFlags(FileFlags::EXISTS)
        };
        2
    ];
    let [het, bet] = encode_extended(&index, &blocks).unwrap();
    let mut bytes = vec![0; 68];
    bytes.extend_from_slice(&het);
    bytes.extend_from_slice(&bet);
    let header = Header {
        version: 2,
        het_table_offset: 68,
        bet_table_offset: 68 + het.len() as u64,
        table_sizes: [0, 0, 0, het.len() as u64, bet.len() as u64],
        ..Header::default()
    };
    let (decoded, decoded_blocks) =
        read_extended(&mut Cursor::new(bytes), 0, &header, &ReadOptions::default()).unwrap();
    assert_eq!(decoded, index);
    assert_eq!(decoded_blocks, blocks);
}

#[test]
fn rejects_invalid_extended_dimensions_without_panicking() {
    let fixture = include_bytes!("../../tests/fixtures/mpq/storm-v3.mpq");
    let archive = Archive::open(Cursor::new(fixture)).unwrap();
    let header = &archive.index().header;
    for (position, size, key, fields) in [
        (
            header.het_table_offset,
            header.table_sizes[3],
            b"(hash table)".as_slice(),
            8,
        ),
        (
            header.bet_table_offset,
            header.table_sizes[4],
            b"(block table)".as_slice(),
            19,
        ),
    ] {
        for field in 0..fields {
            for value in [0, u32::MAX] {
                let mut bytes = fixture.to_vec();
                let table = &mut bytes[position as usize + 12..(position + size) as usize];
                crypt(table, hash(key, 3), true);
                table[field * 4..field * 4 + 4].copy_from_slice(&value.to_le_bytes());
                crypt(table, hash(key, 3), false);
                // Some zero values are legal; every mutation must return normally.
                let _ = Archive::open(Cursor::new(bytes));
            }
        }
    }
}
