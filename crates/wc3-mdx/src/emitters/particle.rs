//! Classic particle emitters stored in `PREM` chunks.
use crate::EncodeError;
use crate::Encoder;
use crate::KnownChunk;
use crate::ValueError;

use crate::{Cursor, ParticleEmittersChunk, TrackTag};
use crate::{Decodable, Encodable};
use std::borrow::Cow;

use crate::FixedText;
use crate::{AnimationTrack, DecodeError, Model, Node};

const PATH_SIZE: usize = 256;

/// A Classic particle emitter with optional animated properties.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleEmitter {
    node: Node,
    emission_rate: f32,
    gravity: f32,
    longitude: f32,
    latitude: f32,
    path: FixedText<PATH_SIZE>,
    reserved: u32,
    life_span: f32,
    initial_velocity: f32,
    tracks: Vec<AnimationTrack>,
}

impl ParticleEmitter {
    /// Creates an emitter with zeroed physical values.
    pub fn new(node: Node, path: &str) -> Result<Self, ValueError> {
        let mut emitter = Self {
            node,
            emission_rate: 0.0,
            gravity: 0.0,
            longitude: 0.0,
            latitude: 0.0,
            path: FixedText::default(),
            reserved: 0,
            life_span: 0.0,
            initial_velocity: 0.0,
            tracks: Vec::new(),
        };
        emitter.set_path(path)?;
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

    /// Returns emission rate.
    pub fn emission_rate(&self) -> f32 {
        self.emission_rate
    }
    /// Sets emission rate.
    pub fn set_emission_rate(&mut self, value: f32) {
        self.emission_rate = value;
    }
    /// Returns gravity.
    pub fn gravity(&self) -> f32 {
        self.gravity
    }
    /// Sets gravity.
    pub fn set_gravity(&mut self, value: f32) {
        self.gravity = value;
    }
    /// Returns longitude.
    pub fn longitude(&self) -> f32 {
        self.longitude
    }
    /// Sets longitude.
    pub fn set_longitude(&mut self, value: f32) {
        self.longitude = value;
    }
    /// Returns latitude.
    pub fn latitude(&self) -> f32 {
        self.latitude
    }
    /// Sets latitude.
    pub fn set_latitude(&mut self, value: f32) {
        self.latitude = value;
    }
    /// Returns particle lifetime.
    pub fn life_span(&self) -> f32 {
        self.life_span
    }
    /// Sets particle lifetime.
    pub fn set_life_span(&mut self, value: f32) {
        self.life_span = value;
    }
    /// Returns initial velocity.
    pub fn initial_velocity(&self) -> f32 {
        self.initial_velocity
    }
    /// Sets initial velocity.
    pub fn set_initial_velocity(&mut self, value: f32) {
        self.initial_velocity = value;
    }

    /// Returns the emitter resource path up to the first NUL.
    pub fn path(&self) -> Cow<'_, str> {
        self.path.text()
    }

    /// Sets the emitter path while retaining all other fields.
    pub fn set_path(&mut self, path: &str) -> Result<(), ValueError> {
        self.path.set_text(path)
    }

    /// Returns the untyped reserved word following the path.
    pub fn reserved(&self) -> u32 {
        self.reserved
    }

    /// Borrows decoded animation tracks.
    pub fn tracks(&self) -> &[AnimationTrack] {
        &self.tracks
    }

    /// Replaces optional animation tracks.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), ValueError> {
        for track in tracks {
            if !is_track(track.tag) {
                return Err(ValueError::InvalidTrackTag {
                    record: ParticleEmittersChunk::TAG,
                    track: track.tag.bytes(),
                });
            }
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

fn is_track(tag: TrackTag) -> bool {
    matches!(
        tag,
        TrackTag::ParticleVisibility
            | TrackTag::ParticleEmissionRate
            | TrackTag::ParticleGravity
            | TrackTag::ParticleLongitude
            | TrackTag::ParticleLatitude
            | TrackTag::ParticleLifespan
            | TrackTag::ParticleSpeed
    )
}

impl Model {
    /// Decodes all `PREM` records in file order.
    pub fn particle_emitters(&self) -> Vec<ParticleEmitter> {
        self.collect_chunk_records::<ParticleEmittersChunk>()
    }

    /// Replaces particle emitters in the first `PREM` chunk.
    pub fn set_particle_emitters(&mut self, emitters: &[ParticleEmitter]) {
        self.replace_chunk(ParticleEmittersChunk::new(emitters.to_vec()));
    }
}

impl Decodable for ParticleEmitter {
    fn decode_one(source: &mut Cursor<'_>, _version: u32) -> Result<Self, DecodeError> {
        let mut cursor = source.slice_u32_sized()?;
        let node = Node::decode_one(&mut cursor, 0)?;
        let emission_rate = cursor.read()?;
        let gravity = cursor.read()?;
        let longitude = cursor.read()?;
        let latitude = cursor.read()?;
        let path = cursor.read()?;
        let reserved = cursor.read()?;
        let life_span = cursor.read()?;
        let initial_velocity = cursor.read()?;
        let mut tracks = Vec::new();
        while !cursor.remaining().is_empty() {
            let offset = cursor.absolute_position();
            let track = AnimationTrack::decode_one(&mut cursor, 0)?;
            if !is_track(track.tag) {
                return Err(DecodeError::MalformedRecord {
                    tag: ParticleEmittersChunk::TAG,
                    offset,
                });
            }

            tracks.push(track);
        }
        cursor.finish()?;
        Ok(Self {
            node,
            emission_rate,
            gravity,
            longitude,
            latitude,
            path,
            reserved,
            life_span,
            initial_velocity,
            tracks,
        })
    }
}

impl Encodable for ParticleEmitter {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let start = bytes.position();
        let marker = bytes.begin_sized();
        self.node.encode_to(bytes)?;
        for value in [
            self.emission_rate,
            self.gravity,
            self.longitude,
            self.latitude,
        ] {
            bytes.write(value);
        }
        bytes.write(&self.path);
        bytes.write(self.reserved);
        bytes.write(self.life_span);
        bytes.write(self.initial_velocity);
        for track in &self.tracks {
            if !is_track(track.tag) {
                return Err(EncodeError::MalformedRecord {
                    tag: ParticleEmittersChunk::TAG,
                    offset: bytes.position() - start,
                });
            }
            track.encode_to(bytes)?;
        }
        bytes.finish_sized(marker, ParticleEmittersChunk::TAG)?;
        Ok(())
    }
}
