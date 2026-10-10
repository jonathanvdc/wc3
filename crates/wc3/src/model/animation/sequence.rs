//! Named animation ranges and playback settings.
use crate::model::mdl;
use crate::model::mdl::{is_positive_zero, is_zero};
use crate::model::mdx;
use crate::model::FixedText;
use crate::model::GeosetExtent;
use crate::model::Model;
use crate::model::ModelDialect;
use crate::model::SequencesChunk;
use crate::model::ValueError;
use bitfield::bitfield;

bitfield! {
    /// Sequence playback flags, with unrecognized bits retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
    #[mdl(flags(NonLooping = 1))]
    pub struct SequenceFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Plays the sequence once instead of looping.
    pub non_looping, set_non_looping: 0;
}

const NAME_SIZE: usize = 80;

/// A named animation range on the model timeline.
#[derive(Clone, Debug, PartialEq, Default, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(
    block = "Anim",
    write_order(interval, flags, move_speed, rarity, sync_point, extent)
)]
pub struct Sequence {
    #[mdl(header)]
    /// Animation name, such as `Stand`, `Walk`, or `Death`.
    pub name: FixedText<NAME_SIZE>,
    #[mdl(property = "Interval")]
    /// Start and end times in milliseconds on the model timeline.
    pub interval: [u32; 2],
    #[mdl(property = "MoveSpeed", default, skip_if = "is_positive_zero")]
    /// Authored movement speed associated with this sequence.
    pub move_speed: f32,
    #[mdl(flatten)]
    /// Playback flags, including whether the sequence loops.
    pub flags: SequenceFlags,
    #[mdl(property = "Rarity", default, skip_if = "is_positive_zero")]
    /// Weight used for random animation selection.
    pub rarity: f32,
    #[mdl(property = "SyncPoint", default, skip_if = "is_zero")]
    /// Authored synchronization point.
    pub sync_point: u32,
    #[mdl(flatten)]
    /// Bounds for the model during this sequence.
    pub extent: GeosetExtent,
}

impl Sequence {
    /// Creates a named sequence with the supplied millisecond interval.
    /// Returns an error if the name contains NUL or exceeds the fixed text capacity.
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

impl<D: ModelDialect> Model<D> {
    /// Returns owned copies of sequence records from every `SEQS` chunk in file order.
    pub fn sequences(&self) -> Vec<Sequence> {
        self.collect_chunk_records::<SequencesChunk>()
    }

    /// Writes all sequences to the first `SEQS` chunk, creating it if needed.
    /// Additional `SEQS` chunks are removed after their records are replaced.
    pub fn set_sequences(&mut self, sequences: &[Sequence]) {
        self.replace_chunk(SequencesChunk::new(sequences.to_vec()));
    }
}
