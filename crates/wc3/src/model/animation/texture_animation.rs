//! Typed texture animation tracks in `TXAN` chunks.
use crate::model::mdl::{MdlWriter, Parser, ReadErrorKind, TokenKind};
use crate::model::ModelVersion;
use crate::model::{mdl, mdx};
use std::io::Write as IoWrite;
crate::model::animation::track_group! {
    pub enum TextureAnimationTrack {
        Translation: TextureTranslation,
        Rotation: TextureRotation,
        Scaling: TextureScaling,
    }
}

use crate::model::KnownChunk;

use crate::model::Model;
use crate::model::TextureAnimationsChunk;

/// A texture animation containing translation, rotation, and scaling tracks.
/// MDL uses a `TVertexAnim` block inside `TextureAnims`. Its transform channels
/// are animation tracks only; there are no `static` transform properties.
#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
#[mdx(sized(tag = TextureAnimationsChunk::TAG))]
pub struct TextureAnimation {
    tracks: Vec<TextureAnimationTrack>,
}

impl TextureAnimation {
    /// Creates an empty texture animation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Borrows decoded tracks without reparsing.
    pub fn tracks(&self) -> &[TextureAnimationTrack] {
        &self.tracks
    }

    /// Replaces texture animation tracks.
    pub fn set_tracks(&mut self, tracks: &[TextureAnimationTrack]) {
        self.tracks = tracks.to_vec();
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes all texture animations in `TXAN` chunks.
    pub fn texture_animations(&self) -> Vec<TextureAnimation> {
        self.collect_chunk_records::<TextureAnimationsChunk>()
    }

    /// Replaces texture animations in the first `TXAN` chunk.
    pub fn set_texture_animations(&mut self, animations: &[TextureAnimation]) {
        self.replace_chunk(TextureAnimationsChunk::new(animations.to_vec()));
    }
}

impl mdl::Read for TextureAnimation {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        parser.expect_ident("TVertexAnim")?;
        parser.expect(TokenKind::OpenBrace)?;
        let mut tracks: Vec<TextureAnimationTrack> = Vec::new();
        loop {
            parser.peek()?;
            if parser.consume(TokenKind::CloseBrace)? {
                break;
            }
            let span = parser.error(ReadErrorKind::DuplicateField).span;
            let track = parser.read::<TextureAnimationTrack>()?;
            if tracks.iter().any(|previous| previous.tag() == track.tag()) {
                return Err(mdl::ReadError::new(span, ReadErrorKind::DuplicateField));
            }
            tracks.push(track);
        }
        Ok(Self { tracks })
    }
}

impl mdl::Write for TextureAnimation {
    fn write_mdl<W: IoWrite>(&self, writer: &mut MdlWriter<W>) -> Result<(), mdl::WriteError> {
        for (index, track) in self.tracks.iter().enumerate() {
            if self.tracks[..index]
                .iter()
                .any(|previous| previous.tag() == track.tag())
            {
                return Err(mdl::WriteError::Unsupported(
                    "duplicate texture animation track",
                ));
            }
        }
        writer.begin_block("TVertexAnim")?;
        for track in &self.tracks {
            writer.write(track)?;
        }
        writer.end_block()
    }
}
