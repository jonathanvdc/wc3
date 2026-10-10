#[cfg(any(unix, windows))]
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};

use super::reader::EntryReader;
use super::{Archive, BlockEntry, Error, Index, ReadOptions, RecoveryDiagnostic};

/// An immutable random-access source. Implementations must allow overlapping reads
/// without changing a shared cursor. The contents must remain stable while mounted.
pub trait ReadAt: Send + Sync {
    /// Returns the source length.
    fn len(&self) -> io::Result<u64>;
    /// Returns whether the source is empty.
    fn is_empty(&self) -> io::Result<bool> {
        Ok(self.len()? == 0)
    }
    /// Reads up to `output.len()` bytes starting at an absolute offset.
    fn read_at(&self, output: &mut [u8], offset: u64) -> io::Result<usize>;
}

#[cfg(any(unix, windows))]
impl ReadAt for File {
    fn len(&self) -> io::Result<u64> {
        Ok(self.metadata()?.len())
    }
    fn read_at(&self, output: &mut [u8], offset: u64) -> io::Result<usize> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileExt;
            FileExt::read_at(self, output, offset)
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::FileExt;
            self.seek_read(output, offset)
        }
    }
}
impl ReadAt for Arc<[u8]> {
    fn len(&self) -> io::Result<u64> {
        Ok(self.as_ref().len() as u64)
    }
    fn read_at(&self, output: &mut [u8], offset: u64) -> io::Result<usize> {
        let offset = usize::try_from(offset).unwrap_or(usize::MAX);
        let bytes = self.get(offset..).unwrap_or_default();
        let count = bytes.len().min(output.len());
        output[..count].copy_from_slice(&bytes[..count]);
        Ok(count)
    }
}

impl<T: ReadAt> ReadAt for Arc<T> {
    fn len(&self) -> io::Result<u64> {
        self.as_ref().len()
    }
    fn read_at(&self, output: &mut [u8], offset: u64) -> io::Result<usize> {
        self.as_ref().read_at(output, offset)
    }
}

/// A positional adapter for arbitrary seekable streams. Only seek/read operations
/// are serialized; entry decompression and validation can run concurrently.
pub struct SeekSource<R>(Mutex<R>);
impl<R> SeekSource<R> {
    /// Wraps a stream with a lock protecting its cursor.
    pub fn new(source: R) -> Self {
        Self(Mutex::new(source))
    }
}
impl<R: Read + Seek + Send> ReadAt for SeekSource<R> {
    fn len(&self) -> io::Result<u64> {
        self.0
            .lock()
            .map_err(|_| io::Error::other("MPQ source lock poisoned"))?
            .seek(SeekFrom::End(0))
    }
    fn read_at(&self, output: &mut [u8], offset: u64) -> io::Result<usize> {
        let mut source = self
            .0
            .lock()
            .map_err(|_| io::Error::other("MPQ source lock poisoned"))?;
        source.seek(SeekFrom::Start(offset))?;
        source.read(output)
    }
}

/// A private logical seek cursor over a shared positional source.
struct SourceCursor<S> {
    source: Arc<S>,
    position: u64,
    len: u64,
}
impl<S: ReadAt> Read for SourceCursor<S> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let count = self.source.read_at(output, self.position)?;
        self.position = self
            .position
            .checked_add(count as u64)
            .ok_or_else(|| io::Error::other("cursor overflow"))?;
        Ok(count)
    }
}
impl<S: ReadAt> Seek for SourceCursor<S> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        let position = match offset {
            SeekFrom::Start(at) => at as i128,
            SeekFrom::Current(delta) => self.position as i128 + delta as i128,
            SeekFrom::End(delta) => self.len as i128 + delta as i128,
        };
        self.position = u64::try_from(position)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid seek"))?;
        Ok(self.position)
    }
}

struct Metadata {
    index: Arc<Index>,
    base: u64,
    len: u64,
    options: ReadOptions,
    diagnostics: Vec<RecoveryDiagnostic>,
}
/// A cheaply cloned archive with shared indexing and independent entry readers.
pub struct SharedArchive<S> {
    source: Arc<S>,
    metadata: Arc<Metadata>,
}
impl<S> Clone for SharedArchive<S> {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            metadata: self.metadata.clone(),
        }
    }
}
/// An owned concurrent entry reader with entry-local recovery diagnostics.
pub struct SharedEntryReader<S> {
    stream: EntryReader<SourceCursor<S>>,
    diagnostics: Vec<RecoveryDiagnostic>,
}
impl<S: ReadAt> Read for SharedEntryReader<S> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.stream.read(output)
    }
}
impl<S: ReadAt> SharedEntryReader<S> {
    /// Recovery diagnostics encountered while opening this entry.
    pub fn diagnostics(&self) -> &[RecoveryDiagnostic] {
        &self.diagnostics
    }
    /// Storage metadata for this entry.
    pub fn metadata(&self) -> &BlockEntry {
        self.stream.metadata()
    }
}
impl<S: ReadAt> SharedArchive<S> {
    /// Indexes a positional source using strict defaults, starting at offset zero.
    pub fn open(source: S) -> Result<Self, Error> {
        Self::with_options(source, ReadOptions::default())
    }
    /// Indexes a positional source with explicit limits and recovery policy.
    pub fn with_options(source: S, options: ReadOptions) -> Result<Self, Error> {
        let len = source.len()?;
        let source = Arc::new(source);
        let archive = Archive::with_options(
            SourceCursor {
                source: source.clone(),
                position: 0,
                len,
            },
            options,
        )?;
        Ok(Self::from_indexed(archive, source))
    }
    fn from_indexed<R>(archive: Archive<R>, source: Arc<S>) -> Self {
        Self {
            source,
            metadata: Arc::new(Metadata {
                index: archive.index,
                base: archive.base,
                len: archive.source_len,
                options: archive.options,
                diagnostics: archive.diagnostics,
            }),
        }
    }
    /// Returns shared decoded lookup tables.
    pub fn index(&self) -> &Index {
        &self.metadata.index
    }
    /// Returns the absolute archive header offset.
    pub fn archive_offset(&self) -> u64 {
        self.metadata.base
    }
    /// Returns indexing recoveries. Payload recoveries are reported by each entry.
    pub fn diagnostics(&self) -> &[RecoveryDiagnostic] {
        &self.metadata.diagnostics
    }
    /// Opens a neutral-locale entry with independent decoding state.
    pub fn open_file(&self, name: impl AsRef<[u8]>) -> Result<SharedEntryReader<S>, Error> {
        self.open_file_with_locale(name, 0, 0)
    }
    /// Opens an exact locale/platform match.
    pub fn open_file_with_locale(
        &self,
        name: impl AsRef<[u8]>,
        locale: u16,
        platform: u16,
    ) -> Result<SharedEntryReader<S>, Error> {
        self.open_entry(|archive| {
            archive
                .open_file_with_locale(name, locale, platform)
                .map(|entry| entry.map_source(|_| ()))
        })
    }
    /// Opens an unnamed block, including supported permissive key recovery.
    pub fn open_file_by_index(&self, id: u32) -> Result<SharedEntryReader<S>, Error> {
        self.open_entry(|archive| {
            archive
                .open_file_by_index(id)
                .map(|entry| entry.map_source(|_| ()))
        })
    }
    fn open_entry(
        &self,
        open: impl FnOnce(&mut Archive<SourceCursor<S>>) -> Result<EntryReader<()>, Error>,
    ) -> Result<SharedEntryReader<S>, Error> {
        let mut archive = Archive {
            source: SourceCursor {
                source: self.source.clone(),
                position: 0,
                len: self.metadata.len,
            },
            source_len: self.metadata.len,
            diagnostics: Vec::new(),
            base: self.metadata.base,
            index: self.metadata.index.clone(),
            options: self.metadata.options.clone(),
        };
        let stream = open(&mut archive)?;
        Ok(SharedEntryReader {
            stream: stream.map_source(|_| archive.source),
            diagnostics: archive.diagnostics,
        })
    }
    /// Extracts a complete entry. Use `open_file` for bounded streaming memory.
    pub fn read_file(&self, name: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        let mut bytes = Vec::new();
        self.open_file(name)?.read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}
impl<R: Read + Seek + Send> Archive<R> {
    /// Shares an already indexed stream, serializing source I/O only.
    pub fn into_shared(self) -> SharedArchive<SeekSource<R>> {
        let Archive {
            source,
            source_len,
            diagnostics,
            base,
            index,
            options,
        } = self;
        SharedArchive {
            source: Arc::new(SeekSource::new(source)),
            metadata: Arc::new(Metadata {
                index,
                base,
                len: source_len,
                options,
                diagnostics,
            }),
        }
    }
}
