//! BLP1 binary layout.
use super::{Blp1, Blp1Content, Blp1Header};
use crate::blp::container::io::{
    append_u32, checked_u32, palette, read_mips, slice, validate_dimensions, validate_write, word,
    write_mips,
};
use crate::blp::{ReadError, ReadErrorKind, WriteError, PALETTE_BYTES};
use std::borrow::Cow;

const TABLE: usize = 28;
const CONTENT: usize = 156;

impl Blp1Header {
    pub(super) fn read(bytes: &[u8]) -> Result<(Self, u32), ReadError> {
        Self::read_version(bytes, b"BLP1")
    }

    pub(crate) fn read_version(bytes: &[u8], magic: &[u8; 4]) -> Result<(Self, u32), ReadError> {
        if slice(bytes, 0, 4)? != magic {
            return Err(ReadError::new(0, ReadErrorKind::InvalidMagic));
        }
        let content_type = word(bytes, 4)?;
        let header = Self {
            alpha_bits: word(bytes, 8)?,
            width: word(bytes, 12)?,
            height: word(bytes, 16)?,
            extra: word(bytes, 20)?,
            has_mipmaps: word(bytes, 24)?,
        };
        validate_dimensions(header.width, header.height)?;
        Ok((header, content_type))
    }

    fn write(&self, bytes: &mut Vec<u8>, content_type: u32) -> Result<(), WriteError> {
        self.write_version(bytes, content_type, b"BLP1")
    }

    pub(crate) fn write_version(
        &self,
        bytes: &mut Vec<u8>,
        content_type: u32,
        magic: &[u8; 4],
    ) -> Result<(), WriteError> {
        validate_write(self.width, self.height)?;
        bytes.extend_from_slice(magic);
        append_u32(bytes, content_type);
        append_u32(bytes, self.alpha_bits);
        append_u32(bytes, self.width);
        append_u32(bytes, self.height);
        append_u32(bytes, self.extra);
        append_u32(bytes, self.has_mipmaps);
        Ok(())
    }
}

impl<'a> Blp1Content<'a> {
    fn read(bytes: &'a [u8], content_type: u32) -> Result<(Self, usize), ReadError> {
        Self::read_at(bytes, content_type, CONTENT)
    }

    pub(crate) fn read_at(
        bytes: &'a [u8],
        content_type: u32,
        start: usize,
    ) -> Result<(Self, usize), ReadError> {
        match content_type {
            0 => {
                let size = word(bytes, start)? as usize;
                let shared_header = slice(bytes, start + 4, size)?;
                let end = (start + 4)
                    .checked_add(size)
                    .ok_or(ReadError::new(start, ReadErrorKind::UnexpectedEnd))?;
                Ok((
                    Self::Jpeg {
                        shared_header: Cow::Borrowed(shared_header),
                    },
                    end,
                ))
            }
            1 => Ok((
                Self::Indexed {
                    palette: palette(bytes, start)?.into(),
                },
                start + PALETTE_BYTES,
            )),
            value => Err(ReadError::new(
                4,
                ReadErrorKind::InvalidValue {
                    field: "content type",
                    value,
                },
            )),
        }
    }

    pub(crate) fn content_type(&self) -> u32 {
        match self {
            Self::Jpeg { .. } => 0,
            Self::Indexed { .. } => 1,
        }
    }

    pub(crate) fn write(&self, bytes: &mut Vec<u8>) -> Result<(), WriteError> {
        match self {
            Self::Jpeg { shared_header } => {
                append_u32(bytes, checked_u32(shared_header.len())?);
                bytes.extend_from_slice(shared_header);
            }
            Self::Indexed { palette } => bytes.extend_from_slice(palette.as_bytes()),
        }
        Ok(())
    }
}

pub(super) fn read(bytes: &[u8]) -> Result<Blp1<'_>, ReadError> {
    let (header, content_type) = Blp1Header::read(bytes)?;
    slice(bytes, TABLE, 128)?;
    let (content, content_end) = Blp1Content::read(bytes, content_type)?;
    Ok(Blp1 {
        header,
        content,
        mipmaps: read_mips(bytes, TABLE, content_end)?,
    })
}

pub(super) fn write(value: &Blp1<'_>) -> Result<Vec<u8>, WriteError> {
    let mut bytes = Vec::new();
    value
        .header
        .write(&mut bytes, value.content.content_type())?;
    bytes.resize(CONTENT, 0);
    value.content.write(&mut bytes)?;
    write_mips(&mut bytes, TABLE, &value.mipmaps)?;
    Ok(bytes)
}

impl Blp1<'_> {
    /// Reads a complete BLP1 file.
    pub fn read(bytes: &[u8]) -> Result<Blp1<'_>, ReadError> {
        read(bytes)
    }

    /// Writes this borrowed container.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        write(self)
    }
}
