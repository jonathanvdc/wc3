//! Lossless, fixed-width MDX text fields.
use super::{Read, ReadOwned, Write};
use crate::model::formats::mdlx::mdx;
use crate::model::formats::mdlx::{Cursor, Encoder, ValueError};
use std::borrow::Cow;

/// A fixed-width text field that retains every source byte.
///
/// New text is NUL-terminated, but decoded or explicitly supplied bytes may
/// be unterminated, contain invalid UTF-8, or have nonzero padding.
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

    /// Sets UTF-8 text and clears the rest of the field.
    /// The text must contain no NUL and occupy fewer than `N` bytes so a
    /// terminator fits. Invalid input leaves the field unchanged.
    pub fn set_text(&mut self, value: &str) -> Result<(), ValueError> {
        if value.len() >= N || value.as_bytes().contains(&0) {
            return Err(ValueError::InvalidString {
                max_bytes: N.saturating_sub(1),
            });
        }
        self.set_bytes(value.as_bytes())
    }

    /// Sets byte text without requiring UTF-8 and clears unused field bytes.
    ///
    /// At most `N` bytes are accepted, without embedded NUL. Shorter values
    /// receive a NUL terminator; full-width values remain unterminated, as is
    /// permitted for decoded native fields. Invalid input leaves the field unchanged.
    pub fn set_bytes(&mut self, value: &[u8]) -> Result<(), ValueError> {
        if value.len() > N || value.contains(&0) {
            return Err(ValueError::InvalidString { max_bytes: N });
        }
        self.0.fill(0);
        self.0[..value.len()].copy_from_slice(value);
        Ok(())
    }
}

impl<const N: usize> Default for FixedText<N> {
    fn default() -> Self {
        Self([0; N])
    }
}

impl<'data, const N: usize> Read<'data> for FixedText<N> {
    fn read_mdx(cursor: &mut Cursor<'data>) -> Result<Self, mdx::ReadError> {
        Ok(Self(
            cursor.read_bytes(N)?.try_into().expect("fixed-width text"),
        ))
    }
}

impl<const N: usize> Write for FixedText<N> {
    fn write_mdx(&self, encoder: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        encoder.write(self.0.as_slice())?;
        Ok(())
    }
}

impl<const N: usize> ReadOwned for FixedText<N> {
    fn decode_owned(bytes: &[u8]) -> Result<Self, mdx::ReadError> {
        Self::decode_mdx(bytes)
    }
}
