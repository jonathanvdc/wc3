//! Borrowed and owned BLP0 conversions.
use super::{Blp0, Blp0Content, Blp0ContentRef, Blp0Ref};

impl Blp0ContentRef<'_> {
    /// Copies the encoded content for editing.
    pub fn to_owned(&self) -> Blp0Content {
        match *self {
            Self::Jpeg { shared_header } => Blp0Content::Jpeg {
                shared_header: shared_header.to_vec(),
            },
            Self::Indexed { palette } => Blp0Content::Indexed {
                palette: Box::new(*palette),
            },
        }
    }
}

impl Blp0Content {
    /// Borrows the encoded content.
    pub fn as_ref(&self) -> Blp0ContentRef<'_> {
        match self {
            Self::Jpeg { shared_header } => Blp0ContentRef::Jpeg { shared_header },
            Self::Indexed { palette } => Blp0ContentRef::Indexed { palette },
        }
    }
}

impl Blp0Ref<'_> {
    /// Copies this BLP0 container for editing.
    pub fn to_owned(&self) -> Blp0 {
        Blp0 {
            header: self.header,
            content: self.content.to_owned(),
            mipmaps: self.mipmaps.map(|mip| mip.map(<[u8]>::to_vec)),
        }
    }
}

impl Blp0 {
    /// Borrows this editable BLP0 container.
    pub fn as_ref(&self) -> Blp0Ref<'_> {
        Blp0Ref {
            header: self.header,
            content: self.content.as_ref(),
            mipmaps: self.mipmaps.each_ref().map(|mip| mip.as_deref()),
        }
    }
}
