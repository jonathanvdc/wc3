use std::io::{self, Cursor, Read};
use std::sync::{Arc, Barrier};
use std::thread;
use wc3::mpq::{
    Archive, ArchiveWriter, FileOptions, ReadAt, ReadOptions, SharedArchive, WriteOptions,
};

fn archive() -> Vec<u8> {
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    for (name, byte) in [("a", 17), ("b", 93)] {
        let bytes = vec![byte; 32_000];
        writer
            .add_file(
                name,
                bytes.len() as u32,
                &mut bytes.as_slice(),
                FileOptions::default(),
            )
            .unwrap();
    }
    writer.finish().unwrap().into_inner()
}

struct OverlappingSource {
    bytes: Arc<[u8]>,
    offsets: [u64; 2],
    barrier: Barrier,
}
impl ReadAt for OverlappingSource {
    fn len(&self) -> io::Result<u64> {
        Ok(self.bytes.as_ref().len() as u64)
    }
    fn read_at(&self, output: &mut [u8], offset: u64) -> io::Result<usize> {
        // Both payload reads must enter concurrently. A shared cursor lock would deadlock.
        if self.offsets.contains(&offset) {
            self.barrier.wait();
        }
        self.bytes.read_at(output, offset)
    }
}
#[test]
fn payload_reads_overlap_and_index_is_shared() {
    let bytes = archive();
    let indexed = Archive::open(Cursor::new(&bytes)).unwrap();
    let offsets = ["a", "b"].map(|name| {
        indexed.index().blocks[indexed.index().find(name, 0, 0).unwrap() as usize].offset
    });
    let shared = SharedArchive::open(OverlappingSource {
        bytes: bytes.into(),
        offsets,
        barrier: Barrier::new(2),
    })
    .unwrap();
    assert!(std::ptr::eq(shared.index(), shared.clone().index()));
    thread::scope(|scope| {
        let a = scope.spawn(|| shared.read_file("a").unwrap());
        let b = scope.spawn(|| shared.read_file("b").unwrap());
        assert_eq!(a.join().unwrap(), vec![17; 32_000]);
        assert_eq!(b.join().unwrap(), vec![93; 32_000]);
    });
}
#[test]
fn independent_streams_survive_dropping_archive() {
    let shared = SharedArchive::open(Arc::<[u8]>::from(archive())).unwrap();
    let mut a = shared.open_file("a").unwrap();
    let mut b = shared.open_file("b").unwrap();
    drop(shared);
    let mut output = [0; 137];
    for _ in 0..20 {
        a.read_exact(&mut output).unwrap();
        assert_eq!(output, [17; 137]);
        b.read_exact(&mut output).unwrap();
        assert_eq!(output, [93; 137]);
    }
}
#[test]
fn seekable_conversion_preserves_index_and_limits() {
    let archive = Archive::with_options(
        Cursor::new(archive()),
        ReadOptions {
            max_file_size: 100,
            ..Default::default()
        },
    )
    .unwrap();
    let pointer = archive.index() as *const _;
    let shared = archive.into_shared();
    assert_eq!(pointer, shared.index() as *const _);
    assert!(shared.open_file("a").is_err());
}
#[cfg(all(feature = "mpq-encode", feature = "mpq-decode"))]
#[test]
fn encrypted_compressed_entries_and_checksum_failure_are_independent() {
    use wc3::mpq::Compression;
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default()).unwrap();
    let data: Vec<u8> = (0..32_000).map(|i| (i % 251) as u8).collect();
    for name in ["a", "b"] {
        writer
            .add_file(
                name,
                data.len() as u32,
                &mut data.as_slice(),
                FileOptions {
                    compression: Compression::Zlib,
                    encrypted: true,
                    sector_checksums: true,
                    ..Default::default()
                },
            )
            .unwrap();
    }
    let bytes = writer.finish().unwrap().into_inner();
    let shared = SharedArchive::open(Arc::<[u8]>::from(bytes.clone())).unwrap();
    thread::scope(|scope| {
        for name in ["a", "b"] {
            let shared = &shared;
            let data = &data;
            scope.spawn(move || assert_eq!(shared.read_file(name).unwrap(), *data));
        }
    });
    let id = shared.index().find("a", 0, 0).unwrap() as usize;
    let block = shared.index().blocks[id];
    // Mutate encoded payload, preserving the encrypted offset table.
    let mut broken = bytes;
    broken[block.offset as usize + 128] ^= 1;
    let shared = SharedArchive::open(Arc::<[u8]>::from(broken)).unwrap();
    assert!(shared.read_file("a").is_err());
    assert_eq!(shared.read_file("b").unwrap(), data);
}
