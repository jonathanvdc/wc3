use super::{Blp0, Blp0Content, Blp0Header};
use crate::blp::{Blp1Content, Blp1Header, BlpOutput, ReadError, WriteError};
use std::borrow::Cow;

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

impl Blp0<'_> {
    /// Reads root metadata without loading external mipmaps.
    ///
    /// All mipmap slots are empty. JPEG content borrows exactly the declared
    /// shared-prefix length; trailing root padding is ignored.
    pub fn read(bytes: &[u8]) -> Result<Blp0<'_>, ReadError> {
        let (header, kind) = Blp1Header::read_version(bytes, b"BLP0")?;
        let (content, _) = Blp1Content::read_at(bytes, kind, 28)?;
        Ok(Blp0 {
            header: Blp0Header {
                width: header.width,
                height: header.height,
                alpha_bits: header.alpha_bits,
                extra: header.extra,
                has_mipmaps: header.has_mipmaps,
            },
            content: match content {
                Blp1Content::Indexed { palette } => Blp0Content::Indexed { palette },
                Blp1Content::Jpeg { shared_header } => Blp0Content::Jpeg { shared_header },
            },
            mipmaps: std::array::from_fn(|_| None),
        })
    }

    /// Serializes the root and attached external mipmap payloads.
    ///
    /// Empty slots are retained, so root-only metadata editing is supported.
    pub fn write_files(&self) -> Result<BlpOutput, WriteError> {
        let content = match &self.content {
            Blp0Content::Indexed { palette } => Blp1Content::Indexed {
                palette: palette.as_ref(),
            },
            Blp0Content::Jpeg { shared_header } => Blp1Content::Jpeg {
                shared_header: Cow::Borrowed(shared_header),
            },
        };
        let mut root = Vec::new();
        self.header
            .as_blp1()
            .write_version(&mut root, content.content_type(), b"BLP0")?;
        content.write(&mut root)?;
        Ok(BlpOutput {
            root,
            mipmaps: self
                .mipmaps
                .each_ref()
                .map(|mip| mip.as_deref().map(<[u8]>::to_vec)),
        })
    }

    /// Returns an error because BLP0 requires multiple output files.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        Err(WriteError::ExternalMipmapsRequired)
    }
}
