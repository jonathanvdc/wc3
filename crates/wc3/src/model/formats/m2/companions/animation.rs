//! External track data. Track descriptors live in the model or skeleton.
use super::super::codec::Encoder;
use super::super::records::ModelFlags;
use super::super::{ReadError, WriteError};
use super::write_chunk;
use crate::model::common::binary::Chunks;
use crate::model::common::Ownership;
use std::borrow::Cow;
#[derive(Clone, Debug, PartialEq, Ownership)]
/// One framed external animation payload whose descriptors reside in a model or skeleton.
pub struct AnimationStream<'data> {
    /// Stream identifier: AFM2 for model tracks, AFSB for skeleton bones, AFSA for attachments; unknown tags remain intact.
    pub tag: [u8; 4],
    /// Timestamp and value bytes addressed by the owning document's track descriptors.
    pub data: Cow<'data, [u8]>,
}
#[derive(Clone, Debug, PartialEq, Ownership)]
/// External animation bytes with framing explicitly selected from their owner.
pub enum AnimationFile<'data> {
    /// Unframed animation bytes; offsets in model tracks address this entire buffer.
    Raw(Cow<'data, [u8]>),
    /// Named animation streams in file order; track offsets address their selected stream payload.
    Chunked(Vec<AnimationStream<'data>>),
}
impl<'data> AnimationFile<'data> {
    /// Select framing from the owning model's flags; raw files have no reliable magic.
    pub fn read(data: &'data [u8], chunked: bool) -> Result<Self, ReadError> {
        if !chunked {
            return Ok(Self::Raw(Cow::Borrowed(data)));
        }
        Ok(Self::Chunked(
            Chunks::new(data, 0)
                .map(|chunk| {
                    let chunk = chunk.map_err(|error| {
                        ReadError::invalid(error.offset, "animation chunk framing")
                    })?;
                    Ok(AnimationStream {
                        tag: chunk.header.tag,
                        data: Cow::Borrowed(chunk.data),
                    })
                })
                .collect::<Result<_, ReadError>>()?,
        ))
    }
    /// Returns the requested animation stream without resolving track descriptors.
    pub fn stream(&self, tag: [u8; 4]) -> Option<&[u8]> {
        match self {
            Self::Raw(data) if matches!(&tag, b"AFM2" | b"AFSA" | b"AFSB") => Some(data),
            Self::Raw(_) => None,
            Self::Chunked(chunks) => chunks
                .iter()
                .find(|c| c.tag == tag)
                .map(|c| c.data.as_ref()),
        }
    }
    /// Encodes the document with freshly assigned local offsets.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        match self {
            Self::Raw(data) => Ok(data.to_vec()),
            Self::Chunked(chunks) => {
                let mut e = Encoder::new(0, ModelFlags::default());
                for c in chunks {
                    write_chunk(&mut e, c.tag, &c.data)?;
                }
                Ok(e.finish())
            }
        }
    }
}
