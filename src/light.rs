//! Light records in `LITE` chunks.

use crate::Record;
use crate::{AnimationTrack, Error, Model, Node};

const FIXED_SIZE: usize = 44;
const EXTENDED_SIZE: usize = 72;

/// A light node with decoded lighting values and animation tracks.
#[derive(Clone, Debug, PartialEq)]
pub struct Light {
    node: Node,
    light_type: u32,
    attenuation_start: f32,
    attenuation_end: f32,
    color: [f32; 3],
    intensity: f32,
    ambient_color: [f32; 3],
    ambient_intensity: f32,
    extended_words: Option<[u32; 7]>,
    tracks: Vec<AnimationTrack>,
}

impl Light {
    /// Creates a light with zeroed lighting values.
    pub fn new(node: Node, light_type: u32) -> Self {
        Self {
            node,
            light_type,
            attenuation_start: 0.0,
            attenuation_end: 0.0,
            color: [0.0; 3],
            intensity: 0.0,
            ambient_color: [0.0; 3],
            ambient_intensity: 0.0,
            extended_words: None,
            tracks: Vec::new(),
        }
    }

    /// Creates a light with the additional fixed fields used by newer models.
    pub fn new_for_version(node: Node, light_type: u32, version: u32) -> Self {
        let mut light = Self::new(node, light_type);
        if version >= 1200 {
            light.extended_words = Some([0; 7]);
        }
        light
    }

    /// Borrows the shared node.
    pub fn node(&self) -> &Node {
        &self.node
    }

    /// Borrows the shared node for editing.
    pub fn node_mut(&mut self) -> &mut Node {
        &mut self.node
    }

    /// Returns raw light type ID.
    pub fn light_type(&self) -> u32 {
        self.light_type
    }
    /// Sets raw light type ID.
    pub fn set_light_type(&mut self, kind: u32) {
        self.light_type = kind;
    }
    /// Returns attenuation start distance.
    pub fn attenuation_start(&self) -> f32 {
        self.attenuation_start
    }
    /// Sets attenuation start distance.
    pub fn set_attenuation_start(&mut self, value: f32) {
        self.attenuation_start = value;
    }
    /// Returns attenuation end distance.
    pub fn attenuation_end(&self) -> f32 {
        self.attenuation_end
    }
    /// Sets attenuation end distance.
    pub fn set_attenuation_end(&mut self, value: f32) {
        self.attenuation_end = value;
    }
    /// Returns RGB light color.
    pub fn color(&self) -> [f32; 3] {
        self.color
    }
    /// Sets RGB light color.
    pub fn set_color(&mut self, color: [f32; 3]) {
        self.color = color;
    }
    /// Returns light intensity.
    pub fn intensity(&self) -> f32 {
        self.intensity
    }
    /// Sets light intensity.
    pub fn set_intensity(&mut self, value: f32) {
        self.intensity = value;
    }
    /// Returns ambient RGB color.
    pub fn ambient_color(&self) -> [f32; 3] {
        self.ambient_color
    }
    /// Sets ambient RGB color.
    pub fn set_ambient_color(&mut self, color: [f32; 3]) {
        self.ambient_color = color;
    }
    /// Returns ambient intensity.
    pub fn ambient_intensity(&self) -> f32 {
        self.ambient_intensity
    }
    /// Sets ambient intensity.
    pub fn set_ambient_intensity(&mut self, value: f32) {
        self.ambient_intensity = value;
    }
    /// Returns the additional seven raw words in newer light records, when present.
    pub fn extended_words(&self) -> Option<[u32; 7]> {
        self.extended_words
    }
    /// Sets the additional seven raw words in a newer light record.
    pub fn set_extended_words(&mut self, words: [u32; 7]) -> Result<(), Error> {
        if self.extended_words.is_none() {
            return Err(Error::MalformedRecord {
                tag: Light::TAG,
                offset: 4 + self.node.encode()?.len() + FIXED_SIZE,
            });
        }
        self.extended_words = Some(words);
        Ok(())
    }
    /// Borrows decoded light animation tracks.
    pub fn tracks(&self) -> &[AnimationTrack] {
        &self.tracks
    }
    /// Replaces optional light animation tracks.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        for track in tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: Light::TAG,
                    offset: 0,
                });
            }
            track.encode()?;
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("validated field"),
    )
}
fn read_f32(bytes: &[u8], offset: usize) -> f32 {
    f32::from_bits(read_u32(bytes, offset))
}
fn read_vec3(bytes: &[u8], offset: usize) -> [f32; 3] {
    std::array::from_fn(|i| read_f32(bytes, offset + i * 4))
}

fn is_track(tag: [u8; 4]) -> bool {
    matches!(
        &tag,
        b"KLAV" | b"KLAC" | b"KLAI" | b"KLBC" | b"KLBI" | b"KLAS" | b"KLAE"
    )
}

pub(crate) fn record_end(data: &[u8], offset: usize) -> Result<usize, Error> {
    let size_bytes = data
        .get(offset..offset.saturating_add(4))
        .ok_or(Error::MalformedRecord {
            tag: Light::TAG,
            offset,
        })?;
    let size = u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize;
    let end = offset
        .checked_add(size)
        .filter(|&end| end <= data.len())
        .ok_or(Error::MalformedRecord {
            tag: Light::TAG,
            offset,
        })?;
    let node_start = offset + 4;
    let node_size_bytes =
        data.get(node_start..node_start.saturating_add(4))
            .ok_or(Error::MalformedRecord {
                tag: Light::TAG,
                offset: node_start,
            })?;
    let node_size =
        u32::from_le_bytes(node_size_bytes.try_into().expect("four-byte size")) as usize;
    let node_end = node_start
        .checked_add(node_size)
        .filter(|&node_end| node_end <= end)
        .ok_or(Error::MalformedRecord {
            tag: Light::TAG,
            offset: node_start,
        })?;
    Node::decode(&data[node_start..node_end], 0)?;
    if node_end
        .checked_add(FIXED_SIZE)
        .map_or(true, |required| required > end)
    {
        return Err(Error::MalformedRecord {
            tag: Light::TAG,
            offset: node_end,
        });
    }
    Ok(end)
}

impl Model {
    /// Decodes all `LITE` records in file order.
    pub fn lights(&self) -> Result<Vec<Light>, Error> {
        self.collect_chunk_records::<crate::LightsChunk>(|chunk| match chunk {
            crate::ModelChunk::Lights(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces lights in the first `LITE` chunk.
    pub fn set_lights(&mut self, lights: &[Light]) -> Result<(), Error> {
        let size = lights.iter().try_fold(0usize, |sum, light| {
            sum.checked_add(light.encode()?.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: Light::TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for light in lights {
            data.extend_from_slice(&light.encode()?);
        }
        self.replace_chunks(Light::TAG, data)?;
        Ok(())
    }
}

impl Record for Light {
    fn decode(bytes: &[u8], _version: u32) -> Result<Self, Error> {
        let end = record_end(bytes, 0)?;
        if end != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: Light::TAG,
                offset: end,
            });
        }
        let node_size = read_u32(bytes, 4) as usize;
        let fixed = 4 + node_size;
        let node = Node::decode(&bytes[4..fixed], 0)?;
        let short = fixed + FIXED_SIZE;
        let extended = fixed + EXTENDED_SIZE;
        let has_extended = bytes.len() >= extended
            && (bytes.len() == extended
                || bytes
                    .get(extended..extended + 4)
                    .is_some_and(|tag| is_track(tag.try_into().expect("four-byte tag"))))
            && bytes.get(short..short + 4).map_or(true, |tag| {
                !is_track(tag.try_into().expect("four-byte tag"))
            });
        let extended_words =
            has_extended.then(|| std::array::from_fn(|i| read_u32(bytes, short + i * 4)));
        let mut tracks = Vec::new();
        let mut offset = if has_extended { extended } else { short };
        while offset < bytes.len() {
            let (track, consumed) = AnimationTrack::parse(bytes, offset)?;
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: Light::TAG,
                    offset,
                });
            }
            tracks.push(track);
            offset += consumed;
        }
        Ok(Self {
            node,
            light_type: read_u32(bytes, fixed),
            attenuation_start: read_f32(bytes, fixed + 4),
            attenuation_end: read_f32(bytes, fixed + 8),
            color: read_vec3(bytes, fixed + 12),
            intensity: read_f32(bytes, fixed + 24),
            ambient_color: read_vec3(bytes, fixed + 28),
            ambient_intensity: read_f32(bytes, fixed + 40),
            extended_words,
            tracks,
        })
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0; 4];
        bytes.extend_from_slice(&self.node.encode()?);
        bytes.extend_from_slice(&self.light_type.to_le_bytes());
        bytes.extend_from_slice(&self.attenuation_start.to_le_bytes());
        bytes.extend_from_slice(&self.attenuation_end.to_le_bytes());
        for value in self.color {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&self.intensity.to_le_bytes());
        for value in self.ambient_color {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&self.ambient_intensity.to_le_bytes());
        if let Some(words) = self.extended_words {
            for word in words {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        for track in &self.tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: Light::TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.encode()?);
        }
        let size = u32::try_from(bytes.len()).map_err(|_| Error::ChunkTooLarge {
            tag: Light::TAG,
            size: bytes.len(),
        })?;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        Ok(bytes)
    }
}

impl Light {
    /// The tag of the chunk containing this record.
    pub const TAG: [u8; 4] = *b"LITE";
}
