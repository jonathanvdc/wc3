use std::io::{self, Cursor, Read};
#[cfg(all(feature = "mpq-encode", feature = "mpq-decode"))]
use wc3::mpq::Compression;
use wc3::mpq::{
    Archive, ArchiveWriter, EncodedEntry, EncodedFileMetadata, Error, FileFlags, FileOptions,
    WriteOptions,
};

fn archive(mut data: &[u8], options: WriteOptions, file: FileOptions) -> Vec<u8> {
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), options).unwrap();
    writer
        .add_file("entry.bin", data.len() as u32, &mut data, file)
        .unwrap();
    writer.finish().unwrap().into_inner()
}
fn stored_metadata(size: u32) -> EncodedFileMetadata {
    EncodedFileMetadata {
        file_size: size,
        stored_size: size,
        flags: FileFlags(FileFlags::EXISTS),
        sector_size: 4096,
        locale: 0,
        platform: 0,
    }
}

#[test]
fn stored_copy_is_bounded_relocatable_and_preserves_locale() {
    for size in [0, 1, 4096, 8193] {
        let data = vec![42; size];
        let bytes = archive(
            &data,
            WriteOptions::default(),
            FileOptions {
                locale: 7,
                platform: 2,
                ..FileOptions::default()
            },
        );
        let mut source = Archive::open(Cursor::new(bytes)).unwrap();
        let entry = source
            .open_encoded_file_with_locale("ENTRY.BIN", 7, 2)
            .unwrap();
        assert_eq!(entry.metadata().file_size, size as u32);
        assert_eq!(entry.metadata().locale, 7);
        let mut writer = ArchiveWriter::new(
            Cursor::new(Vec::new()),
            WriteOptions {
                sector_size_shift: 4,
                ..WriteOptions::default()
            },
        )
        .unwrap();
        writer
            .add_file("prefix", 3, &mut b"abc".as_slice(), FileOptions::default())
            .unwrap();
        writer.add_encoded_file("renamed.bin", entry).unwrap();
        let mut destination =
            Archive::open(Cursor::new(writer.finish().unwrap().into_inner())).unwrap();
        let mut file = destination
            .open_file_with_locale("renamed.bin", 7, 2)
            .unwrap();
        let mut decoded = Vec::new();
        file.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, data);
        assert!(destination.open_file("renamed.bin").is_err());
    }
    let mut input = b"dataextra".as_slice();
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    writer
        .add_encoded_file("file", EncodedEntry::new(stored_metadata(4), &mut input))
        .unwrap();
    assert_eq!(input, b"extra");
    assert_eq!(
        Archive::open(Cursor::new(writer.finish().unwrap().into_inner()))
            .unwrap()
            .read_file("file")
            .unwrap(),
        b"data"
    );
}

#[test]
fn copying_all_entries_reproduces_the_archive_bytes() {
    let bytes = archive(b"content", WriteOptions::default(), FileOptions::default());
    let mut source = Archive::open(Cursor::new(&bytes)).unwrap();
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    writer.copy_file_from(&mut source, "entry.bin").unwrap();
    assert_eq!(writer.finish().unwrap().into_inner(), bytes);
}

#[test]
fn metadata_rejections_do_not_poison_the_writer_but_partial_streams_do() {
    for flags in [
        FileFlags::EXISTS | FileFlags::ENCRYPTED,
        FileFlags::EXISTS | FileFlags::FIX_KEY,
        FileFlags::EXISTS | FileFlags::COMPRESS | FileFlags::IMPLODE,
        0,
    ] {
        let mut writer =
            ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
        let mut metadata = stored_metadata(4);
        metadata.flags = FileFlags(flags);
        assert!(writer
            .add_encoded_file("bad", EncodedEntry::new(metadata, b"data".as_slice()))
            .is_err());
        writer
            .add_file("good", 0, &mut b"".as_slice(), FileOptions::default())
            .unwrap();
        writer.finish().unwrap();
    }
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    assert!(matches!(
        writer.add_encoded_file(
            "short",
            EncodedEntry::new(stored_metadata(4), b"abc".as_slice())
        ),
        Err(Error::SizeMismatch)
    ));
    assert!(matches!(writer.finish(), Err(Error::WriterFailed)));
    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    assert!(writer
        .add_encoded_file("broken", EncodedEntry::new(stored_metadata(4), Broken))
        .is_err());
    assert!(matches!(writer.finish(), Err(Error::WriterFailed)));
}

#[test]
fn encoded_import_and_copy_do_not_require_compression_features() {
    // The compression mask is deliberately unsupported; encoded copying never
    // inspects codec bytes. A single unit can move between archive sector sizes.
    let payload = [0xff, 1, 2];
    let metadata = EncodedFileMetadata {
        file_size: 10,
        stored_size: 3,
        flags: FileFlags(FileFlags::EXISTS | FileFlags::COMPRESS | FileFlags::SINGLE_UNIT),
        sector_size: 4096,
        locale: 0,
        platform: 0,
    };
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    writer
        .add_encoded_file("opaque", EncodedEntry::new(metadata, payload.as_slice()))
        .unwrap();
    let mut source = Archive::open(Cursor::new(writer.finish().unwrap().into_inner())).unwrap();
    let mut writer = ArchiveWriter::new(
        Cursor::new(Vec::new()),
        WriteOptions {
            sector_size_shift: 4,
            ..WriteOptions::default()
        },
    )
    .unwrap();
    writer.copy_file_from(&mut source, "opaque").unwrap();
    let mut destination =
        Archive::open(Cursor::new(writer.finish().unwrap().into_inner())).unwrap();
    let mut copied = Vec::new();
    destination
        .open_encoded_file("opaque")
        .unwrap()
        .read_to_end(&mut copied)
        .unwrap();
    assert_eq!(copied, payload);
}

#[cfg(all(feature = "mpq-encode", feature = "mpq-decode"))]
#[test]
fn compressed_copies_preserve_sectors_checksums_and_destination_raw_digests() {
    let data: Vec<_> = (0..17003).map(|i| (i % 7) as u8).collect();
    for compression in [Compression::Zlib, Compression::Bzip2] {
        for version in 0..=3 {
            let options = WriteOptions {
                header_version: version,
                extended_index: version >= 2,
                raw_chunk_size: if version == 3 { 256 } else { 0 },
                ..WriteOptions::default()
            };
            let bytes = archive(
                &data,
                options.clone(),
                FileOptions {
                    compression,
                    sector_checksums: true,
                    ..FileOptions::default()
                },
            );
            let mut source = Archive::open(Cursor::new(&bytes)).unwrap();
            let mut identical =
                ArchiveWriter::new(Cursor::new(Vec::new()), options.clone()).unwrap();
            identical.copy_file_from(&mut source, "entry.bin").unwrap();
            assert_eq!(identical.finish().unwrap().into_inner(), bytes);
            let mut destination = ArchiveWriter::new(
                Cursor::new(Vec::new()),
                WriteOptions {
                    header_version: 3,
                    extended_index: true,
                    raw_chunk_size: 1024,
                    ..WriteOptions::default()
                },
            )
            .unwrap();
            destination
                .add_file("prefix", 3, &mut b"abc".as_slice(), FileOptions::default())
                .unwrap();
            destination
                .copy_file_from(&mut source, "entry.bin")
                .unwrap();
            let mut archive =
                Archive::open(Cursor::new(destination.finish().unwrap().into_inner())).unwrap();
            assert_eq!(archive.read_file("entry.bin").unwrap(), data);
            let mut mismatched = ArchiveWriter::new(
                Cursor::new(Vec::new()),
                WriteOptions {
                    sector_size_shift: 4,
                    ..WriteOptions::default()
                },
            )
            .unwrap();
            assert!(mismatched.copy_file_from(&mut source, "entry.bin").is_err());
            mismatched
                .add_file("valid", 0, &mut b"".as_slice(), FileOptions::default())
                .unwrap();
            mismatched.finish().unwrap();
        }
    }
}

#[test]
fn encrypted_copy_is_rejected_and_raw_source_corruption_is_detected() {
    let bytes = archive(
        b"private",
        WriteOptions::default(),
        FileOptions {
            encrypted: true,
            adjusted_key: true,
            ..FileOptions::default()
        },
    );
    let mut source = Archive::open(Cursor::new(bytes)).unwrap();
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    assert!(matches!(
        writer.copy_file_from(&mut source, "entry.bin"),
        Err(Error::UnsupportedFlags(_))
    ));
    writer.finish().unwrap();
    let mut bytes = archive(
        b"digest",
        WriteOptions {
            header_version: 3,
            raw_chunk_size: 256,
            ..WriteOptions::default()
        },
        FileOptions::default(),
    );
    let source = Archive::open(Cursor::new(&bytes)).unwrap();
    let offset = source.index().blocks[0].offset as usize;
    bytes[offset] ^= 1;
    let mut source = Archive::open(Cursor::new(bytes)).unwrap();
    assert!(source.open_encoded_file("entry.bin").is_err());
}
