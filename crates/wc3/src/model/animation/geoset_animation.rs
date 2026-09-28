//! Geoset animation records in `GEOA` chunks.
use crate::model::mdl::{Field, Fields, MdlWriter, Parser, ReadErrorKind, TokenKind};
use crate::model::ModelVersion;
use crate::model::{mdl, mdx};
use bitfield::bitfield;
use std::io::Write as IoWrite;
crate::model::animation::track_group! {
    pub enum GeosetTrack {
        Alpha: GeosetAlpha,
        Color: GeosetColor,
    }
}

use crate::model::Color;
use crate::model::KnownChunk;

use crate::model::GeosetAnimationsChunk;
use crate::model::Model;

bitfield! {
    /// Geoset animation rendering flags, retaining unknown bits.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct GeosetAnimationFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the `DROP_SHADOW` bit.
    pub drop_shadow, set_drop_shadow: 0;
    /// Returns or changes the `COLOR` bit.
    pub color, set_color: 1;
}

/// A geoset animation with decoded alpha and color tracks.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write)]
#[mdx(sized(tag = GeosetAnimationsChunk::TAG))]
pub struct GeosetAnimation {
    alpha: f32,
    raw_flags: u32,
    color: Color,
    geoset_id: u32,
    tracks: Vec<GeosetTrack>,
}

impl GeosetAnimation {
    /// Creates a geoset animation with opaque white color and full alpha.
    pub fn new(geoset_id: u32) -> Self {
        Self {
            alpha: 1.0,
            raw_flags: 0,
            color: [1.0; 3],
            geoset_id,
            tracks: Vec::new(),
        }
    }

    /// Returns the base alpha.
    pub fn alpha(&self) -> f32 {
        self.alpha
    }
    /// Changes the base alpha.
    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
    }
    /// Returns decoded rendering flags.
    pub fn flags(&self) -> GeosetAnimationFlags {
        GeosetAnimationFlags(self.raw_flags)
    }
    /// Changes decoded rendering bits.
    pub fn set_flags(&mut self, flags: GeosetAnimationFlags) {
        self.raw_flags = flags.bits();
    }
    /// Returns base RGB color.
    pub fn color(&self) -> Color {
        self.color
    }
    /// Changes base RGB color.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    /// Returns the referenced geoset index.
    pub fn geoset_id(&self) -> u32 {
        self.geoset_id
    }
    /// Changes the referenced geoset index.
    pub fn set_geoset_id(&mut self, id: u32) {
        self.geoset_id = id;
    }
    /// Borrows alpha and color tracks without reparsing.
    pub fn tracks(&self) -> &[GeosetTrack] {
        &self.tracks
    }
    /// Replaces alpha and color tracks.
    pub fn set_tracks(&mut self, tracks: &[GeosetTrack]) {
        self.tracks = tracks.to_vec();
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes all `GEOA` records in file order.
    pub fn geoset_animations(&self) -> Vec<GeosetAnimation> {
        self.collect_chunk_records::<GeosetAnimationsChunk>()
    }

    /// Replaces geoset animations in the first `GEOA` chunk.
    pub fn set_geoset_animations(&mut self, animations: &[GeosetAnimation]) {
        self.replace_chunk(GeosetAnimationsChunk::new(animations.to_vec()));
    }
}

impl mdl::Read for GeosetAnimation {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        parser.expect_ident("GeosetAnim")?;
        let mut body = parser.begin_block()?;
        let mut value = Self::new(0);
        let mut fields = Fields::default();
        while let Some(token) = body.peek()? {
            if token.kind == TokenKind::CloseBrace {
                break;
            }
            let mut field = match token.kind {
                TokenKind::Ident(name) => Field {
                    name,
                    span: token.span,
                },
                _ => return Err(body.error(ReadErrorKind::Expected("a field name or '}'"))),
            };
            if field.name == "static" {
                body.next_token()?;
                let token = body.next_token()?;
                field = match token.kind {
                    TokenKind::Ident(name) => Field {
                        name,
                        span: token.span,
                    },
                    _ => {
                        return Err(mdl::ReadError::new(
                            token.span,
                            ReadErrorKind::Expected("an animatable property name"),
                        ))
                    }
                };
                match field.name {
                    "Alpha" => {
                        fields.mark(0, field)?;
                        value.alpha = body.read_property()?;
                    }
                    "Color" => {
                        fields.mark(1, field)?;
                        value.color = body.read_property()?;
                        value.raw_flags |= 2;
                    }
                    _ => return Err(mdl::ReadError::new(field.span, ReadErrorKind::UnknownField)),
                }
            } else {
                match field.name {
                    "Alpha" | "Color" => {
                        fields.mark(if field.name == "Alpha" { 0 } else { 1 }, field)?;
                        value.tracks.push(body.read::<GeosetTrack>()?);
                        if field.name == "Color" {
                            value.raw_flags |= 2;
                        }
                    }
                    "DropShadow" => {
                        fields.mark(2, field)?;
                        body.next_token()?;
                        body.expect(TokenKind::Comma)?;
                        value.raw_flags |= 1;
                    }
                    "GeosetId" => {
                        fields.mark(3, field)?;
                        body.next_token()?;
                        value.geoset_id = body.read_property()?;
                    }
                    _ => return Err(mdl::ReadError::new(field.span, ReadErrorKind::UnknownField)),
                }
            }
        }
        fields.require(
            3,
            "GeosetId",
            body.error(ReadErrorKind::MissingField("GeosetId")).span,
        )?;
        body.finish()?;
        Ok(value)
    }
}

impl mdl::Write for GeosetAnimation {
    fn write_mdl<W: IoWrite>(&self, writer: &mut MdlWriter<W>) -> Result<(), mdl::WriteError> {
        if self.raw_flags & !3 != 0 {
            return Err(mdl::WriteError::Unsupported(
                "unknown geoset animation flags",
            ));
        }
        let mut alpha_track = false;
        let mut color_track = false;
        for track in &self.tracks {
            let seen = match track {
                GeosetTrack::Alpha(_) => &mut alpha_track,
                GeosetTrack::Color(_) => &mut color_track,
            };
            if *seen {
                return Err(mdl::WriteError::Unsupported(
                    "duplicate geoset animation track",
                ));
            }
            *seen = true;
        }
        if alpha_track && self.alpha.to_bits() != 1.0f32.to_bits() {
            return Err(mdl::WriteError::Unsupported(
                "nondefault base alpha alongside an animation track",
            ));
        }
        let white = self
            .color
            .iter()
            .all(|value| value.to_bits() == 1.0f32.to_bits());
        let uses_color = self.flags().color();
        if color_track && !uses_color {
            return Err(mdl::WriteError::Unsupported(
                "color track without the color-use flag",
            ));
        }
        if (color_track || !uses_color) && !white {
            return Err(mdl::WriteError::Unsupported("base color omitted by MDL"));
        }
        writer.begin_block("GeosetAnim")?;
        if !alpha_track {
            writer.static_property("Alpha", &self.alpha)?;
        }
        if self.flags().drop_shadow() {
            writer.flag("DropShadow")?;
        }
        writer.property("GeosetId", &self.geoset_id)?;
        if uses_color && !color_track {
            writer.static_property("Color", &self.color)?;
        }
        // Preserve the order of stored tracks through text round trips.
        for track in &self.tracks {
            writer.write(track)?;
        }
        writer.end_block()
    }
}
