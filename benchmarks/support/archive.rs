use std::io::Cursor;
use wc3::mpq::{ArchiveWriter, Compression, FileOptions, WriteOptions};

pub fn payload(size: usize, noisy: bool) -> Vec<u8> {
    let mut state = 0x91e1_0da5_u32;
    (0..size)
        .map(|i| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            if noisy {
                state as u8
            } else {
                ((i / 32 + i % 7) % 64) as u8
            }
        })
        .collect()
}

pub fn archive(
    data: &[u8],
    entries: usize,
    compression: Compression,
    encrypted: bool,
    version: u16,
) -> Vec<u8> {
    let mut writer = ArchiveWriter::new(
        Cursor::new(Vec::new()),
        WriteOptions {
            header_version: version,
            extended_index: version >= 2,
            listfile: false,
            ..Default::default()
        },
    )
    .unwrap();
    for i in 0..entries {
        let mut input = data;
        writer
            .add_file(
                format!("entry-{i}"),
                data.len() as u32,
                &mut input,
                FileOptions {
                    compression,
                    encrypted,
                    adjusted_key: encrypted,
                    sector_checksums: compression != Compression::Stored,
                    ..Default::default()
                },
            )
            .unwrap();
    }
    writer.finish().unwrap().into_inner()
}
