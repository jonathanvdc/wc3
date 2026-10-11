use super::{Blp0, Blp0ContentRef, Blp0Header, Blp0Ref};
use crate::blp::{Blp1ContentRef, Blp1Header, BlpOutput, ReadError, WriteError, MIPMAP_SLOTS};

impl Blp0Header {
    fn as_blp1(self) -> Blp1Header {
        Blp1Header {
            width: self.width,
            height: self.height,
            alpha_bits: self.alpha_bits,
            extra: self.extra,
            has_mipmaps: self.has_mipmaps,
        }
    }
}

impl Blp0Ref<'_> {
    /// Reads root metadata without loading external mipmaps.
    ///
    /// All mipmap slots are empty. JPEG content borrows exactly the declared
    /// shared-prefix length; trailing root padding is ignored.
    pub fn read(bytes: &[u8]) -> Result<Blp0Ref<'_>, ReadError> {
        let (header, kind) = Blp1Header::read_version(bytes, b"BLP0")?;
        let (content, _) = Blp1ContentRef::read_at(bytes, kind, 28)?;
        Ok(Blp0Ref {
            header: Blp0Header {
                width: header.width,
                height: header.height,
                alpha_bits: header.alpha_bits,
                extra: header.extra,
                has_mipmaps: header.has_mipmaps,
            },
            content: match content {
                Blp1ContentRef::Indexed { palette } => Blp0ContentRef::Indexed { palette },
                Blp1ContentRef::Jpeg { shared_header } => Blp0ContentRef::Jpeg { shared_header },
            },
            mipmaps: [None; MIPMAP_SLOTS],
        })
    }

    /// Serializes the root and attached external mipmap payloads.
    ///
    /// Empty slots are retained, so root-only metadata editing is supported.
    pub fn write_files(&self) -> Result<BlpOutput, WriteError> {
        let content = match self.content {
            Blp0ContentRef::Indexed { palette } => Blp1ContentRef::Indexed { palette },
            Blp0ContentRef::Jpeg { shared_header } => Blp1ContentRef::Jpeg { shared_header },
        };
        let mut root = Vec::new();
        self.header
            .as_blp1()
            .write_version(&mut root, content.content_type(), b"BLP0")?;
        content.write(&mut root)?;
        Ok(BlpOutput {
            root,
            mipmaps: self.mipmaps.map(|mip| mip.map(<[u8]>::to_vec)),
        })
    }

    /// Returns an error because BLP0 requires multiple output files.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        Err(WriteError::ExternalMipmapsRequired)
    }
}

impl Blp0 {
    /// Serializes the root and attached external mipmap payloads.
    ///
    /// Empty slots are retained, so root-only metadata editing is supported.
    pub fn write_files(&self) -> Result<BlpOutput, WriteError> {
        self.as_ref().write_files()
    }
    /// Returns an error because BLP0 requires multiple output files.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        self.as_ref().write()
    }
}
