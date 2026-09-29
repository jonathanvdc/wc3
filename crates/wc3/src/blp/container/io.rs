//! Shared checked byte and mipmap table operations.
use crate::blp::{ReadError, ReadErrorKind, WriteError, MIPMAP_SLOTS, PALETTE_BYTES};

pub(crate) fn slice(bytes: &[u8], start: usize, len: usize) -> Result<&[u8], ReadError> {
    let end = start
        .checked_add(len)
        .ok_or(ReadError::new(start, ReadErrorKind::UnexpectedEnd))?;
    bytes
        .get(start..end)
        .ok_or(ReadError::new(start, ReadErrorKind::UnexpectedEnd))
}

pub(super) fn word(bytes: &[u8], start: usize) -> Result<u32, ReadError> {
    Ok(u32::from_le_bytes(
        slice(bytes, start, 4)?.try_into().expect("four bytes"),
    ))
}

pub(super) fn byte(bytes: &[u8], start: usize) -> Result<u8, ReadError> {
    Ok(slice(bytes, start, 1)?[0])
}

pub(super) fn palette(bytes: &[u8], start: usize) -> Result<&[u8; PALETTE_BYTES], ReadError> {
    Ok(slice(bytes, start, PALETTE_BYTES)?
        .try_into()
        .expect("palette length"))
}

pub(super) fn read_mips(
    bytes: &[u8],
    table: usize,
    content_end: usize,
) -> Result<[Option<&[u8]>; MIPMAP_SLOTS], ReadError> {
    let mut mips = [None; MIPMAP_SLOTS];
    for (level, slot) in mips.iter_mut().enumerate() {
        let offset_position = table + level * 4;
        let size_position = table + 64 + level * 4;
        let offset = word(bytes, offset_position)? as usize;
        let size = word(bytes, size_position)? as usize;
        if offset == 0 && size == 0 {
            continue;
        }
        if offset == 0 || size == 0 || offset < content_end {
            return Err(ReadError::new(
                offset_position,
                ReadErrorKind::InvalidRange { level },
            ));
        }
        *slot =
            Some(slice(bytes, offset, size).map_err(|_| {
                ReadError::new(offset_position, ReadErrorKind::InvalidRange { level })
            })?);
    }
    if mips[0].is_none() {
        return Err(ReadError::new(
            table,
            ReadErrorKind::MissingMipmap { level: 0 },
        ));
    }
    Ok(mips)
}

pub(super) fn validate_dimensions(width: u32, height: u32) -> Result<(), ReadError> {
    if width == 0 {
        return Err(ReadError::new(
            12,
            ReadErrorKind::InvalidValue {
                field: "width",
                value: width,
            },
        ));
    }
    if height == 0 {
        return Err(ReadError::new(
            16,
            ReadErrorKind::InvalidValue {
                field: "height",
                value: height,
            },
        ));
    }
    Ok(())
}

pub(super) fn checked_u32(value: usize) -> Result<u32, WriteError> {
    u32::try_from(value).map_err(|_| WriteError::SizeOverflow)
}

pub(super) fn append_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub(super) fn write_mips(
    bytes: &mut Vec<u8>,
    table: usize,
    mips: &[Option<&[u8]>; MIPMAP_SLOTS],
) -> Result<(), WriteError> {
    if mips[0].is_none() {
        return Err(WriteError::MissingMipmap { level: 0 });
    }
    for (level, mip) in mips.iter().enumerate() {
        if let Some(mip) = mip {
            if mip.is_empty() {
                return Err(WriteError::InvalidValue {
                    field: "empty mipmap",
                });
            }
            let offset = checked_u32(bytes.len())?;
            let size = checked_u32(mip.len())?;
            let end = bytes
                .len()
                .checked_add(mip.len())
                .ok_or(WriteError::SizeOverflow)?;
            checked_u32(end)?;
            bytes[table + level * 4..table + level * 4 + 4].copy_from_slice(&offset.to_le_bytes());
            bytes[table + 64 + level * 4..table + 64 + level * 4 + 4]
                .copy_from_slice(&size.to_le_bytes());
            bytes.extend_from_slice(mip);
        }
    }
    Ok(())
}

pub(super) fn validate_write(width: u32, height: u32) -> Result<(), WriteError> {
    if width == 0 || height == 0 {
        return Err(WriteError::InvalidValue {
            field: "dimensions",
        });
    }
    Ok(())
}
