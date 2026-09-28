//! Binary Warcraft III MDX decoding and encoding.

mod cursor;
pub use cursor::{Cursor, Read};
mod encoder;
pub use encoder::{Encoder, SizeMarker, Write};
mod error;
pub use error::{ReadError, ValueError, WriteError};
mod fixed_text;
pub use fixed_text::FixedText;

pub use wc3_derive::{MdxRead as Read, MdxWrite as Write};
