//! Reforged popcorn particle emitters in `CORN` chunks.

use std::borrow::Cow;

use crate::{AnimationTrack, Error, Model, Node};

const TAG: [u8; 4] = *b"CORN";
const PATH_SIZE: usize = 260;
const FIXED_SIZE: usize = 32 + PATH_SIZE * 2;

/// A popcorn particle emitter with fixed fields and optional animation tracks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PopcornEmitter {
    bytes: Vec<u8>,
}

impl PopcornEmitter {
    /// Creates an emitter with zeroed physical values.
    pub fn new(node: Node, path: &str, visibility_guide: &str) -> Result<Self, Error> {
        let mut bytes = vec![0; 4];
        bytes.extend_from_slice(node.as_bytes());
        bytes.resize(bytes.len() + FIXED_SIZE, 0);
        if bytes.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: bytes.len(),
            });
        }
        let size = bytes.len() as u32;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        let mut emitter = Self { bytes };
        emitter.set_path(path)?;
        emitter.set_visibility_guide(visibility_guide)?;
        Ok(emitter)
    }

    /// Wraps one inclusive-size emitter record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if record_end(bytes, 0)? != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete emitter record.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the shared node.
    pub fn node(&self) -> Node {
        let size = self.node_size();
        Node::from_bytes(&self.bytes[4..4 + size]).expect("validated node")
    }

    /// Returns particle lifetime.
    pub fn life_span(&self) -> f32 {
        self.f32_at(self.fixed_offset())
    }

    /// Sets particle lifetime.
    pub fn set_life_span(&mut self, value: f32) {
        self.set_f32_at(self.fixed_offset(), value);
    }

    /// Returns emission rate.
    pub fn emission_rate(&self) -> f32 {
        self.f32_at(self.fixed_offset() + 4)
    }

    /// Sets emission rate.
    pub fn set_emission_rate(&mut self, value: f32) {
        self.set_f32_at(self.fixed_offset() + 4, value);
    }

    /// Returns particle speed.
    pub fn speed(&self) -> f32 {
        self.f32_at(self.fixed_offset() + 8)
    }

    /// Sets particle speed.
    pub fn set_speed(&mut self, value: f32) {
        self.set_f32_at(self.fixed_offset() + 8, value);
    }

    /// Returns RGB particle color.
    pub fn color(&self) -> [f32; 3] {
        std::array::from_fn(|axis| self.f32_at(self.fixed_offset() + 12 + axis * 4))
    }

    /// Sets RGB particle color.
    pub fn set_color(&mut self, color: [f32; 3]) {
        for (axis, value) in color.into_iter().enumerate() {
            self.set_f32_at(self.fixed_offset() + 12 + axis * 4, value);
        }
    }

    /// Returns base alpha.
    pub fn alpha(&self) -> f32 {
        self.f32_at(self.fixed_offset() + 24)
    }

    /// Sets base alpha.
    pub fn set_alpha(&mut self, value: f32) {
        self.set_f32_at(self.fixed_offset() + 24, value);
    }

    /// Returns the replaceable texture ID.
    pub fn replaceable_id(&self) -> u32 {
        self.u32_at(self.fixed_offset() + 28)
    }

    /// Sets the replaceable texture ID.
    pub fn set_replaceable_id(&mut self, id: u32) {
        self.set_u32_at(self.fixed_offset() + 28, id);
    }

    /// Returns the model path.
    pub fn path(&self) -> Cow<'_, str> {
        self.text_at(self.fixed_offset() + 32)
    }

    /// Sets the model path.
    pub fn set_path(&mut self, path: &str) -> Result<(), Error> {
        self.set_text_at(self.fixed_offset() + 32, path)
    }

    /// Returns the animation visibility guide path.
    pub fn visibility_guide(&self) -> Cow<'_, str> {
        self.text_at(self.fixed_offset() + 32 + PATH_SIZE)
    }

    /// Sets the animation visibility guide path.
    pub fn set_visibility_guide(&mut self, guide: &str) -> Result<(), Error> {
        self.set_text_at(self.fixed_offset() + 32 + PATH_SIZE, guide)
    }

    /// Decodes optional `KPP*` animation tracks.
    pub fn tracks(&self) -> Result<Vec<AnimationTrack>, Error> {
        let mut tracks = Vec::new();
        let mut offset = self.fixed_offset() + FIXED_SIZE;
        while offset < self.bytes.len() {
            let (track, consumed) = AnimationTrack::parse(&self.bytes, offset)?;
            if !is_track_tag(track.tag) {
                return Err(Error::MalformedRecord { tag: TAG, offset });
            }
            tracks.push(track);
            offset += consumed;
        }
        Ok(tracks)
    }

    /// Replaces optional animation tracks and updates the inclusive size.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        let mut bytes = self.bytes[..self.fixed_offset() + FIXED_SIZE].to_vec();
        for track in tracks {
            if !is_track_tag(track.tag) {
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

    fn node_size(&self) -> usize {
        u32::from_le_bytes(self.bytes[4..8].try_into().expect("validated node size")) as usize
    }

    fn fixed_offset(&self) -> usize {
        4 + self.node_size()
    }

    fn u32_at(&self, offset: usize) -> u32 {
        u32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("four-byte field"),
        )
    }

    fn set_u32_at(&mut self, offset: usize, value: u32) {
        self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn f32_at(&self, offset: usize) -> f32 {
        f32::from_bits(self.u32_at(offset))
    }

    fn set_f32_at(&mut self, offset: usize, value: f32) {
        self.set_u32_at(offset, value.to_bits());
    }

    fn text_at(&self, offset: usize) -> Cow<'_, str> {
        let field = &self.bytes[offset..offset + PATH_SIZE];
        let end = field
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(PATH_SIZE);
        String::from_utf8_lossy(&field[..end])
    }

    fn set_text_at(&mut self, offset: usize, value: &str) -> Result<(), Error> {
        if value.len() >= PATH_SIZE || value.as_bytes().contains(&0) {
            return Err(Error::InvalidString {
                max_bytes: PATH_SIZE - 1,
            });
        }
        self.bytes[offset..offset + PATH_SIZE].fill(0);
        self.bytes[offset..offset + value.len()].copy_from_slice(value.as_bytes());
        Ok(())
    }
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
