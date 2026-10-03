use bevy_asset::io::{AssetReader, AssetReaderError, Reader};
use bevy_mpq::{MpqAssetReader, OverlayAssetReader};
use futures_lite::future::block_on;
use std::io::Cursor;
use std::path::Path;
use wc3::mpq::{Archive, ArchiveWriter, Compression, FileOptions, WriteOptions};

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

fn mount(bytes: Vec<u8>) -> MpqAssetReader<Cursor<Vec<u8>>> {
    MpqAssetReader::new(Archive::open(Cursor::new(bytes)).unwrap(), "test archive")
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
        Box::new(mount(archive(&[("shared", b"map", 0)]))),
        Box::new(mount(archive(&[
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
        .unwrap(),
        "broken.mpq",
    );
    let overlay = OverlayAssetReader::new(vec![
        Box::new(reader),
        Box::new(mount(archive(&[("broken", b"good", 0)]))),
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
        Box::new(mount(archive(&[("shared", b"map", 0)]))),
        Box::new(MetadataReader),
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
