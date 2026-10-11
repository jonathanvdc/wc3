//! BLP2 binary layout.
use super::{Blp2, Blp2Content, Blp2Header, DxtFormat};
use crate::blp::container::io::{
    append_u32, byte, checked_u32, palette, read_mips, slice, validate_dimensions, validate_write,
    word, write_mips,
};
use crate::blp::{ReadError, ReadErrorKind, WriteError, PALETTE_BYTES};
use std::borrow::Cow;

const REGION: usize = 148;
const END: usize = REGION + PALETTE_BYTES;
const TABLE: usize = 20;

impl Blp2Header {
    fn read(bytes: &[u8]) -> Result<(Self, u8, u8), ReadError> {
        if slice(bytes, 0, 4)? != b"BLP2" {
            return Err(ReadError::new(0, ReadErrorKind::InvalidMagic));
        }
        let version = word(bytes, 4)?;
        if version != 1 {
            return Err(ReadError::new(
                4,
                ReadErrorKind::InvalidValue {
                    field: "BLP2 version",
                    value: version,
                },
            ));
        }
        let encoding = byte(bytes, 8)?;
        let alpha_type = byte(bytes, 10)?;
        let header = Self {
            alpha_bits: byte(bytes, 9)?,
            mipmap_flags: byte(bytes, 11)?,
            width: word(bytes, 12)?,
            height: word(bytes, 16)?,
        };
        validate_dimensions(header.width, header.height)?;
        Ok((header, encoding, alpha_type))
    }

    fn write(&self, bytes: &mut Vec<u8>, encoding: u8, alpha_type: u8) -> Result<(), WriteError> {
        validate_write(self.width, self.height)?;
        bytes.extend_from_slice(b"BLP2");
        append_u32(bytes, 1);
        bytes.extend_from_slice(&[encoding, self.alpha_bits, alpha_type, self.mipmap_flags]);
        append_u32(bytes, self.width);
        append_u32(bytes, self.height);
        Ok(())
    }
}

impl<'a> Blp2Content<'a> {
    fn read(bytes: &'a [u8], encoding: u8, alpha_type: u8) -> Result<Self, ReadError> {
        let region = palette(bytes, REGION)?;
        match encoding {
            0 => {
                let size = u32::from_le_bytes(region[..4].try_into().expect("four bytes")) as usize;
                if size > PALETTE_BYTES - 4 {
                    return Err(ReadError::new(
                        REGION,
                        ReadErrorKind::InvalidValue {
                            field: "BLP2 JPEG header size",
                            value: size as u32,
                        },
                    ));
                }
                Ok(Self::Jpeg {
                    alpha_type,
                    shared_header: Cow::Borrowed(&region[4..4 + size]),
                    unused: Cow::Borrowed(&region[4 + size..]),
                })
            }
            1 => Ok(Self::Indexed {
                alpha_type,
                palette: region.into(),
            }),
            2 => Ok(Self::Dxt {
                format: match alpha_type {
                    0 => DxtFormat::Dxt1,
                    1 => DxtFormat::Dxt3,
                    7 => DxtFormat::Dxt5,
                    value => {
                        return Err(ReadError::new(
                            10,
                            ReadErrorKind::InvalidValue {
                                field: "DXT alpha type",
                                value: value.into(),
                            },
                        ));
                    }
                },
                palette_region: region.into(),
            }),
            3 | 4 => Ok(Self::Bgra {
                encoding,
                alpha_type,
                palette_region: region.into(),
            }),
            value => Err(ReadError::new(
                8,
                ReadErrorKind::InvalidValue {
                    field: "color encoding",
                    value: value.into(),
                },
            )),
        }
    }

    fn encoding(&self) -> Result<(u8, u8), WriteError> {
        match self {
            Self::Jpeg { alpha_type, .. } => Ok((0, *alpha_type)),
            Self::Indexed { alpha_type, .. } => Ok((1, *alpha_type)),
            Self::Dxt { format, .. } => Ok((
                2,
                match format {
                    DxtFormat::Dxt1 => 0,
                    DxtFormat::Dxt3 => 1,
                    DxtFormat::Dxt5 => 7,
                },
            )),
            Self::Bgra {
                encoding,
                alpha_type,
                ..
            } if *encoding == 3 || *encoding == 4 => Ok((*encoding, *alpha_type)),
            Self::Bgra { .. } => Err(WriteError::InvalidValue {
                field: "BGRA encoding",
            }),
        }
    }

    fn write(&self, bytes: &mut Vec<u8>) -> Result<(), WriteError> {
        match self {
            Self::Jpeg {
                shared_header,
                unused,
                ..
            } => {
                if shared_header.len() > PALETTE_BYTES - 4 {
                    return Err(WriteError::InvalidValue {
                        field: "BLP2 JPEG header size",
                    });
                }
                append_u32(bytes, checked_u32(shared_header.len())?);
                bytes.extend_from_slice(shared_header);
                let remaining = PALETTE_BYTES - 4 - shared_header.len();
                bytes.extend_from_slice(&unused[..unused.len().min(remaining)]);
                bytes.resize(END, 0);
            }
            Self::Indexed { palette, .. } => bytes.extend_from_slice(palette.as_bytes()),
            Self::Dxt { palette_region, .. } | Self::Bgra { palette_region, .. } => {
                bytes.extend_from_slice(palette_region.as_bytes());
            }
        }
        Ok(())
    }
}

pub(super) fn read(bytes: &[u8]) -> Result<Blp2<'_>, ReadError> {
    let (header, encoding, alpha_type) = Blp2Header::read(bytes)?;
    let content = Blp2Content::read(bytes, encoding, alpha_type)?;
    Ok(Blp2 {
        header,
        content,
        mipmaps: read_mips(bytes, TABLE, END)?,
    })
}

pub(super) fn write(value: &Blp2<'_>) -> Result<Vec<u8>, WriteError> {
    let mut bytes = Vec::new();
    let (encoding, alpha_type) = value.content.encoding()?;
    value.header.write(&mut bytes, encoding, alpha_type)?;
    bytes.resize(REGION, 0);
    value.content.write(&mut bytes)?;
    write_mips(&mut bytes, TABLE, &value.mipmaps)?;
    Ok(bytes)
}

impl Blp2<'_> {
    /// Reads a complete BLP2 file, borrowing its encoded content.
    pub fn read(bytes: &[u8]) -> Result<Blp2<'_>, ReadError> {
        read(bytes)
    }

    /// Writes a canonical layout from borrowed or owned content.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        write(self)
    }
}
