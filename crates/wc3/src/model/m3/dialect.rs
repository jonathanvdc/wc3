//! Envelope layouts controlling native section-reference widths.
use std::fmt::Debug;
mod sealed {
    pub trait Envelope {}
}
/// An M3 envelope layout; section revisions remain independently selected.
pub trait Envelope: sealed::Envelope + Clone + Copy + Debug + Default + Eq + 'static {
    /// Canonical envelope FourCC; its four bytes are reversed on disk.
    const MAGIC: [u8; 4];
    /// Native section-reference byte width.
    const REFERENCE_SIZE: usize;
}
/// Older envelope with eight-byte references and no reference flag word.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Md33;
impl sealed::Envelope for Md33 {}
impl Envelope for Md33 {
    const MAGIC: [u8; 4] = *b"MD33";
    const REFERENCE_SIZE: usize = 8;
}
/// Envelope with twelve-byte references including the reference flag word.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Md34;
impl sealed::Envelope for Md34 {}
impl Envelope for Md34 {
    const MAGIC: [u8; 4] = *b"MD34";
    const REFERENCE_SIZE: usize = 12;
}
