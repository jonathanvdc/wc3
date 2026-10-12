//! Sparse character customization bone transforms.
use super::super as m2;
use super::super::codec::{ArrayRef, Decoder, Encoder};
use super::super::{ReadError, WriteError, ZeroVec};
use super::{chunk_bytes, encode_records, flat_records, write_chunk, BoneMatrix};
use crate::model::common::Ownership;
#[derive(Clone, Debug, Default, PartialEq, Ownership)]
/// Sparse replacement bone transforms for one character-customization choice.
pub struct BoneFile<'data> {
    /// Bone-file format version preceding BIDA/BOMT framing; known files use 1.
    pub version: u32,
    /// BIDA indices into the owning model's bone array, paired one-to-one with `matrices`.
    pub indices: ZeroVec<'data, u16>,
    /// BOMT replacement transforms corresponding to `indices`, in row-major order with translation in the last row.
    pub matrices: Vec<BoneMatrix>,
}
impl<'data> BoneFile<'data> {
    /// Decodes the document from a bounded byte buffer.
    pub fn read(data: &'data [u8]) -> Result<Self, ReadError> {
        let mut d = Decoder::new(data, 0, 0);
        let version = d.u32()?;
        if version != 1 {
            return Err(ReadError::invalid(0, "unsupported bone-file version"));
        }
        let chunks = chunk_bytes(d.bytes(data.len() - 4)?)?;
        if chunks.len() != 2 || chunks[0].0 != *b"BIDA" || chunks[1].0 != *b"BOMT" {
            return Err(ReadError::invalid(4, "bone file requires BIDA then BOMT"));
        }
        let bytes = chunks[0].1;
        if !bytes.len().is_multiple_of(2) {
            return Err(ReadError::invalid(4, "bone indices length"));
        }
        let indices = Decoder::new(bytes, 0, 0).array(ArrayRef {
            count: (bytes.len() / 2) as u32,
            offset: 0,
        })?;
        let matrices = flat_records(chunks[1].1, 0, 0)?;
        if indices.len() != matrices.len() {
            return Err(ReadError::invalid(
                4,
                "bone indices/matrices count mismatch",
            ));
        }
        Ok(Self {
            version,
            indices,
            matrices,
        })
    }
    /// Encodes the document with freshly assigned local offsets.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        if self.version != 1 {
            return Err(WriteError::invalid("unsupported bone-file version"));
        }
        if self.indices.len() != self.matrices.len() {
            return Err(WriteError::invalid("bone indices/matrices count mismatch"));
        }
        let mut e = Encoder::new(0, 0);
        e.u32(self.version)?;
        write_chunk(&mut e, *b"BIDA", self.indices.as_bytes())?;
        write_chunk(&mut e, *b"BOMT", &encode_records(&self.matrices, 0, 0)?)?;
        Ok(e.finish())
    }
}

impl<'data> m2::Read<'data> for BoneFile<'data> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let origin = input.absolute_position();
        let remaining = input.data().len() - input.position();
        Self::read(input.bytes(remaining)?).map_err(|mut error| {
            error.offset = origin.saturating_add(error.offset);
            error
        })
    }
}
impl m2::Write for BoneFile<'_> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.write()?)
    }
}
