//! Fixed-width fields shared by MDX record implementations.

use std::borrow::Cow;

use crate::Error;

pub(crate) fn text(field: &[u8]) -> Cow<'_, str> {
    let end = field
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(field.len());
    String::from_utf8_lossy(&field[..end])
}

pub(crate) fn set_text(field: &mut [u8], value: &str) -> Result<(), Error> {
    if value.len() >= field.len() || value.as_bytes().contains(&0) {
        return Err(Error::InvalidString {
            max_bytes: field.len() - 1,
        });
    }
    field.fill(0);
    field[..value.len()].copy_from_slice(value.as_bytes());
    Ok(())
}

pub(crate) fn f32_at(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("four-byte field"),
    )
}

pub(crate) fn set_f32_at(bytes: &mut [u8], offset: usize, value: f32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub(crate) fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("four-byte field"),
    )
}

pub(crate) fn set_u32_at(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
