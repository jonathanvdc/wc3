//! Modern chunked geometry accompanying M2 models.
//!
//! The documented vertex streams begin with a format FourCC followed by packed
//! elements. Structural and collision payload layouts remain unverified and are
//! retained verbatim. This is distinct from the separate WoW Model3 format.
use super::super as m2;
use super::super::codec::{Decoder, Encoder};
use super::super::{ReadError, WriteError};
use crate::model::common::binary::{ChunkHeader, Chunks};
use crate::model::common::{Ownership, ZeroVec};
use std::borrow::Cow;

/// Packed stream elements selected by their literal format code.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum StreamValues<'data> {
    /// `1U16`: one unsigned sixteen-bit value per element.
    U16(ZeroVec<'data, u16>),
    /// `2F32`: two floating-point components per element.
    Float2(ZeroVec<'data, [f32; 2]>),
    /// `3F32`: three floating-point components per element.
    Float3(ZeroVec<'data, [f32; 3]>),
    /// `4U8N`: four unsigned normalized components, retained as native bytes.
    Normalized4(ZeroVec<'data, [u8; 4]>),
    /// An unknown format's payload, without inventing an element width.
    Raw(Cow<'data, [u8]>),
}
impl StreamValues<'_> {
    fn bytes(&self) -> &[u8] {
        match self {
            Self::U16(values) => values.as_bytes(),
            Self::Float2(values) => values.as_bytes(),
            Self::Float3(values) => values.as_bytes(),
            Self::Normalized4(values) => values.as_bytes(),
            Self::Raw(values) => values,
        }
    }
    fn format(&self) -> Option<[u8; 4]> {
        match self {
            Self::U16(_) => Some(*b"1U16"),
            Self::Float2(_) => Some(*b"2F32"),
            Self::Float3(_) => Some(*b"3F32"),
            Self::Normalized4(_) => Some(*b"4U8N"),
            Self::Raw(_) => None,
        }
    }
}

/// A format-prefixed packed vertex or index stream.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub struct VertexStream<'data> {
    /// Literal format code in wire order.
    pub format: [u8; 4],
    /// Borrowed or edited packed elements.
    pub values: StreamValues<'data>,
}
impl<'data> VertexStream<'data> {
    /// Constructs a stream, checking that known values match their format code.
    pub fn new(format: [u8; 4], values: StreamValues<'data>) -> Result<Self, WriteError> {
        let stream = Self { format, values };
        stream.check_format()?;
        Ok(stream)
    }
    fn read(data: &'data [u8], origin: usize) -> Result<Self, ReadError> {
        let format = data
            .get(..4)
            .ok_or_else(|| ReadError::invalid(origin, "geometry stream format"))?
            .try_into()
            .unwrap();
        let bytes = &data[4..];
        let bad = || ReadError::invalid(origin + 4, "geometry stream element width");
        let values = match &format {
            b"1U16" => StreamValues::U16(ZeroVec::parse_bytes(bytes).map_err(|_| bad())?),
            b"2F32" => StreamValues::Float2(ZeroVec::parse_bytes(bytes).map_err(|_| bad())?),
            b"3F32" => StreamValues::Float3(ZeroVec::parse_bytes(bytes).map_err(|_| bad())?),
            b"4U8N" => StreamValues::Normalized4(ZeroVec::parse_bytes(bytes).map_err(|_| bad())?),
            _ => StreamValues::Raw(Cow::Borrowed(bytes)),
        };
        Ok(Self { format, values })
    }
    fn check_format(&self) -> Result<(), WriteError> {
        match self.values.format() {
            Some(format) if format != self.format => {
                Err(WriteError::invalid("geometry stream format/value mismatch"))
            }
            None if matches!(&self.format, b"1U16" | b"2F32" | b"3F32" | b"4U8N") => Err(
                WriteError::invalid("known geometry stream format requires typed values"),
            ),
            _ => Ok(()),
        }
    }
}

/// A geometry chunk retaining its position and literal native payload.
#[derive(Clone, Debug, PartialEq, Ownership)]
pub enum GeometryChunk<'data> {
    /// A vertex or index stream; the tag retains the channel or UV-set identity.
    Stream {
        /// Native stream tag, such as VPOS, VUV0, or VINX.
        tag: [u8; 4],
        /// Packed values and format code.
        stream: VertexStream<'data>,
    },
    /// M3VR container-version payload; its scalar layout remains unverified.
    ContainerVersion(Cow<'data, [u8]>),
    /// VGEO geometry descriptor; payload layout remains unverified.
    Geometry(Cow<'data, [u8]>),
    /// RBAT render batches; payload layout remains unverified.
    Batches(Cow<'data, [u8]>),
    /// LODS level-of-detail table; payload layout remains unverified.
    Lods(Cow<'data, [u8]>),
    /// VSTR stream descriptor; payload layout remains unverified.
    StreamDescriptor(Cow<'data, [u8]>),
    /// SKIN profile container; payload layout remains unverified.
    Skin(Cow<'data, [u8]>),
    /// CPOS collision positions; no format prefix is assumed.
    CollisionPositions(Cow<'data, [u8]>),
    /// CNML collision normals; no format prefix is assumed.
    CollisionNormals(Cow<'data, [u8]>),
    /// CINX collision indices; no format prefix is assumed.
    CollisionIndices(Cow<'data, [u8]>),
    /// An unknown chunk retained without interpretation.
    Unknown {
        /// Literal tag bytes in wire order.
        tag: [u8; 4],
        /// Original or edited payload bytes.
        data: Cow<'data, [u8]>,
    },
}
impl GeometryChunk<'_> {
    /// Returns the literal native chunk tag.
    pub fn tag(&self) -> [u8; 4] {
        match self {
            Self::Stream { tag, .. } | Self::Unknown { tag, .. } => *tag,
            Self::ContainerVersion(_) => *b"M3VR",
            Self::Geometry(_) => *b"VGEO",
            Self::Batches(_) => *b"RBAT",
            Self::Lods(_) => *b"LODS",
            Self::StreamDescriptor(_) => *b"VSTR",
            Self::Skin(_) => *b"SKIN",
            Self::CollisionPositions(_) => *b"CPOS",
            Self::CollisionNormals(_) => *b"CNML",
            Self::CollisionIndices(_) => *b"CINX",
        }
    }
    fn raw_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Stream { .. } => None,
            Self::ContainerVersion(data)
            | Self::Geometry(data)
            | Self::Batches(data)
            | Self::Lods(data)
            | Self::StreamDescriptor(data)
            | Self::Skin(data)
            | Self::CollisionPositions(data)
            | Self::CollisionNormals(data)
            | Self::CollisionIndices(data)
            | Self::Unknown { data, .. } => Some(data),
        }
    }
}

/// An ordered modern geometry document; duplicate chunks remain distinct.
#[derive(Clone, Debug, Default, PartialEq, Ownership)]
pub struct GeometryFile<'data> {
    /// Native chunks in their original order.
    pub chunks: Vec<GeometryChunk<'data>>,
}
impl<'data> GeometryFile<'data> {
    /// Creates an empty geometry document for explicit construction.
    pub fn new() -> Self {
        Self::default()
    }
    /// Decodes bounded framing and documented stream element layouts.
    ///
    /// Structural payloads remain opaque; this does not claim complete geometry
    /// semantics or require an undocumented mandatory-chunk ordering.
    pub fn read(data: &'data [u8]) -> Result<Self, ReadError> {
        let mut chunks = Vec::new();
        for chunk in Chunks::new(data, 0) {
            let chunk = chunk
                .map_err(|error| ReadError::invalid(error.offset, "geometry chunk framing"))?;
            let tag = chunk.header.tag;
            let payload = Cow::Borrowed(chunk.data);
            let value = if stream_tag(tag) {
                GeometryChunk::Stream {
                    tag,
                    stream: VertexStream::read(chunk.data, chunk.offset)?,
                }
            } else {
                match &tag {
                    b"M3VR" => GeometryChunk::ContainerVersion(payload),
                    b"VGEO" => GeometryChunk::Geometry(payload),
                    b"RBAT" => GeometryChunk::Batches(payload),
                    b"LODS" => GeometryChunk::Lods(payload),
                    b"VSTR" => GeometryChunk::StreamDescriptor(payload),
                    b"SKIN" => GeometryChunk::Skin(payload),
                    b"CPOS" => GeometryChunk::CollisionPositions(payload),
                    b"CNML" => GeometryChunk::CollisionNormals(payload),
                    b"CINX" => GeometryChunk::CollisionIndices(payload),
                    _ => GeometryChunk::Unknown { tag, data: payload },
                }
            };
            chunks.push(value);
        }
        Ok(Self { chunks })
    }
    /// Writes native framing and packed streams without dropping opaque chunks.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        let mut result = Vec::new();
        for chunk in &self.chunks {
            let tag = chunk.tag();
            let (prefix, bytes) = match chunk {
                GeometryChunk::Stream { tag, stream } => {
                    if !stream_tag(*tag) {
                        return Err(WriteError::invalid("geometry stream tag"));
                    }
                    stream.check_format()?;
                    (Some(stream.format), stream.values.bytes())
                }
                GeometryChunk::Unknown { tag, .. } if known_tag(*tag) => {
                    return Err(WriteError::invalid("known geometry tag in opaque chunk"));
                }
                _ => (None, chunk.raw_bytes().unwrap()),
            };
            let size = bytes
                .len()
                .checked_add(if prefix.is_some() { 4 } else { 0 })
                .and_then(|size| u32::try_from(size).ok())
                .ok_or_else(|| WriteError::invalid("geometry chunk size"))?;
            result.extend_from_slice(&ChunkHeader { tag, size }.to_bytes());
            if let Some(prefix) = prefix {
                result.extend_from_slice(&prefix);
            }
            result.extend_from_slice(bytes);
        }
        Ok(result)
    }
}
fn stream_tag(tag: [u8; 4]) -> bool {
    matches!(
        &tag,
        b"VPOS"
            | b"VNML"
            | b"VTAN"
            | b"VWTS"
            | b"VIBP"
            | b"VINX"
            | b"VUV0"
            | b"VUV1"
            | b"VUV2"
            | b"VUV3"
            | b"VUV4"
            | b"VUV5"
            | b"VCL0"
            | b"VCL1"
    )
}
fn known_tag(tag: [u8; 4]) -> bool {
    stream_tag(tag)
        || matches!(
            &tag,
            b"M3VR" | b"VGEO" | b"RBAT" | b"LODS" | b"VSTR" | b"SKIN" | b"CPOS" | b"CNML" | b"CINX"
        )
}

impl<'data> m2::Read<'data> for GeometryFile<'data> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let origin = input.absolute_position();
        let remaining = input.data().len() - input.position();
        Self::read(input.bytes(remaining)?).map_err(|mut error| {
            error.offset = origin.saturating_add(error.offset);
            error
        })
    }
}
impl m2::Write for GeometryFile<'_> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.write()?)
    }
}
