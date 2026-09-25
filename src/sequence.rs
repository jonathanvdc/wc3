//! Animation sequence records in the `SEQS` chunk.
use crate::Encoder;
use crate::{Tag, Vec3};

use crate::Record;
use crate::{Cursor, ModelChunk, SequencesChunk};
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
    name: [u8; NAME_SIZE],
    interval: [u32; 2],
    move_speed: u32,
    flags: u32,
    rarity: u32,
    sync_point: u32,
    bounds_radius: u32,
    minimum_extent: [u32; 3],
    maximum_extent: [u32; 3],
}

impl Sequence {
    pub fn new(name: &str, interval: [u32; 2]) -> Result<Self, Error> {
        let mut sequence = Self {
            name: [0; NAME_SIZE],
            interval,
            move_speed: 0,
            flags: 0,
            rarity: 0,
            sync_point: 0,
            bounds_radius: 0,
            minimum_extent: [0; 3],
            maximum_extent: [0; 3],
        };
        sequence.set_name(name)?;
        Ok(sequence)
    }
    pub fn as_bytes(&self) -> [u8; SIZE] {
        self.encode()
            .expect("fixed-size record")
            .try_into()
            .expect("fixed-size record")
    }
    pub fn name(&self) -> Cow<'_, str> {
        field::text(&self.name)
    }
    pub fn set_name(&mut self, name: &str) -> Result<(), Error> {
        field::set_text(&mut self.name, name)
    }
    pub fn interval(&self) -> [u32; 2] {
        self.interval
    }
    pub fn set_interval(&mut self, interval: [u32; 2]) {
        self.interval = interval;
    }
    pub fn move_speed(&self) -> f32 {
        f32::from_bits(self.move_speed)
    }
    pub fn set_move_speed(&mut self, speed: f32) {
        self.move_speed = speed.to_bits();
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
        f32::from_bits(self.rarity)
    }
    pub fn set_rarity(&mut self, rarity: f32) {
        self.rarity = rarity.to_bits();
    }
    pub fn sync_point(&self) -> u32 {
        self.sync_point
    }
    pub fn set_sync_point(&mut self, point: u32) {
        self.sync_point = point;
    }
    pub fn bounds_radius(&self) -> f32 {
        f32::from_bits(self.bounds_radius)
    }
    pub fn set_bounds_radius(&mut self, radius: f32) {
        self.bounds_radius = radius.to_bits();
    }
    pub fn minimum_extent(&self) -> Vec3 {
        self.minimum_extent.map(f32::from_bits)
    }
    pub fn set_minimum_extent(&mut self, extent: Vec3) {
        self.minimum_extent = extent.map(f32::to_bits);
    }
    pub fn maximum_extent(&self) -> Vec3 {
        self.maximum_extent.map(f32::from_bits)
    }
    pub fn set_maximum_extent(&mut self, extent: Vec3) {
        self.maximum_extent = extent.map(f32::to_bits);
    }
}

impl Model {
    /// Decodes every `SEQS` chunk in file order.
    pub fn sequences(&self) -> Result<Vec<Sequence>, Error> {
        self.collect_chunk_records::<SequencesChunk>(|chunk| match chunk {
            ModelChunk::Sequences(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Writes all sequences to the first `SEQS` chunk, creating it if needed.
    /// Additional `SEQS` chunks are removed after their records are replaced.
    pub fn set_sequences(&mut self, sequences: &[Sequence]) -> Result<(), Error> {
        self.replace_chunk(ModelChunk::Sequences(SequencesChunk::new(
            sequences.to_vec(),
        )))
    }
}

impl Record for Sequence {
    fn decode_one(cursor: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let size = cursor.remaining().len();
        if size < SIZE {
            return Err(Error::MalformedChunk {
                tag: Self::TAG,
                size,
                expected: SIZE,
            });
        }
        let name = cursor
            .read_exact(NAME_SIZE)?
            .try_into()
            .expect("fixed-width name");
        let interval = [cursor.read_u32()?, cursor.read_u32()?];
        let move_speed = cursor.read_u32()?;
        let flags = cursor.read_u32()?;
        let rarity = cursor.read_u32()?;
        let sync_point = cursor.read_u32()?;
        let bounds_radius = cursor.read_u32()?;
        let mut minimum_extent = [0; 3];
        let mut maximum_extent = [0; 3];
        for value in &mut minimum_extent {
            *value = cursor.read_u32()?;
        }
        for value in &mut maximum_extent {
            *value = cursor.read_u32()?;
        }
        Ok(Self {
            name,
            interval,
            move_speed,
            flags,
            rarity,
            sync_point,
            bounds_radius,
            minimum_extent,
            maximum_extent,
        })
    }
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), Error> {
        bytes.write_bytes(&self.name);
        for value in self
            .interval
            .into_iter()
            .chain([
                self.move_speed,
                self.flags,
                self.rarity,
                self.sync_point,
                self.bounds_radius,
            ])
            .chain(self.minimum_extent)
            .chain(self.maximum_extent)
        {
            bytes.write(value);
        }
        Ok(())
    }
}

impl Sequence {
    /// The tag of the chunk containing this record.
    pub const TAG: Tag = *b"SEQS";
}
