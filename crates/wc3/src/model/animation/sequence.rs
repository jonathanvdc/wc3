//! Animation sequence records in the `SEQS` chunk.
use crate::model::mdl;
use crate::model::mdl::{is_positive_zero, is_zero};
use crate::model::mdx;
use crate::model::ModelVersion;
use crate::model::ValueError;
use crate::model::{GeosetExtent, Vec3};
use bitfield::bitfield;

use crate::model::SequencesChunk;

use std::borrow::Cow;

use crate::model::FixedText;
use crate::model::Model;

bitfield! {
    /// Sequence playback flags, with unrecognized bits retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, mdx::Read, mdx::Write)]
    pub struct SequenceFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the `NON_LOOPING` bit.
    pub non_looping, set_non_looping: 0;
}

const NAME_SIZE: usize = 80;

/// A fixed-size animation sequence with lossless playback flags.
#[derive(Clone, Debug, PartialEq, Default, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(
    block = "Anim",
    write_order(interval, flags, move_speed, rarity, sync_point, extent)
)]
pub struct Sequence {
    #[mdl(header)]
    name: FixedText<NAME_SIZE>,
    #[mdl(property = "Interval")]
    interval: [u32; 2],
    #[mdl(property = "MoveSpeed", default, skip_if = "is_positive_zero")]
    move_speed: f32,
    #[mdl(flags(NonLooping = 1))]
    flags: SequenceFlags,
    #[mdl(property = "Rarity", default, skip_if = "is_positive_zero")]
    rarity: f32,
    #[mdl(property = "SyncPoint", default, skip_if = "is_zero")]
    sync_point: u32,
    #[mdl(flatten)]
    extent: GeosetExtent,
}

impl Sequence {
    pub fn new(name: &str, interval: [u32; 2]) -> Result<Self, ValueError> {
        let mut sequence = Self {
            name: FixedText::default(),
            interval,
            move_speed: 0.0,
            flags: SequenceFlags::default(),
            rarity: 0.0,
            sync_point: 0,
            extent: GeosetExtent::default(),
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
        self.flags
    }
    pub fn set_flags(&mut self, flags: SequenceFlags) {
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
        self.extent.bounds_radius
    }
    pub fn set_bounds_radius(&mut self, radius: f32) {
        self.extent.bounds_radius = radius;
    }
    pub fn minimum_extent(&self) -> Vec3 {
        self.extent.minimum
    }
    pub fn set_minimum_extent(&mut self, extent: Vec3) {
        self.extent.minimum = extent;
    }
    pub fn maximum_extent(&self) -> Vec3 {
        self.extent.maximum
    }
    pub fn set_maximum_extent(&mut self, extent: Vec3) {
        self.extent.maximum = extent;
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
