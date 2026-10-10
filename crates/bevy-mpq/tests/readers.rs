use bevy_asset::io::{AssetReader, AssetReaderError, Reader};
use bevy_mpq::{MpqAssetReader, OverlayAssetReader, OverlayMount};
use futures_lite::future::block_on;
use std::io::Cursor;
use std::path::Path;
use wc3::mpq::{Archive, ArchiveWriter, Compression, FileOptions, SeekSource, WriteOptions};

fn archive(entries: &[(&str, &[u8], u16)]) -> Vec<u8> {
    let mut writer = ArchiveWriter::new(
        Cursor::new(Vec::new()),
        WriteOptions {
            listfile: false,
            ..Default::default()
        },
    )
    .unwrap();
    for &(name, bytes, locale) in entries {
        writer
            .add_file(
                name,
                bytes.len() as u32,
                &mut &*bytes,
                FileOptions {
                    locale,
                    ..Default::default()
                },
            )
            .unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn mount(bytes: Vec<u8>) -> MpqAssetReader<SeekSource<Cursor<Vec<u8>>>> {
    MpqAssetReader::new(
        Archive::open(Cursor::new(bytes)).unwrap().into_shared(),
        "test archive",
    )
}

fn read(reader: &impl AssetReader, path: &str) -> Result<Vec<u8>, AssetReaderError> {
    block_on(async {
        let mut stream = reader.read(Path::new(path)).await?;
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).await?;
        Ok(bytes)
    })
}

#[test]
fn names_work_without_listfile_and_locale_falls_back_to_neutral() {
    let reader = mount(archive(&[
        ("Textures\\Body.blp", b"neutral", 0),
        ("Textures\\Body.blp", b"localized", 1033),
        ("other", b"fallback", 0),
    ]))
    .with_locale(1033, 0);
    assert_eq!(read(&reader, "textures/body.BLP").unwrap(), b"localized");
    assert_eq!(read(&reader, "other").unwrap(), b"fallback");
    assert!(matches!(
        read(&reader, "missing"),
        Err(AssetReaderError::NotFound(_))
    ));
    assert!(read(&reader, "../other").is_err());
    assert!(read(&reader, "C:\\other").is_err());
    assert!(read(&reader, "\\other").is_err());
    assert!(read(&reader, "foo\\..\\other").is_err());
}

#[test]
fn overlay_respects_order_and_missing_fallback() {
    let reader = OverlayAssetReader::new(vec![
        OverlayMount::mpq(mount(archive(&[("shared", b"map", 0)]))),
        OverlayMount::mpq(mount(archive(&[
            ("shared", b"base", 0),
            ("base-only", b"fallback", 0),
        ]))),
    ]);
    assert_eq!(read(&reader, "shared").unwrap(), b"map");
    assert_eq!(read(&reader, "base-only").unwrap(), b"fallback");
    assert!(matches!(
        read(&reader, "missing"),
        Err(AssetReaderError::NotFound(_))
    ));
}

#[test]
fn payload_errors_do_not_fall_through() {
    let bytes = archive(&[("broken", b"bad", 0)]);
    let index = Archive::open(Cursor::new(bytes.clone())).unwrap();
    let id = index.index().find("broken", 0, 0).unwrap();
    let block = &index.index().blocks[id as usize];
    let offset = block.offset as usize;

    // Use a source that allows indexing then rejects payload reads.
    let reader = MpqAssetReader::new(
        Archive::open(FailingPayload {
            inner: Cursor::new(bytes),
            offset: offset as u64,
        })
        .unwrap()
        .into_shared(),
        "broken.mpq",
    );
    let overlay = OverlayAssetReader::new(vec![
        OverlayMount::mpq(reader),
        OverlayMount::mpq(mount(archive(&[("broken", b"good", 0)]))),
    ]);
    let error = read(&overlay, "broken").unwrap_err();
    assert!(!matches!(error, AssetReaderError::NotFound(_)));
    assert!(error.to_string().contains("broken.mpq"));
}

struct FailingPayload {
    inner: Cursor<Vec<u8>>,
    offset: u64,
}
impl std::io::Read for FailingPayload {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        if self.inner.position() == self.offset {
            return Err(std::io::Error::other("payload unavailable"));
        }
        std::io::Read::read(&mut self.inner, bytes)
    }
}
impl std::io::Seek for FailingPayload {
    fn seek(&mut self, position: std::io::SeekFrom) -> std::io::Result<u64> {
        std::io::Seek::seek(&mut self.inner, position)
    }
}

#[test]
fn overlay_metadata_does_not_come_from_a_shadowed_mount() {
    let reader = OverlayAssetReader::new(vec![
        OverlayMount::mpq(mount(archive(&[("shared", b"map", 0)]))),
        OverlayMount::reader(Box::new(MetadataReader)),
    ]);
    let result = block_on(reader.read_meta(Path::new("shared")));
    assert!(matches!(result, Err(AssetReaderError::NotFound(_))));
}

struct MetadataReader;
impl AssetReader for MetadataReader {
    async fn read<'a>(&'a self, _path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        Ok(bevy_asset::io::VecReader::new(b"lower".to_vec()))
    }
    async fn read_meta<'a>(
        &'a self,
        _path: &'a Path,
    ) -> Result<impl Reader + 'a, AssetReaderError> {
        Ok(bevy_asset::io::VecReader::new(b"lower metadata".to_vec()))
    }
    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<bevy_asset::io::PathStream>, AssetReaderError> {
        Err(AssetReaderError::NotFound(path.to_owned()))
    }
    async fn is_directory<'a>(&'a self, _path: &'a Path) -> Result<bool, AssetReaderError> {
        Ok(false)
    }
}

#[test]
fn compressed_encrypted_entries_are_decoded_on_read() {
    let bytes = vec![42; 12_000];
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    writer
        .add_file(
            "compressed.blp",
            bytes.len() as u32,
            &mut bytes.as_slice(),
            FileOptions {
                compression: Compression::Zlib,
                encrypted: true,
                sector_checksums: true,
                ..Default::default()
            },
        )
        .unwrap();
    let reader = mount(writer.finish().unwrap().into_inner());
    assert_eq!(read(&reader, "compressed.blp").unwrap(), bytes);
}

#[test]
fn indexed_metadata_selection_does_not_read_payloads() {
    let bytes = archive(&[("broken", b"bad", 0)]);
    let indexed = Archive::open(Cursor::new(bytes.clone())).unwrap();
    let offset =
        indexed.index().blocks[indexed.index().find("broken", 0, 0).unwrap() as usize].offset;
    let reader = MpqAssetReader::new(
        Archive::open(FailingPayload {
            inner: Cursor::new(bytes),
            offset,
        })
        .unwrap()
        .into_shared(),
        "broken",
    );
    let overlay = OverlayAssetReader::new(vec![
        OverlayMount::mpq(reader),
        OverlayMount::reader(Box::new(MetadataReader)),
    ]);
    assert!(matches!(
        block_on(overlay.read_meta(Path::new("broken"))),
        Err(AssetReaderError::NotFound(_))
    ));
    assert!(read(&overlay, "broken").is_err());
}

#[test]
fn concurrent_clones_share_extraction_limit() {
    use std::num::NonZeroUsize;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use std::thread;
    use std::time::Duration;
    use wc3::mpq::{ReadAt, SharedArchive};
    struct Source {
        bytes: Arc<[u8]>,
        offset: u64,
        active: AtomicUsize,
        peak: AtomicUsize,
    }
    impl ReadAt for Source {
        fn len(&self) -> std::io::Result<u64> {
            Ok(self.bytes.as_ref().len() as u64)
        }
        fn read_at(&self, output: &mut [u8], offset: u64) -> std::io::Result<usize> {
            if offset == self.offset {
                let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
                self.peak.fetch_max(active, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(20));
                let result = self.bytes.read_at(output, offset);
                self.active.fetch_sub(1, Ordering::SeqCst);
                result
            } else {
                self.bytes.read_at(output, offset)
            }
        }
    }
    let bytes = archive(&[("asset", &[42; 512], 0)]);
    let indexed = Archive::open(Cursor::new(&bytes)).unwrap();
    let offset =
        indexed.index().blocks[indexed.index().find("asset", 0, 0).unwrap() as usize].offset;
    for limit in [1, 2] {
        let source = Arc::new(Source {
            bytes: bytes.clone().into(),
            offset,
            active: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
        });
        let reader =
            MpqAssetReader::new(SharedArchive::open(source.clone()).unwrap(), "concurrent")
                .with_max_concurrent_reads(NonZeroUsize::new(limit).unwrap());
        thread::scope(|scope| {
            for _ in 0..8 {
                let reader = reader.clone();
                scope.spawn(move || assert_eq!(read(&reader, "asset").unwrap(), [42; 512]));
            }
        });
        assert_eq!(source.peak.load(Ordering::SeqCst), limit);
    }
}
