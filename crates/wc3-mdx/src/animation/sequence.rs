//! Animation sequence records in the `SEQS` chunk.
use crate::EncodeError;
use crate::Encoder;
use crate::ModelVersion;
use crate::ValueError;
use crate::Vec3;

use crate::SequencesChunk;
use crate::{Encodable, Readable, Writable};
use std::borrow::Cow;

use crate::FixedText;
use crate::Model;

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
const NAME_SIZE: usize = 80;

/// A fixed-size animation sequence, including reserved fields.
#[derive(Clone, Debug, PartialEq, Readable, Writable)]
pub struct Sequence {
    name: FixedText<NAME_SIZE>,
    interval: [u32; 2],
    move_speed: f32,
    flags: u32,
    rarity: f32,
    sync_point: u32,
    bounds_radius: f32,
    minimum_extent: Vec3,
    maximum_extent: Vec3,
}

impl Sequence {
    pub fn new(name: &str, interval: [u32; 2]) -> Result<Self, ValueError> {
        let mut sequence = Self {
            name: FixedText::default(),
            interval,
            move_speed: 0.0,
            flags: 0,
            rarity: 0.0,
            sync_point: 0,
            bounds_radius: 0.0,
            minimum_extent: [0.0; 3],
            maximum_extent: [0.0; 3],
        };
        sequence.set_name(name)?;
        Ok(sequence)
    }

    pub fn name(&self) -> Cow<'_, str> {
        self.name.text()
    }
    pub fn set_name(&mut self, name: &str) -> Result<(), ValueError> {
        self.name.set_text(name)
    }
    pub fn interval(&self) -> [u32; 2] {
        self.interval
    }
    pub fn set_interval(&mut self, interval: [u32; 2]) {
        self.interval = interval;
    }
    pub fn move_speed(&self) -> f32 {
        self.move_speed
    }
    pub fn set_move_speed(&mut self, speed: f32) {
        self.move_speed = speed;
    }
    pub fn flags(&self) -> SequenceFlags {
        SequenceFlags::from_bits(self.flags)
    }
    pub fn raw_flags(&self) -> u32 {
        self.flags
    }
    pub fn set_flags(&mut self, flags: SequenceFlags) {
        self.flags = flags.bits();
    }
    pub fn set_raw_flags(&mut self, flags: u32) {
        self.flags = flags;
    }
    pub fn rarity(&self) -> f32 {
        self.rarity
    }
    pub fn set_rarity(&mut self, rarity: f32) {
        self.rarity = rarity;
    }
    pub fn sync_point(&self) -> u32 {
        self.sync_point
    }
    pub fn set_sync_point(&mut self, point: u32) {
        self.sync_point = point;
    }
    pub fn bounds_radius(&self) -> f32 {
        self.bounds_radius
    }
    pub fn set_bounds_radius(&mut self, radius: f32) {
        self.bounds_radius = radius;
    }
    pub fn minimum_extent(&self) -> Vec3 {
        self.minimum_extent
    }
    pub fn set_minimum_extent(&mut self, extent: Vec3) {
        self.minimum_extent = extent;
    }
    pub fn maximum_extent(&self) -> Vec3 {
        self.maximum_extent
    }
    pub fn set_maximum_extent(&mut self, extent: Vec3) {
        self.maximum_extent = extent;
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes every `SEQS` chunk in file order.
    pub fn sequences(&self) -> Vec<Sequence> {
        self.collect_chunk_records::<SequencesChunk>()
    }

    /// Writes all sequences to the first `SEQS` chunk, creating it if needed.
    /// Additional `SEQS` chunks are removed after their records are replaced.
    pub fn set_sequences(&mut self, sequences: &[Sequence]) {
        self.replace_chunk(SequencesChunk::new(sequences.to_vec()));
    }
}

impl Encodable for Sequence {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        bytes.write(self);
        Ok(())
    }
}
