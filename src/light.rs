//! Light records in `LITE` chunks.

use crate::{AnimationTrack, Error, Model, Node};

const TAG: [u8; 4] = *b"LITE";
const FIXED_SIZE: usize = 44;

/// A light node with fixed lighting values and preserved animation bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Light {
    bytes: Vec<u8>,
}

impl Light {
    /// Creates a light with zeroed lighting values.
    pub fn new(node: Node, light_type: u32) -> Self {
        let mut bytes = vec![0; 4];
        bytes.extend_from_slice(node.as_bytes());
        bytes.extend_from_slice(&light_type.to_le_bytes());
        bytes.resize(bytes.len() + FIXED_SIZE - 4, 0);
        let size = bytes.len() as u32;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        Self { bytes }
    }

    /// Wraps one inclusive-size light record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let end = record_end(bytes, 0)?;
        if end != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: end,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete light record.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the shared node.
    pub fn node(&self) -> Node {
        let start = 4;
        let end = start + self.node_size();
        Node::from_bytes(&self.bytes[start..end]).expect("validated node")
    }

    /// Returns raw light type ID.
    pub fn light_type(&self) -> u32 {
        self.u32_at(self.fixed_offset())
    }

    /// Sets raw light type ID.
    pub fn set_light_type(&mut self, kind: u32) {
        self.set_u32_at(self.fixed_offset(), kind);
    }

    /// Returns attenuation start distance.
    pub fn attenuation_start(&self) -> f32 {
        self.f32_at(self.fixed_offset() + 4)
    }

    /// Sets attenuation start distance.
    pub fn set_attenuation_start(&mut self, value: f32) {
        self.set_f32_at(self.fixed_offset() + 4, value);
    }

    /// Returns attenuation end distance.
    pub fn attenuation_end(&self) -> f32 {
        self.f32_at(self.fixed_offset() + 8)
    }

    /// Sets attenuation end distance.
    pub fn set_attenuation_end(&mut self, value: f32) {
        self.set_f32_at(self.fixed_offset() + 8, value);
    }

    /// Returns RGB light color.
    pub fn color(&self) -> [f32; 3] {
        self.vec3_at(self.fixed_offset() + 12)
    }

    /// Sets RGB light color.
    pub fn set_color(&mut self, color: [f32; 3]) {
        self.set_vec3_at(self.fixed_offset() + 12, color);
    }

    /// Returns light intensity.
    pub fn intensity(&self) -> f32 {
        self.f32_at(self.fixed_offset() + 24)
    }

    /// Sets light intensity.
    pub fn set_intensity(&mut self, value: f32) {
        self.set_f32_at(self.fixed_offset() + 24, value);
    }

    /// Returns ambient RGB color.
    pub fn ambient_color(&self) -> [f32; 3] {
        self.vec3_at(self.fixed_offset() + 28)
    }

    /// Sets ambient RGB color.
    pub fn set_ambient_color(&mut self, color: [f32; 3]) {
        self.set_vec3_at(self.fixed_offset() + 28, color);
    }

    /// Returns ambient intensity.
    pub fn ambient_intensity(&self) -> f32 {
        self.f32_at(self.fixed_offset() + 40)
    }

    /// Sets ambient intensity.
    pub fn set_ambient_intensity(&mut self, value: f32) {
        self.set_f32_at(self.fixed_offset() + 40, value);
    }

    /// Returns optional light track bytes without interpretation.
    pub fn track_bytes(&self) -> &[u8] {
        &self.bytes[self.fixed_offset() + FIXED_SIZE..]
    }

    /// Decodes optional visibility, color, intensity, and attenuation tracks.
    pub fn tracks(&self) -> Result<Vec<AnimationTrack>, Error> {
        let mut result = Vec::new();
        let mut offset = self.fixed_offset() + FIXED_SIZE;
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

    /// Replaces optional light animation tracks.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        let mut bytes = self.bytes[..self.fixed_offset() + FIXED_SIZE].to_vec();
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

    fn vec3_at(&self, offset: usize) -> [f32; 3] {
        std::array::from_fn(|axis| self.f32_at(offset + axis * 4))
    }

    fn set_vec3_at(&mut self, offset: usize, color: [f32; 3]) {
        for (axis, value) in color.into_iter().enumerate() {
            self.set_f32_at(offset + axis * 4, value);
        }
    }
}

fn is_track(tag: [u8; 4]) -> bool {
    matches!(
        &tag,
        b"KLAV" | b"KLAC" | b"KLAI" | b"KLBC" | b"KLBI" | b"KLAS" | b"KLAE"
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
    /// Decodes all `LITE` records in file order.
    pub fn lights(&self) -> Result<Vec<Light>, Error> {
        let mut lights = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            let mut offset = 0;
            while offset < chunk.data.len() {
                let end = record_end(&chunk.data, offset)?;
                lights.push(Light::from_bytes(&chunk.data[offset..end])?);
                offset = end;
            }
        }
        Ok(lights)
    }

    /// Replaces lights in the first `LITE` chunk.
    pub fn set_lights(&mut self, lights: &[Light]) -> Result<(), Error> {
        let size = lights.iter().try_fold(0usize, |sum, light| {
            sum.checked_add(light.bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for light in lights {
            data.extend_from_slice(light.as_bytes());
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
