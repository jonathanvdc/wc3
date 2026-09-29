//! Particle2 choice flags, three segment colors and projected UV intervals.
use super::{Particle2FilterMode, Particle2Frames, ParticleEmitter2};
use crate::model::scene::{set_node_kind, validate_node_kind};
use crate::model::{mdl, Color};
use mdl::{Field, Parser, ReadErrorKind, Span, TokenKind, Writer};
use std::io::Write as IoWrite;

#[derive(mdl::Read, mdl::Write)]
#[mdl(property = "Color")]
struct ColorEntry(Color);
pub(super) struct SegmentColors([Color; 3]);
impl mdl::ReadProperty for SegmentColors {
    fn read_mdl_property(parser: &mut Parser<'_>, _: Field<'_>) -> Result<Self, mdl::ReadError> {
        parser.expect(TokenKind::OpenBrace)?;
        let mut values = [[0.0; 3]; 3];
        for value in &mut values {
            *value = parser.read::<ColorEntry>()?.0;
        }
        parser.expect(TokenKind::CloseBrace)?;
        Ok(Self(values))
    }
    fn missing_mdl_property(_: &'static str, _: Span) -> Result<Self, mdl::ReadError> {
        Ok(Self([[0.0; 3]; 3]))
    }
}
pub(super) struct SegmentColorsRef<'a>(&'a [Color; 3]);
impl mdl::WriteProperty for SegmentColorsRef<'_> {
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), mdl::WriteError> {
        writer.begin_block(name)?;
        for value in self.0 {
            writer.write(&ColorEntry(*value))?;
        }
        writer.end_block()
    }
}
fn choice(value: u32, span: Span) -> Result<u32, mdl::ReadError> {
    if value == 0 {
        return Ok(0);
    }
    if !value.is_power_of_two() {
        return Err(mdl::ReadError::new(
            span,
            ReadErrorKind::Expected("one choice flag"),
        ));
    }
    Ok(value.trailing_zeros())
}
impl ParticleEmitter2 {
    pub(super) fn mdl_filter(&self) -> u32 {
        1u32.checked_shl(self.filter_mode.raw()).unwrap_or(0)
    }
    pub(super) fn set_mdl_filter(
        &mut self,
        value: u32,
        _: bool,
        span: Span,
    ) -> Result<(), mdl::ReadError> {
        self.filter_mode = Particle2FilterMode::from_raw(choice(value, span)?);
        Ok(())
    }
    pub(super) fn mdl_frames(&self) -> u32 {
        1u32.checked_shl(self.frames.raw()).unwrap_or(0)
    }
    pub(super) fn set_mdl_frames(
        &mut self,
        value: u32,
        _: bool,
        span: Span,
    ) -> Result<(), mdl::ReadError> {
        self.frames = Particle2Frames::from_raw(choice(value, span)?);
        Ok(())
    }
    pub(super) fn mdl_segments(&self) -> SegmentColorsRef<'_> {
        SegmentColorsRef(&self.segment_colors)
    }
    pub(super) fn set_mdl_segments(
        &mut self,
        value: SegmentColors,
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.segment_colors = value.0;
        Ok(())
    }
    pub(super) fn finish_mdl(&mut self, _: Span) -> Result<(), mdl::ReadError> {
        set_node_kind(&mut self.node, 0x1000);
        Ok(())
    }
    pub(super) fn validate_mdl(&self) -> Result<(), mdl::WriteError> {
        validate_node_kind(&self.node, 0x1000 | (self.node.flags.bits() & 0x1f8000))?;
        if self.filter_mode.raw() > 4 {
            return Err(mdl::WriteError::Unsupported("particle2 filter mode"));
        }
        if self.frames.raw() > 2 {
            return Err(mdl::WriteError::Unsupported("particle2 head/tail mode"));
        }
        Ok(())
    }
}

impl ParticleEmitter2 {
    pub(super) fn mdl_life_uv(&self) -> [u32; 3] {
        self.uv_animations[0]
    }
    pub(super) fn set_mdl_life_uv(
        &mut self,
        value: [u32; 3],
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.uv_animations[0] = value;
        Ok(())
    }
    pub(super) fn mdl_decay_uv(&self) -> [u32; 3] {
        self.uv_animations[1]
    }
    pub(super) fn set_mdl_decay_uv(
        &mut self,
        value: [u32; 3],
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.uv_animations[1] = value;
        Ok(())
    }
    pub(super) fn mdl_tail_uv(&self) -> [u32; 3] {
        self.uv_animations[2]
    }
    pub(super) fn set_mdl_tail_uv(
        &mut self,
        value: [u32; 3],
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.uv_animations[2] = value;
        Ok(())
    }
    pub(super) fn mdl_tail_decay_uv(&self) -> [u32; 3] {
        self.uv_animations[3]
    }
    pub(super) fn set_mdl_tail_decay_uv(
        &mut self,
        value: [u32; 3],
        _: bool,
        _: Span,
    ) -> Result<(), mdl::ReadError> {
        self.uv_animations[3] = value;
        Ok(())
    }
}
