use super::{Blp1, Blp1Content};
use std::borrow::Cow;
impl Blp1Content<'_> {
    /// Borrows all encoded content without copying payloads.
    pub fn as_ref(&self) -> Blp1Content<'_> {
        match self {
            Self::Jpeg { shared_header } => Blp1Content::Jpeg {
                shared_header: Cow::Borrowed(shared_header),
            },
            Self::Indexed { palette } => Blp1Content::Indexed {
                palette: palette.as_ref(),
            },
        }
    }
    /// Moves owned buffers and copies borrowed content into independent storage.
    pub fn into_owned(self) -> Blp1Content<'static> {
        match self {
            Self::Jpeg { shared_header } => Blp1Content::Jpeg {
                shared_header: Cow::Owned(shared_header.into_owned()),
            },
            Self::Indexed { palette } => Blp1Content::Indexed {
                palette: palette.into_owned(),
            },
        }
    }
}
impl Blp1<'_> {
    /// Borrows all payloads without allocating.
    pub fn as_ref(&self) -> Blp1<'_> {
        Blp1 {
            header: self.header,
            content: self.content.as_ref(),
            mipmaps: self
                .mipmaps
                .each_ref()
                .map(|mip| mip.as_deref().map(Cow::Borrowed)),
        }
    }
    /// Moves owned buffers and copies borrowed payloads.
    pub fn into_owned(self) -> Blp1<'static> {
        Blp1 {
            header: self.header,
            content: self.content.into_owned(),
            mipmaps: self
                .mipmaps
                .map(|mip| mip.map(|bytes| Cow::Owned(bytes.into_owned()))),
        }
    }
}
