//! Borrowed and owned BLP1 conversions.
use super::{Blp1, Blp1Content, Blp1ContentRef, Blp1Ref};

impl Blp1ContentRef<'_> {
    /// Copies the encoded content for editing.
    pub fn to_owned(&self) -> Blp1Content {
        match *self {
            Self::Jpeg { shared_header } => Blp1Content::Jpeg {
                shared_header: shared_header.to_vec(),
            },
            Self::Indexed { palette } => Blp1Content::Indexed {
                palette: Box::new(*palette),
            },
        }
    }
}

impl Blp1Content {
    /// Borrows the encoded content.
    pub fn as_ref(&self) -> Blp1ContentRef<'_> {
        match self {
            Self::Jpeg { shared_header } => Blp1ContentRef::Jpeg { shared_header },
            Self::Indexed { palette } => Blp1ContentRef::Indexed { palette },
        }
    }
}

impl Blp1Ref<'_> {
    /// Copies this BLP1 container for editing.
    pub fn to_owned(&self) -> Blp1 {
        Blp1 {
            header: self.header,
            content: self.content.to_owned(),
            mipmaps: self.mipmaps.map(|mip| mip.map(<[u8]>::to_vec)),
        }
    }
}

impl Blp1 {
    /// Borrows this editable BLP1 container.
    pub fn as_ref(&self) -> Blp1Ref<'_> {
        Blp1Ref {
            header: self.header,
            content: self.content.as_ref(),
            mipmaps: self.mipmaps.each_ref().map(|mip| mip.as_deref()),
        }
    }
}
