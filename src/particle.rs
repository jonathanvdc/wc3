//! Classic particle emitters stored in `PREM` chunks.

use std::borrow::Cow;

use crate::{sized_node, AnimationTrack, Error, Model, Node};

const TAG: [u8; 4] = *b"PREM";
const FIXED_SIZE: usize = 284;
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

    /// Wraps one inclusive-size emitter record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let layout = sized_node::layout(bytes, TAG, FIXED_SIZE)?;
        let node = Node::from_bytes(&bytes[4..layout.fixed_start])?;
        let fixed = bytes[layout.fixed_start..layout.track_start]
            .try_into()
            .expect("validated fixed fields");
        let mut tracks = Vec::new();
        let mut offset = layout.track_start;
        while offset < bytes.len() {
            let (track, consumed) = AnimationTrack::parse(bytes, offset)?;
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord { tag: TAG, offset });
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

    /// Serializes the complete record.
    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0; 4];
        bytes.extend_from_slice(&self.node.to_bytes());
        bytes.extend_from_slice(&self.fixed);
        for track in &self.tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.to_bytes()?);
        }
        let size = u32::try_from(bytes.len()).map_err(|_| Error::ChunkTooLarge {
            tag: TAG,
            size: bytes.len(),
        })?;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        Ok(bytes)
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
        let field = &self.fixed[16..16 + PATH_SIZE];
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
        self.fixed[16..16 + PATH_SIZE].fill(0);
        self.fixed[16..16 + path.len()].copy_from_slice(path.as_bytes());
        Ok(())
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
                    tag: TAG,
                    offset: 0,
                });
            }
            track.to_bytes()?;
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }

    fn f32_at(&self, relative: usize) -> f32 {
        f32::from_le_bytes(
            self.fixed[relative..relative + 4]
                .try_into()
                .expect("four-byte field"),
        )
    }

    fn set_f32_at(&mut self, relative: usize, value: f32) {
        self.fixed[relative..relative + 4].copy_from_slice(&value.to_le_bytes());
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
            sum.checked_add(emitter.to_bytes()?.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for emitter in emitters {
            data.extend_from_slice(&emitter.to_bytes()?);
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
