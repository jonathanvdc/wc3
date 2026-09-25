//! Animation sequence records in the `SEQS` chunk.

use crate::Record;
use std::borrow::Cow;

use crate::utils::field;
use crate::{Error, Model};

/// Sequence playback flags, with unrecognized bits retained.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SequenceFlags(u32);

impl SequenceFlags {
    pub const NON_LOOPING: Self = Self(1);
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
    pub const fn bits(self) -> u32 {
        self.0
    }
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    pub fn set(&mut self, other: Self, enabled: bool) {
        if enabled {
            self.0 |= other.0;
        } else {
            self.0 &= !other.0;
        }
    }
}
pub(crate) const SIZE: usize = 132;
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

    /// Returns the original 132-byte record.
    pub fn as_bytes(&self) -> &[u8; SIZE] {
        &self.bytes
    }

    /// Returns the name up to the first NUL, replacing invalid UTF-8.
    pub fn name(&self) -> Cow<'_, str> {
        field::text(&self.bytes[..NAME_SIZE])
    }

    /// Replaces the name without disturbing other sequence fields.
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        field::set_text(&mut self.bytes[..NAME_SIZE], name)
    }

    /// Returns the start and end frame times.
    pub fn interval(&self) -> [u32; 2] {
        [
            field::u32_at(&self.bytes, 80),
            field::u32_at(&self.bytes, 84),
        ]
    }

    /// Sets the start and end frame times.
    pub fn set_interval(&mut self, interval: [u32; 2]) {
        field::set_u32_at(&mut self.bytes, 80, interval[0]);
        field::set_u32_at(&mut self.bytes, 84, interval[1]);
    }

    /// Returns the ground movement speed.
    pub fn move_speed(&self) -> f32 {
        f32::from_bits(field::u32_at(&self.bytes, 88))
    }

    /// Sets the ground movement speed.
    pub fn set_move_speed(&mut self, speed: f32) {
        field::set_u32_at(&mut self.bytes, 88, speed.to_bits());
    }

    /// Returns decoded sequence playback flags.
    pub fn flags(&self) -> SequenceFlags {
        SequenceFlags::from_bits(self.raw_flags())
    }

    /// Returns exact raw sequence flag bits.
    pub fn raw_flags(&self) -> u32 {
        field::u32_at(&self.bytes, 92)
    }

    /// Sets decoded sequence playback flags.
    pub fn set_flags(&mut self, flags: SequenceFlags) {
        self.set_raw_flags(flags.bits());
    }

    /// Sets exact raw sequence flag bits.
    pub fn set_raw_flags(&mut self, flags: u32) {
        field::set_u32_at(&mut self.bytes, 92, flags);
    }

    /// Returns the sequence rarity.
    pub fn rarity(&self) -> f32 {
        f32::from_bits(field::u32_at(&self.bytes, 96))
    }

    /// Sets the sequence rarity.
    pub fn set_rarity(&mut self, rarity: f32) {
        field::set_u32_at(&mut self.bytes, 96, rarity.to_bits());
    }

    /// Returns the raw sync point field.
    pub fn sync_point(&self) -> u32 {
        field::u32_at(&self.bytes, 100)
    }

    /// Sets the raw sync point field.
    pub fn set_sync_point(&mut self, point: u32) {
        field::set_u32_at(&mut self.bytes, 100, point);
    }

    /// Returns the bounding sphere radius.
    pub fn bounds_radius(&self) -> f32 {
        f32::from_bits(field::u32_at(&self.bytes, 104))
    }

    /// Sets the bounding sphere radius.
    pub fn set_bounds_radius(&mut self, radius: f32) {
        field::set_u32_at(&mut self.bytes, 104, radius.to_bits());
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

    fn extent_at(&self, offset: usize) -> [f32; 3] {
        std::array::from_fn(|index| f32::from_bits(field::u32_at(&self.bytes, offset + index * 4)))
    }

    fn set_extent_at(&mut self, offset: usize, extent: [f32; 3]) {
        for (index, value) in extent.into_iter().enumerate() {
            field::set_u32_at(&mut self.bytes, offset + index * 4, value.to_bits());
        }
    }
}

impl Model {
    /// Decodes every `SEQS` chunk in file order.
    pub fn sequences(&self) -> Result<Vec<Sequence>, Error> {
        self.collect_chunk_records::<crate::SequencesChunk>(|chunk| match chunk {
            crate::ModelChunk::Sequences(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Writes all sequences to the first `SEQS` chunk, creating it if needed.
    /// Additional `SEQS` chunks are removed after their records are replaced.
    pub fn set_sequences(&mut self, sequences: &[Sequence]) -> Result<(), Error> {
        let size = sequences
            .len()
            .checked_mul(SIZE)
            .ok_or(Error::ChunkTooLarge {
                tag: Sequence::TAG,
                size: usize::MAX,
            })?;
        if size > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: Sequence::TAG,
                size,
            });
        }
        let mut data = Vec::with_capacity(size);
        for sequence in sequences {
            data.extend_from_slice(sequence.as_bytes());
        }
        self.replace_raw_chunk(Sequence::TAG, data)?;
        Ok(())
    }
}

impl Record for Sequence {
    fn decode_one(cursor: &mut crate::Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let size = cursor.remaining().len();
        let bytes = cursor
            .read_exact(SIZE)
            .map_err(|_| Error::MalformedChunk {
                tag: Self::TAG,
                size,
                expected: SIZE,
            })?
            .try_into()
            .expect("fixed-width record");
        Ok(Self { bytes })
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        Ok(self.bytes.to_vec())
    }
}

impl Sequence {
    /// The tag of the chunk containing this record.
    pub const TAG: [u8; 4] = *b"SEQS";
}
