//! MPQ asset readers and caller-ordered overlays for Bevy.
//! Register readers before adding `AssetPlugin`. Archives are immutable snapshots;
//! directory enumeration, asset processing, patch deltas and watching are unsupported.
#![deny(missing_docs)]

use bevy_asset::io::{
    AssetReader, AssetReaderError, ErasedAssetReader, PathStream, Reader, VecReader,
};
use blocking::unblock;
use std::io::{self, Read, Seek};
use std::path::{Component, Path};
use std::sync::{Arc, Mutex};
use wc3::mpq::{Archive, Error};

/// An indexed archive. Extraction runs on blocking workers and is serialized per
/// archive. Returned readers own their bytes and do not retain the archive lock.
pub struct MpqAssetReader<R> {
    archive: Arc<Mutex<Archive<R>>>,
    label: Arc<str>,
    locale: u16,
    platform: u16,
}

impl<R> Clone for MpqAssetReader<R> {
    fn clone(&self) -> Self {
        Self {
            archive: self.archive.clone(),
            label: self.label.clone(),
            locale: self.locale,
            platform: self.platform,
        }
    }
}

impl<R: Read + Seek + Send + 'static> MpqAssetReader<R> {
    /// Takes an already-opened archive, allowing callers to configure read limits
    /// and strict/permissive parsing before mounting it.
    pub fn new(archive: Archive<R>, label: impl Into<Arc<str>>) -> Self {
        Self {
            archive: Arc::new(Mutex::new(archive)),
            label: label.into(),
            locale: 0,
            platform: 0,
        }
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

impl<R: Read + Seek + Send + 'static> AssetReader for MpqAssetReader<R> {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        if path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "asset paths must stay within the source",
            )
            .into());
        }
        let name = path
            .to_str()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "MPQ asset path must be UTF-8")
            })?
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
        let archive = self.archive.clone();
        let label = self.label.clone();
        let path = path.to_owned();
        let locale = self.locale;
        let platform = self.platform;
        let bytes = unblock(move || {
            let mut archive = archive
                .lock()
                .map_err(|_| io::Error::other("MPQ lock poisoned"))?;
            let selected = if archive
                .index()
                .find(name.as_bytes(), locale, platform)
                .is_some()
            {
                locale
            } else {
                0
            };
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
    readers: Vec<Box<dyn ErasedAssetReader>>,
}

impl OverlayAssetReader {
    /// Creates an overlay that searches `readers` from first to last.
    ///
    /// Missing assets fall through to the next reader; other errors stop the
    /// search. An empty overlay reports every asset as missing. Metadata comes
    /// from the reader containing the asset, even when that reader has no metadata.
    pub fn new(readers: Vec<Box<dyn ErasedAssetReader>>) -> Self {
        Self { readers }
    }
}

impl AssetReader for OverlayAssetReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        for reader in &self.readers {
            match reader.read(path).await {
                Err(AssetReaderError::NotFound(_)) => continue,
                result => return result,
            }
        }
        Err(AssetReaderError::NotFound(path.to_owned()))
    }
    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        // Select metadata from the same mount as the asset, never a lower mount.
        for reader in &self.readers {
            match reader.read(path).await {
                Err(AssetReaderError::NotFound(_)) => continue,
                Err(error) => return Err(error),
                Ok(_) => return reader.read_meta(path).await,
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
