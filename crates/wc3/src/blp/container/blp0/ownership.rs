use super::{Blp0, Blp0Content};
use std::borrow::Cow;
impl Blp0Content<'_> {
    /// Borrows all encoded content without copying payloads.
    pub fn as_ref(&self) -> Blp0Content<'_> {
        match self {
            Self::Jpeg { shared_header } => Blp0Content::Jpeg {
                shared_header: Cow::Borrowed(shared_header),
            },
            Self::Indexed { palette } => Blp0Content::Indexed {
                palette: palette.as_ref(),
            },
        }
    }
    /// Moves owned buffers and copies borrowed content into independent storage.
    pub fn into_owned(self) -> Blp0Content<'static> {
        match self {
            Self::Jpeg { shared_header } => Blp0Content::Jpeg {
                shared_header: Cow::Owned(shared_header.into_owned()),
            },
            Self::Indexed { palette } => Blp0Content::Indexed {
                palette: palette.into_owned(),
            },
        }
    }
}
impl Blp0<'_> {
    /// Borrows all payloads without allocating.
    pub fn as_ref(&self) -> Blp0<'_> {
        Blp0 {
            header: self.header,
            content: self.content.as_ref(),
            mipmaps: self
                .mipmaps
                .each_ref()
                .map(|mip| mip.as_deref().map(Cow::Borrowed)),
        }
    }
    /// Moves owned buffers and copies borrowed payloads.
    pub fn into_owned(self) -> Blp0<'static> {
        Blp0 {
            header: self.header,
            content: self.content.into_owned(),
            mipmaps: self
                .mipmaps
                .map(|mip| mip.map(|bytes| Cow::Owned(bytes.into_owned()))),
        }
    }
}
