//! Read and write binary Warcraft III models and individual records.
//!
//! Import [`Read`] and [`Write`] as `_` for `decode_mdx()` and `encode_mdx()`.
//! Use [`crate::model::Model`] for a known version, or
//! [`crate::model::DynamicModel::decode_mdx`] to select the version from a file.
//! The runtime reader requires a fallback version for files without a version chunk.
//!
//! ```
//! use wc3::model::{Model, V800};
//! use wc3::model::mdx::{Read as _, Write as _};
//! use wc3::model::scene::ModelInfo;
//!
//! let mut model = Model::<V800>::new();
//! model.set_model_info(&ModelInfo::new("Example")?);
//! let bytes = model.encode_mdx()?;
//! let decoded = Model::<V800>::decode_mdx(&bytes)?;
//! assert_eq!(decoded.version(), 800);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Decoding preserves chunk order, unknown chunk payloads, unknown flag bits, and
//! fixed-width text bytes. Malformed known records return [`ReadError`].
//!
//! For custom codecs, [`Cursor`] reads within a byte slice and [`Encoder`] appends
//! to a buffer. `read_mdx()` consumes one value; `decode_mdx()` additionally rejects
//! trailing bytes. A record codec encodes that record, not a complete model file.
//! The [`Read`] and [`Write`] derives support fields in declaration order and
//! explicit numeric enum mappings. An `#[mdx(tag = *b"XXXX")]` field binds an
//! `Animatable<T>` or `Option<Track<T>>` to an animation tag. Sized records read
//! their fixed fields first, then dispatch the animation tail. Repeated tags
//! replace the previous animation; writers emit tracks in field order.
//! `#[mdx(flatten)]` forwards track dispatch to a nested fixed field group.
//! Unsized groups read and write their fixed fields through `Read`/`Write`, and
//! their parent handles the tail through `ReadTracks`/`WriteTracks`.

//!
//! # Standard I/O
//!
//! [`from_reader`] buffers the entire source through EOF and rejects trailing
//! input. [`to_writer`] buffers the encoded value before writing it.
//! Writers do not flush their sinks; I/O failures may leave partial output.
//!
//! ```
//! use wc3::model::{mdx, Model, V800};
//!
//! let model = Model::<V800>::new();
//! let mut output = Vec::new();
//! mdx::to_writer(&mut output, &model)?;
//! let decoded: Model<V800> = mdx::from_reader(output.as_slice())?;
//! assert_eq!(decoded.version(), 800);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod cursor;
pub use cursor::{Cursor, Read};
mod encoder;
pub use encoder::{Encoder, SizeMarker, Write};
mod error;
pub use error::{ReadError, ValueError, WriteError};
mod fixed_text;
pub use fixed_text::FixedText;

pub use wc3_derive::{MdxRead as Read, MdxValue as Value, MdxWrite as Write};

mod io;
pub use io::{from_reader, from_reader_with_version, to_writer, FromReaderError, ToWriterError};

mod tracks;
pub use tracks::{ReadTrackProperty, ReadTracks, WriteTrackProperty, WriteTracks};
