use super::PALETTE_BYTES;
use std::{borrow::Cow, ops::Deref};

/// A palette-sized byte region with borrowed or owned storage.
///
/// Construction requires exactly 1,024 bytes. Mutating borrowed storage copies
/// it once; mutating owned storage reuses its buffer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Palette<'a>(Cow<'a, [u8]>);

impl Palette<'_> {
    /// Borrows the fixed-size bytes without allocating.
    pub fn as_ref(&self) -> Palette<'_> {
        Palette(Cow::Borrowed(&self.0))
    }

    /// Returns the fixed-size bytes.
    pub fn as_bytes(&self) -> &[u8; PALETTE_BYTES] {
        self.0.as_ref().try_into().expect("fixed-size palette")
    }

    /// Returns mutable bytes, copying borrowed storage if necessary.
    pub fn to_mut(&mut self) -> &mut [u8; PALETTE_BYTES] {
        self.0
            .to_mut()
            .as_mut_slice()
            .try_into()
            .expect("fixed-size palette")
    }

    /// Moves owned bytes and copies borrowed bytes into independent storage.
    pub fn into_owned(self) -> Palette<'static> {
        Palette(Cow::Owned(self.0.into_owned()))
    }
}

impl<'a> From<&'a [u8; PALETTE_BYTES]> for Palette<'a> {
    fn from(bytes: &'a [u8; PALETTE_BYTES]) -> Self {
        Self(Cow::Borrowed(bytes))
    }
}

impl From<Box<[u8; PALETTE_BYTES]>> for Palette<'static> {
    fn from(bytes: Box<[u8; PALETTE_BYTES]>) -> Self {
        let bytes: Box<[u8]> = bytes;
        Self(Cow::Owned(bytes.into_vec()))
    }
}

impl Deref for Palette<'_> {
    type Target = [u8; PALETTE_BYTES];
    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}
