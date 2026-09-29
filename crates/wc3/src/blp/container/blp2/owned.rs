//! Borrowed and owned BLP2 conversions.
use super::{Blp2, Blp2Content, Blp2ContentRef, Blp2Ref};

impl Blp2ContentRef<'_> {
    /// Copies the encoded content for editing.
    pub fn to_owned(&self) -> Blp2Content {
        match *self {
            Self::Jpeg {
                alpha_type,
                shared_header,
                unused,
            } => Blp2Content::Jpeg {
                alpha_type,
                shared_header: shared_header.to_vec(),
                unused: unused.to_vec(),
            },
            Self::Indexed {
                alpha_type,
                palette,
            } => Blp2Content::Indexed {
                alpha_type,
                palette: Box::new(*palette),
            },
            Self::Dxt {
                format,
                palette_region,
            } => Blp2Content::Dxt {
                format,
                palette_region: Box::new(*palette_region),
            },
            Self::Bgra {
                encoding,
                alpha_type,
                palette_region,
            } => Blp2Content::Bgra {
                encoding,
                alpha_type,
                palette_region: Box::new(*palette_region),
            },
        }
    }
}

impl Blp2Content {
    /// Borrows the encoded content.
    pub fn as_ref(&self) -> Blp2ContentRef<'_> {
        match self {
            Self::Jpeg {
                alpha_type,
                shared_header,
                unused,
            } => Blp2ContentRef::Jpeg {
                alpha_type: *alpha_type,
                shared_header,
                unused,
            },
            Self::Indexed {
                alpha_type,
                palette,
            } => Blp2ContentRef::Indexed {
                alpha_type: *alpha_type,
                palette,
            },
            Self::Dxt {
                format,
                palette_region,
            } => Blp2ContentRef::Dxt {
                format: *format,
                palette_region,
            },
            Self::Bgra {
                encoding,
                alpha_type,
                palette_region,
            } => Blp2ContentRef::Bgra {
                encoding: *encoding,
                alpha_type: *alpha_type,
                palette_region,
            },
        }
    }
}

impl Blp2Ref<'_> {
    /// Copies this BLP2 container for editing.
    pub fn to_owned(&self) -> Blp2 {
        Blp2 {
            header: self.header,
            content: self.content.to_owned(),
            mipmaps: self.mipmaps.map(|mip| mip.map(<[u8]>::to_vec)),
        }
    }
}

impl Blp2 {
    /// Borrows this editable BLP2 container.
    pub fn as_ref(&self) -> Blp2Ref<'_> {
        Blp2Ref {
            header: self.header,
            content: self.content.as_ref(),
            mipmaps: self.mipmaps.each_ref().map(|mip| mip.as_deref()),
        }
    }
}
