//! Classic particle emitters stored in `PREM` chunks.

use std::borrow::Cow;

use crate::{sized_node, AnimationTrack, Error, Model, Node};

const TAG: [u8; 4] = *b"PREM";
const FIXED_SIZE: usize = 284;
const PATH_SIZE: usize = 256;

/// A Classic particle emitter with optional animated properties.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticleEmitter {
    bytes: Vec<u8>,
}

impl ParticleEmitter {
    /// Creates an emitter with zeroed physical values.
    pub fn new(node: Node, path: &str) -> Result<Self, Error> {
        let bytes = sized_node::new_record(&node, FIXED_SIZE, TAG)?;
        let mut emitter = Self { bytes };
        emitter.set_path(path)?;
        Ok(emitter)
    }

    /// Wraps one inclusive-size emitter record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        sized_node::layout(bytes, TAG, FIXED_SIZE)?;
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete record.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the shared node.
    pub fn node(&self) -> Node {
        let start = 4;
        let end = self.fixed_start();
        Node::from_bytes(&self.bytes[start..end]).expect("validated node")
    }

    /// Returns emission rate.
    pub fn emission_rate(&self) -> f32 {
        self.f32_at(0)
    }
    /// Sets emission rate.
    pub fn set_emission_rate(&mut self, value: f32) {
        self.set_f32_at(0, value);
    }
    /// Returns gravity.
    pub fn gravity(&self) -> f32 {
        self.f32_at(4)
    }
    /// Sets gravity.
    pub fn set_gravity(&mut self, value: f32) {
        self.set_f32_at(4, value);
    }
    /// Returns longitude.
    pub fn longitude(&self) -> f32 {
        self.f32_at(8)
    }
    /// Sets longitude.
    pub fn set_longitude(&mut self, value: f32) {
        self.set_f32_at(8, value);
    }
    /// Returns latitude.
    pub fn latitude(&self) -> f32 {
        self.f32_at(12)
    }
    /// Sets latitude.
    pub fn set_latitude(&mut self, value: f32) {
        self.set_f32_at(12, value);
    }
    /// Returns particle lifetime.
    pub fn life_span(&self) -> f32 {
        self.f32_at(276)
    }
    /// Sets particle lifetime.
    pub fn set_life_span(&mut self, value: f32) {
        self.set_f32_at(276, value);
    }
    /// Returns initial velocity.
    pub fn initial_velocity(&self) -> f32 {
        self.f32_at(280)
    }
    /// Sets initial velocity.
    pub fn set_initial_velocity(&mut self, value: f32) {
        self.set_f32_at(280, value);
    }

    /// Returns the emitter resource path up to the first NUL.
    pub fn path(&self) -> Cow<'_, str> {
        let start = self.fixed_start() + 16;
        let field = &self.bytes[start..start + PATH_SIZE];
        let end = field
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(PATH_SIZE);
        String::from_utf8_lossy(&field[..end])
    }

    /// Sets the emitter path while retaining all other fields.
    pub fn set_path(&mut self, path: &str) -> Result<(), Error> {
        if path.len() >= PATH_SIZE || path.as_bytes().contains(&0) {
            return Err(Error::InvalidString {
                max_bytes: PATH_SIZE - 1,
            });
        }
        let start = self.fixed_start() + 16;
        self.bytes[start..start + PATH_SIZE].fill(0);
        self.bytes[start..start + path.len()].copy_from_slice(path.as_bytes());
        Ok(())
    }

    /// Returns the untyped reserved word following the path.
    pub fn reserved(&self) -> u32 {
        let start = self.fixed_start() + 272;
        u32::from_le_bytes(
            self.bytes[start..start + 4]
                .try_into()
                .expect("four-byte field"),
        )
    }

    /// Decodes the optional animation tracks.
    pub fn tracks(&self) -> Result<Vec<AnimationTrack>, Error> {
        let mut result = Vec::new();
        let mut offset = self.track_start();
        while offset < self.bytes.len() {
            let (track, consumed) = AnimationTrack::parse(&self.bytes, offset)?;
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord { tag: TAG, offset });
            }
            result.push(track);
            offset += consumed;
        }
        Ok(result)
    }

    /// Replaces optional animation tracks and updates the record size.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        let mut bytes = self.bytes[..self.track_start()].to_vec();
        for track in tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.to_bytes()?);
        }
        if bytes.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: bytes.len(),
            });
        }
        let size = bytes.len() as u32;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        self.bytes = bytes;
        Ok(())
    }

    fn fixed_start(&self) -> usize {
        sized_node::layout(&self.bytes, TAG, FIXED_SIZE)
            .expect("validated record")
            .fixed_start
    }

    fn track_start(&self) -> usize {
        sized_node::layout(&self.bytes, TAG, FIXED_SIZE)
            .expect("validated record")
            .track_start
    }

    fn f32_at(&self, relative: usize) -> f32 {
        let start = self.fixed_start() + relative;
        f32::from_le_bytes(
            self.bytes[start..start + 4]
                .try_into()
                .expect("four-byte field"),
        )
    }

    fn set_f32_at(&mut self, relative: usize, value: f32) {
        let start = self.fixed_start() + relative;
        self.bytes[start..start + 4].copy_from_slice(&value.to_le_bytes());
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
        let mut result = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            result.extend(
                sized_node::records(&chunk.data, TAG, FIXED_SIZE)?
                    .into_iter()
                    .map(ParticleEmitter::from_bytes)
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        Ok(result)
    }

    /// Replaces particle emitters in the first `PREM` chunk.
    pub fn set_particle_emitters(&mut self, emitters: &[ParticleEmitter]) -> Result<(), Error> {
        let size = emitters.iter().try_fold(0usize, |sum, emitter| {
            sum.checked_add(emitter.bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for emitter in emitters {
            data.extend_from_slice(emitter.as_bytes());
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
