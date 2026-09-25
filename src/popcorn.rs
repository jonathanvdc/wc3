//! Reforged popcorn particle emitters in `CORN` chunks.
use crate::Encoder;
use crate::{Color, Tag};

use crate::Record;
use crate::{Cursor, ModelChunk, PopcornEmittersChunk};
use std::borrow::Cow;

use crate::utils::field;
use crate::{AnimationTrack, Error, Model, Node};

const PATH_SIZE: usize = 260;
const FIXED_SIZE: usize = 32 + PATH_SIZE * 2;

/// A popcorn particle emitter with decoded fields and animation tracks.
#[derive(Clone, Debug, PartialEq)]
pub struct PopcornEmitter {
    node: Node,
    life_span: f32,
    emission_rate: f32,
    speed: f32,
    color: Color,
    alpha: f32,
    replaceable_id: u32,
    path: [u8; PATH_SIZE],
    visibility_guide: [u8; PATH_SIZE],
    tracks: Vec<AnimationTrack>,
}

impl PopcornEmitter {
    /// Creates an emitter with zeroed physical values.
    pub fn new(node: Node, path: &str, visibility_guide: &str) -> Result<Self, Error> {
        let mut emitter = Self {
            node,
            life_span: 0.0,
            emission_rate: 0.0,
            speed: 0.0,
            color: [0.0; 3],
            alpha: 0.0,
            replaceable_id: 0,
            path: [0; PATH_SIZE],
            visibility_guide: [0; PATH_SIZE],
            tracks: Vec::new(),
        };
        emitter.set_path(path)?;
        emitter.set_visibility_guide(visibility_guide)?;
        emitter.encode()?;
        Ok(emitter)
    }

    /// Borrows the shared node.
    pub fn node(&self) -> &Node {
        &self.node
    }
    /// Borrows the shared node for editing.
    pub fn node_mut(&mut self) -> &mut Node {
        &mut self.node
    }
    /// Returns particle lifetime.
    pub fn life_span(&self) -> f32 {
        self.life_span
    }
    /// Sets particle lifetime.
    pub fn set_life_span(&mut self, value: f32) {
        self.life_span = value;
    }
    /// Returns emission rate.
    pub fn emission_rate(&self) -> f32 {
        self.emission_rate
    }
    /// Sets emission rate.
    pub fn set_emission_rate(&mut self, value: f32) {
        self.emission_rate = value;
    }
    /// Returns particle speed.
    pub fn speed(&self) -> f32 {
        self.speed
    }
    /// Sets particle speed.
    pub fn set_speed(&mut self, value: f32) {
        self.speed = value;
    }
    /// Returns RGB particle color.
    pub fn color(&self) -> Color {
        self.color
    }
    /// Sets RGB particle color.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    /// Returns base alpha.
    pub fn alpha(&self) -> f32 {
        self.alpha
    }
    /// Sets base alpha.
    pub fn set_alpha(&mut self, value: f32) {
        self.alpha = value;
    }
    /// Returns the replaceable texture ID.
    pub fn replaceable_id(&self) -> u32 {
        self.replaceable_id
    }
    /// Sets the replaceable texture ID.
    pub fn set_replaceable_id(&mut self, id: u32) {
        self.replaceable_id = id;
    }
    /// Returns the model path.
    pub fn path(&self) -> Cow<'_, str> {
        field::text(&self.path)
    }
    /// Sets the model path.
    pub fn set_path(&mut self, path: &str) -> Result<(), Error> {
        field::set_text(&mut self.path, path)
    }
    /// Returns the animation visibility guide path.
    pub fn visibility_guide(&self) -> Cow<'_, str> {
        field::text(&self.visibility_guide)
    }
    /// Sets the animation visibility guide path.
    pub fn set_visibility_guide(&mut self, guide: &str) -> Result<(), Error> {
        field::set_text(&mut self.visibility_guide, guide)
    }
    /// Borrows decoded animation tracks.
    pub fn tracks(&self) -> &[AnimationTrack] {
        &self.tracks
    }
    /// Replaces optional animation tracks.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        for track in tracks {
            if !is_track_tag(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: PopcornEmitter::TAG,
                    offset: 0,
                });
            }
            track.encode()?;
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

fn is_track_tag(tag: Tag) -> bool {
    matches!(
        &tag,
        b"KPPA" | b"KPPC" | b"KPPE" | b"KPPL" | b"KPPS" | b"KPPV"
    )
}

impl Model {
    /// Decodes all popcorn emitters in `CORN` chunks.
    pub fn popcorn_emitters(&self) -> Result<Vec<PopcornEmitter>, Error> {
        self.collect_chunk_records::<PopcornEmittersChunk>(|chunk| match chunk {
            ModelChunk::PopcornEmitters(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces popcorn emitters in the first `CORN` chunk.
    pub fn set_popcorn_emitters(&mut self, emitters: &[PopcornEmitter]) -> Result<(), Error> {
        self.replace_chunk(ModelChunk::PopcornEmitters(PopcornEmittersChunk::new(
            emitters.to_vec(),
        )))
    }
}

impl Record for PopcornEmitter {
    fn decode_one(source: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let mut cursor = source.slice_u32_sized()?;
        let node = Node::decode_one(&mut cursor, 0)?;
        let life_span = cursor.read_f32()?;
        let emission_rate = cursor.read_f32()?;
        let speed = cursor.read_f32()?;
        let color = cursor.read_vec3()?;
        let alpha = cursor.read_f32()?;
        let replaceable_id = cursor.read_u32()?;
        let path = cursor
            .read_exact(PATH_SIZE)?
            .try_into()
            .expect("fixed-width path");
        let visibility_guide = cursor
            .read_exact(FIXED_SIZE - 32 - PATH_SIZE)?
            .try_into()
            .expect("fixed-width guide");
        let mut tracks = Vec::new();
        while !cursor.remaining().is_empty() {
            let offset = cursor.absolute_position();
            let track = AnimationTrack::decode_one(&mut cursor, 0)?;
            if !is_track_tag(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: Self::TAG,
                    offset,
                });
            }

            tracks.push(track);
        }
        cursor.finish()?;
        Ok(Self {
            node,
            life_span,
            emission_rate,
            speed,
            color,
            alpha,
            replaceable_id,
            path,
            visibility_guide,
            tracks,
        })
    }

    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), Error> {
        let start = bytes.position();
        let marker = bytes.begin_sized();
        self.node.encode_to(bytes)?;
        for value in [
            self.life_span,
            self.emission_rate,
            self.speed,
            self.color[0],
            self.color[1],
            self.color[2],
            self.alpha,
        ] {
            bytes.write(value);
        }
        bytes.write(self.replaceable_id);
        bytes.write_bytes(&self.path);
        bytes.write_bytes(&self.visibility_guide);
        for track in &self.tracks {
            if !is_track_tag(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: PopcornEmitter::TAG,
                    offset: bytes.position() - start,
                });
            }
            track.encode_to(bytes)?;
        }
        bytes.finish_sized(marker, PopcornEmitter::TAG)?;
        Ok(())
    }
}

impl PopcornEmitter {
    /// The tag of the chunk containing this record.
    pub const TAG: Tag = *b"CORN";
}
