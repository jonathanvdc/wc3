//! Streaming archive inspection, extraction, creation, and copy-and-append editing.
use clap::{Parser, Subcommand, ValueEnum};
use std::error::Error;
use std::fs::File;
use std::io::{self, Seek, Write};
use std::path::PathBuf;
use std::str::FromStr;

use wc3::mpq::{Archive, ArchiveWriter, Compression, FileOptions, WriteOptions};

#[derive(Parser)]
#[command(about = "List, extract, create, and edit MPQ archives")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List known archive entry names.
    List { archive: PathBuf },
    /// Extract one entry to a new file.
    Extract {
        archive: PathBuf,
        name: String,
        output: PathBuf,
    },
    /// Create an archive, optionally adding NAME=PATH entries.
    Create {
        output: PathBuf,
        #[arg(value_enum)]
        compression: CompressionArg,
        #[arg(value_enum)]
        encryption: Encryption,
        #[arg(value_name = "NAME=PATH")]
        entries: Vec<Entry>,
    },
    /// Copy an archive and add or replace NAME=PATH entries.
    Edit {
        archive: PathBuf,
        output: PathBuf,
        #[arg(value_name = "NAME=PATH")]
        entries: Vec<Entry>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum CompressionArg {
    Stored,
    Zlib,
    Bzip2,
}

#[derive(Clone, Copy, ValueEnum)]
enum Encryption {
    Plain,
    Encrypted,
    Adjusted,
}

#[derive(Clone, Debug)]
struct Entry {
    name: String,
    path: PathBuf,
}

impl FromStr for Entry {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (name, path) = value.split_once('=').ok_or("expected NAME=PATH")?;
        if name.is_empty() || path.is_empty() {
            return Err("NAME and PATH must both be nonempty".into());
        }
        Ok(Self {
            name: name.into(),
            path: path.into(),
        })
    }
}

fn add<W: Write + Seek>(
    writer: &mut ArchiveWriter<W>,
    entry: &Entry,
    options: FileOptions,
    replace: bool,
) -> Result<(), Box<dyn Error>> {
    let mut source = File::open(&entry.path)?;
    let size = source.metadata()?.len().try_into()?;
    if replace {
        writer.replace_file(&entry.name, size, &mut source, options)?;
    } else {
        writer.add_file(&entry.name, size, &mut source, options)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    match Args::parse().command {
        Command::List { archive } => {
            let mut archive = Archive::open(File::open(archive)?)?;
            for name in archive.known_names()? {
                println!("{}", String::from_utf8_lossy(&name));
            }
        }
        Command::Extract {
            archive,
            name,
            output,
        } => {
            let mut archive = Archive::open(File::open(archive)?)?;
            let mut entry = archive.open_file(&name)?;
            // Avoid implicit extraction paths from untrusted archive filenames.
            let mut output = File::options().write(true).create_new(true).open(output)?;
            io::copy(&mut entry, &mut output)?;
        }
        Command::Create {
            output,
            compression,
            encryption,
            entries,
        } => {
            let compression = match compression {
                CompressionArg::Stored => Compression::Stored,
                CompressionArg::Zlib => Compression::Zlib,
                CompressionArg::Bzip2 => Compression::Bzip2,
            };
            let (encrypted, adjusted_key) = match encryption {
                Encryption::Plain => (false, false),
                Encryption::Encrypted => (true, false),
                Encryption::Adjusted => (true, true),
            };
            let output = File::options().write(true).create_new(true).open(output)?;
            let mut writer = ArchiveWriter::new(output, WriteOptions::default())?;
            let options = FileOptions {
                compression,
                encrypted,
                adjusted_key,
                sector_checksums: compression != Compression::Stored,
                ..FileOptions::default()
            };
            for entry in &entries {
                add(&mut writer, entry, options, false)?;
            }
            writer.finish()?;
        }
        Command::Edit {
            archive,
            output,
            entries,
        } => {
            if archive == output {
                return Err("input and output archive paths must differ".into());
            }
            let mut archive = Archive::open(File::open(archive)?)?;
            let output = File::options().write(true).create_new(true).open(output)?;
            let mut writer = ArchiveWriter::from_archive(output, &mut archive)?;
            for entry in &entries {
                add(&mut writer, entry, FileOptions::default(), true)?;
            }
            writer.finish()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_paths_can_contain_equals() {
        let entry: Entry = "units\\model.mdx=/tmp/model=variant.mdx".parse().unwrap();
        assert_eq!(entry.name, "units\\model.mdx");
        assert_eq!(entry.path, PathBuf::from("/tmp/model=variant.mdx"));
    }

    #[test]
    fn malformed_entries_are_rejected_before_archive_creation() {
        for value in ["missing-separator", "=path", "name="] {
            assert!(
                Args::try_parse_from(["mpq", "create", "out.mpq", "stored", "plain", value])
                    .is_err()
            );
        }
    }
}
