//! Animation sequence records in the `SEQS` chunk.
use crate::model::mdl;
use crate::model::mdl::{is_positive_zero, is_zero};
use crate::model::mdx;
use crate::model::GeosetExtent;
use crate::model::ModelVersion;
use crate::model::ValueError;
use bitfield::bitfield;

use crate::model::SequencesChunk;

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
    pub name: FixedText<NAME_SIZE>,
    #[mdl(property = "Interval")]
    pub interval: [u32; 2],
    #[mdl(property = "MoveSpeed", default, skip_if = "is_positive_zero")]
    pub move_speed: f32,
    #[mdl(flags(NonLooping = 1))]
    pub flags: SequenceFlags,
    #[mdl(property = "Rarity", default, skip_if = "is_positive_zero")]
    pub rarity: f32,
    #[mdl(property = "SyncPoint", default, skip_if = "is_zero")]
    pub sync_point: u32,
    #[mdl(flatten)]
    pub extent: GeosetExtent,
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
        sequence.name.set_text(name)?;
        Ok(sequence)
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
