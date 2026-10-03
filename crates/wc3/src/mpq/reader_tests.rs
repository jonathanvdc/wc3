use std::io::{Cursor, Read};

use super::super::crypto::{crypt, hash};
#[cfg(all(feature = "mpq-decode", feature = "mpq-encode"))]
use super::super::Compression;
use super::super::{ArchiveWriter, FileOptions, WriteOptions};
use super::ReadOptions;
use super::{Archive, Error, FileFlags};

fn archive() -> Vec<u8> {
    let mut writer = ArchiveWriter::new(
        Cursor::new(Vec::new()),
        WriteOptions {
            listfile: false,
            ..WriteOptions::default()
        },
    )
    .unwrap();
    writer
        .add_file(
            "data",
            7,
            &mut b"payload".as_slice(),
            FileOptions::default(),
        )
        .unwrap();
    writer.finish().unwrap().into_inner()
}

fn change_table(bytes: &mut [u8], hash_table: bool, mutate: impl FnOnce(&mut [u8])) {
    let index = Archive::open(Cursor::new(&*bytes)).unwrap().index().clone();
    let (offset, count, name): (_, _, &[u8]) = if hash_table {
        (
            index.header.hash_table_offset,
            index.header.hash_table_entries,
            b"(hash table)",
        )
    } else {
        (
            index.header.block_table_offset,
            index.header.block_table_entries,
            b"(block table)",
        )
    };
    let table = &mut bytes[offset as usize..offset as usize + count as usize * 16];
    crypt(table, hash(name, 3), true);
    mutate(table);
    crypt(table, hash(name, 3), false);
}

#[test]
fn rejects_dangling_hashes_and_block_extents() {
    let mut bytes = archive();
    change_table(&mut bytes, true, |table| {
        for entry in table.as_chunks_mut::<16>().0.iter_mut() {
            if u32::from_le_bytes(entry[12..16].try_into().unwrap()) == 0 {
                entry[12..16].copy_from_slice(&999u32.to_le_bytes());
            }
        }
    });
    assert!(matches!(
        Archive::open(Cursor::new(bytes)),
        Err(Error::InvalidArchive(_))
    ));
    let mut bytes = archive();
    change_table(&mut bytes, false, |table| {
        table[4..8].copy_from_slice(&u32::MAX.to_le_bytes())
    });
    assert!(matches!(
        Archive::open(Cursor::new(bytes)),
        Err(Error::InvalidArchive(_))
    ));
}

#[test]
fn preserves_unsupported_flags_without_claiming_to_decode() {
    let mut bytes = archive();
    change_table(&mut bytes, false, |table| {
        table[12..16].copy_from_slice(&(FileFlags::EXISTS | 0x100000).to_le_bytes())
    });
    let mut archive = Archive::open(Cursor::new(bytes)).unwrap();
    assert!(matches!(
        archive.open_file("data"),
        Err(Error::UnsupportedFlags(_))
    ));
    let mut raw = Vec::new();
    archive
        .encoded_file(0)
        .unwrap()
        .read_to_end(&mut raw)
        .unwrap();
    assert_eq!(raw, b"payload");
    let writer = ArchiveWriter::from_archive(Cursor::new(Vec::new()), &mut archive).unwrap();
    let mut copied = Archive::open(Cursor::new(writer.finish().unwrap().into_inner())).unwrap();
    assert!(matches!(
        copied.open_file("data"),
        Err(Error::UnsupportedFlags(_))
    ));
    assert_eq!(copied.index().blocks[0], archive.index().blocks[0]);
}

#[test]
fn tolerates_empty_block_at_zero() {
    let mut bytes = archive();
    change_table(&mut bytes, false, |table| table[..12].fill(0));
    assert!(Archive::open(Cursor::new(bytes))
        .unwrap()
        .read_file("data")
        .unwrap()
        .is_empty());
}

/// Construct encoded entry storage independently of the MPQ compression writer.
fn encoded_unit(encoded: &[u8], decoded_size: u32, flags: u32) -> Vec<u8> {
    let mut writer = ArchiveWriter::new(
        Cursor::new(Vec::new()),
        WriteOptions {
            listfile: false,
            ..WriteOptions::default()
        },
    )
    .unwrap();
    writer
        .add_file(
            "data",
            encoded.len() as u32,
            &mut &*encoded,
            FileOptions::default(),
        )
        .unwrap();
    let mut bytes = writer.finish().unwrap().into_inner();
    change_table(&mut bytes, false, |table| {
        table[8..12].copy_from_slice(&decoded_size.to_le_bytes());
        table[12..16]
            .copy_from_slice(&(FileFlags::EXISTS | FileFlags::SINGLE_UNIT | flags).to_le_bytes());
    });
    bytes
}

#[test]
fn reads_and_limits_single_unit_storage() {
    let data = b"single unit with trailing bytes";
    let bytes = encoded_unit(data, data.len() as u32, 0);
    assert_eq!(
        Archive::open(Cursor::new(&bytes))
            .unwrap()
            .read_file("data")
            .unwrap(),
        data
    );
    let mut limited = Archive::with_options(
        Cursor::new(bytes),
        ReadOptions {
            max_single_unit_bytes: 16,
            ..ReadOptions::default()
        },
    )
    .unwrap();
    assert!(matches!(
        limited.open_file("data"),
        Err(Error::LimitExceeded(_))
    ));
    // Equality with decoded size means stored data, even with COMPRESS set.
    let bytes = encoded_unit(data, data.len() as u32, FileFlags::COMPRESS);
    assert_eq!(
        Archive::open(Cursor::new(bytes))
            .unwrap()
            .read_file("data")
            .unwrap(),
        data
    );
}

#[test]
fn reads_encrypted_single_units_with_adjusted_keys() {
    let data = b"an encrypted unit with an odd length";
    let mut writer = ArchiveWriter::new(
        Cursor::new(Vec::new()),
        WriteOptions {
            listfile: false,
            ..WriteOptions::default()
        },
    )
    .unwrap();
    writer
        .add_file(
            "data",
            data.len() as u32,
            &mut data.as_slice(),
            FileOptions {
                encrypted: true,
                adjusted_key: true,
                ..FileOptions::default()
            },
        )
        .unwrap();
    let mut bytes = writer.finish().unwrap().into_inner();
    // This fits one sector, whose encryption is identical to a single unit.
    change_table(&mut bytes, false, |table| {
        table[12..16].copy_from_slice(
            &(FileFlags::EXISTS
                | FileFlags::SINGLE_UNIT
                | FileFlags::ENCRYPTED
                | FileFlags::FIX_KEY)
                .to_le_bytes(),
        );
    });
    assert_eq!(
        Archive::open(Cursor::new(bytes))
            .unwrap()
            .read_file("data")
            .unwrap(),
        data
    );
}

#[test]
fn encoded_copy_is_independent_of_decoder_availability() {
    // PKWARE DCL's standard AIAIAIAIAIAIA example; no encoder dependency.
    let encoded = [0, 4, 0x82, 0x24, 0x25, 0x8f, 0x80, 0x7f];
    let bytes = encoded_unit(&encoded, 13, FileFlags::IMPLODE);
    let mut original = Archive::open(Cursor::new(bytes)).unwrap();
    #[cfg(not(feature = "mpq-decode"))]
    assert!(matches!(
        original.read_file("data"),
        Err(Error::FeatureDisabled("mpq-decode"))
    ));
    let mut actual = Vec::new();
    original
        .encoded_file(0)
        .unwrap()
        .read_to_end(&mut actual)
        .unwrap();
    assert_eq!(actual, encoded);
    let writer = ArchiveWriter::from_archive(Cursor::new(Vec::new()), &mut original).unwrap();
    let mut copied = Archive::open(Cursor::new(writer.finish().unwrap().into_inner())).unwrap();
    let mut after = Vec::new();
    copied
        .encoded_file(0)
        .unwrap()
        .read_to_end(&mut after)
        .unwrap();
    assert_eq!(after, encoded);
}

#[cfg(feature = "mpq-decode")]
#[test]
fn decodes_known_pkware_streams_in_both_framings() {
    let encoded = [0, 4, 0x82, 0x24, 0x25, 0x8f, 0x80, 0x7f];
    let bytes = encoded_unit(&encoded, 13, FileFlags::IMPLODE);
    assert_eq!(
        Archive::open(Cursor::new(bytes))
            .unwrap()
            .read_file("data")
            .unwrap(),
        b"AIAIAIAIAIAIA"
    );
    let mut masked = vec![8];
    masked.extend_from_slice(&encoded);
    let bytes = encoded_unit(&masked, 13, FileFlags::COMPRESS);
    assert_eq!(
        Archive::open(Cursor::new(bytes))
            .unwrap()
            .read_file("data")
            .unwrap(),
        b"AIAIAIAIAIAIA"
    );
}

#[cfg(feature = "mpq-decode")]
#[test]
fn decodes_known_zlib_bzip2_sparse_and_chained_streams() {
    // Fixed zlib and bzip2 streams representing 128 zero bytes.
    let streams: &[&[u8]] = &[
        &[2, 120, 156, 99, 96, 24, 88, 0, 0, 0, 128, 0, 1],
        &[
            16, 66, 90, 104, 57, 49, 65, 89, 38, 83, 89, 185, 95, 21, 67, 0, 0, 0, 64, 128, 64, 0,
            0, 4, 32, 0, 33, 0, 130, 131, 23, 114, 69, 56, 80, 144, 185, 95, 21, 67,
        ],
        // Sparse: big-endian output size followed by a 128-byte zero run.
        &[32, 0, 0, 0, 128, 125],
        // Zlib wrapping that sparse stream; decompress in reverse order.
        &[34, 120, 156, 99, 96, 96, 104, 168, 5, 0, 1, 130, 0, 254],
    ];
    for stream in streams {
        let bytes = encoded_unit(stream, 128, FileFlags::COMPRESS);
        assert_eq!(
            Archive::open(Cursor::new(bytes))
                .unwrap()
                .read_file("data")
                .unwrap(),
            [0; 128]
        );
    }
    // The decoder must enforce the declared size independently of encoding.
    let bytes = encoded_unit(streams[0], 32, FileFlags::COMPRESS);
    assert!(Archive::open(Cursor::new(bytes))
        .unwrap()
        .read_file("data")
        .is_err());
}

#[cfg(all(feature = "mpq-decode", feature = "mpq-encode"))]
#[test]
fn rejects_sector_offsets_bombs_unknown_masks_and_bad_checksums() {
    let mut writer = ArchiveWriter::new(
        Cursor::new(Vec::new()),
        WriteOptions {
            listfile: false,
            ..WriteOptions::default()
        },
    )
    .unwrap();
    writer
        .add_file(
            "data",
            10000,
            &mut vec![42; 10000].as_slice(),
            FileOptions {
                compression: Compression::Zlib,
                sector_checksums: true,
                ..FileOptions::default()
            },
        )
        .unwrap();
    let bytes = writer.finish().unwrap().into_inner();
    let mut original = Archive::open(Cursor::new(&bytes)).unwrap();
    assert_eq!(original.read_file("data").unwrap(), vec![42; 10000]);
    let block = original.index().blocks[0];
    let start = block.offset as usize;
    let first = u32::from_le_bytes(bytes[start..start + 4].try_into().unwrap()) as usize;
    let mut corrupt = bytes.clone();
    corrupt[start + 4..start + 8].copy_from_slice(&0u32.to_le_bytes());
    assert!(Archive::open(Cursor::new(corrupt))
        .unwrap()
        .open_file("data")
        .is_err());
    let mut corrupt = bytes.clone();
    corrupt[start + first + 3] ^= 1;
    assert!(matches!(
        Archive::open(Cursor::new(corrupt))
            .unwrap()
            .read_file("data"),
        Err(Error::ChecksumMismatch(0))
    ));
    let mut corrupt = bytes.clone();
    corrupt[start + first] = 4; // Unknown compression bit.
    let mut archive = Archive::with_options(
        Cursor::new(corrupt),
        ReadOptions {
            verify_sector_checksums: false,
            ..ReadOptions::default()
        },
    )
    .unwrap();
    assert!(matches!(
        archive.read_file("data"),
        Err(Error::UnsupportedCompression(4))
    ));
    // Reduce declared size without changing the compressed stream's output.
    let mut bomb = bytes;
    change_table(&mut bomb, false, |table| {
        table[8..12].copy_from_slice(&100u32.to_le_bytes())
    });
    assert!(Archive::with_options(
        Cursor::new(bomb),
        ReadOptions {
            verify_sector_checksums: false,
            ..ReadOptions::default()
        }
    )
    .unwrap()
    .read_file("data")
    .is_err());
}

#[test]
fn full_hash_tables_allow_replacement_and_reuse_tombstones() {
    let mut bytes = archive();
    change_table(&mut bytes, true, |table| {
        let existing: [u8; 16] = *table
            .as_chunks::<16>()
            .0
            .iter()
            .find(|e| u32::from_le_bytes(e[12..16].try_into().unwrap()) == 0)
            .unwrap();
        for (i, entry) in table.as_chunks_mut::<16>().0.iter_mut().enumerate() {
            entry.copy_from_slice(&existing);
            entry[8..10].copy_from_slice(&(i as u16).to_le_bytes());
        }
    });
    let mut source = Archive::open(Cursor::new(bytes)).unwrap();
    let mut writer = ArchiveWriter::from_archive(Cursor::new(Vec::new()), &mut source).unwrap();
    assert!(matches!(
        writer.start_file("new", 0, FileOptions::default()),
        Err(Error::LimitExceeded(_))
    ));
    writer
        .replace_file("data", 3, &mut b"new".as_slice(), FileOptions::default())
        .unwrap();
    writer.remove_file("data", 1, 0).unwrap();
    writer
        .add_file("added", 3, &mut b"add".as_slice(), FileOptions::default())
        .unwrap();
    let mut edited = Archive::open(Cursor::new(writer.finish().unwrap().into_inner())).unwrap();
    assert_eq!(edited.read_file("data").unwrap(), b"new");
    assert_eq!(edited.read_file("added").unwrap(), b"add");
    assert!(matches!(
        edited.open_file_with_locale("data", 1, 0),
        Err(Error::FileNotFound)
    ));
    let mut original = Vec::new();
    edited
        .open_file_with_locale("data", 2, 0)
        .unwrap()
        .read_to_end(&mut original)
        .unwrap();
    assert_eq!(original, b"payload");
}

#[cfg(feature = "mpq-decode")]
#[test]
fn reads_new_codecs_through_archive_framing() {
    use super::super::codec_test_vectors::{
        HUFFMAN_ZEROS, LZMA_EOS, LZMA_SIZED, MONO, MONO_CHAIN, MONO_PCM, RAW, STEREO, STEREO_CHAIN,
        STEREO_PCM,
    };

    let zeros = [0u8; 64];
    let cases: &[(u8, &[u8], &[u8])] = &[
        (1, HUFFMAN_ZEROS, &zeros),
        (0x40, MONO, MONO_PCM),
        (0x80, STEREO, STEREO_PCM),
        (0x41, MONO_CHAIN, MONO_PCM),
        (0x81, STEREO_CHAIN, STEREO_PCM),
        (0x12, LZMA_EOS, RAW),
        (0x12, LZMA_SIZED, RAW),
    ];
    for &(mask, compressed, expected) in cases {
        let mut encoded = vec![mask];
        encoded.extend_from_slice(compressed);
        for single_unit in [true, false] {
            let payload = if single_unit {
                encoded.clone()
            } else {
                let end = 8 + encoded.len() as u32;
                let mut sectors = 8u32.to_le_bytes().to_vec();
                sectors.extend_from_slice(&end.to_le_bytes());
                sectors.extend_from_slice(&encoded);
                sectors
            };
            let mut bytes = encoded_unit(&payload, expected.len() as u32, FileFlags::COMPRESS);
            if !single_unit {
                change_table(&mut bytes, false, |table| {
                    table[12..16]
                        .copy_from_slice(&(FileFlags::EXISTS | FileFlags::COMPRESS).to_le_bytes());
                });
            }
            assert_eq!(
                Archive::open(Cursor::new(bytes))
                    .unwrap()
                    .read_file("data")
                    .unwrap(),
                expected
            );
        }
    }
}

#[cfg(all(feature = "mpq-decode", feature = "mpq-encode"))]
#[test]
fn compressed_classic_and_extended_tables() {
    use super::super::codec::encode;
    use super::super::Header;

    for version in 1..=3 {
        let fixture = if version == 1 {
            include_bytes!("../../tests/fixtures/mpq/storm-v2.mpq").as_slice()
        } else if version == 2 {
            include_bytes!("../../tests/fixtures/mpq/storm-v3.mpq").as_slice()
        } else {
            include_bytes!("../../tests/fixtures/mpq/storm-v4.mpq").as_slice()
        };
        let archive = Archive::open(Cursor::new(fixture)).unwrap();
        let mut header = archive.index().header.clone();
        let offsets = [
            header.hash_table_offset,
            header.block_table_offset,
            header.hi_block_table_offset,
            header.het_table_offset,
            header.bet_table_offset,
        ];
        let first = offsets.iter().copied().filter(|&n| n != 0).min().unwrap();
        let mut output = fixture[..first as usize].to_vec();
        let mut tables: [Vec<u8>; 5] = Default::default();
        for i in 0..5 {
            if offsets[i] == 0 {
                continue;
            }
            let start = offsets[i] as usize;
            let stored = header.table_sizes[i] as usize;
            let mut bytes = fixture[start..start + stored].to_vec();
            if i != 2 {
                let key = hash(
                    if i == 0 || i == 3 {
                        b"(hash table)"
                    } else {
                        b"(block table)"
                    },
                    3,
                );
                let prefix = if i >= 3 { 12 } else { 0 };
                crypt(&mut bytes[prefix..], key, true);
                let encoded = encode(&bytes[prefix..], Compression::Zlib).unwrap();
                bytes.truncate(prefix);
                bytes.extend(encoded);
                crypt(&mut bytes[prefix..], key, false);
            }
            tables[i] = bytes;
        }
        let mut positions = [0; 5];
        for i in [3, 4, 0, 1, 2] {
            if offsets[i] != 0 {
                positions[i] = output.len() as u64;
                header.table_sizes[i] = tables[i].len() as u64;
                let digest_index = [1, 0, 2, 4, 3][i];
                header.md5[digest_index] = md5::compute(&tables[i]).0;
                output.extend_from_slice(&tables[i]);
            }
        }
        header.hash_table_offset = positions[0];
        header.block_table_offset = positions[1];
        header.hi_block_table_offset = positions[2];
        header.het_table_offset = positions[3];
        header.bet_table_offset = positions[4];
        header.archive_size = output.len() as u64;
        header.raw_chunk_size = 0;
        header.write(&mut Cursor::new(&mut output)).unwrap();
        let mut archive = Archive::open(Cursor::new(&output)).unwrap();
        assert_eq!(archive.read_file("stored.bin").unwrap(), b"payload");
        assert_eq!(
            archive.read_file("units/encrypted.bin").unwrap(),
            (0..17003).map(|i| (i % 7) as u8).collect::<Vec<_>>()
        );
        let options = super::ReadOptions {
            max_table_bytes: 16,
            ..Default::default()
        };
        assert!(Archive::with_options(Cursor::new(&output), options).is_err());
        // Header codec still accepts the expected version after table repacking.
        assert_eq!(
            Header::decode(&output[..header.header_size() as usize])
                .unwrap()
                .version,
            version
        );
    }
}

#[test]
fn differing_classic_and_bet_record_order_is_remapped() {
    let fixture = include_bytes!("../../tests/fixtures/mpq/storm-v3.mpq");
    let header = Archive::open(Cursor::new(fixture))
        .unwrap()
        .index()
        .header
        .clone();
    let mut bytes = fixture.to_vec();
    let start = header.block_table_offset as usize;
    let table = &mut bytes[start..start + header.block_table_entries as usize * 16];
    let key = hash(b"(block table)", 3);
    crypt(table, key, true);
    let first: Vec<_> = table[..16].to_vec();
    table.copy_within(16..32, 0);
    table[16..32].copy_from_slice(&first);
    crypt(table, key, false);
    // Adjust the classic filename references to retain their original meaning.
    let start = header.hash_table_offset as usize;
    let table = &mut bytes[start..start + header.hash_table_entries as usize * 16];
    let key = hash(b"(hash table)", 3);
    crypt(table, key, true);
    for record in table.as_chunks_mut::<16>().0.iter_mut() {
        let id = u32::from_le_bytes(record[12..16].try_into().unwrap());
        if id < 2 {
            record[12..16].copy_from_slice(&(1 - id).to_le_bytes());
        }
    }
    crypt(table, key, false);
    let mut archive = Archive::open(Cursor::new(bytes)).unwrap();
    assert_eq!(archive.read_file("stored.bin").unwrap(), b"payload");
    assert_eq!(archive.index().blocks.len(), 2);
    assert_eq!(archive.index().find("Units/Encrypted.bin", 0, 0), Some(1));
}
