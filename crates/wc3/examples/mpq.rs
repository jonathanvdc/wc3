//! Streaming archive inspection, extraction, creation, and copy-and-append editing.
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, ErrorKind, Seek, Write};
use std::path::Path;

use wc3::mpq::{Archive, ArchiveWriter, Compression, FileOptions, WriteOptions};

fn usage() -> io::Error {
    io::Error::new(ErrorKind::InvalidInput,
        "usage: mpq list ARCHIVE | extract ARCHIVE NAME OUTPUT | create OUTPUT stored|zlib|bzip2 plain|encrypted|adjusted NAME=PATH... | edit ARCHIVE OUTPUT NAME=PATH...")
}

fn add<W: Write + Seek>(
    writer: &mut ArchiveWriter<W>,
    specification: &str,
    options: FileOptions,
    replace: bool,
) -> Result<(), Box<dyn Error>> {
    let (name, path) = specification.split_once('=').ok_or_else(usage)?;
    let mut source = File::open(path)?;
    let size = source.metadata()?.len().try_into()?;
    if replace {
        writer.replace_file(name, size, &mut source, options)?;
    } else {
        writer.add_file(name, size, &mut source, options)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("list") if args.len() == 2 => {
            let mut archive = Archive::open(File::open(&args[1])?)?;
            for name in archive.known_names()? {
                println!("{}", String::from_utf8_lossy(&name));
            }
        }
        Some("extract") if args.len() == 4 => {
            let mut archive = Archive::open(File::open(&args[1])?)?;
            let mut entry = archive.open_file(&args[2])?;
            // Avoid implicit extraction paths from untrusted archive filenames.
            let mut output = File::options()
                .write(true)
                .create_new(true)
                .open(&args[3])?;
            io::copy(&mut entry, &mut output)?;
        }
        Some("create") if args.len() >= 4 => {
            let compression = match args[2].as_str() {
                "stored" => Compression::Stored,
                "zlib" => Compression::Zlib,
                "bzip2" => Compression::Bzip2,
                _ => return Err(usage().into()),
            };
            let (encrypted, adjusted_key) = match args[3].as_str() {
                "plain" => (false, false),
                "encrypted" => (true, false),
                "adjusted" => (true, true),
                _ => return Err(usage().into()),
            };
            let output = File::options()
                .write(true)
                .create_new(true)
                .open(&args[1])?;
            let mut writer = ArchiveWriter::new(output, WriteOptions::default())?;
            let options = FileOptions {
                compression,
                encrypted,
                adjusted_key,
                sector_checksums: compression != Compression::Stored,
                ..FileOptions::default()
            };
            for specification in &args[4..] {
                add(&mut writer, specification, options, false)?;
            }
            writer.finish()?;
        }
        Some("edit") if args.len() >= 3 => {
            if Path::new(&args[1]) == Path::new(&args[2]) {
                return Err(usage().into());
            }
            let mut archive = Archive::open(File::open(&args[1])?)?;
            let output = File::options()
                .write(true)
                .create_new(true)
                .open(&args[2])?;
            let mut writer = ArchiveWriter::from_archive(output, &mut archive)?;
            for specification in &args[3..] {
                add(&mut writer, specification, FileOptions::default(), true)?;
            }
            writer.finish()?;
        }
        _ => return Err(usage().into()),
    }
    Ok(())
}
