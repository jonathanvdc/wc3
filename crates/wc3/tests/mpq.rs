use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

use wc3::mpq::{
    Archive, ArchiveWriter, Compression, Error, FileOptions, ReadOptions, WriteOptions,
};

fn payload(size: usize) -> Vec<u8> {
    (0..size)
        .map(|i| {
            if i / 4096 % 2 == 0 {
                (i % 7) as u8
            } else {
                ((i * 73 + i / 11) % 256) as u8
            }
        })
        .collect()
}

fn editable_archive() -> Vec<u8> {
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    let data = payload(17003);
    for name in ["stored.bin", "encrypted.bin", "Units\\Encrypted.bin"] {
        writer
            .add_file(
                name,
                data.len() as u32,
                &mut data.as_slice(),
                FileOptions {
                    encrypted: name != "stored.bin",
                    adjusted_key: name != "stored.bin",
                    ..FileOptions::default()
                },
            )
            .unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn build(name: &[u8], data: &[u8], options: FileOptions) -> Vec<u8> {
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    writer
        .add_file(name, data.len() as u32, &mut &*data, options)
        .unwrap();
    writer.finish().unwrap().into_inner()
}

#[test]
fn streaming_round_trips_sector_boundaries_and_encryption() {
    let compressions = vec![Compression::Stored];
    #[cfg(all(feature = "mpq-decode", feature = "mpq-encode"))]
    let compressions = [compressions, vec![Compression::Zlib, Compression::Bzip2]].concat();
    for compression in compressions {
        for (encrypted, adjusted_key) in [(false, false), (true, false), (true, true)] {
            for size in [0, 1, 3, 511, 512, 513, 4095, 4096, 4097, 17003] {
                let data = payload(size);
                let mut writer = ArchiveWriter::new(
                    Cursor::new(Vec::new()),
                    WriteOptions {
                        sector_size_shift: 0,
                        ..WriteOptions::default()
                    },
                )
                .unwrap();
                let mut entry = writer
                    .start_file(
                        b"Units\\Test.bin",
                        size as u32,
                        FileOptions {
                            compression,
                            encrypted,
                            adjusted_key,
                            ..FileOptions::default()
                        },
                    )
                    .unwrap();
                for chunk in data.chunks(17) {
                    entry.write_all(chunk).unwrap();
                }
                entry.finish().unwrap();
                let bytes = writer.finish().unwrap().into_inner();
                let mut archive = Archive::open(Cursor::new(bytes)).unwrap();
                let mut entry = archive.open_file(b"units/test.BIN").unwrap();
                assert_eq!(entry.read(&mut []).unwrap(), 0);
                let mut actual = Vec::new();
                let mut buffer = [0; 13];
                loop {
                    let count = entry.read(&mut buffer).unwrap();
                    if count == 0 {
                        break;
                    }
                    actual.extend_from_slice(&buffer[..count]);
                }
                assert_eq!(actual, data);
                drop(entry);
                assert_eq!(archive.read_file(b"Units\\Test.bin").unwrap(), data);
            }
        }
    }
}

#[test]
fn exact_locale_lookup_duplicates_and_listfile() {
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    writer
        .add_file("a.txt", 1, &mut b"a".as_slice(), FileOptions::default())
        .unwrap();
    writer
        .add_file(
            "A.TXT",
            1,
            &mut b"b".as_slice(),
            FileOptions {
                locale: 1033,
                ..FileOptions::default()
            },
        )
        .unwrap();
    assert!(matches!(
        writer.start_file("a.txt", 0, FileOptions::default()),
        Err(Error::DuplicateFile)
    ));
    assert!(matches!(
        writer.start_file("(listfile)", 0, FileOptions::default()),
        Err(Error::DuplicateFile)
    ));
    assert!(matches!(
        writer.start_file(b"bad\0name", 0, FileOptions::default()),
        Err(Error::InvalidName)
    ));
    let mut archive = Archive::open(Cursor::new(writer.finish().unwrap().into_inner())).unwrap();
    assert_eq!(archive.read_file("a.txt").unwrap(), b"a");
    let mut bytes = Vec::new();
    archive
        .open_file_with_locale("a.txt", 1033, 0)
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    assert_eq!(bytes, b"b");
    assert!(matches!(
        archive.open_file_with_locale("a.txt", 1031, 0),
        Err(Error::FileNotFound)
    ));
    assert_eq!(
        archive.known_names().unwrap(),
        vec![b"(listfile)".to_vec(), b"A.TXT".to_vec(), b"a.txt".to_vec()]
    );
}

#[test]
fn map_prefix_userdata_and_explicit_offsets() {
    let bytes = build(b"a", b"data", FileOptions::default());
    let mut prefixed = vec![0; 512];
    prefixed[..4].copy_from_slice(b"HM3W");
    prefixed.extend_from_slice(&bytes);
    let mut archive = Archive::open(Cursor::new(&prefixed)).unwrap();
    assert_eq!(archive.archive_offset(), 512);
    assert_eq!(archive.read_file("a").unwrap(), b"data");
    prefixed[..4].copy_from_slice(b"MPQ\x1b");
    prefixed[8..12].copy_from_slice(&512u32.to_le_bytes());
    let mut archive = Archive::open(Cursor::new(&prefixed)).unwrap();
    assert_eq!(archive.read_file("a").unwrap(), b"data");
    let mut arbitrary = vec![0; 7];
    arbitrary.extend_from_slice(&bytes);
    let mut archive = Archive::at(Cursor::new(arbitrary), 7, ReadOptions::default()).unwrap();
    assert_eq!(archive.read_file("a").unwrap(), b"data");
    let options = ReadOptions {
        max_header_search_bytes: 511,
        ..ReadOptions::default()
    };
    prefixed[..4].copy_from_slice(b"HM3W");
    assert!(Archive::with_options(Cursor::new(prefixed), options).is_err());
}

#[test]
fn writer_can_embed_at_an_aligned_offset() {
    let mut output = Cursor::new(vec![0; 512]);
    output.set_position(512);
    let mut writer = ArchiveWriter::new(output, WriteOptions::default()).unwrap();
    writer
        .add_file(
            "a",
            3,
            &mut b"abc".as_slice(),
            FileOptions {
                encrypted: true,
                adjusted_key: true,
                ..FileOptions::default()
            },
        )
        .unwrap();
    let mut archive = Archive::open(Cursor::new(writer.finish().unwrap().into_inner())).unwrap();
    assert_eq!(archive.read_file("a").unwrap(), b"abc");
}

#[test]
fn copy_edit_preserves_unknown_names_encoded_data_and_offsets() {
    let mut original = Archive::open(Cursor::new(editable_archive())).unwrap();
    let original_index = original.index().clone();
    let mut encoded = Vec::new();
    let id = original.index().find("Units\\Encrypted.bin", 0, 0).unwrap();
    original
        .encoded_file(id)
        .unwrap()
        .read_to_end(&mut encoded)
        .unwrap();
    let mut destination = Cursor::new(vec![0; 512]);
    destination.set_position(512);
    let mut writer = ArchiveWriter::from_archive(destination, &mut original).unwrap();
    writer.remove_file("stored.bin", 0, 0).unwrap();
    writer.remove_file("(listfile)", 0, 0).unwrap();
    writer
        .replace_file(
            "encrypted.bin",
            3,
            &mut b"new".as_slice(),
            FileOptions::default(),
        )
        .unwrap();
    writer
        .add_file(
            "added.bin",
            4,
            &mut b"more".as_slice(),
            FileOptions::default(),
        )
        .unwrap();
    let mut edited = Archive::open(Cursor::new(writer.finish().unwrap().into_inner())).unwrap();
    assert!(matches!(
        edited.read_file("stored.bin"),
        Err(Error::FileNotFound)
    ));
    assert_eq!(edited.read_file("encrypted.bin").unwrap(), b"new");
    assert_eq!(edited.read_file("added.bin").unwrap(), b"more");
    assert!(edited.known_names().unwrap().is_empty());
    assert_eq!(
        edited.index().blocks[id as usize],
        original_index.blocks[id as usize]
    );
    let mut after = Vec::new();
    edited
        .encoded_file(id)
        .unwrap()
        .read_to_end(&mut after)
        .unwrap();
    assert_eq!(after, encoded);
    assert_eq!(
        edited.read_file("Units\\Encrypted.bin").unwrap(),
        payload(17003)
    );
}

#[test]
fn writer_rejects_abandoned_short_overlong_and_failed_entries() {
    for case in 0..3 {
        let mut writer =
            ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
        let mut entry = writer.start_file("a", 2, FileOptions::default()).unwrap();
        match case {
            0 => drop(entry),
            1 => {
                entry.write_all(b"a").unwrap();
                assert!(matches!(entry.finish(), Err(Error::SizeMismatch)));
            }
            _ => {
                assert!(entry.write_all(b"abc").is_err());
                assert!(matches!(entry.finish(), Err(Error::WriterFailed)));
            }
        }
        assert!(matches!(writer.finish(), Err(Error::WriterFailed)));
    }
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    assert!(matches!(
        writer.add_file(
            "short",
            10,
            &mut b"short".as_slice(),
            FileOptions::default()
        ),
        Err(Error::SizeMismatch)
    ));
    assert!(matches!(writer.finish(), Err(Error::WriterFailed)));
}

#[test]
fn reads_only_declared_input_length() {
    let mut input = b"abc trailing".as_slice();
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    writer
        .add_file("a", 3, &mut input, FileOptions::default())
        .unwrap();
    assert_eq!(input, b" trailing");
    writer.finish().unwrap();
}

#[test]
fn invalid_headers_and_read_limits_are_rejected() {
    let original = build(b"a", b"data", FileOptions::default());
    for (at, bytes) in [
        (12, vec![1, 0]),
        (14, vec![23, 0]),
        (24, 3u32.to_le_bytes().to_vec()),
        (16, u32::MAX.to_le_bytes().to_vec()),
        (8, u32::MAX.to_le_bytes().to_vec()),
    ] {
        let mut corrupt = original.clone();
        corrupt[at..at + bytes.len()].copy_from_slice(&bytes);
        assert!(
            Archive::open(Cursor::new(corrupt)).is_err(),
            "field at {at}"
        );
    }
    let options = ReadOptions {
        max_table_entries: 1,
        ..ReadOptions::default()
    };
    assert!(matches!(
        Archive::with_options(Cursor::new(&original), options),
        Err(Error::LimitExceeded(_))
    ));
    let options = ReadOptions {
        max_file_size: 3,
        ..ReadOptions::default()
    };
    let mut archive = Archive::with_options(Cursor::new(original), options).unwrap();
    assert!(matches!(
        archive.open_file("a"),
        Err(Error::LimitExceeded(_))
    ));
}

#[cfg(not(feature = "mpq-encode"))]
#[test]
fn disabled_encoder_rejects_before_poisoning_writer() {
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    assert!(matches!(
        writer.start_file(
            "a",
            1,
            FileOptions {
                compression: Compression::Zlib,
                ..FileOptions::default()
            }
        ),
        Err(Error::FeatureDisabled("mpq-encode"))
    ));
    writer
        .add_file("a", 1, &mut b"x".as_slice(), FileOptions::default())
        .unwrap();
    writer.finish().unwrap();
}

/// Rejects whole-file buffering and tracks the largest source read request.
struct BoundedSource {
    inner: Cursor<Vec<u8>>,
    max_read: usize,
}
impl Read for BoundedSource {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        self.max_read = self.max_read.max(out.len());
        self.inner.read(out)
    }
}
impl Seek for BoundedSource {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        self.inner.seek(from)
    }
}

#[test]
fn large_payload_reads_are_sector_bounded() {
    let data = vec![42; 2 << 20];
    let bytes = build(
        b"large",
        &data,
        FileOptions {
            encrypted: true,
            ..FileOptions::default()
        },
    );
    let mut archive = Archive::open(BoundedSource {
        inner: Cursor::new(bytes),
        max_read: 0,
    })
    .unwrap();
    let mut sink = io::sink();
    assert_eq!(
        io::copy(&mut archive.open_file("large").unwrap(), &mut sink).unwrap(),
        data.len() as u64
    );
    assert!(archive.into_inner().max_read <= 4096);
}

struct FailingSink {
    inner: Cursor<Vec<u8>>,
    budget: usize,
}
impl Write for FailingSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.budget == 0 {
            return Err(io::Error::other("injected write failure"));
        }
        let count = bytes.len().min(self.budget);
        self.budget -= count;
        self.inner.write(&bytes[..count])
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Seek for FailingSink {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        self.inner.seek(from)
    }
}

#[test]
fn partial_destination_failure_prevents_archive_completion() {
    let sink = FailingSink {
        inner: Cursor::new(Vec::new()),
        budget: 42,
    };
    let mut writer = ArchiveWriter::new(
        sink,
        WriteOptions {
            sector_size_shift: 0,
            listfile: false,
            ..WriteOptions::default()
        },
    )
    .unwrap();
    let mut entry = writer
        .start_file("data", 512, FileOptions::default())
        .unwrap();
    assert!(entry.write_all(&[42; 512]).is_err());
    assert!(matches!(entry.finish(), Err(Error::WriterFailed)));
    assert!(matches!(writer.finish(), Err(Error::WriterFailed)));
}

#[test]
fn removed_names_do_not_survive_generated_listfile() {
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    writer
        .add_file("removed", 0, &mut [].as_slice(), FileOptions::default())
        .unwrap();
    writer.remove_file("REMOVED", 0, 0).unwrap();
    let mut archive = Archive::open(Cursor::new(writer.finish().unwrap().into_inner())).unwrap();
    assert_eq!(archive.known_names().unwrap(), vec![b"(listfile)".to_vec()]);
}

#[test]
fn newer_headers_round_trip_and_edit() {
    for version in 0..=3 {
        for extended_index in [false, true] {
            if extended_index && version < 2 {
                continue;
            }
            let mut writer = ArchiveWriter::new(
                Cursor::new(Vec::new()),
                WriteOptions {
                    header_version: version,
                    extended_index,
                    listfile: false,
                    ..WriteOptions::default()
                },
            )
            .unwrap();
            for name in ["Units/Encrypted.bin", "stored.bin", "removed.bin"] {
                writer
                    .add_file(
                        name,
                        7,
                        &mut b"payload".as_slice(),
                        FileOptions {
                            encrypted: true,
                            adjusted_key: true,
                            ..FileOptions::default()
                        },
                    )
                    .unwrap();
            }
            let bytes = writer.finish().unwrap().into_inner();
            let mut archive = Archive::open(Cursor::new(bytes)).unwrap();
            assert_eq!(archive.index().header.version, version);
            assert_eq!(archive.index().extended.is_some(), extended_index);
            assert_eq!(
                archive.read_file("UNITS\\ENCRYPTED.BIN").unwrap(),
                b"payload"
            );
            let mut writer =
                ArchiveWriter::from_archive(Cursor::new(Vec::new()), &mut archive).unwrap();
            writer.remove_file("removed.bin", 0, 0).unwrap();
            writer
                .replace_file(
                    "stored.bin",
                    3,
                    &mut b"new".as_slice(),
                    FileOptions::default(),
                )
                .unwrap();
            let mut archive =
                Archive::open(Cursor::new(writer.finish().unwrap().into_inner())).unwrap();
            assert_eq!(archive.read_file("stored.bin").unwrap(), b"new");
            assert!(matches!(
                archive.open_file("removed.bin"),
                Err(Error::FileNotFound)
            ));
            assert_eq!(
                archive.read_file("Units/Encrypted.bin").unwrap(),
                b"payload"
            );
        }
    }
}

#[cfg(feature = "mpq-decode")]
#[test]
fn stormlib_newer_archives_and_het_only_lookup() {
    for fixture in [
        include_bytes!("fixtures/mpq/storm-v2.mpq").as_slice(),
        include_bytes!("fixtures/mpq/storm-v3.mpq").as_slice(),
        include_bytes!("fixtures/mpq/storm-v4.mpq").as_slice(),
    ] {
        let mut archive = Archive::open(Cursor::new(fixture)).unwrap();
        assert_eq!(archive.read_file("stored.bin").unwrap(), b"payload");
        let expected: Vec<_> = (0..17003).map(|i| (i % 7) as u8).collect();
        assert_eq!(archive.read_file("units/encrypted.bin").unwrap(), expected);
        if archive.index().header.version >= 2 {
            let mut bytes = fixture.to_vec();
            // Remove the classic tables from the header to require HET/BET.
            bytes[16..32].fill(0);
            bytes[40..44].fill(0);
            if archive.index().header.version == 2 {
                let end =
                    archive.index().header.bet_table_offset + archive.index().header.table_sizes[4];
                bytes.truncate(end as usize);
                bytes[44..52].copy_from_slice(&end.to_le_bytes());
            }
            if archive.index().header.version == 3 {
                bytes[68..84].fill(0);
                bytes[112..144].fill(0);
                let digest = md5::compute(&bytes[..192]).0;
                bytes[192..208].copy_from_slice(&digest);
            }
            let mut archive = Archive::open(Cursor::new(bytes)).unwrap();
            assert!(archive.index().hashes.is_empty());
            assert_eq!(archive.read_file("UNITS/ENCRYPTED.BIN").unwrap(), expected);
            let mut writer =
                ArchiveWriter::from_archive(Cursor::new(Vec::new()), &mut archive).unwrap();
            writer
                .replace_file(
                    "stored.bin",
                    3,
                    &mut b"new".as_slice(),
                    FileOptions::default(),
                )
                .unwrap();
            let mut edited =
                Archive::open(Cursor::new(writer.finish().unwrap().into_inner())).unwrap();
            assert_eq!(edited.read_file("units/encrypted.bin").unwrap(), expected);
            assert_eq!(edited.read_file("stored.bin").unwrap(), b"new");
        }
    }
}

#[test]
fn v4_raw_digests_cover_chunks_and_detect_corruption() {
    for chunk in [1, 7, 1024, 4096] {
        for compression in [Compression::Stored, Compression::Zlib] {
            if compression != Compression::Stored
                && !cfg!(all(feature = "mpq-encode", feature = "mpq-decode"))
            {
                continue;
            }
            let mut writer = ArchiveWriter::new(
                Cursor::new(Vec::new()),
                WriteOptions {
                    header_version: 3,
                    extended_index: true,
                    raw_chunk_size: chunk,
                    listfile: false,
                    ..WriteOptions::default()
                },
            )
            .unwrap();
            let data = payload(17003);
            writer
                .add_file(
                    "data",
                    data.len() as u32,
                    &mut data.as_slice(),
                    FileOptions {
                        compression,
                        encrypted: true,
                        adjusted_key: true,
                        ..FileOptions::default()
                    },
                )
                .unwrap();
            let bytes = writer.finish().unwrap().into_inner();
            let mut archive = Archive::open(Cursor::new(&bytes)).unwrap();
            assert_eq!(archive.read_file("data").unwrap(), data);
            let block = archive.index().blocks[0];
            for offset in [block.offset, block.offset + block.stored_size as u64] {
                let mut corrupt = bytes.clone();
                corrupt[offset as usize] ^= 1;
                let mut archive = Archive::open(Cursor::new(corrupt)).unwrap();
                assert!(archive.open_file("data").is_err());
            }
            let mut corrupt = bytes.clone();
            corrupt[14] ^= 1;
            assert!(Archive::open(Cursor::new(corrupt)).is_err());
            let mut corrupt = bytes.clone();
            let header = &archive.index().header;
            corrupt[(header.het_table_offset + header.table_sizes[3]) as usize] ^= 1;
            assert!(Archive::open(Cursor::new(corrupt)).is_err());
        }
    }
}

#[test]
fn shortened_v3_headers_survive_encoded_editing() {
    let mut writer = ArchiveWriter::new(
        Cursor::new(Vec::new()),
        WriteOptions {
            header_version: 1,
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
    let mut bytes = writer.finish().unwrap().into_inner();
    bytes[12..14].copy_from_slice(&2u16.to_le_bytes());
    let mut archive = Archive::open(Cursor::new(bytes)).unwrap();
    assert_eq!(archive.index().header.header_size(), 44);
    let writer = ArchiveWriter::from_archive(Cursor::new(Vec::new()), &mut archive).unwrap();
    let mut edited = Archive::open(Cursor::new(writer.finish().unwrap().into_inner())).unwrap();
    assert_eq!(edited.index().header.header_size(), 44);
    assert_eq!(edited.read_file("data").unwrap(), b"payload");
}

#[test]
fn editing_het_only_archives_preserves_unknown_hashes() {
    let mut writer = ArchiveWriter::new(
        Cursor::new(Vec::new()),
        WriteOptions {
            header_version: 3,
            extended_index: true,
            listfile: false,
            ..WriteOptions::default()
        },
    )
    .unwrap();
    for i in 0..40 {
        writer
            .add_file(
                format!("file-{i}"),
                7,
                &mut b"payload".as_slice(),
                FileOptions::default(),
            )
            .unwrap();
    }
    let mut bytes = writer.finish().unwrap().into_inner();
    bytes[16..32].fill(0);
    bytes[40..44].fill(0);
    bytes[68..84].fill(0);
    bytes[112..144].fill(0);
    let digest = md5::compute(&bytes[..192]).0;
    bytes[192..208].copy_from_slice(&digest);
    let mut archive = Archive::open(Cursor::new(bytes)).unwrap();
    let mut writer = ArchiveWriter::from_archive(Cursor::new(Vec::new()), &mut archive).unwrap();
    for _ in 0..12 {
        writer
            .replace_file("file-0", 3, &mut b"new".as_slice(), FileOptions::default())
            .unwrap();
    }
    let mut archive = Archive::open(Cursor::new(writer.finish().unwrap().into_inner())).unwrap();
    assert_eq!(archive.index().blocks.len(), 40);
    assert!(
        archive.index().header.hash_table_entries >= archive.index().header.block_table_entries
    );
    for i in 1..40 {
        assert_eq!(archive.read_file(format!("file-{i}")).unwrap(), b"payload");
    }
    assert_eq!(archive.read_file("file-0").unwrap(), b"new");
}

#[test]
fn v4_table_md5s_detect_corruption_without_raw_chunks() {
    let mut writer = ArchiveWriter::new(
        Cursor::new(Vec::new()),
        WriteOptions {
            header_version: 3,
            extended_index: true,
            listfile: false,
            ..WriteOptions::default()
        },
    )
    .unwrap();
    writer
        .add_file(
            "file",
            7,
            &mut b"payload".as_slice(),
            FileOptions::default(),
        )
        .unwrap();
    let bytes = writer.finish().unwrap().into_inner();
    let archive = Archive::open(Cursor::new(&bytes)).unwrap();
    let h = &archive.index().header;
    for offset in [
        h.hash_table_offset,
        h.block_table_offset,
        h.het_table_offset,
        h.bet_table_offset,
    ] {
        let mut corrupt = bytes.clone();
        corrupt[offset as usize] ^= 1;
        assert!(Archive::open(Cursor::new(corrupt)).is_err());
    }
}

#[cfg(feature = "mpq-decode")]
#[test]
fn permissive_mode_preserves_newer_format_lookup_and_integrity() {
    use wc3::mpq::ReadMode;

    let options = ReadOptions {
        mode: ReadMode::Permissive,
        ..ReadOptions::default()
    };
    for bytes in [
        include_bytes!("fixtures/mpq/storm-v2.mpq").as_slice(),
        include_bytes!("fixtures/mpq/storm-v3.mpq").as_slice(),
        include_bytes!("fixtures/mpq/storm-v4.mpq").as_slice(),
    ] {
        let mut archive = Archive::with_options(Cursor::new(bytes), options.clone()).unwrap();
        assert_eq!(archive.read_file("stored.bin").unwrap(), b"payload");
        assert_eq!(
            archive.read_file("units/encrypted.bin").unwrap(),
            (0..17003).map(|i| (i % 7) as u8).collect::<Vec<_>>()
        );
        assert!(archive.diagnostics().is_empty());
    }
    let mut bytes = include_bytes!("fixtures/mpq/storm-v4.mpq").to_vec();
    bytes[14] ^= 1;
    assert!(Archive::with_options(Cursor::new(bytes), options.clone()).is_err());
    let mut bytes = include_bytes!("fixtures/mpq/storm-v4.mpq").to_vec();
    bytes[4..8].copy_from_slice(&999u32.to_le_bytes());
    assert!(Archive::with_options(Cursor::new(bytes), options).is_err());
}
