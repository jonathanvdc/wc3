use super::{Blp2, Blp2Content};
use std::borrow::Cow;
impl Blp2Content<'_> {
    /// Borrows all encoded content without copying payloads.
    pub fn as_ref(&self) -> Blp2Content<'_> {
        match self {
            Self::Jpeg {
                alpha_type,
                shared_header,
                unused,
            } => Blp2Content::Jpeg {
                alpha_type: *alpha_type,
                shared_header: Cow::Borrowed(shared_header),
                unused: Cow::Borrowed(unused),
            },
            Self::Indexed {
                alpha_type,
                palette,
            } => Blp2Content::Indexed {
                alpha_type: *alpha_type,
                palette: palette.as_ref(),
            },
            Self::Dxt {
                format,
                palette_region,
            } => Blp2Content::Dxt {
                format: *format,
                palette_region: palette_region.as_ref(),
            },
            Self::Bgra {
                encoding,
                alpha_type,
                palette_region,
            } => Blp2Content::Bgra {
                encoding: *encoding,
                alpha_type: *alpha_type,
                palette_region: palette_region.as_ref(),
            },
        }
    }
    /// Moves owned buffers and copies borrowed content into independent storage.
    pub fn into_owned(self) -> Blp2Content<'static> {
        match self {
            Self::Jpeg {
                alpha_type,
                shared_header,
                unused,
            } => Blp2Content::Jpeg {
                alpha_type,
                shared_header: Cow::Owned(shared_header.into_owned()),
                unused: Cow::Owned(unused.into_owned()),
            },
            Self::Indexed {
                alpha_type,
                palette,
            } => Blp2Content::Indexed {
                alpha_type,
                palette: palette.into_owned(),
            },
            Self::Dxt {
                format,
                palette_region,
            } => Blp2Content::Dxt {
                format,
                palette_region: palette_region.into_owned(),
            },
            Self::Bgra {
                encoding,
                alpha_type,
                palette_region,
            } => Blp2Content::Bgra {
                encoding,
                alpha_type,
                palette_region: palette_region.into_owned(),
            },
        }
    }
}
impl Blp2<'_> {
    /// Borrows all payloads without allocating.
    pub fn as_ref(&self) -> Blp2<'_> {
        Blp2 {
            header: self.header,
            content: self.content.as_ref(),
            mipmaps: self
                .mipmaps
                .each_ref()
                .map(|mip| mip.as_deref().map(Cow::Borrowed)),
        }
    }
    /// Moves owned buffers and copies borrowed payloads.
    pub fn into_owned(self) -> Blp2<'static> {
        Blp2 {
            header: self.header,
            content: self.content.into_owned(),
            mipmaps: self
                .mipmaps
                .map(|mip| mip.map(|bytes| Cow::Owned(bytes.into_owned()))),
        }
    }
}
