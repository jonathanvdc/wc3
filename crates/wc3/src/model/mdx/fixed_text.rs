//! Lossless, fixed-width MDX text fields.
use super::{Read, Write};
use std::borrow::Cow;

use crate::model::{Cursor, Encoder, ReadError, ValueError, WriteError};

/// A fixed-width, NUL-terminated text field that retains every source byte.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FixedText<const N: usize>([u8; N]);

impl<const N: usize> FixedText<N> {
    /// Wraps the exact bytes of a field without validating or changing them.
    pub const fn from_bytes(bytes: [u8; N]) -> Self {
        Self(bytes)
    }

    /// Returns the exact bytes, including unused bytes after the first NUL.
    pub const fn as_bytes(&self) -> &[u8; N] {
        &self.0
    }

    /// Returns text up to the first NUL, replacing invalid UTF-8.
    pub fn text(&self) -> Cow<'_, str> {
        let end = self.0.iter().position(|&byte| byte == 0).unwrap_or(N);
        String::from_utf8_lossy(&self.0[..end])
    }

    /// Sets text and clears the rest of the field.
    pub fn set_text(&mut self, value: &str) -> Result<(), ValueError> {
        if value.len() >= N || value.as_bytes().contains(&0) {
            return Err(ValueError::InvalidString {
                max_bytes: N.saturating_sub(1),
            });
        }
        self.0.fill(0);
        self.0[..value.len()].copy_from_slice(value.as_bytes());
        Ok(())
    }
}

impl<const N: usize> Default for FixedText<N> {
    fn default() -> Self {
        Self([0; N])
    }
}

impl<const N: usize> Read for FixedText<N> {
    fn read_mdx(cursor: &mut Cursor<'_>) -> Result<Self, ReadError> {
        Ok(Self(
            cursor.read_bytes(N)?.try_into().expect("fixed-width text"),
        ))
    }
}

impl<const N: usize> Write for FixedText<N> {
    fn write_mdx(&self, encoder: &mut Encoder<'_>) -> Result<(), WriteError> {
        encoder.write(self.0.as_slice())?;
        Ok(())
    }
}
