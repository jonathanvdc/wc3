//! Binary conversion for typed MDX records.

use crate::model::LATEST_VERSION;
use crate::Chunk;
use crate::Error;

/// A typed MDX record that can be converted to and from bytes.
pub trait Record: Sized {
    /// Parses a record.
    fn decode(bytes: &[u8], version: u32) -> Result<Self, Error>;

    /// Writes a record.
    fn encode(&self) -> Result<Vec<u8>, Error>;

    /// Parses a record, assuming the model has the latest version
    /// if it has no `VERS` chunk.
    fn decode_latest(bytes: &[u8]) -> Result<Self, Error> {
        Self::decode(bytes, LATEST_VERSION)
    }
}

/// A typed record that represents the complete payload of one MDX chunk.
/// Types for individual entries in a multi-record chunk implement `Record`
/// instead; their collection types implement `ChunkRecord`.
pub trait ChunkRecord: Record {
    /// The four-byte chunk tag for this record type.
    const TAG: [u8; 4];

    /// Encodes the record into a chunk with the appropriate tag.
    fn encode_chunk(&self) -> Result<Chunk, Error> {
        Ok(Chunk::new(Self::TAG, self.encode()?))
    }

    /// Decodes a record of this type from a chunk.
    fn decode_chunk(chunk: &Chunk) -> Result<Self, Error> {
        if chunk.tag != Self::TAG {
            return Err(Error::MalformedRecord {
                tag: chunk.tag,
                offset: 0,
            });
        }
        Self::decode(&chunk.data, LATEST_VERSION)
    }
}

#[cfg(test)]
mod tests {
    use super::Record;
    use crate::{Geoset, Model, Sequence};

    fn round_trip<T: Record + PartialEq + std::fmt::Debug>(value: &T, version: u32) {
        let bytes = value.encode().unwrap();
        assert_eq!(T::decode(&bytes, version).unwrap(), *value);
    }

    #[test]
    fn one_api_handles_model_fixed_and_versioned_records() {
        round_trip(&Model::new(800), 800);
        round_trip(&Sequence::new("Stand", [0, 100]).unwrap(), 800);
        round_trip(&Geoset::new(1800, &[], &[], &[]).unwrap(), 1800);
        assert!(Sequence::decode(&[0; 131], 800).is_err());
    }
}
