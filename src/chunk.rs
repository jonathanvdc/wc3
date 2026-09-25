/// A tagged top-level MDX chunk. The payload is stored without interpretation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chunk {
    /// Four-byte binary identifier, such as `VERS`, `MODL`, or `GEOS`.
    pub tag: [u8; 4],
    /// Original payload bytes, preserved on round-trip.
    pub data: Vec<u8>,
}

impl Chunk {
    /// Creates a chunk from its tag and payload.
    pub fn new(tag: [u8; 4], data: Vec<u8>) -> Self {
        Self { tag, data }
    }
}
