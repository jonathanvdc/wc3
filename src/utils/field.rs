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
