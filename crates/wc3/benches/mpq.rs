use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{
    env,
    fs::{self, File},
    hint::black_box,
    io::{Cursor, Read},
    path::PathBuf,
    process::id,
    sync::Arc,
};
use wc3::mpq::{
    Archive, ArchiveWriter, Compression, EncodedEntry, EncodedFileMetadata, FileFlags,
    SharedArchive, WriteOptions,
};

#[path = "../../../benchmarks/support/archive.rs"]
mod fixtures;

// The fixed codec inputs are shared with the codec's correctness tests.
#[path = "../src/mpq/codec_test_vectors.rs"]
mod vectors;

fn single_unit(payload: &[u8], decoded_size: usize, flags: u32) -> Vec<u8> {
    let metadata = EncodedFileMetadata {
        file_size: decoded_size as u32,
        stored_size: payload.len() as u32,
        flags: FileFlags(FileFlags::EXISTS | FileFlags::SINGLE_UNIT | flags),
        sector_size: 4096,
        locale: 0,
        platform: 0,
    };
    let mut writer = ArchiveWriter::new(
        Cursor::new(Vec::new()),
        WriteOptions {
            listfile: false,
            ..Default::default()
        },
    )
    .unwrap();
    writer
        .add_encoded_file("entry", EncodedEntry::new(metadata, payload))
        .unwrap();
    writer.finish().unwrap().into_inner()
}

struct TemporaryArchive(PathBuf);
impl Drop for TemporaryArchive {
    fn drop(&mut self) {
        fs::remove_file(&self.0).unwrap();
    }
}

fn complete_reads(c: &mut Criterion) {
    let payload = fixtures::payload(1 << 20, false);
    let mut group = c.benchmark_group("complete-read");
    group.throughput(Throughput::Bytes(payload.len() as u64));
    for (name, bytes) in [
        ("single-stored", single_unit(&payload, payload.len(), 0)),
        (
            "sector-stored",
            fixtures::archive(&payload, 1, Compression::Stored, false, 0),
        ),
        (
            "sector-zlib",
            fixtures::archive(&payload, 1, Compression::Zlib, true, 0),
        ),
    ] {
        let archive = SharedArchive::open(Arc::<[u8]>::from(bytes)).unwrap();
        let entry = if name == "single-stored" {
            "entry"
        } else {
            "entry-0"
        };
        assert_eq!(archive.read_file(entry).unwrap(), payload);
        group.bench_function(format!("{name}/stream"), |b| {
            b.iter(|| {
                let mut bytes = Vec::new();
                archive
                    .open_file(entry)
                    .unwrap()
                    .read_to_end(&mut bytes)
                    .unwrap();
                black_box(bytes)
            })
        });
        group.bench_function(format!("{name}/consume"), |b| {
            b.iter(|| black_box(archive.open_file(entry).unwrap().into_bytes().unwrap()))
        });
    }
    group.finish();
}

fn archives(c: &mut Criterion) {
    let mut group = c.benchmark_group("mpq");
    let quick = env::var_os("WC3_BENCH_QUICK").is_some();
    let sizes = if quick {
        vec![65536]
    } else if env::var_os("WC3_BENCH_LARGE").is_some() {
        vec![1024, 65536, 1048576]
    } else {
        vec![1024, 65536]
    };
    for size in sizes {
        for noisy in if quick {
            vec![false]
        } else {
            vec![false, true]
        } {
            let payload = fixtures::payload(size, noisy);
            for (compression_name, compression) in [
                ("stored", Compression::Stored),
                ("zlib", Compression::Zlib),
                ("bzip2", Compression::Bzip2),
            ] {
                for encrypted in if quick { vec![true] } else { vec![false, true] } {
                    let case = format!(
                        "{compression_name}/{}/{}/{}",
                        if encrypted { "encrypted" } else { "plain" },
                        if noisy { "noise" } else { "structured" },
                        size
                    );
                    let bytes: Arc<[u8]> =
                        fixtures::archive(&payload, 8, compression, encrypted, 0).into();
                    let shared = SharedArchive::open(bytes.clone()).unwrap();
                    let mut serial = Archive::open(Cursor::new(bytes.as_ref())).unwrap();
                    assert_eq!(shared.read_file("entry-3").unwrap(), payload);
                    assert_eq!(serial.read_file("entry-3").unwrap(), payload);
                    group.throughput(Throughput::Bytes(size as u64));
                    group.bench_function(format!("extract/serial/{case}"), |b| {
                        b.iter(|| black_box(serial.read_file(black_box("entry-3")).unwrap()))
                    });
                    group.bench_function(format!("extract/shared/{case}"), |b| {
                        b.iter(|| black_box(shared.read_file(black_box("entry-3")).unwrap()))
                    });
                    group.throughput(Throughput::Bytes((size * 8) as u64));
                    group.bench_function(format!("write/{case}"), |b| {
                        b.iter(|| {
                            black_box(fixtures::archive(
                                black_box(&payload),
                                8,
                                compression,
                                encrypted,
                                0,
                            ))
                        })
                    });
                }
            }
        }
    }
    let payload = fixtures::payload(4096, false);
    for version in if quick { vec![0, 3] } else { vec![0, 1, 2, 3] } {
        let bytes: Arc<[u8]> =
            fixtures::archive(&payload, 128, Compression::Zlib, true, version).into();
        let archive = SharedArchive::open(bytes.clone()).unwrap();
        assert_eq!(archive.read_file("entry-127").unwrap(), payload);
        group.throughput(Throughput::Elements(128));
        group.bench_function(BenchmarkId::new("open/serial", version), |b| {
            b.iter(|| black_box(Archive::open(Cursor::new(bytes.as_ref())).unwrap()))
        });
        group.bench_function(BenchmarkId::new("open/shared", version), |b| {
            b.iter(|| black_box(SharedArchive::open(bytes.clone()).unwrap()))
        });
        group.throughput(Throughput::Elements(1));
        for name in ["entry-0", "entry-127", "missing"] {
            group.bench_function(BenchmarkId::new(format!("lookup/{name}"), version), |b| {
                b.iter(|| black_box(archive.index().find(black_box(name), 0, 0)))
            });
        }
    }
    // Tiny known streams isolate all decode-only compression paths through the public API.
    let mut streams = vec![
        (
            "pkware-implode".to_owned(),
            vec![0, 4, 0x82, 0x24, 0x25, 0x8f, 0x80, 0x7f],
            b"AIAIAIAIAIAIA".to_vec(),
            FileFlags::IMPLODE,
        ),
        (
            "pkware-mask".to_owned(),
            vec![8, 0, 4, 0x82, 0x24, 0x25, 0x8f, 0x80, 0x7f],
            b"AIAIAIAIAIAIA".to_vec(),
            FileFlags::COMPRESS,
        ),
        (
            "sparse".to_owned(),
            vec![32, 0, 0, 0, 128, 125],
            vec![0; 128],
            FileFlags::COMPRESS,
        ),
        (
            "sparse-zlib".to_owned(),
            vec![34, 120, 156, 99, 96, 96, 104, 168, 5, 0, 1, 130, 0, 254],
            vec![0; 128],
            FileFlags::COMPRESS,
        ),
    ];
    for (name, mask, encoded, expected) in [
        ("huffman-zero", 1, vectors::HUFFMAN_ZEROS, &[0; 64][..]),
        ("adpcm-mono", 0x40, vectors::MONO, vectors::MONO_PCM),
        ("adpcm-stereo", 0x80, vectors::STEREO, vectors::STEREO_PCM),
        (
            "huffman-adpcm-mono",
            0x41,
            vectors::MONO_CHAIN,
            vectors::MONO_PCM,
        ),
        (
            "huffman-adpcm-stereo",
            0x81,
            vectors::STEREO_CHAIN,
            vectors::STEREO_PCM,
        ),
        ("lzma-eos", 0x12, vectors::LZMA_EOS, vectors::RAW),
        ("lzma-sized", 0x12, vectors::LZMA_SIZED, vectors::RAW),
    ] {
        let mut payload = vec![mask];
        payload.extend_from_slice(encoded);
        streams.push((
            name.to_owned(),
            payload,
            expected.to_vec(),
            FileFlags::COMPRESS,
        ));
    }
    for (distribution, encoded) in vectors::HUFFMAN.iter().enumerate() {
        let mut payload = vec![1];
        payload.extend_from_slice(encoded);
        // MPQ stores streams that would grow as raw bytes instead.
        if payload.len() < vectors::RAW.len() {
            streams.push((
                format!("huffman-{distribution}"),
                payload,
                vectors::RAW.to_vec(),
                FileFlags::COMPRESS,
            ));
        }
    }
    for (name, encoded, expected, flags) in streams {
        let bytes: Arc<[u8]> = single_unit(&encoded, expected.len(), flags).into();
        let archive = SharedArchive::open(bytes).unwrap();
        assert_eq!(archive.read_file("entry").unwrap(), expected);
        group.throughput(Throughput::Bytes(expected.len() as u64));
        group.bench_function(format!("single-unit/{name}"), |b| {
            b.iter(|| black_box(archive.read_file("entry").unwrap()))
        });
    }
    let bytes = fixtures::archive(
        &fixtures::payload(65536, false),
        16,
        Compression::Zlib,
        true,
        0,
    );
    let file = TemporaryArchive(env::temp_dir().join(format!("wc3-mpq-criterion-{}.mpq", id())));
    fs::write(&file.0, bytes).unwrap();
    let file_archive = SharedArchive::open(File::open(&file.0).unwrap()).unwrap();
    assert_eq!(
        file_archive.read_file("entry-0").unwrap(),
        fixtures::payload(65536, false)
    );
    group.throughput(Throughput::Bytes(65536));
    group.bench_function("file/warm-cache", |b| {
        b.iter(|| black_box(file_archive.read_file("entry-0").unwrap()))
    });
    // Independent StormLib fixtures exercise extended tables and raw chunk digests.
    for (name, bytes) in [
        (
            "storm-v2",
            include_bytes!("../tests/fixtures/mpq/storm-v2.mpq").as_slice(),
        ),
        (
            "storm-v3",
            include_bytes!("../tests/fixtures/mpq/storm-v3.mpq").as_slice(),
        ),
        (
            "storm-v4",
            include_bytes!("../tests/fixtures/mpq/storm-v4.mpq").as_slice(),
        ),
    ] {
        let mut archive = Archive::open(Cursor::new(bytes)).unwrap();
        let names = [b"Units\\Encrypted.bin".to_vec(), b"stored.bin".to_vec()];
        for entry in names {
            let expected = archive.read_file(&entry).unwrap();
            group.throughput(Throughput::Bytes(expected.len() as u64));
            group.bench_function(
                format!("compat/{name}/{}", String::from_utf8_lossy(&entry)),
                |b| b.iter(|| black_box(archive.read_file(black_box(&entry)).unwrap())),
            );
        }
    }
    group.finish();
}

criterion_group!(benches, archives, complete_reads);
criterion_main!(benches);
