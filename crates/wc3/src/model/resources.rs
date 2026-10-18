//! Model-owned external dependency declarations and profile-specific addressing.
use crate::model::{Error, Link, ValidateLinks};
use std::borrow::Cow;

/// An image source addressed externally, supplied by the application, or embedded.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImageResource<'a> {
    /// Unresolved caller-provided asset root.
    Root,
    /// Resource resolved relative to another logical asset identity.
    Relative {
        /// Identity supplying the filename stem.
        owner: Box<ResourceLocator<'a>>,
        /// Suffix appended to that stem.
        suffix: Cow<'a, str>,
    },
    /// Native path spelling, including non-UTF-8 paths.
    Path(Cow<'a, [u8]>),
    /// World of Warcraft resource identity.
    FileDataId {
        /// Primary numeric identity.
        id: u32,
        /// Accompanying native filename; empty when none was authored.
        filename: Cow<'a, [u8]>,
    },

    /// Application-provided replacement texture category.
    Replaceable {
        /// Primary numeric identity.
        id: u32,
        /// Accompanying native filename; empty when none was authored.
        filename: Cow<'a, [u8]>,
    },
    /// Image bytes supplied with the authoring data, with no established encoding interpretation.
    EmbeddedImage {
        /// Authored image bytes, borrowed when available.
        bytes: Cow<'a, [u8]>,
    },
    /// A logical resource with an explicit namespace.
    Named {
        /// Provider namespace.
        namespace: Cow<'a, str>,
        /// Provider identity.
        name: Cow<'a, str>,
    },
}
impl ImageResource<'_> {
    /// Detaches the complete resource identity, including nested owners and raw paths.
    pub fn into_owned(self) -> ImageResource<'static> {
        match self {
            Self::Root => ImageResource::Root,
            Self::Relative { owner, suffix } => ImageResource::Relative {
                owner: Box::new(owner.into_owned()),
                suffix: Cow::Owned(suffix.into_owned()),
            },
            Self::Path(value) => ImageResource::Path(Cow::Owned(value.into_owned())),
            Self::FileDataId { id, filename } => ImageResource::FileDataId {
                id,
                filename: Cow::Owned(filename.into_owned()),
            },

            Self::Replaceable { id, filename } => ImageResource::Replaceable {
                id,
                filename: Cow::Owned(filename.into_owned()),
            },
            Self::EmbeddedImage { bytes } => ImageResource::EmbeddedImage {
                bytes: Cow::Owned(bytes.into_owned()),
            },
            Self::Named { namespace, name } => ImageResource::Named {
                namespace: Cow::Owned(namespace.into_owned()),
                name: Cow::Owned(name.into_owned()),
            },
        }
    }
}

impl ValidateLinks for ImageResource<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Self::FileDataId { filename, .. } | Self::Replaceable { filename, .. } = self {
            if filename.contains(&0) {
                visitor(Link::InvalidValue);
            }
        }
    }
}

/// A dependency addressed by native path bytes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PathResource<'a> {
    /// Native path spelling, including non-UTF-8 bytes.
    Path(Cow<'a, [u8]>),
}
impl<'a> From<PathResource<'a>> for ImageResource<'a> {
    fn from(value: PathResource<'a>) -> Self {
        match value {
            PathResource::Path(value) => ImageResource::Path(value),
        }
    }
}
impl<'a> TryFrom<ImageResource<'a>> for PathResource<'a> {
    type Error = Error;
    fn try_from(value: ImageResource<'a>) -> Result<Self, Error> {
        match value {
            ImageResource::Path(value) => Ok(Self::Path(value)),
            _ => Err(Error::new(
                "resources",
                "address is not admitted by the destination resource vocabulary",
            )),
        }
    }
}
impl ValidateLinks for PathResource<'_> {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
/// Warcraft III image paths and application replacement categories.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MdlxImageResource<'a> {
    /// Native path spelling, including non-UTF-8 bytes.
    Path(Cow<'a, [u8]>),
    /// Application-selected replacement category.
    Replaceable {
        /// Primary numeric identity.
        id: u32,
        /// Accompanying native filename; empty when none was authored.
        filename: Cow<'a, [u8]>,
    },
}
impl<'a> From<MdlxImageResource<'a>> for ImageResource<'a> {
    fn from(value: MdlxImageResource<'a>) -> Self {
        match value {
            MdlxImageResource::Path(value) => ImageResource::Path(value),
            MdlxImageResource::Replaceable { id, filename } => {
                ImageResource::Replaceable { id, filename }
            }
        }
    }
}
impl<'a> TryFrom<ImageResource<'a>> for MdlxImageResource<'a> {
    type Error = Error;
    fn try_from(value: ImageResource<'a>) -> Result<Self, Error> {
        match value {
            ImageResource::Path(value) => Ok(Self::Path(value)),
            ImageResource::Replaceable { id, filename } => Ok(Self::Replaceable { id, filename }),
            _ => Err(Error::new(
                "resources",
                "address is not admitted by the destination resource vocabulary",
            )),
        }
    }
}
impl ValidateLinks for MdlxImageResource<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Self::Replaceable { filename, .. } = self {
            if filename.contains(&0) {
                visitor(Link::InvalidValue);
            }
        }
    }
}
/// WoW image paths, file-data identities, and replacement categories.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum M2ImageResource<'a> {
    /// Native path spelling, including non-UTF-8 bytes.
    Path(Cow<'a, [u8]>),
    /// External file-data table identity.
    FileDataId {
        /// Primary numeric identity.
        id: u32,
        /// Accompanying native filename; empty when none was authored.
        filename: Cow<'a, [u8]>,
    },
    /// Application-selected replacement category.
    Replaceable {
        /// Primary numeric identity.
        id: u32,
        /// Accompanying native filename; empty when none was authored.
        filename: Cow<'a, [u8]>,
    },
}
impl<'a> From<M2ImageResource<'a>> for ImageResource<'a> {
    fn from(value: M2ImageResource<'a>) -> Self {
        match value {
            M2ImageResource::Path(value) => ImageResource::Path(value),
            M2ImageResource::FileDataId { id, filename } => {
                ImageResource::FileDataId { id, filename }
            }
            M2ImageResource::Replaceable { id, filename } => {
                ImageResource::Replaceable { id, filename }
            }
        }
    }
}
impl<'a> TryFrom<ImageResource<'a>> for M2ImageResource<'a> {
    type Error = Error;
    fn try_from(value: ImageResource<'a>) -> Result<Self, Error> {
        match value {
            ImageResource::Path(value) => Ok(Self::Path(value)),
            ImageResource::FileDataId { id, filename } => Ok(Self::FileDataId { id, filename }),
            ImageResource::Replaceable { id, filename } => Ok(Self::Replaceable { id, filename }),
            _ => Err(Error::new(
                "resources",
                "address is not admitted by the destination resource vocabulary",
            )),
        }
    }
}
impl ValidateLinks for M2ImageResource<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Self::FileDataId { filename, .. } | Self::Replaceable { filename, .. } = self {
            if filename.contains(&0) {
                visitor(Link::InvalidValue);
            }
        }
        if matches!(
            self,
            Self::FileDataId { id: 0, .. } | Self::Replaceable { id: 0, .. }
        ) {
            visitor(Link::InvalidValue);
        }
    }
}
/// World image paths, file-data identities, and embedded lightmaps.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum WorldImageResource<'a> {
    /// Native path spelling, including non-UTF-8 bytes.
    Path(Cow<'a, [u8]>),
    /// External file-data table identity.
    FileDataId(u32),
    /// Embedded image bytes whose encoding is unspecified.
    EmbeddedImage {
        /// Image bytes with unspecified encoding.
        bytes: Cow<'a, [u8]>,
    },
}
impl<'a> From<WorldImageResource<'a>> for ImageResource<'a> {
    fn from(value: WorldImageResource<'a>) -> Self {
        match value {
            WorldImageResource::Path(value) => ImageResource::Path(value),
            WorldImageResource::FileDataId(id) => ImageResource::FileDataId {
                id,
                filename: Cow::Borrowed(&[]),
            },
            WorldImageResource::EmbeddedImage { bytes } => ImageResource::EmbeddedImage { bytes },
        }
    }
}
impl<'a> TryFrom<ImageResource<'a>> for WorldImageResource<'a> {
    type Error = Error;
    fn try_from(value: ImageResource<'a>) -> Result<Self, Error> {
        match value {
            ImageResource::Path(value) => Ok(Self::Path(value)),
            ImageResource::FileDataId { id, filename } if filename.is_empty() => {
                Ok(Self::FileDataId(id))
            }
            ImageResource::EmbeddedImage { bytes } => Ok(Self::EmbeddedImage { bytes }),
            _ => Err(Error::new(
                "resources",
                "address is not admitted by the destination resource vocabulary",
            )),
        }
    }
}
impl ValidateLinks for WorldImageResource<'_> {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
/// World child-model and skybox addresses.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum WorldModelResource<'a> {
    /// Native path spelling, including non-UTF-8 bytes.
    Path(Cow<'a, [u8]>),
    /// External file-data table identity.
    FileDataId(u32),
}
impl<'a> From<WorldModelResource<'a>> for ImageResource<'a> {
    fn from(value: WorldModelResource<'a>) -> Self {
        match value {
            WorldModelResource::Path(value) => ImageResource::Path(value),
            WorldModelResource::FileDataId(id) => ImageResource::FileDataId {
                id,
                filename: Cow::Borrowed(&[]),
            },
        }
    }
}
impl<'a> TryFrom<ImageResource<'a>> for WorldModelResource<'a> {
    type Error = Error;
    fn try_from(value: ImageResource<'a>) -> Result<Self, Error> {
        match value {
            ImageResource::Path(value) => Ok(Self::Path(value)),
            ImageResource::FileDataId { id, filename } if filename.is_empty() => {
                Ok(Self::FileDataId(id))
            }
            _ => Err(Error::new(
                "resources",
                "address is not admitted by the destination resource vocabulary",
            )),
        }
    }
}
impl ValidateLinks for WorldModelResource<'_> {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}

/// Resource identity interpreted by an application resolver, not by a file layout.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResourceLocator<'a> {
    /// Unresolved caller-provided asset root.
    Root,
    /// Resource resolved relative to another logical asset identity.
    Relative {
        /// Identity supplying the filename stem.
        owner: Box<ResourceLocator<'a>>,
        /// Suffix appended to that stem.
        suffix: Cow<'a, str>,
    },
    /// Native path spelling, including non-UTF-8 paths.
    Path(Cow<'a, [u8]>),
    /// World of Warcraft resource identity.
    FileDataId(u32),

    /// A logical resource with an explicit namespace.
    Named {
        /// Provider namespace.
        namespace: Cow<'a, str>,
        /// Provider identity.
        name: Cow<'a, str>,
    },
}
impl ResourceLocator<'_> {
    /// Detaches the complete resource identity, including nested owners and raw paths.
    pub fn into_owned(self) -> ResourceLocator<'static> {
        match self {
            Self::Root => ResourceLocator::Root,
            Self::Relative { owner, suffix } => ResourceLocator::Relative {
                owner: Box::new(owner.into_owned()),
                suffix: Cow::Owned(suffix.into_owned()),
            },
            Self::Path(value) => ResourceLocator::Path(Cow::Owned(value.into_owned())),
            Self::FileDataId(value) => ResourceLocator::FileDataId(value),

            Self::Named { namespace, name } => ResourceLocator::Named {
                namespace: Cow::Owned(namespace.into_owned()),
                name: Cow::Owned(name.into_owned()),
            },
        }
    }
}

impl ValidateLinks for ResourceLocator<'_> {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl<'a> From<ResourceLocator<'a>> for ImageResource<'a> {
    fn from(value: ResourceLocator<'a>) -> Self {
        match value {
            ResourceLocator::Root => Self::Root,
            ResourceLocator::Relative { owner, suffix } => Self::Relative { owner, suffix },
            ResourceLocator::Path(value) => Self::Path(value),
            ResourceLocator::FileDataId(id) => Self::FileDataId {
                id,
                filename: Cow::Borrowed(&[]),
            },

            ResourceLocator::Named { namespace, name } => Self::Named { namespace, name },
        }
    }
}
impl<'a> TryFrom<ImageResource<'a>> for ResourceLocator<'a> {
    type Error = Error;
    fn try_from(value: ImageResource<'a>) -> Result<Self, Error> {
        match value {
            ImageResource::Root => Ok(Self::Root),
            ImageResource::Relative { owner, suffix } => Ok(Self::Relative { owner, suffix }),
            ImageResource::Path(value) => Ok(Self::Path(value)),
            ImageResource::FileDataId { id, filename } if filename.is_empty() => {
                Ok(Self::FileDataId(id))
            }

            ImageResource::Named { namespace, name } => Ok(Self::Named { namespace, name }),
            _ => Err(Error::new(
                "resources",
                "image-only declaration cannot address an external dependency",
            )),
        }
    }
}
impl<'a> From<PathResource<'a>> for ResourceLocator<'a> {
    fn from(value: PathResource<'a>) -> Self {
        let PathResource::Path(path) = value;
        Self::Path(path)
    }
}
impl<'a> TryFrom<ResourceLocator<'a>> for PathResource<'a> {
    type Error = Error;
    fn try_from(value: ResourceLocator<'a>) -> Result<Self, Error> {
        ImageResource::from(value).try_into()
    }
}
impl<'a> From<WorldModelResource<'a>> for ResourceLocator<'a> {
    fn from(value: WorldModelResource<'a>) -> Self {
        match value {
            WorldModelResource::Path(path) => Self::Path(path),
            WorldModelResource::FileDataId(id) => Self::FileDataId(id),
        }
    }
}
impl<'a> TryFrom<ResourceLocator<'a>> for WorldModelResource<'a> {
    type Error = Error;
    fn try_from(value: ResourceLocator<'a>) -> Result<Self, Error> {
        ImageResource::from(value).try_into()
    }
}

impl<'a> From<PathResource<'a>> for MdlxImageResource<'a> {
    fn from(value: PathResource<'a>) -> Self {
        let PathResource::Path(path) = value;
        Self::Path(path)
    }
}
impl<'a> From<PathResource<'a>> for M2ImageResource<'a> {
    fn from(value: PathResource<'a>) -> Self {
        let PathResource::Path(path) = value;
        Self::Path(path)
    }
}
impl<'a> From<PathResource<'a>> for WorldImageResource<'a> {
    fn from(value: PathResource<'a>) -> Self {
        let PathResource::Path(path) = value;
        Self::Path(path)
    }
}

#[cfg(test)]
mod resource_tests {}

/// Companion addresses supported by the WoW model vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum M2ResourceLocator<'a> {
    /// The application-provided root asset.
    Root,
    /// A sibling obtained by replacing the owner's extension with a suffix.
    Relative {
        /// Address supplying the file stem.
        owner: Box<M2ResourceLocator<'a>>,
        /// Suffix appended to that stem.
        suffix: Cow<'a, str>,
    },
    /// Native path bytes.
    Path(Cow<'a, [u8]>),
    /// File-data table identity.
    FileDataId(u32),
}
impl<'a> From<M2ResourceLocator<'a>> for ResourceLocator<'a> {
    fn from(value: M2ResourceLocator<'a>) -> Self {
        match value {
            M2ResourceLocator::Root => Self::Root,
            M2ResourceLocator::Relative { owner, suffix } => Self::Relative {
                owner: Box::new((*owner).into()),
                suffix,
            },
            M2ResourceLocator::Path(path) => Self::Path(path),
            M2ResourceLocator::FileDataId(id) => Self::FileDataId(id),
        }
    }
}
impl<'a> TryFrom<ResourceLocator<'a>> for M2ResourceLocator<'a> {
    type Error = Error;
    fn try_from(value: ResourceLocator<'a>) -> Result<Self, Error> {
        Ok(match value {
            ResourceLocator::Root => Self::Root,
            ResourceLocator::Relative { owner, suffix } => Self::Relative {
                owner: Box::new((*owner).try_into()?),
                suffix,
            },
            ResourceLocator::Path(path) => Self::Path(path),
            ResourceLocator::FileDataId(id) => Self::FileDataId(id),
            _ => {
                return Err(Error::new(
                    "m2.resources",
                    "companion address is unavailable in the M2 vocabulary",
                ))
            }
        })
    }
}
impl ValidateLinks for M2ResourceLocator<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Relative { owner, .. } => owner.visit_links(visitor),
            Self::FileDataId(0) => visitor(Link::InvalidValue),
            _ => {}
        }
    }
}

impl ImageResource<'_> {
    /// Declares a numeric image identity without an accompanying filename.
    pub fn file_data_id(id: u32) -> Self {
        Self::FileDataId {
            id,
            filename: Cow::Borrowed(&[]),
        }
    }
    /// Declares a numeric image identity without an accompanying filename.
    pub fn replaceable(id: u32) -> Self {
        Self::Replaceable {
            id,
            filename: Cow::Borrowed(&[]),
        }
    }
}

impl MdlxImageResource<'_> {
    /// Declares a numeric image identity without an accompanying filename.
    pub fn replaceable(id: u32) -> Self {
        Self::Replaceable {
            id,
            filename: Cow::Borrowed(&[]),
        }
    }
}

impl M2ImageResource<'_> {
    /// Declares a numeric image identity without an accompanying filename.
    pub fn file_data_id(id: u32) -> Self {
        Self::FileDataId {
            id,
            filename: Cow::Borrowed(&[]),
        }
    }
    /// Declares a numeric image identity without an accompanying filename.
    pub fn replaceable(id: u32) -> Self {
        Self::Replaceable {
            id,
            filename: Cow::Borrowed(&[]),
        }
    }
}
