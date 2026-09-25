//! Reforged popcorn particle emitters in `CORN` chunks.

use std::borrow::Cow;

use crate::{AnimationTrack, Error, Model, Node};

const TAG: [u8; 4] = *b"CORN";
const PATH_SIZE: usize = 260;
const FIXED_SIZE: usize = 32 + PATH_SIZE * 2;

/// A popcorn particle emitter with decoded fields and animation tracks.
#[derive(Clone, Debug, PartialEq)]
pub struct PopcornEmitter {
    node: Node,
    life_span: f32,
    emission_rate: f32,
    speed: f32,
    color: [f32; 3],
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
        emitter.to_bytes()?;
        Ok(emitter)
    }

    /// Parses one inclusive-size emitter record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if record_end(bytes, 0)? != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        let node_size =
            u32::from_le_bytes(bytes[4..8].try_into().expect("validated node size")) as usize;
        let fixed = 4 + node_size;
        let node = Node::from_bytes(&bytes[4..fixed])?;
        let word = |offset| {
            u32::from_le_bytes(
                bytes[fixed + offset..fixed + offset + 4]
                    .try_into()
                    .expect("validated field"),
            )
        };
        let float = |offset| f32::from_bits(word(offset));
        let path = bytes[fixed + 32..fixed + 32 + PATH_SIZE]
            .try_into()
            .expect("validated path");
        let visibility_guide = bytes[fixed + 32 + PATH_SIZE..fixed + FIXED_SIZE]
            .try_into()
            .expect("validated guide");
        let mut tracks = Vec::new();
        let mut offset = fixed + FIXED_SIZE;
        while offset < bytes.len() {
            let (track, consumed) = AnimationTrack::parse(bytes, offset)?;
            if !is_track_tag(track.tag) {
                return Err(Error::MalformedRecord { tag: TAG, offset });
            }
            tracks.push(track);
            offset += consumed;
        }
        Ok(Self {
            node,
            life_span: float(0),
            emission_rate: float(4),
            speed: float(8),
            color: [float(12), float(16), float(20)],
            alpha: float(24),
            replaceable_id: word(28),
            path,
            visibility_guide,
            tracks,
        })
    }

    /// Serializes the inclusive-size emitter record.
    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0; 4];
        bytes.extend_from_slice(&self.node.to_bytes());
        for value in [
            self.life_span,
            self.emission_rate,
            self.speed,
            self.color[0],
            self.color[1],
            self.color[2],
            self.alpha,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&self.replaceable_id.to_le_bytes());
        bytes.extend_from_slice(&self.path);
        bytes.extend_from_slice(&self.visibility_guide);
        for track in &self.tracks {
            if !is_track_tag(track.tag) {
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
    pub fn color(&self) -> [f32; 3] {
        self.color
    }
    /// Sets RGB particle color.
    pub fn set_color(&mut self, color: [f32; 3]) {
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
        text_field(&self.path)
    }
    /// Sets the model path.
    pub fn set_path(&mut self, path: &str) -> Result<(), Error> {
        set_text_field(&mut self.path, path)
    }
    /// Returns the animation visibility guide path.
    pub fn visibility_guide(&self) -> Cow<'_, str> {
        text_field(&self.visibility_guide)
    }
    /// Sets the animation visibility guide path.
    pub fn set_visibility_guide(&mut self, guide: &str) -> Result<(), Error> {
        set_text_field(&mut self.visibility_guide, guide)
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
                    tag: TAG,
                    offset: 0,
                });
            }
            track.to_bytes()?;
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

fn text_field(field: &[u8; PATH_SIZE]) -> Cow<'_, str> {
    let end = field
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(PATH_SIZE);
    String::from_utf8_lossy(&field[..end])
}

fn set_text_field(field: &mut [u8; PATH_SIZE], value: &str) -> Result<(), Error> {
    if value.len() >= PATH_SIZE || value.as_bytes().contains(&0) {
        return Err(Error::InvalidString {
            max_bytes: PATH_SIZE - 1,
        });
    }
    field.fill(0);
    field[..value.len()].copy_from_slice(value.as_bytes());
    Ok(())
}

fn is_track_tag(tag: [u8; 4]) -> bool {
    matches!(
        &tag,
        b"KPPA" | b"KPPC" | b"KPPE" | b"KPPL" | b"KPPS" | b"KPPV"
    )
}

fn record_end(data: &[u8], offset: usize) -> Result<usize, Error> {
    let size_bytes = data
        .get(offset..offset.saturating_add(4))
        .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
    let size = u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize;
    let end = offset
        .checked_add(size)
        .filter(|&end| end <= data.len())
        .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
    let node_start = offset + 4;
    let node_size_bytes =
        data.get(node_start..node_start.saturating_add(4))
            .ok_or(Error::MalformedRecord {
                tag: TAG,
                offset: node_start,
            })?;
    let node_size =
        u32::from_le_bytes(node_size_bytes.try_into().expect("four-byte size")) as usize;
    let node_end = node_start
        .checked_add(node_size)
        .filter(|&node_end| node_end <= end)
        .ok_or(Error::MalformedRecord {
            tag: TAG,
            offset: node_start,
        })?;
    Node::from_bytes(&data[node_start..node_end])?;
    if node_end
        .checked_add(FIXED_SIZE)
        .map_or(true, |required| required > end)
    {
        return Err(Error::MalformedRecord {
            tag: TAG,
            offset: node_end,
        });
    }
    Ok(end)
}

impl Model {
    /// Decodes all popcorn emitters in `CORN` chunks.
    pub fn popcorn_emitters(&self) -> Result<Vec<PopcornEmitter>, Error> {
        let mut emitters = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            let mut offset = 0;
            while offset < chunk.data.len() {
                let end = record_end(&chunk.data, offset)?;
                emitters.push(PopcornEmitter::from_bytes(&chunk.data[offset..end])?);
                offset = end;
            }
        }
        Ok(emitters)
    }

    /// Replaces popcorn emitters in the first `CORN` chunk.
    pub fn set_popcorn_emitters(&mut self, emitters: &[PopcornEmitter]) -> Result<(), Error> {
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
