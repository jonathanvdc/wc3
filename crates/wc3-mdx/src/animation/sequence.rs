//! Animation sequence records in the `SEQS` chunk.
use crate::mdl::{
    fixed_text, ReadError as MdlError, ReadErrorKind, Fields, MdlRead, MdlWrite, MdlWriter, Parser,
    TokenKind, WriteError,
};
use crate::ModelVersion;
use crate::ValueError;
use crate::Vec3;
use bitfield::bitfield;
use std::io::Write;

use crate::SequencesChunk;
use crate::{Readable, Writable};
use std::borrow::Cow;

use crate::FixedText;
use crate::Model;

bitfield! {
    /// Sequence playback flags, with unrecognized bits retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct SequenceFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the `NON_LOOPING` bit.
    pub non_looping, set_non_looping: 0;
}

const NAME_SIZE: usize = 80;

/// A fixed-size animation sequence, including reserved fields.
#[derive(Clone, Debug, PartialEq, Default, Readable, Writable)]
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
        SequenceFlags(self.flags)
    }
    pub fn set_flags(&mut self, flags: SequenceFlags) {
        self.flags = flags.bits();
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

impl MdlRead for Sequence {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, MdlError> {
        parser.expect_ident("Anim")?;
        let mut sequence = Self::default();
        sequence.name = parser.read_fixed_text()?;
        let mut fields = Fields::default();
        let mut body = parser.begin_block()?;
        while let Some(field) = body.next_field()? {
            match field.name {
                "Interval" => {
                    fields.mark(0, field)?;
                    sequence.interval = body.read_property()?;
                }
                "NonLooping" => {
                    fields.mark(1, field)?;
                    body.expect(TokenKind::Comma)?;
                    sequence.flags = 1;
                }
                "MoveSpeed" => {
                    fields.mark(2, field)?;
                    sequence.move_speed = body.read_property()?;
                }
                "Rarity" => {
                    fields.mark(3, field)?;
                    sequence.rarity = body.read_property()?;
                }
                "SyncPoint" => {
                    fields.mark(4, field)?;
                    sequence.sync_point = body.read_property()?;
                }
                "MinimumExtent" => {
                    fields.mark(5, field)?;
                    sequence.minimum_extent = body.read_property()?;
                }
                "MaximumExtent" => {
                    fields.mark(6, field)?;
                    sequence.maximum_extent = body.read_property()?;
                }
                "BoundsRadius" => {
                    fields.mark(7, field)?;
                    sequence.bounds_radius = body.read_property()?;
                }
                _ => return Err(MdlError::new(field.span, ReadErrorKind::UnknownField)),
            }
        }
        fields.require(
            0,
            "Interval",
            body.error(ReadErrorKind::MissingField("Interval")).span,
        )?;
        body.finish()?;
        Ok(sequence)
    }
}
impl MdlWrite for Sequence {
    fn write_mdl<W: Write>(&self, writer: &mut MdlWriter<W>) -> Result<(), WriteError> {
        if self.flags & !1 != 0 {
            return Err(WriteError::Unsupported("unknown sequence flags"));
        }
        writer.begin_named_block("Anim", fixed_text(&self.name)?)?;
        writer.property("Interval", &self.interval)?;
        if self.flags & 1 != 0 {
            writer.flag("NonLooping")?;
        }
        // Check bits rather than equality so omission does not discard negative zero.
        if self.move_speed.to_bits() != 0 {
            writer.property("MoveSpeed", &self.move_speed)?;
        }
        if self.rarity.to_bits() != 0 {
            writer.property("Rarity", &self.rarity)?;
        }
        if self.sync_point != 0 {
            writer.property("SyncPoint", &self.sync_point)?;
        }
        writer.property("MinimumExtent", &self.minimum_extent)?;
        writer.property("MaximumExtent", &self.maximum_extent)?;
        writer.property("BoundsRadius", &self.bounds_radius)?;
        writer.end_block()
    }
}
