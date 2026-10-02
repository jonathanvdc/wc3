use std::collections::BTreeMap;
use std::io::{self, Read, Seek, SeekFrom, Write};

use super::{ArchiveWriter, FileOptions, WriteOptions};
use crate::mpq::{Archive, Error};

#[derive(Default)]
struct Sparse {
    bytes: BTreeMap<u64, u8>,
    position: u64,
    len: u64,
}
impl Write for Sparse {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        for &byte in bytes {
            self.bytes.insert(self.position, byte);
            self.position += 1;
        }
        self.len = self.len.max(self.position);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Read for Sparse {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let n = (self.len - self.position.min(self.len)).min(bytes.len() as u64) as usize;
        for byte in &mut bytes[..n] {
            *byte = self.bytes.get(&self.position).copied().unwrap_or(0);
            self.position += 1;
        }
        Ok(n)
    }
}
impl Seek for Sparse {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        self.position = match from {
            SeekFrom::Start(n) => n,
            SeekFrom::Current(n) => self.position.checked_add_signed(n).unwrap(),
            SeekFrom::End(n) => self.len.checked_add_signed(n).unwrap(),
        };
        Ok(self.position)
    }
}

#[test]
fn large_offsets_tables_and_adjusted_keys() {
    for version in 1..=3 {
        let mut writer = ArchiveWriter::new(
            Sparse::default(),
            WriteOptions {
                header_version: version,
                extended_index: version >= 2,
                listfile: false,
                raw_chunk_size: if version == 3 { 4 } else { 0 },
                ..WriteOptions::default()
            },
        )
        .unwrap();
        let offset = (1u64 << 32) + 17;
        writer.output.seek(SeekFrom::Start(offset)).unwrap();
        writer
            .add_file(
                "data",
                7,
                &mut b"payload".as_slice(),
                FileOptions {
                    encrypted: true,
                    adjusted_key: true,
                    ..FileOptions::default()
                },
            )
            .unwrap();
        let mut source = writer.finish().unwrap();
        source.seek(SeekFrom::Start(0)).unwrap();
        let mut archive = Archive::open(source).unwrap();
        let header = &archive.index().header;
        assert!(header.hash_table_offset > u32::MAX as u64);
        assert!(header.block_table_offset > u32::MAX as u64);
        assert!(header.hi_block_table_offset > u32::MAX as u64);
        assert_eq!(archive.index().blocks[0].offset, offset);
        assert_eq!(archive.read_file("data").unwrap(), b"payload");
    }
    let mut writer = ArchiveWriter::new(Sparse::default(), WriteOptions::default()).unwrap();
    writer.output.seek(SeekFrom::Start(1u64 << 32)).unwrap();
    assert!(matches!(
        writer.start_file("data", 7, FileOptions::default()),
        Err(Error::LimitExceeded(_))
    ));
}
