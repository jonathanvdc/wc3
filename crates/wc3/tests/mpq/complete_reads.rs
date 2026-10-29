use std::io::{Cursor, Read};
use std::sync::Arc;
use wc3::mpq::{
    Archive, ArchiveWriter, Compression, EncodedEntry, EncodedFileMetadata, Error, FileFlags,
    FileOptions, ReadOptions, SharedArchive, WriteOptions,
};

fn archive(data: &[u8], compression: Compression, single: bool) -> Vec<u8> {
    let mut writer = ArchiveWriter::new(
        Cursor::new(Vec::new()),
        WriteOptions {
            sector_size_shift: 0,
            listfile: false,
            ..Default::default()
        },
    )
    .unwrap();
    if single {
        writer
            .add_encoded_file(
                "entry",
                EncodedEntry::new(
                    EncodedFileMetadata {
                        file_size: data.len() as u32,
                        stored_size: data.len() as u32,
                        flags: FileFlags(FileFlags::EXISTS | FileFlags::SINGLE_UNIT),
                        sector_size: 512,
                        locale: 0,
                        platform: 0,
                    },
                    data,
                ),
            )
            .unwrap();
    } else {
        writer
            .add_file(
                "entry",
                data.len() as u32,
                &mut &*data,
                FileOptions {
                    compression,
                    encrypted: true,
                    adjusted_key: true,
                    sector_checksums: compression != Compression::Stored,
                    ..Default::default()
                },
            )
            .unwrap();
    }
    writer.finish().unwrap().into_inner()
}

#[test]
fn consuming_read_returns_only_unread_bytes_across_unit_boundaries() {
    let formats = vec![(Compression::Stored, false), (Compression::Stored, true)];
    #[cfg(all(feature = "mpq-decode", feature = "mpq-encode"))]
    let formats = [
        formats,
        vec![(Compression::Zlib, false), (Compression::Bzip2, false)],
    ]
    .concat();
    for (compression, single) in formats {
        for size in [0, 1, 511, 512, 513, 4096, 17003] {
            let data: Vec<_> = (0..size).map(|i| (i % 251) as u8).collect();
            let bytes = archive(&data, compression, single);
            let mut serial = Archive::open(Cursor::new(&bytes)).unwrap();
            let shared = SharedArchive::open(Arc::<[u8]>::from(bytes.clone())).unwrap();
            for prefix in [0, 1.min(size), 512.min(size), 513.min(size), size] {
                let mut entry = serial.open_file("ENTRY").unwrap();
                let mut consumed = vec![0; prefix];
                entry.read_exact(&mut consumed).unwrap();
                assert_eq!(consumed, data[..prefix]);
                assert_eq!(entry.into_bytes().unwrap(), data[prefix..]);
                let mut entry = shared.open_file("entry").unwrap();
                entry.read_exact(&mut consumed).unwrap();
                assert_eq!(entry.into_bytes().unwrap(), data[prefix..]);
            }
            assert_eq!(serial.read_file("entry").unwrap(), data);
            assert_eq!(shared.read_file("entry").unwrap(), data);
        }
    }
}

#[test]
fn complete_reads_preserve_missing_name_and_size_limit_errors() {
    let bytes = archive(&[42; 513], Compression::Stored, false);
    let options = ReadOptions {
        max_file_size: 512,
        ..Default::default()
    };
    let mut serial = Archive::with_options(Cursor::new(&bytes), options.clone()).unwrap();
    let shared = SharedArchive::with_options(Arc::<[u8]>::from(bytes.clone()), options).unwrap();
    assert!(matches!(
        serial.read_file("entry"),
        Err(Error::LimitExceeded("file size"))
    ));
    assert!(matches!(
        shared.read_file("entry"),
        Err(Error::LimitExceeded("file size"))
    ));
    assert!(matches!(
        serial.read_file("missing"),
        Err(Error::FileNotFound)
    ));
    assert!(matches!(
        shared.read_file("missing"),
        Err(Error::FileNotFound)
    ));
    assert!(matches!(
        serial.read_file(b"bad\0name"),
        Err(Error::InvalidName)
    ));
    assert!(matches!(
        shared.read_file(b"bad\0name"),
        Err(Error::InvalidName)
    ));
}

#[cfg(all(feature = "mpq-decode", feature = "mpq-encode"))]
#[test]
fn consuming_read_rejects_corruption_after_valid_earlier_sectors() {
    let data = vec![42; 4096];
    let mut writer = ArchiveWriter::new(
        Cursor::new(Vec::new()),
        WriteOptions {
            sector_size_shift: 0,
            ..Default::default()
        },
    )
    .unwrap();
    writer
        .add_file(
            "entry",
            data.len() as u32,
            &mut data.as_slice(),
            FileOptions {
                compression: Compression::Zlib,
                sector_checksums: true,
                ..Default::default()
            },
        )
        .unwrap();
    let mut bytes = writer.finish().unwrap().into_inner();
    let archive = Archive::open(Cursor::new(&bytes)).unwrap();
    let block = archive.index().blocks[archive.index().find("entry", 0, 0).unwrap() as usize];
    let at = block.offset as usize;
    let second = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()) as usize;
    bytes[at + second + 1] ^= 1;
    let mut serial = Archive::open(Cursor::new(&bytes)).unwrap();
    let shared = SharedArchive::open(Arc::<[u8]>::from(bytes.clone())).unwrap();
    assert!(matches!(
        serial.open_file("entry").unwrap().into_bytes(),
        Err(Error::ChecksumMismatch(1))
    ));
    assert!(matches!(
        shared.open_file("entry").unwrap().into_bytes(),
        Err(Error::ChecksumMismatch(1))
    ));
}
