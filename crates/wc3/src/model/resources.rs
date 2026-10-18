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
    FileDataId(u32),

    /// Application-provided replacement texture category.
    Replaceable(u32),
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
            Self::FileDataId(value) => ImageResource::FileDataId(value),

            Self::Replaceable(value) => ImageResource::Replaceable(value),
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
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
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
    Replaceable(u32),
}
impl<'a> From<MdlxImageResource<'a>> for ImageResource<'a> {
    fn from(value: MdlxImageResource<'a>) -> Self {
        match value {
            MdlxImageResource::Path(value) => ImageResource::Path(value),
            MdlxImageResource::Replaceable(value) => ImageResource::Replaceable(value),
        }
    }
}
impl<'a> TryFrom<ImageResource<'a>> for MdlxImageResource<'a> {
    type Error = Error;
    fn try_from(value: ImageResource<'a>) -> Result<Self, Error> {
        match value {
            ImageResource::Path(value) => Ok(Self::Path(value)),
            ImageResource::Replaceable(value) => Ok(Self::Replaceable(value)),
            _ => Err(Error::new(
                "resources",
                "address is not admitted by the destination resource vocabulary",
            )),
        }
    }
}
impl ValidateLinks for MdlxImageResource<'_> {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
/// WoW image paths, file-data identities, and replacement categories.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum M2ImageResource<'a> {
    /// Native path spelling, including non-UTF-8 bytes.
    Path(Cow<'a, [u8]>),
    /// External file-data table identity.
    FileDataId(u32),
    /// Application-selected replacement category.
    Replaceable(u32),
}
impl<'a> From<M2ImageResource<'a>> for ImageResource<'a> {
    fn from(value: M2ImageResource<'a>) -> Self {
        match value {
            M2ImageResource::Path(value) => ImageResource::Path(value),
            M2ImageResource::FileDataId(value) => ImageResource::FileDataId(value),
            M2ImageResource::Replaceable(value) => ImageResource::Replaceable(value),
        }
    }
}
impl<'a> TryFrom<ImageResource<'a>> for M2ImageResource<'a> {
    type Error = Error;
    fn try_from(value: ImageResource<'a>) -> Result<Self, Error> {
        match value {
            ImageResource::Path(value) => Ok(Self::Path(value)),
            ImageResource::FileDataId(value) => Ok(Self::FileDataId(value)),
            ImageResource::Replaceable(value) => Ok(Self::Replaceable(value)),
            _ => Err(Error::new(
                "resources",
                "address is not admitted by the destination resource vocabulary",
            )),
        }
    }
}
impl ValidateLinks for M2ImageResource<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if matches!(self, Self::FileDataId(0) | Self::Replaceable(0)) {
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
            WorldImageResource::FileDataId(value) => ImageResource::FileDataId(value),
            WorldImageResource::EmbeddedImage { bytes } => ImageResource::EmbeddedImage { bytes },
        }
    }
}
impl<'a> TryFrom<ImageResource<'a>> for WorldImageResource<'a> {
    type Error = Error;
    fn try_from(value: ImageResource<'a>) -> Result<Self, Error> {
        match value {
            ImageResource::Path(value) => Ok(Self::Path(value)),
            ImageResource::FileDataId(value) => Ok(Self::FileDataId(value)),
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
            WorldModelResource::FileDataId(value) => ImageResource::FileDataId(value),
        }
    }
}
impl<'a> TryFrom<ImageResource<'a>> for WorldModelResource<'a> {
    type Error = Error;
    fn try_from(value: ImageResource<'a>) -> Result<Self, Error> {
        match value {
            ImageResource::Path(value) => Ok(Self::Path(value)),
            ImageResource::FileDataId(value) => Ok(Self::FileDataId(value)),
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
            ResourceLocator::FileDataId(value) => Self::FileDataId(value),

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
            ImageResource::FileDataId(value) => Ok(Self::FileDataId(value)),

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
