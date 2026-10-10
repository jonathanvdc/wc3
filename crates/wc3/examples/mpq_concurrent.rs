//! Compare serialized extraction with concurrent positional file reads.
use std::fs::{self, File};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use wc3::mpq::{Archive, ArchiveWriter, Compression, FileOptions, SharedArchive, WriteOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::temp_dir().join(format!("wc3-mpq-benchmark-{}.mpq", std::process::id()));
    let mut writer = ArchiveWriter::new(
        Cursor::new(Vec::new()),
        WriteOptions {
            listfile: false,
            ..Default::default()
        },
    )?;
    let payload: Vec<u8> = (0..4 * 1024 * 1024)
        .map(|i| ((i * 17 + i / 251) % 256) as u8)
        .collect();
    for id in 0..16 {
        writer.add_file(
            format!("entry-{id}"),
            payload.len() as u32,
            &mut payload.as_slice(),
            FileOptions {
                compression: Compression::Zlib,
                encrypted: true,
                sector_checksums: true,
                ..Default::default()
            },
        )?;
    }
    fs::write(&path, writer.finish()?.into_inner())?;
    let serial = Arc::new(Mutex::new(Archive::open(File::open(&path)?)?));
    let shared = SharedArchive::open(File::open(&path)?)?;
    // Warm the filesystem cache, then verify every decoded result in both paths.
    assert_eq!(shared.read_file("entry-0")?, payload);
    for workers in [1, 2, 4, 8] {
        for parallel in [false, true] {
            let start = Instant::now();
            thread::scope(|scope| {
                for worker in 0..workers {
                    let serial = &serial;
                    let shared = &shared;
                    let payload = &payload;
                    scope.spawn(move || {
                        for id in (worker..64).step_by(workers) {
                            let name = format!("entry-{}", id % 16);
                            let bytes = if parallel {
                                shared.read_file(name).unwrap()
                            } else {
                                serial.lock().unwrap().read_file(name).unwrap()
                            };
                            assert_eq!(bytes, *payload);
                        }
                    });
                }
            });
            println!(
                "{} workers={workers}: {:.1} MiB/s ({:.3}s)",
                if parallel { "positional" } else { "serialized" },
                256.0 / start.elapsed().as_secs_f64(),
                start.elapsed().as_secs_f64()
            );
        }
    }
    fs::remove_file(path)?;
    Ok(())
}
