//! Classic particle emitters stored in `PREM` chunks.

use crate::Record;
use std::borrow::Cow;

use crate::utils::field;
use crate::{sized_node, AnimationTrack, Error, Model, Node};

pub(crate) const FIXED_SIZE: usize = 284;
const PATH_SIZE: usize = 256;

/// A Classic particle emitter with optional animated properties.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleEmitter {
    node: Node,
    fixed: [u8; FIXED_SIZE],
    tracks: Vec<AnimationTrack>,
}

impl ParticleEmitter {
    /// Creates an emitter with zeroed physical values.
    pub fn new(node: Node, path: &str) -> Result<Self, Error> {
        let mut emitter = Self {
            node,
            fixed: [0; FIXED_SIZE],
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
        field::f32_at(&self.fixed, 0)
    }
    /// Sets emission rate.
    pub fn set_emission_rate(&mut self, value: f32) {
        field::set_f32_at(&mut self.fixed, 0, value);
    }
    /// Returns gravity.
    pub fn gravity(&self) -> f32 {
        field::f32_at(&self.fixed, 4)
    }
    /// Sets gravity.
    pub fn set_gravity(&mut self, value: f32) {
        field::set_f32_at(&mut self.fixed, 4, value);
    }
    /// Returns longitude.
    pub fn longitude(&self) -> f32 {
        field::f32_at(&self.fixed, 8)
    }
    /// Sets longitude.
    pub fn set_longitude(&mut self, value: f32) {
        field::set_f32_at(&mut self.fixed, 8, value);
    }
    /// Returns latitude.
    pub fn latitude(&self) -> f32 {
        field::f32_at(&self.fixed, 12)
    }
    /// Sets latitude.
    pub fn set_latitude(&mut self, value: f32) {
        field::set_f32_at(&mut self.fixed, 12, value);
    }
    /// Returns particle lifetime.
    pub fn life_span(&self) -> f32 {
        field::f32_at(&self.fixed, 276)
    }
    /// Sets particle lifetime.
    pub fn set_life_span(&mut self, value: f32) {
        field::set_f32_at(&mut self.fixed, 276, value);
    }
    /// Returns initial velocity.
    pub fn initial_velocity(&self) -> f32 {
        field::f32_at(&self.fixed, 280)
    }
    /// Sets initial velocity.
    pub fn set_initial_velocity(&mut self, value: f32) {
        field::set_f32_at(&mut self.fixed, 280, value);
    }

    /// Returns the emitter resource path up to the first NUL.
    pub fn path(&self) -> Cow<'_, str> {
        field::text(&self.fixed[16..16 + PATH_SIZE])
    }

    /// Sets the emitter path while retaining all other fields.
    pub fn set_path(&mut self, path: &str) -> Result<(), Error> {
        field::set_text(&mut self.fixed[16..16 + PATH_SIZE], path)
    }

    /// Returns the untyped reserved word following the path.
    pub fn reserved(&self) -> u32 {
        u32::from_le_bytes(self.fixed[272..276].try_into().expect("four-byte field"))
    }

    /// Borrows decoded animation tracks.
    pub fn tracks(&self) -> &[AnimationTrack] {
        &self.tracks
    }

    /// Replaces optional animation tracks.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        for track in tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: ParticleEmitter::TAG,
                    offset: 0,
                });
            }
            track.encode()?;
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

fn is_track(tag: [u8; 4]) -> bool {
    matches!(
        &tag,
        b"KPEV" | b"KPEE" | b"KPEG" | b"KPLN" | b"KPLT" | b"KPEL" | b"KPES"
    )
}

impl Model {
    /// Decodes all `PREM` records in file order.
    pub fn particle_emitters(&self) -> Result<Vec<ParticleEmitter>, Error> {
        self.collect_chunk_records::<crate::ParticleEmittersChunk>(|chunk| match chunk {
            crate::ModelChunk::ParticleEmitters(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces particle emitters in the first `PREM` chunk.
    pub fn set_particle_emitters(&mut self, emitters: &[ParticleEmitter]) -> Result<(), Error> {
        let size = emitters.iter().try_fold(0usize, |sum, emitter| {
            sum.checked_add(emitter.encode()?.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: ParticleEmitter::TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for emitter in emitters {
            data.extend_from_slice(&emitter.encode()?);
        }
        self.replace_chunks(ParticleEmitter::TAG, data)?;
        Ok(())
    }
}

impl Record for ParticleEmitter {
    fn decode(bytes: &[u8], _version: u32) -> Result<Self, Error> {
        let layout = sized_node::layout(bytes, ParticleEmitter::TAG, FIXED_SIZE)?;
        let node = Node::decode(&bytes[4..layout.fixed_start], 0)?;
        let fixed = bytes[layout.fixed_start..layout.track_start]
            .try_into()
            .expect("validated fixed fields");
        let mut tracks = Vec::new();
        let mut offset = layout.track_start;
        while offset < bytes.len() {
            let (track, consumed) = AnimationTrack::parse(bytes, offset)?;
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: ParticleEmitter::TAG,
                    offset,
                });
            }
            tracks.push(track);
            offset += consumed;
        }
        Ok(Self {
            node,
            fixed,
            tracks,
        })
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0; 4];
        bytes.extend_from_slice(&self.node.encode()?);
        bytes.extend_from_slice(&self.fixed);
        for track in &self.tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: ParticleEmitter::TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.encode()?);
        }
        let size = u32::try_from(bytes.len()).map_err(|_| Error::ChunkTooLarge {
            tag: ParticleEmitter::TAG,
            size: bytes.len(),
        })?;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        Ok(bytes)
    }
}

impl ParticleEmitter {
    /// The tag of the chunk containing this record.
    pub const TAG: [u8; 4] = *b"PREM";
}
