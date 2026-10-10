//! MPQ asset readers and caller-ordered overlays for Bevy.
//! Register readers before adding `AssetPlugin`. Archives are immutable snapshots;
//! directory enumeration, asset processing, patch deltas and watching are unsupported.
#![deny(missing_docs)]

use async_lock::Semaphore;
use bevy_asset::io::{
    AssetReader, AssetReaderError, ErasedAssetReader, PathStream, Reader, VecReader,
};
use blocking::unblock;
use std::io::{self, Read};
use std::num::NonZeroUsize;
use std::path::{Component, Path};
use std::sync::Arc;
use wc3::mpq::{Error, ReadAt, SharedArchive};

/// An indexed archive with bounded concurrent extraction on blocking workers.
/// Returned readers own their bytes and release extraction capacity.
pub struct MpqAssetReader<R> {
    archive: SharedArchive<R>,
    permits: Arc<Semaphore>,
    label: Arc<str>,
    locale: u16,
    platform: u16,
}

impl<R> Clone for MpqAssetReader<R> {
    fn clone(&self) -> Self {
        Self {
            archive: self.archive.clone(),
            permits: self.permits.clone(),
            label: self.label.clone(),
            locale: self.locale,
            platform: self.platform,
        }
    }
}

impl<R: ReadAt + 'static> MpqAssetReader<R> {
    /// Mounts a concurrent archive. The default allows four active extractions.
    pub fn new(archive: SharedArchive<R>, label: impl Into<Arc<str>>) -> Self {
        Self {
            archive,
            permits: Arc::new(Semaphore::new(4)),
            label: label.into(),
            locale: 0,
            platform: 0,
        }
    }

    /// Sets the extraction limit. Configure before cloning to share the limit.
    /// Capacity is acquired asynchronously before scheduling blocking work.
    pub fn with_max_concurrent_reads(mut self, limit: NonZeroUsize) -> Self {
        self.permits = Arc::new(Semaphore::new(limit.get()));
        self
    }

    fn contains(&self, path: &Path) -> Result<bool, AssetReaderError> {
        let name = asset_name(path)?;
        Ok(self
            .archive
            .index()
            .find(name.as_bytes(), self.locale, self.platform)
            .is_some()
            || self
                .archive
                .index()
                .find(name.as_bytes(), 0, self.platform)
                .is_some())
    }

    /// Selects the requested locale/platform, falling back to neutral locale on
    /// the same platform only when the requested entry is absent from the index.
    /// Extraction errors for an indexed entry do not trigger locale fallback.
    pub fn with_locale(mut self, locale: u16, platform: u16) -> Self {
        self.locale = locale;
        self.platform = platform;
        self
    }
}

impl<R: ReadAt + 'static> AssetReader for MpqAssetReader<R> {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        let name = asset_name(path)?;
        let archive = self.archive.clone();
        let label = self.label.clone();
        let path = path.to_owned();
        let platform = self.platform;
        let selected = if archive
            .index()
            .find(name.as_bytes(), self.locale, platform)
            .is_some()
        {
            self.locale
        } else if archive.index().find(name.as_bytes(), 0, platform).is_some() {
            0
        } else {
            return Err(AssetReaderError::NotFound(path));
        };
        let permit = self.permits.acquire_arc().await;
        let bytes = unblock(move || {
            let _permit = permit;
            let result = (|| {
                let mut entry =
                    archive.open_file_with_locale(name.as_bytes(), selected, platform)?;
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes)?;
                Ok::<_, Error>(bytes)
            })();
            match result {
                Ok(bytes) => Ok(bytes),
                Err(Error::FileNotFound) => Err(AssetReaderError::NotFound(path)),
                Err(error) => {
                    Err(io::Error::other(format!("MPQ {label}, entry {name}: {error}")).into())
                }
            }
        })
        .await?;
        Ok(VecReader::new(bytes))
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        Err::<VecReader, _>(AssetReaderError::NotFound(path.to_owned()))
    }
    async fn read_directory<'a>(
        &'a self,
        _path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "MPQ directory enumeration is unsupported",
        )
        .into())
    }
    async fn is_directory<'a>(&'a self, _path: &'a Path) -> Result<bool, AssetReaderError> {
        Ok(false)
    }
}

/// Searches readers in caller-supplied order. Only `NotFound` allows fallback.
/// No mount order, locale policy, or Warcraft-specific paths are imposed.
pub struct OverlayAssetReader {
    readers: Vec<OverlayMount>,
}

impl OverlayAssetReader {
    /// Searches mounts in caller order. MPQ mounts select metadata using their
    /// index; generic mounts probe by reading. Only missing entries fall through.
    pub fn new(readers: Vec<OverlayMount>) -> Self {
        Self { readers }
    }
}

impl AssetReader for OverlayAssetReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        for reader in &self.readers {
            match reader.reader.read(path).await {
                Err(AssetReaderError::NotFound(_)) => continue,
                result => return result,
            }
        }
        Err(AssetReaderError::NotFound(path.to_owned()))
    }
    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        // Select metadata from the same mount as the asset, never a lower mount.
        for reader in &self.readers {
            if let Some(contains) = &reader.contains {
                if !contains(path)? {
                    continue;
                }
                return reader.reader.read_meta(path).await;
            }
            match reader.reader.read(path).await {
                Err(AssetReaderError::NotFound(_)) => continue,
                Err(error) => return Err(error),
                Ok(_) => return reader.reader.read_meta(path).await,
            }
        }
        Err(AssetReaderError::NotFound(path.to_owned()))
    }
    async fn read_directory<'a>(
        &'a self,
        _path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "overlay directory enumeration is unsupported",
        )
        .into())
    }
    async fn is_directory<'a>(&'a self, _path: &'a Path) -> Result<bool, AssetReaderError> {
        Ok(false)
    }
}

/// An overlay mount with optional index-only existence lookup.
pub struct OverlayMount {
    reader: Box<dyn ErasedAssetReader>,
    contains: Option<Box<Contains>>,
}
type Contains = dyn Fn(&Path) -> Result<bool, AssetReaderError> + Send + Sync;
impl OverlayMount {
    /// Mounts a generic Bevy reader, probing it by reading when selecting metadata.
    pub fn reader(reader: Box<dyn ErasedAssetReader>) -> Self {
        Self {
            reader,
            contains: None,
        }
    }
    /// Mounts an MPQ reader, selecting metadata without extracting payloads.
    pub fn mpq<R: ReadAt + 'static>(reader: MpqAssetReader<R>) -> Self {
        let probe = reader.clone();
        Self {
            reader: Box::new(reader),
            contains: Some(Box::new(move |path| probe.contains(path))),
        }
    }
}

fn asset_name(path: &Path) -> Result<String, AssetReaderError> {
    if path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "asset paths must stay within the source",
        )
        .into());
    }
    let name = path
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "MPQ asset path must be UTF-8"))?
        .replace('\\', "/");
    if name.starts_with('/')
        || name.as_bytes().get(1) == Some(&b':')
        || name.split('/').any(|part| part == "..")
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "absolute path or parent traversal in MPQ path",
        )
        .into());
    }
    Ok(name)
}
