//! Animation sequence records in the `SEQS` chunk.

use std::borrow::Cow;

use crate::{Error, Model};

const TAG: [u8; 4] = *b"SEQS";
const SIZE: usize = 132;
const NAME_SIZE: usize = 80;

/// A fixed-size animation sequence, including reserved fields and raw float bits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sequence {
    bytes: [u8; SIZE],
}

impl Sequence {
    /// Creates a zero-initialized sequence with a name and frame interval.
    pub fn new(name: &str, interval: [u32; 2]) -> Result<Self, Error> {
        let mut sequence = Self { bytes: [0; SIZE] };
        sequence.set_name(name)?;
        sequence.set_interval(interval);
        Ok(sequence)
    }

    fn parse(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.try_into().expect("fixed-size record"),
        }
    }

    /// Returns the original 132-byte record.
    pub fn as_bytes(&self) -> &[u8; SIZE] {
        &self.bytes
    }

    /// Returns the name up to the first NUL, replacing invalid UTF-8.
    pub fn name(&self) -> Cow<'_, str> {
        let end = self.bytes[..NAME_SIZE]
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(NAME_SIZE);
        String::from_utf8_lossy(&self.bytes[..end])
    }

    /// Replaces the name without disturbing other sequence fields.
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        if name.len() >= NAME_SIZE || name.as_bytes().contains(&0) {
            return Err(Error::InvalidString {
                max_bytes: NAME_SIZE - 1,
            });
        }
        self.bytes[..NAME_SIZE].fill(0);
        self.bytes[..name.len()].copy_from_slice(name.as_bytes());
        Ok(())
    }

    /// Returns the start and end frame times.
    pub fn interval(&self) -> [u32; 2] {
        [self.u32_at(80), self.u32_at(84)]
    }

    /// Sets the start and end frame times.
    pub fn set_interval(&mut self, interval: [u32; 2]) {
        self.set_u32_at(80, interval[0]);
        self.set_u32_at(84, interval[1]);
    }

    /// Returns the ground movement speed.
    pub fn move_speed(&self) -> f32 {
        f32::from_bits(self.u32_at(88))
    }

    /// Sets the ground movement speed.
    pub fn set_move_speed(&mut self, speed: f32) {
        self.set_u32_at(88, speed.to_bits());
    }

    /// Returns the raw sequence flags. Bit 0 marks a non-looping sequence.
    pub fn flags(&self) -> u32 {
        self.u32_at(92)
    }

    /// Sets the raw sequence flags.
    pub fn set_flags(&mut self, flags: u32) {
        self.set_u32_at(92, flags);
    }

    /// Returns the sequence rarity.
    pub fn rarity(&self) -> f32 {
        f32::from_bits(self.u32_at(96))
    }

    /// Sets the sequence rarity.
    pub fn set_rarity(&mut self, rarity: f32) {
        self.set_u32_at(96, rarity.to_bits());
    }

    /// Returns the raw sync point field.
    pub fn sync_point(&self) -> u32 {
        self.u32_at(100)
    }

    /// Sets the raw sync point field.
    pub fn set_sync_point(&mut self, point: u32) {
        self.set_u32_at(100, point);
    }

    /// Returns the bounding sphere radius.
    pub fn bounds_radius(&self) -> f32 {
        f32::from_bits(self.u32_at(104))
    }

    /// Sets the bounding sphere radius.
    pub fn set_bounds_radius(&mut self, radius: f32) {
        self.set_u32_at(104, radius.to_bits());
    }

    /// Returns the minimum XYZ extent.
    pub fn minimum_extent(&self) -> [f32; 3] {
        self.extent_at(108)
    }

    /// Sets the minimum XYZ extent.
    pub fn set_minimum_extent(&mut self, extent: [f32; 3]) {
        self.set_extent_at(108, extent);
    }

    /// Returns the maximum XYZ extent.
    pub fn maximum_extent(&self) -> [f32; 3] {
        self.extent_at(120)
    }

    /// Sets the maximum XYZ extent.
    pub fn set_maximum_extent(&mut self, extent: [f32; 3]) {
        self.set_extent_at(120, extent);
    }

    fn u32_at(&self, offset: usize) -> u32 {
        u32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("fixed-size field"),
        )
    }

    fn set_u32_at(&mut self, offset: usize, value: u32) {
        self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn extent_at(&self, offset: usize) -> [f32; 3] {
        std::array::from_fn(|index| f32::from_bits(self.u32_at(offset + index * 4)))
    }

    fn set_extent_at(&mut self, offset: usize, extent: [f32; 3]) {
        for (index, value) in extent.into_iter().enumerate() {
            self.set_u32_at(offset + index * 4, value.to_bits());
        }
    }
}

impl Model {
    /// Decodes every `SEQS` chunk in file order.
    pub fn sequences(&self) -> Result<Vec<Sequence>, Error> {
        let mut sequences = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            if chunk.data.len() % SIZE != 0 {
                return Err(Error::MalformedChunk {
                    tag: TAG,
                    size: chunk.data.len(),
                    expected: SIZE,
                });
            }
            sequences.extend(chunk.data.chunks_exact(SIZE).map(Sequence::parse));
        }
        Ok(sequences)
    }

    /// Writes all sequences to the first `SEQS` chunk, creating it if needed.
    /// Additional `SEQS` chunks are removed after their records are replaced.
    pub fn set_sequences(&mut self, sequences: &[Sequence]) -> Result<(), Error> {
        let size = sequences
            .len()
            .checked_mul(SIZE)
            .ok_or(Error::ChunkTooLarge {
                tag: TAG,
                size: usize::MAX,
            })?;
        if size > u32::MAX as usize {
            return Err(Error::ChunkTooLarge { tag: TAG, size });
        }
        let mut data = Vec::with_capacity(size);
        for sequence in sequences {
            data.extend_from_slice(sequence.as_bytes());
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
